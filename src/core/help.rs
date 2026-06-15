use std::env;
use std::io::{self, IsTerminal};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const FG: &str = "\x1b[38;2;248;248;242m";
const COMMENT: &str = "\x1b[38;2;98;114;164m";
const CYAN: &str = "\x1b[38;2;139;233;253m";
const PINK: &str = "\x1b[38;2;255;121;198m";
const PURPLE: &str = "\x1b[38;2;189;147;249m";

pub(crate) fn print(text: &str) {
    print!("{}", format(text));
}

pub(crate) fn println(text: &str) {
    println!("{}", format(text));
}

pub(crate) fn format(text: &str) -> String {
    if !color_enabled() {
        return text.to_string();
    }

    let mut out = String::new();
    for (index, line) in text.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&format_line(line));
    }
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn color_enabled() -> bool {
    match env::var("AW_COLOR").ok().as_deref() {
        Some("always" | "1" | "true" | "yes") => true,
        Some("never" | "0" | "false" | "no") => false,
        _ if env::var_os("NO_COLOR").is_some() => false,
        _ => io::stdout().is_terminal(),
    }
}

fn format_line(line: &str) -> String {
    if line.trim().is_empty() {
        return String::new();
    }
    if let Some((name, description)) = line.split_once(':') {
        if !line.starts_with(' ') && !description.trim().is_empty() {
            return format!("{BOLD}{PURPLE}{name}:{RESET}{FG}{description}{RESET}");
        }
    }
    let trimmed = line.trim_end();
    if !line.starts_with(' ') && trimmed.ends_with(':') {
        return format!("{BOLD}{PINK}{trimmed}{RESET}");
    }
    if let Some(command) = line.strip_prefix("  ") {
        let (command, description) = split_command_description(command);
        if description.is_empty() {
            return format!("  {CYAN}{command}{RESET}");
        }
        return format!("  {CYAN}{command}{RESET}  {COMMENT}{description}{RESET}");
    }
    if line.starts_with("Options:") {
        return format!("{BOLD}{PINK}{line}{RESET}");
    }
    format!("{FG}{line}{RESET}")
}

fn split_command_description(line: &str) -> (&str, &str) {
    let Some((index, _)) = line
        .match_indices("  ")
        .find(|(index, _)| line[*index..].starts_with("  "))
    else {
        return (line.trim_end(), "");
    };
    let command = line[..index].trim_end();
    let description = line[index..].trim();
    if description.is_empty() {
        (line.trim_end(), "")
    } else {
        (command, description)
    }
}
