# Final-source local acceptance. Serial workloads, original VM limits, no retries.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE = 'true'
# Keep independent host test cases from contending with the 3-second child-start fixture.
$env:RUST_TEST_THREADS = '1'
$run = Join-Path $root ('build/m3/P3-05/acceptance-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
$report = [ordered]@{schema_version=1;story='P3-05';status='running';commands=@();runs=@{};source_commit=(git rev-parse HEAD);user_execution=$true;host_test_threads=1}
function Check([string]$name,[string]$exe,[string[]]$arguments) {
    $timer=[Diagnostics.Stopwatch]::StartNew()
    & $exe @arguments *> (Join-Path $run ($name+'.log'))
    $code=$LASTEXITCODE
    $report.commands+=@{name=$name;program=$exe;arguments=$arguments;exit_code=$code;elapsed_ms=$timer.ElapsedMilliseconds}
    if ($code -ne 0) { throw "$name failed; see $run" }
    [IO.File]::ReadAllText((Join-Path $run ($name+'.log')))
}
function Extract([string]$text,[string]$pattern) {
    $match=[regex]::Match($text,$pattern)
    if (-not $match.Success) { throw "Missing artifact path: $pattern" }
    $match.Groups[1].Value.Trim()
}
Push-Location $root
try {
    git status --porcelain=v1 --untracked-files=all | Set-Content (Join-Path $run 'source-status.txt')
    git diff --binary | Set-Content (Join-Path $run 'source.patch')
    $inputs=foreach ($path in @(git ls-files --cached --others --exclude-standard | Sort-Object -Unique)) {
        if (Test-Path -LiteralPath $path -PathType Leaf) { @{path=$path;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()} }
    }
    ConvertTo-Json -InputObject @($inputs) -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    $pre=Check prerequisites pwsh @('-NoProfile','-File','tools/Recheck-P2Prerequisites.ps1')
    $report.runs.prerequisites=Extract $pre 'Prerequisite evidence: ([^\r\n]+)'
    $baseline=Check m1 pwsh @('-NoProfile','-File','tools/Test-P1-07.ps1')
    $report.runs.m1=Extract $baseline 'M1 artifacts: ([^\r\n]+)'
    Check m1-review pwsh @('-NoProfile','-File','tools/Review-P1-07.ps1','-Run',$report.runs.m1) | Out-Null
    $memory=Check owned-memory cargo @('xtask','test','--suite','m1','--case','owned-memory')
    $report.runs.memory=Extract $memory 'M1 artifacts: ([^\r\n]+)'
    $loader=Check static-loader cargo @('xtask','test','--suite','m1','--case','static-loader')
    $report.runs.loader=Extract $loader 'M1 artifacts: ([^\r\n]+)'
    $boundary=Check user-boundary cargo @('xtask','test','--suite','m1','--case','user-boundary')
    $report.runs.boundary=Extract $boundary 'M1 artifacts: ([^\r\n]+)'
    $scheduled=Check scheduler cargo @('xtask','test','--suite','m1','--case','scheduler')
    $report.runs.scheduler=Extract $scheduled 'M1 artifacts: ([^\r\n]+)'
    $report.status='passed'
} catch {
    $report.status='failed'
    $report['error']=$_.Exception.Message
    throw
} finally {
    $report | ConvertTo-Json -Depth 16 | Set-Content (Join-Path $run 'acceptance.json')
    Pop-Location
    "P3-05 acceptance: $run/acceptance.json"
}
