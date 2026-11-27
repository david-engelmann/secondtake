# fish completion for secondtake
complete -c secondtake -n '__fish_use_subcommand' -a 'compare' -d 'secondtake compare'
complete -c secondtake -s h -l help -d 'Show help'
complete -c secondtake -n '__fish_seen_subcommand_from compare' -l 'registration' -r -F
complete -c secondtake -n '__fish_seen_subcommand_from compare' -l 'judge' -r -F
complete -c secondtake -n '__fish_seen_subcommand_from compare' -s 'o' -r -F
complete -c secondtake -n '__fish_seen_subcommand_from compare' -l 'output' -r -F
