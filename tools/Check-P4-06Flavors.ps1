param([Parameter(Mandatory)][string]$Descriptors,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
& (Join-Path $PSScriptRoot 'Check-P4-05Flavors.ps1') -Descriptors $Descriptors -Output $Output
if($LASTEXITCODE -ne 0){throw 'Prior separate flavor compile failed'}
foreach($flavor in @('input-runtime','input-test')) {
    $arguments=@('check','--locked','--offline','-p','continuum-kernel','--bin','continuum-kernel','--target','x86_64-unknown-none','--profile','kernel-dev','--features',$flavor)
    & cargo @arguments *> (Join-Path $Output "$flavor.log")
    $code=$LASTEXITCODE
    $rows=@(Get-Content (Join-Path $Output 'flavors.json') -Raw | ConvertFrom-Json)
    $rows+=@{flavor=$flavor;arguments=$arguments;exit_code=$code}
    $rows | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Output 'flavors.json')
    if($code -ne 0){throw "Input flavor $flavor failed"}
}
