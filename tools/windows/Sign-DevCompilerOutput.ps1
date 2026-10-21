param([Parameter(Mandatory)][string]$Executable, [Parameter(Mandatory)][string]$Source)
. (Join-Path $PSScriptRoot 'DevSigning.ps1')
$null = Set-DevExecutableSignature -Path $Executable -CompilerSource $Source
