# bash completion for secondtake
_secondtake() {
    local cur prev words cword
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "compare -h --help" -- "${cur}") )
        return 0
    fi
    case "${cur}" in
        -*)
            COMPREPLY=( $(compgen -W "--registration --judge -o --output -h --help" -- "${cur}") )
            return 0
            ;;
    esac
    COMPREPLY=( $(compgen -f -- "${cur}") )
}
complete -F _secondtake secondtake
