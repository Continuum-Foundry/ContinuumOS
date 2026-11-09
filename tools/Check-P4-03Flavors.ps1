param([Parameter(Mandatory)][string]$Descriptors,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
& (Join-Path $PSScriptRoot 'Check-P4-02Flavors.ps1') -Descriptors $Descriptors -Output $Output
if ($LASTEXITCODE -ne 0) {throw 'Historical flavor compile failed'}
$env:CONTINUUM_TRUSTED_IMAGES=(Resolve-Path (Join-Path $Descriptors 'trusted-images.rs')).Path
$env:CONTINUUM_RUNTIME_IDS=(Resolve-Path (Join-Path $Descriptors 'runtime-ids.rs')).Path
$env:CONTINUUM_BUILD_ID='p4-03-metadata-only-never-booted'
$arguments=@('check','--locked','--offline','-p','continuum-kernel','--bin','continuum-kernel','--target','x86_64-unknown-none','--profile','kernel-dev','--features','device-test')
& cargo @arguments *> (Join-Path $Output 'device-test.log')
$code=$LASTEXITCODE
$rows=@(Get-Content (Join-Path $Output 'flavors.json') -Raw | ConvertFrom-Json)
$rows+=@{flavor='device-test';arguments=$arguments;exit_code=$code}
$rows | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Output 'flavors.json')
if ($code -ne 0) {throw 'Device flavor compile failed'}
