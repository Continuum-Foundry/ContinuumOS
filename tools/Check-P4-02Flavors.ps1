param([Parameter(Mandatory)][string]$Descriptors,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CONTINUUM_TRUSTED_IMAGES=(Resolve-Path (Join-Path $Descriptors 'trusted-images.rs')).Path
$env:CONTINUUM_RUNTIME_IDS=(Resolve-Path (Join-Path $Descriptors 'runtime-ids.rs')).Path
$env:CONTINUUM_BUILD_ID='p4-02-metadata-only-never-booted'
New-Item -ItemType Directory -Force $Output | Out-Null
$rows=@()
foreach ($flavor in @('ordinary-minimal','desktop-runtime','desktop-test','user-boundary-test','scheduler-test','capability-test','channel-test','wait-test','supervisor-test','policy-test','route-test','sdk-test','sdk-runtime-test','stress-test','stress-trace','stress-supervisor-test')) {
    $arguments=@('check','--locked','--offline','-p','continuum-kernel','--bin','continuum-kernel','--target','x86_64-unknown-none','--profile','kernel-dev')
    if ($flavor -ne 'ordinary-minimal') {$arguments+=@('--features',$flavor)}
    & cargo @arguments *> (Join-Path $Output ($flavor+'.log'))
    $code=$LASTEXITCODE
    $rows+=@{flavor=$flavor;arguments=$arguments;exit_code=$code}
    $rows | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Output 'flavors.json')
    if ($code -ne 0) {throw "Metadata check failed: $flavor"}
}
