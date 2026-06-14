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
    local aw_home="${AW_HOME:-$HOME/.aw}"
    default_profile="$aw_home/default-profile"
    if [[ -f "$default_profile" ]]; then
      profile_name="$(sed -n '1p' "$default_profile")"
      printf '%s/profiles/%s' "$aw_home" "$profile_name"
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

  _aw_single_workspace() {
    local workspace single count
    count=0
    while IFS= read -r workspace; do
      [[ -n "$workspace" ]] || continue
      single="$workspace"
      count=$((count + 1))
    done < <(_aw_workspaces)
    [[ "$count" -eq 1 ]] && printf '%s' "$single"
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

  _aw_complete_tab_command() {
    local workspace="$1" action="$2" first_arg_index="$3" single_workspace
    if [[ -z "$action" ]]; then
      COMPREPLY=( $(compgen -W "list add move rename remove refresh" -- "$cur") )
      return
    fi

    if [[ "$cur" == --* ]]; then
      COMPREPLY=( $(compgen -W "--session" -- "$cur") )
      return
    fi

    case "$action" in
      list|refresh)
        if [[ -z "$workspace" && "$COMP_CWORD" -eq "$first_arg_index" && -z "$(_aw_single_workspace)" ]]; then
          COMPREPLY=( $(compgen -W "$(_aw_workspaces)" -- "$cur") )
        fi
        ;;
      add|move|remove|rename)
        if [[ -n "$workspace" ]]; then
          [[ "$COMP_CWORD" -eq "$first_arg_index" ]] && COMPREPLY=( $(compgen -W "$(_aw_tabs "$workspace")" -- "$cur") )
          return
        fi
        single_workspace="$(_aw_single_workspace)"
        if [[ "$COMP_CWORD" -eq "$first_arg_index" && -n "$single_workspace" ]]; then
          COMPREPLY=( $(compgen -W "$(_aw_tabs "$single_workspace")" -- "$cur") )
        elif [[ "$COMP_CWORD" -eq "$first_arg_index" ]]; then
          COMPREPLY=( $(compgen -W "$(_aw_workspaces)" -- "$cur") )
        elif [[ "$COMP_CWORD" -eq $((first_arg_index + 1)) && -z "$single_workspace" ]]; then
          COMPREPLY=( $(compgen -W "$(_aw_tabs "${COMP_WORDS[$first_arg_index]}")" -- "$cur") )
        fi
        ;;
    esac
  }

  case "${COMP_WORDS[1]:-}" in
    commit)
      if [[ "$COMP_CWORD" -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "setup request status doctor wait poke" -- "$cur") )
      else
        case "${COMP_WORDS[2]:-}" in
          setup)
            if [[ "$COMP_CWORD" -eq 3 ]]; then
              COMPREPLY=( $(compgen -W "$(_aw_workspaces) --tab --session --agent --no-agent" -- "$cur") )
            else
              COMPREPLY=( $(compgen -W "--tab --session --agent --no-agent" -- "$cur") )
            fi
            ;;
          request)
            COMPREPLY=( $(compgen -W "--check --verify --queue-root --root --summary --owner --must-contain --must-not-contain --poke --workspace --session --wait --timeout --poll" -- "$cur") )
            _aw_file_replies
            ;;
          status|doctor)
            COMPREPLY=( $(compgen -W "--queue-root --root" -- "$cur") )
            ;;
          wait)
            COMPREPLY=( $(compgen -W "--queue-root --root --timeout --poll" -- "$cur") )
            ;;
          poke)
            COMPREPLY=( $(compgen -W "git --queue-root --root --workspace --session" -- "$cur") )
            ;;
        esac
      fi
      ;;
    session)
      [[ "$COMP_CWORD" -eq 2 ]] && COMPREPLY=( $(compgen -W "name" -- "$cur") )
      [[ "$COMP_CWORD" -eq 3 && "${COMP_WORDS[2]:-}" == "name" ]] && COMPREPLY=( $(compgen -W "$(_aw_workspaces)" -- "$cur") )
      ;;
    repo)
      if [[ "$COMP_CWORD" -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "doctor migrate clean measure-git probe-git-config routes worktree" -- "$cur") )
      else
        case "${COMP_WORDS[2]:-}" in
          migrate)
            COMPREPLY=( $(compgen -W "--dry-run" -- "$cur") )
            ;;
          clean)
            COMPREPLY=( $(compgen -W "--delete --generated --rust-targets --nested-node-modules --all-safe --build-outputs --preprocessed" -- "$cur") )
            ;;
          probe-git-config)
            COMPREPLY=( $(compgen -W "--path --apply" -- "$cur") )
            ;;
          routes)
            COMPREPLY=( $(compgen -W "doctor --config" -- "$cur") )
            ;;
          worktree)
            COMPREPLY=( $(compgen -W "--branch --base --skip-deps --copy-deps" -- "$cur") )
            _aw_file_replies
            ;;
        esac
      fi
      ;;
    owner)
      [[ "$COMP_CWORD" -eq 2 ]] && COMPREPLY=( $(compgen -W "git pkg" -- "$cur") )
      ;;
    tab)
      _aw_complete_tab_command "" "${COMP_WORDS[2]:-}" 3
      ;;
    refresh|remove|rename)
      [[ "$COMP_CWORD" -eq 2 ]] && COMPREPLY=( $(compgen -W "$(_aw_workspaces)" -- "$cur") )
      ;;
    doctor)
      [[ "$COMP_CWORD" -eq 2 ]] && COMPREPLY=( $(compgen -W "repo --config" -- "$cur") )
      ;;
    *)
      if [[ "${COMP_WORDS[2]:-}" == "tab" ]]; then
        _aw_complete_tab_command "${COMP_WORDS[1]}" "${COMP_WORDS[3]:-}" 4
      elif [[ "$COMP_CWORD" -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "help install setup doctor paths repo list create refresh rename remove tab session commit owner ps kill $(_aw_workspaces)" -- "$cur") )
      elif [[ "$COMP_CWORD" -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "tab -s --session -r --root" -- "$cur") )
      elif [[ "$COMP_CWORD" -gt 1 ]]; then
        COMPREPLY=( $(compgen -W "-s --session -r --root" -- "$cur") )
      fi
      ;;
  esac
}

complete -F _aw_completion aw
