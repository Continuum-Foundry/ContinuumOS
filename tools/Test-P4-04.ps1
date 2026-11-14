param([string]$Manifest='config/p4-04-validation.json',[string]$Resume,[switch]$Plan)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$env:RUST_TEST_THREADS='1'
$env:RUST_MIN_STACK='16777216'
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    $arguments=@('tools/p4_display_acceptance.py',$(if ($Plan) {'plan'} else {'run'}),'--manifest',$Manifest)
    if ($Resume) {$arguments+=@('--resume',$Resume)}
    & python @arguments
    if ($LASTEXITCODE -ne 0) {throw 'P4-04 failed; preserve the reported local run.'}
} finally {Pop-Location}
