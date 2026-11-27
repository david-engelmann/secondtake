# PowerShell completion for secondtake
Register-ArgumentCompleter -Native -CommandName secondtake -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $candidates = @('compare', '--registration', '--judge', '-o', '--output', '-h', '--help')
    $candidates | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
