param([Parameter(Mandatory)][string]$Descriptors,[Parameter(Mandatory)][string]$Output)
# Metadata compilation only. Real xtask cases still build/package/execute guests.
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CONTINUUM_TRUSTED_IMAGES=(Resolve-Path (Join-Path $Descriptors 'trusted-images.rs')).Path
$env:CONTINUUM_RUNTIME_IDS=(Resolve-Path (Join-Path $Descriptors 'runtime-ids.rs')).Path
$env:CONTINUUM_BUILD_ID='metadata-only-p3-10' # never packaged or booted
New-Item -ItemType Directory -Force $Output | Out-Null
$results=@()
foreach ($flavor in @('ordinary','user-boundary-test','scheduler-test','capability-test','channel-test','wait-test','supervisor-test','policy-test')) {
    $arguments=@('check','--locked','--offline','-p','continuum-kernel','--bin','continuum-kernel','--target','x86_64-unknown-none','--profile','kernel-dev')
    if ($flavor -ne 'ordinary') {$arguments+=@('--features',$flavor)}
    & cargo @arguments *> (Join-Path $Output ($flavor+'.log'))
    $results+=@{flavor=$flavor;exit_code=$LASTEXITCODE;arguments=$arguments}
    if ($LASTEXITCODE -ne 0) { $results | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Output 'flavors.json');throw "Compile failed: $flavor" }
}
$results | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Output 'flavors.json')
