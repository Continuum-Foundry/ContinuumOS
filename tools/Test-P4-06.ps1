param([string]$Manifest='config/p4-06-validation.json',[string]$Resume,[switch]$Plan,[switch]$Probe)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true';$env:RUST_TEST_THREADS='1';$env:RUST_MIN_STACK='16777216'
$arguments=@('tools/p4_input_acceptance.py',$(if($Plan){if($Probe){'probe-plan'}else{'plan'}}elseif($Probe){'probe'}else{'run'}),'--manifest',$Manifest)
if($Resume){$arguments+=@('--resume',$Resume)}
& python @arguments
if($LASTEXITCODE -ne 0){throw 'P4-06 failed; retain the reported local attempt.'}
