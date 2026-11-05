param()
# Serial final-source gate. No historical-run reuse or automatic retries.
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$env:RUST_TEST_THREADS='1'
$env:RUST_MIN_STACK='16777216'
Push-Location $root
try {
    & python tools/p3_acceptance.py run
    if ($LASTEXITCODE -ne 0) {throw 'P3-14 failed; preserve the reported local run.'}
} finally {Pop-Location}
