param([string]$Manifest='config/p4-02-validation.json',[string]$Resume,[switch]$FullGate)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$env:RUST_TEST_THREADS='1'
$env:RUST_MIN_STACK='16777216'
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    if ($FullGate) {
        if ($Resume) {throw 'The original full gate does not reuse or resume earlier attempts.'}
        $arguments=@('tools/p4_runtime_full_acceptance.py','run')
    } else {
        $arguments=@('tools/p4_runtime_acceptance.py','run','--manifest',$Manifest)
        if ($Resume) {$arguments+=@('--resume',$Resume)}
    }
    & python @arguments
    if ($LASTEXITCODE -ne 0) {throw 'P4-02 failed; preserve the reported local run.'}
} finally {Pop-Location}
