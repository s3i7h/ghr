#!/usr/bin/env bash

__ghr_complete__static() {
  local options

  options=("${@:2}")

  compgen -W "${options[*]}" -- "$1"
}

__ghr_complete__repos() {
  local repositories suggestions

  repositories="$(ghr list)"
  suggestions="$(compgen -W "${repositories[*]}" -- "$1")"

  if [[ $1 != -* && ${COMP_CWORD} -ne 2 ]]; then
    return
  fi

  echo "$suggestions"
}

__ghr_complete__profiles() {
  local profiles suggestions

  profiles="$(ghr profile list --short)"
  suggestions="$(compgen -W "${profiles[*]}" -- "$1")"

  if [[ $1 != -* && ${COMP_CWORD} -ne 3 ]]; then
    return
  fi

  echo "$suggestions"
}

__ghr_complete() {
  local cword

  # Replaces ':' in $COMP_WORDBREAKS to prevent bash appends the suggestion after ':' repeatedly
  COMP_WORDBREAKS=${COMP_WORDBREAKS//:/}

  cword="${COMP_WORDS[COMP_CWORD]}"

  if [ "${COMP_CWORD}" = 1 ]; then
    COMPREPLY=($(__ghr_complete__static "${cword}" --help add browse cd clone delete help init list open path profile shell sync version))
    return 0
  fi

  case "${COMP_WORDS[1]}" in
  add)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  browse)
    COMPREPLY=($(__ghr_complete__repos "${cword}"))
    ;;
  cd)
    COMPREPLY=($(__ghr_complete__repos "${cword}"))
    ;;
  clone)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  delete)
    COMPREPLY=($(__ghr_complete__repos "${cword}"))
    ;;
  init)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  list)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help --no-host --no-owner -p --path))
    ;;
  open)
    if [ "${COMP_CWORD}" = 2 ]; then
      COMPREPLY=($(__ghr_complete__repos "${cword}" --help))
    elif [ "${COMP_CWORD}" = 3 ]; then
      # Complete a known command to open the repository using
      COMPREPLY=($(compgen -c -- "${cword}"))
    fi
    ;;
  path)
    COMPREPLY=($(__ghr_complete__repos "${cword}"))
    ;;
  profile)
    if [ "$COMP_CWORD" = 2 ]; then
      COMPREPLY=($(__ghr_complete__static "${cword}" --help list show apply))
    else
      case "${COMP_WORDS[2]}" in
      show|apply)
        COMPREPLY=($(__ghr_complete__profiles "${cword}" --help))
        ;;
      *)
        ;;
      esac
    fi
    ;;
  search)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  shell)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  sync)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help dump restore))
    ;;
  version)
    COMPREPLY=($(__ghr_complete__static "${cword}" --help))
    ;;
  help)
    COMPREPLY=()
    ;;
  *)
    ;;
  esac
}

# complete is a bash builtin, but recent versions of ZSH come with a function
# called bashcompinit that will create a complete in ZSH. If the user is in
# ZSH, load and run bashcompinit before calling the complete function.
if [[ -n ${ZSH_VERSION-} ]]; then
  # First calling compinit (only if not called yet!)
  # and then bashcompinit as mentioned by zsh man page.
  if ! command -v compinit >/dev/null; then
    autoload -U +X compinit && if [[ ${ZSH_DISABLE_COMPFIX-} = true ]]; then
      compinit -u
    else
      compinit
    fi
  fi
  autoload -U +X bashcompinit && bashcompinit
fi

complete -F __ghr_complete -o bashdefault -o default ghr
