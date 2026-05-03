$ErrorActionPreference = 'Stop'

$DocsRoot = Split-Path -Parent $PSScriptRoot
$Input = Join-Path $PSScriptRoot 'thesis.typo'
$Output = Join-Path $PSScriptRoot 'winsentials-thesis.pdf'

typst compile --root $DocsRoot $Input $Output
