param([switch]$Replay, [int]$TimeoutSeconds=120)
$ErrorActionPreference='Stop'
& (Join-Path $PSScriptRoot 'Preview-Custom.ps1') -Candidate Slint -Replay:$Replay -TimeoutSeconds $TimeoutSeconds
