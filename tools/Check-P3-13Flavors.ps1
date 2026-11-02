param([Parameter(Mandatory)][string]$Descriptors,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
& (Join-Path $PSScriptRoot 'Check-P3-12Flavors.ps1') -Descriptors $Descriptors -Output $Output
if ($LASTEXITCODE -ne 0) {throw 'Prior separate metadata build matrix failed.'}
$results=@(Get-Content (Join-Path $Output 'flavors.json') -Raw | ConvertFrom-Json)
$env:CONTINUUM_BUILD_ID='metadata-only-p3-13'
foreach ($flavor in @('stress-test','stress-trace','stress-supervisor-test')) {
    $arguments=@('check','--locked','--offline','-p','continuum-kernel','--bin','continuum-kernel','--target','x86_64-unknown-none','--profile','kernel-stress','--features',$flavor)
    & cargo @arguments *> (Join-Path $Output ($flavor+'.log'))
    $results+=@{flavor=$flavor;exit_code=$LASTEXITCODE;arguments=$arguments}
    $results | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Output 'flavors.json')
    if ($LASTEXITCODE -ne 0) {throw "Metadata compile failed: $flavor"}
}
