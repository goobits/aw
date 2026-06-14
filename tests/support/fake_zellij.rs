use std::path::Path;

use super::command::TestHome;
use super::temp;

pub fn installed_home(name: &str) -> TestHome {
    let home = TestHome::new(name);
    install(&home.bin);
    home.install_aw();
    home
}

pub fn install(bin_dir: &Path) {
    let path = bin_dir.join("zellij");
    temp::write(&path, FAKE_ZELLIJ);
    temp::make_executable(path);
}

pub fn sorted_tab_names(path: impl AsRef<Path>) -> Vec<String> {
    let mut rows = temp::read(path)
        .lines()
        .map(|line| {
            let columns: Vec<_> = line.split('\t').collect();
            (
                columns[1].parse::<usize>().expect("tab position"),
                columns[3].to_string(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.0);
    rows.into_iter().map(|row| row.1).collect()
}

pub fn tab_name(path: impl AsRef<Path>, tab_id: &str) -> String {
    temp::read(path)
        .lines()
        .find_map(|line| {
            let columns: Vec<_> = line.split('\t').collect();
            (columns[0] == tab_id).then(|| columns[3].to_string())
        })
        .expect("tab name")
}

pub const FAKE_ZELLIJ: &str = r#"#!/usr/bin/env bash
set -euo pipefail

write_state() {
  local next
  next="$(mktemp)"
  cat > "$next"
  mv "$next" "$state"
}

write_panes() {
  local next
  next="$(mktemp)"
  cat > "$next"
  mv "$next" "$panes"
}

print_sessions() {
  if [[ -v FAKE_ZELLIJ_SESSIONS ]]; then
    printf '%s' "$FAKE_ZELLIJ_SESSIONS"
    if [[ -n "$FAKE_ZELLIJ_SESSIONS" && "$FAKE_ZELLIJ_SESSIONS" != *$'\n' ]]; then
      printf '\n'
    fi
  else
    printf 'test-live\n'
  fi
}

if [[ "${1:-}" == "--version" ]]; then
  printf 'zellij 0.44.3\n'
  exit 0
fi

if [[ "${1:-}" == "setup" && "${2:-}" == "--check" ]]; then
  exit 0
fi

if [[ "${1:-}" == "list-sessions" ]]; then
  print_sessions
  exit 0
fi

if [[ "${1:-}" == "delete-session" ]]; then
  session="${@: -1}"
  if [[ -n "${FAKE_ZELLIJ_DELETED_SESSION:-}" ]]; then
    printf '%s\n' "$session" > "$FAKE_ZELLIJ_DELETED_SESSION"
  fi
  if [[ "$*" != *"--force"* ]]; then
    printf 'expected force delete\n' >&2
    exit 1
  fi
  exit 0
fi

if [[ "${1:-}" == "attach" || "${1:-}" == "--layout" ]]; then
  if [[ "${1:-}" == "attach" && "${FAKE_ZELLIJ_ATTACH_CURRENT_PANIC:-0}" == "1" ]]; then
    printf 'thread '\''main'\'' panicked at src/commands.rs:858:25:\nYou are trying to attach to the current session ("%s"). This is not supported.\n' "${@: -1}" >&2
    exit 101
  fi
  if [[ "${FAKE_ZELLIJ_FAIL_ON_LAUNCH_ENV_LEAK:-0}" == "1" && ( -n "${ZELLIJ:-}" || -n "${ZELLIJ_SESSION_NAME:-}" ) ]]; then
    printf 'zellij launch inherited ZELLIJ=%s ZELLIJ_SESSION_NAME=%s\n' "${ZELLIJ:-}" "${ZELLIJ_SESSION_NAME:-}" >&2
    exit 1
  fi
  if [[ "${FAKE_ZELLIJ_FAIL_ON_ATTACH:-0}" == "1" ]]; then
    printf 'workspace assignment must not attach or create a zellij client\n' >&2
    exit 1
  fi
  if [[ -n "${FAKE_ZELLIJ_LAUNCH_ARGS:-}" ]]; then
    printf '%s\n' "$*" > "$FAKE_ZELLIJ_LAUNCH_ARGS"
  fi
  exit 0
fi

if [[ "${1:-}" == "action" && "${2:-}" == "switch-session" && "${FAKE_ZELLIJ_FAIL_ON_SWITCH:-0}" == "1" ]]; then
  printf 'workspace assignment must not switch zellij sessions\n' >&2
  exit 1
fi

if [[ "${1:-}" == "action" && "${2:-}" == "switch-session" ]]; then
  if [[ -n "${FAKE_ZELLIJ_SWITCH_ARGS:-}" ]]; then
    printf '%s\n' "$*" > "$FAKE_ZELLIJ_SWITCH_ARGS"
  fi
  exit 0
fi

if [[ "${1:-}" != "action" ]]; then
  exit 1
fi

state="${FAKE_ZELLIJ_TABS:?}"
panes="${state}.panes"
if [[ -n "${FAKE_ZELLIJ_SESSION_NAMES:-}" ]]; then
  printf '%s\n' "${ZELLIJ_SESSION_NAME:-}" >> "$FAKE_ZELLIJ_SESSION_NAMES"
fi

shift
case "${1:-}" in
  list-tabs)
    awk -F '\t' '
      BEGIN { printf "[" }
      {
        if (NR > 1) printf ","
        printf "{\"tab_id\":%s,\"position\":%s,\"active\":%s,\"name\":\"%s\"}", $1, $2, $3, $4
      }
      END { printf "]" }
    ' "$state"
    ;;
  list-panes)
    if [[ -s "$panes" && "${FAKE_ZELLIJ_GENERATE_PANES_FROM_TABS:-0}" != "1" ]]; then
      awk -F '\t' '
        BEGIN { printf "[" }
        {
          if (NR > 1) printf ","
          printf "{\"id\":%s,\"tab_id\":%s,\"tab_name\":\"%s\",\"is_plugin\":%s,\"title\":\"%s\"}", $1, $2, $3, $4, $5
        }
        END { printf "]" }
      ' "$panes"
    else
      awk -F '\t' '
        BEGIN { printf "[" }
        {
          if (NR > 1) printf ","
          printf "{\"id\":%s,\"tab_id\":%s,\"tab_name\":\"%s\",\"is_plugin\":false,\"title\":\"pane\"}", $1, $1, $4
        }
        END { printf "]" }
      ' "$state"
    fi
    ;;
  dump-screen)
    target=""
    while [[ "$#" -gt 0 ]]; do
      case "$1" in
        --pane-id | -p)
          target="${2:-}"
          target="${target#terminal_}"
          shift 2
          ;;
        *)
          shift
          ;;
      esac
    done
    [[ -n "${FAKE_ZELLIJ_SCREEN_DIR:-}" && -f "$FAKE_ZELLIJ_SCREEN_DIR/$target.txt" ]] || exit 0
    cat "$FAKE_ZELLIJ_SCREEN_DIR/$target.txt"
    ;;
  new-tab)
    name=""
    cwd=""
    while [[ "$#" -gt 0 ]]; do
      case "$1" in
        --name | -n)
          name="${2:-}"
          shift 2
          ;;
        --cwd | -c)
          cwd="${2:-}"
          shift 2
          ;;
        *)
          shift
          ;;
      esac
    done
    next_id="$(awk -F '\t' 'BEGIN { max = -1 } { if ($1 > max) max = $1 } END { print max + 1 }' "$state")"
    next_position="$(awk 'END { print NR }' "$state")"
    next_pane_id="$(awk -F '\t' 'BEGIN { max = 99 } { if ($1 > max) max = $1 } END { print max + 1 }' "$panes")"
    printf '%s\t%s\tfalse\t%s\n' "$next_id" "$next_position" "$name" >> "$state"
    printf '%s\t%s\t%s\ttrue\tzellij:status-bar\n' "$next_pane_id" "$next_id" "$name" >> "$panes"
    printf '%s\t%s\n' "$name" "$cwd" >> "${state}.cwds"
    printf '%s\n' "$next_id"
    ;;
  close-tab-by-id)
    target="${2:-}"
    awk -F '\t' -v target="$target" 'BEGIN { OFS = FS } $1 != target { print }' "$state" |
      awk -F '\t' 'BEGIN { OFS = FS } { $2 = NR - 1; print }' |
      write_state
    awk -F '\t' -v target="$target" 'BEGIN { OFS = FS } $2 != target { print }' "$panes" | write_panes
    ;;
  close-pane)
    target=""
    while [[ "$#" -gt 0 ]]; do
      case "$1" in
        --pane-id | -p)
          target="${2:-}"
          target="${target#plugin_}"
          shift 2
          ;;
        *)
          shift
          ;;
      esac
    done
    awk -F '\t' -v target="$target" 'BEGIN { OFS = FS } $1 != target { print }' "$panes" | write_panes
    ;;
  go-to-tab-by-id)
    target="${2:-}"
    awk -F '\t' -v target="$target" 'BEGIN { OFS = FS } { $3 = ($1 == target ? "true" : "false"); print }' "$state" | write_state
    ;;
  move-tab)
    direction="${2:-}"
    [[ "$direction" == "left" ]] || exit 0
    active_position="$(awk -F '\t' '$3 == "true" { print $2 }' "$state")"
    [[ "$active_position" =~ ^[0-9]+$ ]] || exit 0
    if (( active_position == 0 )); then
      exit 0
    fi
    previous_position=$((active_position - 1))
    awk -F '\t' -v active="$active_position" -v previous="$previous_position" '
      BEGIN { OFS = FS }
      $2 == active { $2 = previous; print; next }
      $2 == previous { $2 = active; print; next }
      { print }
    ' "$state" | write_state
    ;;
  rename-tab-by-id)
    target="${2:-}"
    name="${3:-}"
    awk -F '\t' -v target="$target" -v name="$name" 'BEGIN { OFS = FS } { if ($1 == target) $4 = name; print }' "$state" | write_state
    ;;
  save-session)
    touch "${state}.saved"
    ;;
  write-chars)
    shift
    pane_id=""
    if [[ "${1:-}" == "--pane-id" ]]; then
      pane_id="${2:-}"
      shift 2
    fi
    if [[ -n "${FAKE_ZELLIJ_WRITTEN_PANES:-}" ]]; then
      printf '%s\n' "$pane_id" >> "$FAKE_ZELLIJ_WRITTEN_PANES"
    fi
    printf '%s' "${1:-}" >> "${FAKE_ZELLIJ_WRITTEN_CHARS:?}"
    ;;
  send-keys)
    shift
    pane_id=""
    if [[ "${1:-}" == "--pane-id" ]]; then
      pane_id="${2:-}"
      shift 2
    fi
    if [[ -n "${FAKE_ZELLIJ_KEY_PANES:-}" ]]; then
      printf '%s\n' "$pane_id" >> "$FAKE_ZELLIJ_KEY_PANES"
    fi
    printf '%s\n' "$*" >> "${FAKE_ZELLIJ_SENT_KEYS:?}"
    ;;
esac
"#;
