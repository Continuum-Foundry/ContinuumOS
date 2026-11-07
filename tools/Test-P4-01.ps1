param()
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$env:RUST_TEST_THREADS='1'
$env:RUST_MIN_STACK='16777216'
Push-Location $root
try {
    & python tools/p4_acceptance.py run
    if ($LASTEXITCODE -ne 0) {throw 'P4-01 failed; preserve the printed local run.'}
} finally {Pop-Location}
