param()
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
Push-Location $root
try {
    & python tools/Run-P2Acceptance.py
    if ($LASTEXITCODE) {throw 'P2-13 acceptance failed; inspect the printed local run directory'}
} finally {Pop-Location}
