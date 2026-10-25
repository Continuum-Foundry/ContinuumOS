param([Parameter(Mandatory=$true)][string]$Baseline)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
Push-Location $root
try {
    & python tools/Run-UiPerformanceRemediation.py $Baseline
    if ($LASTEXITCODE) {throw 'UI remediation failed; retain the printed local run directory'}
} finally {Pop-Location}
