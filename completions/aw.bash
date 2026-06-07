_aw_completion() {
  local cur prev config_dir default_profile profile_name
  COMPREPLY=()
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"

  _aw_config_dir() {
    if [[ -n "${AW_CONFIG_DIR:-}" && -f "$AW_CONFIG_DIR/profile.conf" ]]; then
      printf '%s' "$AW_CONFIG_DIR"
      return
    fi
    if [[ -f "$PWD/config/aw/profile.conf" ]]; then
      printf '%s' "$PWD/config/aw"
      return
    fi
    default_profile="$HOME/.local/share/agent-workspace/default-profile"
    if [[ -f "$default_profile" ]]; then
      profile_name="$(sed -n '1p' "$default_profile")"
      printf '%s/.local/share/agent-workspace/profiles/%s' "$HOME" "$profile_name"
    fi
  }

  _aw_workspaces() {
    local tabs_file dir
    dir="$(_aw_config_dir)"
    [[ -d "$dir" ]] || return
    for tabs_file in "$dir"/*.tabs; do
      [[ -f "$tabs_file" ]] || continue
      basename "$tabs_file" .tabs
    done
  }

  _aw_tabs() {
    local workspace="$1" dir tabs_file
    dir="$(_aw_config_dir)"
    tabs_file="$dir/$workspace.tabs"
    [[ -f "$tabs_file" ]] || return
    awk -F '\t' '{ print $1 }' "$tabs_file"
  }

  _aw_file_replies() {
    local file
    while IFS= read -r file; do
      COMPREPLY+=("$file")
    done < <(compgen -f -- "$cur")
  }

  case "${COMP_WORDS[1]:-}" in
    commit)
      if [[ "$COMP_CWORD" -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "setup add status doctor wait poke" -- "$cur") )
      else
        case "${COMP_WORDS[2]:-}" in
          setup)
            if [[ "$COMP_CWORD" -eq 3 ]]; then
              COMPREPLY=( $(compgen -W "$(_aw_workspaces) --tab --session --agent --no-agent" -- "$cur") )
            else
              COMPREPLY=( $(compgen -W "--tab --session --agent --no-agent" -- "$cur") )
            fi
            ;;
          add)
            COMPREPLY=( $(compgen -W "--check --verify --root --summary --owner --must-contain --must-not-contain --poke --wait --timeout --poll" -- "$cur") )
            _aw_file_replies
            ;;
          status|doctor)
            COMPREPLY=( $(compgen -W "--root" -- "$cur") )
            ;;
          wait)
            COMPREPLY=( $(compgen -W "--root --timeout --poll" -- "$cur") )
            ;;
          poke)
            COMPREPLY=( $(compgen -W "git --root" -- "$cur") )
            ;;
        esac
      fi
      ;;
    tab)
      case "${COMP_WORDS[2]:-}" in
        "")
          COMPREPLY=( $(compgen -W "list add move rename remove refresh" -- "$cur") )
          ;;
        list|refresh)
          [[ "$COMP_CWORD" -eq 3 ]] && COMPREPLY=( $(compgen -W "$(_aw_workspaces)" -- "$cur") )
          ;;
        add|move|remove)
          if [[ "$COMP_CWORD" -eq 3 ]]; then
            COMPREPLY=( $(compgen -W "$(_aw_workspaces)" -- "$cur") )
          elif [[ "$COMP_CWORD" -eq 4 ]]; then
            COMPREPLY=( $(compgen -W "$(_aw_tabs "${COMP_WORDS[3]}")" -- "$cur") )
          fi
          ;;
        rename)
          if [[ "$COMP_CWORD" -eq 3 ]]; then
            COMPREPLY=( $(compgen -W "$(_aw_workspaces)" -- "$cur") )
          elif [[ "$COMP_CWORD" -eq 4 ]]; then
            COMPREPLY=( $(compgen -W "$(_aw_tabs "${COMP_WORDS[3]}")" -- "$cur") )
          fi
          ;;
      esac
      ;;
    refresh|remove|rename)
      [[ "$COMP_CWORD" -eq 2 ]] && COMPREPLY=( $(compgen -W "$(_aw_workspaces)" -- "$cur") )
      ;;
    doctor)
      [[ "$COMP_CWORD" -eq 2 ]] && COMPREPLY=( $(compgen -W "repo --config" -- "$cur") )
      ;;
    migrate)
      if [[ "$COMP_CWORD" -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "repo" -- "$cur") )
      elif [[ "${COMP_WORDS[2]:-}" == "repo" ]]; then
        COMPREPLY=( $(compgen -W "--dry-run" -- "$cur") )
      fi
      ;;
    *)
      if [[ "$COMP_CWORD" -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "help install setup doctor migrate list create refresh rename remove tab commit ps kill $(_aw_workspaces)" -- "$cur") )
      elif [[ "$COMP_CWORD" -gt 1 ]]; then
        COMPREPLY=( $(compgen -W "-s --session -r --root" -- "$cur") )
      fi
      ;;
  esac
}

complete -F _aw_completion aw
