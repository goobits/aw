use std::fs;
use std::path::Path;

use crate::error::{AwError, Result};
use crate::paths::validate_name;

#[derive(Clone, Debug)]
pub(crate) struct IndexedTab {
    pub name: String,
    pub index: Option<usize>,
}

pub fn tab_name_from_line(line: &str) -> &str {
    line.split('\t').next().unwrap_or(line)
}

pub fn parse_tabs_csv(csv: &str) -> Result<Vec<String>> {
    let tabs: Vec<String> = csv.split(',').map(str::to_string).collect();
    if tabs.is_empty() {
        return Err(AwError::new("aw: expected at least one tab", 2));
    }
    for tab in &tabs {
        if tab.is_empty() {
            return Err(AwError::new(format!("aw: empty tab name in {}", csv), 2));
        }
        validate_name("tab", tab)?;
    }
    Ok(tabs)
}

pub fn parse_tabs_args(args: &[String]) -> Result<Vec<String>> {
    if args.len() == 1 && args[0].contains(',') {
        return parse_tabs_csv(&args[0]);
    }
    if args.is_empty() {
        return Err(AwError::new("aw: expected at least one tab", 2));
    }
    for tab in args {
        if tab.is_empty() {
            return Err(AwError::new("aw: empty tab name", 2));
        }
        validate_name("tab", tab)?;
    }
    Ok(args.to_vec())
}

pub(crate) fn parse_indexed_tab_spec(spec: &str) -> Result<IndexedTab> {
    if let Some((name, index)) = spec.rsplit_once('@') {
        if name.is_empty() {
            return Err(AwError::new(
                "aw: tab@index needs a tab name before @, for example keyboard@1",
                2,
            ));
        }
        if index.is_empty() || !index.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(AwError::new(
                format!("aw: tab index must be a number, for example {name}@1"),
                2,
            ));
        }
        validate_name("tab", name)?;
        let index = index.parse::<usize>().map_err(|_| {
            AwError::new(
                format!("aw: tab index is too large, for example {name}@1"),
                2,
            )
        })?;
        return Ok(IndexedTab {
            name: name.to_string(),
            index: Some(index),
        });
    }

    validate_name("tab", spec)?;
    Ok(IndexedTab {
        name: spec.to_string(),
        index: None,
    })
}

pub fn write_tabs_file(config_dir: &Path, workspace: &str, tabs: &[String]) -> Result<()> {
    validate_name("workspace", workspace)?;
    if tabs.is_empty() {
        return Err(AwError::new(
            format!("aw: workspace {} must have at least one tab", workspace),
            2,
        ));
    }
    for tab in tabs {
        validate_name("tab", tab)?;
    }
    fs::write(
        config_dir.join(format!("{}.tabs", workspace)),
        format!("{}\n", tabs.join("\n")),
    )?;
    Ok(())
}

pub fn read_tab_lines(tabs_file: &Path) -> Result<Vec<String>> {
    Ok(fs::read_to_string(tabs_file)?
        .lines()
        .map(str::to_string)
        .collect())
}

pub(crate) fn upsert_workspace_tab_line(tabs_file: &Path, spec: &str) -> Result<IndexedTab> {
    let indexed = parse_indexed_tab_spec(spec)?;
    let mut existing_line = None;
    let mut next_lines = Vec::new();

    for line in read_tab_lines(tabs_file)? {
        if tab_name_from_line(&line) == indexed.name {
            existing_line = Some(line);
        } else {
            next_lines.push(line);
        }
    }

    let line = existing_line.unwrap_or_else(|| indexed.name.clone());
    match indexed.index {
        Some(index) if index <= next_lines.len() => next_lines.insert(index, line),
        Some(index) => {
            return Err(index_out_of_range(index, next_lines.len()));
        }
        None => next_lines.push(line),
    }

    fs::write(tabs_file, format!("{}\n", next_lines.join("\n")))?;
    Ok(indexed)
}

pub fn remove_workspace_tab_line(tabs_file: &Path, remove_name: &str) -> Result<()> {
    validate_name("tab", remove_name)?;
    let mut found = false;
    let mut next_lines = Vec::new();
    for line in read_tab_lines(tabs_file)? {
        if tab_name_from_line(&line) == remove_name {
            found = true;
        } else {
            next_lines.push(line);
        }
    }
    if !found {
        return Err(AwError::new(
            format!("aw: tab not found in workspace file: {}", remove_name),
            1,
        ));
    }
    if next_lines.is_empty() {
        return Err(AwError::new("aw: workspace must have at least one tab", 2));
    }
    fs::write(tabs_file, format!("{}\n", next_lines.join("\n")))?;
    Ok(())
}

pub(crate) fn rename_workspace_tab_line_from_spec(
    tabs_file: &Path,
    old_name: &str,
    new_spec: &str,
) -> Result<IndexedTab> {
    let indexed = parse_indexed_tab_spec(new_spec)?;
    let mut next_lines = renamed_workspace_tab_lines(tabs_file, old_name, &indexed.name)?;
    if let Some(index) = indexed.index {
        let mut moved_line = None;
        let mut remaining_lines = Vec::new();
        for line in next_lines {
            if tab_name_from_line(&line) == indexed.name {
                moved_line = Some(line);
            } else {
                remaining_lines.push(line);
            }
        }
        let line = moved_line.ok_or_else(|| {
            AwError::new(
                format!("aw: tab not found in workspace file: {}", indexed.name),
                1,
            )
        })?;
        if index <= remaining_lines.len() {
            remaining_lines.insert(index, line);
        } else {
            return Err(index_out_of_range(index, remaining_lines.len()));
        }
        next_lines = remaining_lines;
    }
    fs::write(tabs_file, format!("{}\n", next_lines.join("\n")))?;
    Ok(indexed)
}

pub fn validate_workspace_tab_rename(
    tabs_file: &Path,
    old_name: &str,
    new_name: &str,
) -> Result<()> {
    renamed_workspace_tab_lines(tabs_file, old_name, new_name).map(|_| ())
}

fn renamed_workspace_tab_lines(
    tabs_file: &Path,
    old_name: &str,
    new_name: &str,
) -> Result<Vec<String>> {
    validate_name("tab", old_name)?;
    validate_name("tab", new_name)?;
    let current_lines = read_tab_lines(tabs_file)?;
    if current_lines
        .iter()
        .any(|line| tab_name_from_line(line) == new_name)
    {
        return Err(AwError::new(
            format!("aw: tab already exists in workspace file: {}", new_name),
            1,
        ));
    }

    let mut found = false;
    let mut next_lines = Vec::new();
    for line in current_lines {
        let current_name = tab_name_from_line(&line);
        if current_name == old_name {
            found = true;
            let rest = &line[current_name.len()..];
            next_lines.push(format!("{}{}", new_name, rest));
        } else {
            next_lines.push(line);
        }
    }
    if !found {
        return Err(AwError::new(
            format!("aw: tab not found in workspace file: {}", old_name),
            1,
        ));
    }
    Ok(next_lines)
}

fn index_out_of_range(index: usize, max: usize) -> AwError {
    AwError::new(
        format!("aw: tab index {index} is past the end; use 0 through {max}"),
        2,
    )
}
