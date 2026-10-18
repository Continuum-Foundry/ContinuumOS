# Explicit acceptance: launches only bounded, offline QEMU processes via xtask.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$run = Join-Path $root ('build/m1/acceptance-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory $run | Out-Null
$results = [ordered]@{schema_version=1;status='running';commands=@();runs=@{}}
function Invoke-Check([string]$name, [string]$executable, [string[]]$arguments) {
    $log = Join-Path $run "$name.log"
    & $executable @arguments *> $log
    $exitCode = $LASTEXITCODE
    $results.commands += @{name=$name;executable=$executable;arguments=$arguments;exit_code=$exitCode;log="$name.log"}
    if ($exitCode -ne 0) { throw "$name failed ($exitCode); see $log" }
    return [IO.File]::ReadAllText($log)
}
Push-Location $root
try {
    $m0 = Invoke-Check m0 pwsh @('-NoProfile','-File',(Join-Path $PSScriptRoot 'Test-M0.ps1'))
    $results.runs.m0 = [regex]::Match($m0,'Acceptance evidence: ([^\r\n]+)').Groups[1].Value
    Invoke-Check doctor cargo @('xtask','doctor','--profile','m1','--json') | Out-Null
    $p101 = Invoke-Check prerequisites pwsh @('-NoProfile','-File',(Join-Path $PSScriptRoot 'Test-P1-01.ps1'))
    $results.runs.prerequisites = [regex]::Match($p101,'P1-01 test evidence: ([^\r\n]+)').Groups[1].Value
    foreach ($case in 'run','iso-smoke','iso-negative') {
        $arguments = if ($case -eq 'run') { @('xtask','run','--profile','vm-baseline') } else { @('xtask','test','--suite','m1','--case',$case) }
        $text = Invoke-Check $case cargo $arguments
        $path = [regex]::Match($text,'M1 artifacts: ([^\r\n]+)').Groups[1].Value
        if (-not $path) { throw "Missing artifact path for $case" }
        $result = Get-Content (Join-Path $path 'result.json') -Raw | ConvertFrom-Json
        if ($result.status -ne 'passed') { throw "$case did not pass" }
        $results.runs[$case] = $path
    }
    $results.status = 'passed'
} catch {
    $results.status = 'failed'
    $results['error'] = $_.Exception.Message
    throw
} finally {
    $results | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance-results.json')
    Pop-Location
    "P1-02 acceptance evidence: $run"
}
