_aw_complete() {
  local current previous
  current="${COMP_WORDS[COMP_CWORD]}"
  previous="${COMP_WORDS[COMP_CWORD-1]}"

  if [[ ${COMP_CWORD} -eq 1 ]]; then
    COMPREPLY=($(compgen -W "commit doctor help install owner paths repo" -- "$current"))
    return
  fi

  case "${COMP_WORDS[1]}" in
    commit)
      if [[ ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=($(compgen -W "block check doctor done help list next poke raw-request request setup status wait" -- "$current"))
      elif [[ "$previous" == "poke" || "$previous" == "--tab" ]]; then
        COMPREPLY=($(compgen -W "git" -- "$current"))
      else
        COMPREPLY=($(compgen -W "--check --owner --poke --poll --queue-root --root --summary --tab --timeout --verify --wait" -- "$current"))
      fi
      ;;
    install)
      COMPREPLY=($(compgen -W "--dry-run --repo" -- "$current"))
      ;;
    owner)
      COMPREPLY=($(compgen -W "git pkg" -- "$current"))
      ;;
    repo)
      COMPREPLY=($(compgen -W "clean doctor measure-git migrate probe-git-config routes worktree" -- "$current"))
      ;;
  esac
}

complete -F _aw_complete aw
