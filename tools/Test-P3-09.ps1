param([string]$Prerequisites,[string]$SourceManifest)
# Final-source local acceptance. Serial workloads, original VM limits, no retries.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE = 'true'
# Keep independent host test cases from contending with the 3-second child-start fixture.
$env:RUST_TEST_THREADS = '1'
$env:RUST_MIN_STACK = '16777216'
$run = Join-Path $root ('build/m3/P3-09/acceptance-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
$report = [ordered]@{schema_version=1;story='P3-09';status='running';commands=@();runs=@{};source_commit=(git rev-parse HEAD);user_execution=$true;host_test_threads=1}
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
    if ($Prerequisites) {
        if (-not $SourceManifest) { throw 'Prerequisite reuse requires the exact prior M1 source manifest.' }
        $prerequisiteDirectory=(Resolve-Path -LiteralPath $Prerequisites).Path
        $sourceManifestPath=(Resolve-Path -LiteralPath $SourceManifest).Path
        $previous=Get-Content (Join-Path $prerequisiteDirectory 'prerequisites.json') -Raw | ConvertFrom-Json
        if ($previous.status -ne 'passed' -or -not $previous.offline -or $previous.commands.Count -ne 10 -or @($previous.commands | Where-Object exit_code -ne 0).Count -ne 0) { throw 'Cannot reuse an incomplete prerequisite run.' }
        $priorInputs=@{}
        foreach ($inputFile in @(Get-Content (Join-Path $prerequisiteDirectory 'source-inputs.json') -Raw | ConvertFrom-Json)) { $priorInputs[$inputFile.path]=$inputFile.sha256 }
        $measured=Get-Content -LiteralPath $sourceManifestPath -Raw | ConvertFrom-Json -AsHashtable
        if ($measured.Count -eq 0) { throw 'Empty prior source manifest.' }
        foreach ($inputPath in $measured.Keys) {
            $absolute=[IO.Path]::GetFullPath((Join-Path $root $inputPath))
            if (-not $absolute.StartsWith($root+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) { throw 'Source manifest escaped workspace.' }
            $actual=(Get-FileHash -LiteralPath $absolute -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($actual -ne $measured[$inputPath] -or $actual -ne $priorInputs[$inputPath]) { throw "Changed prerequisite input: $inputPath" }
        }
        $report.runs.prerequisites=$prerequisiteDirectory
        $report['prerequisite_reuse']=@{source_manifest=$sourceManifestPath;source_manifest_sha256=(Get-FileHash $sourceManifestPath -Algorithm SHA256).Hash.ToLowerInvariant();validated_files=$measured.Count;directory=$prerequisiteDirectory;reason='identical measured source/tool/configuration inputs; subsequent documentation-only audit-count fix'}
        $report.commands+=@{name='prerequisites';program='validated-local-reference';arguments=@($prerequisiteDirectory,$sourceManifestPath);exit_code=0;elapsed_ms=0}
    } else {
        $pre=Check prerequisites pwsh @('-NoProfile','-File','tools/Recheck-P2Prerequisites.ps1')
        $report.runs.prerequisites=Extract $pre 'Prerequisite evidence: ([^\r\n]+)'
    }
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
    $objects=Check capabilities cargo @('xtask','test','--suite','m1','--case','capabilities')
    $report.runs.capabilities=Extract $objects 'M1 artifacts: ([^\r\n]+)'
    $channels=Check channels cargo @('xtask','test','--suite','m1','--case','channels')
    $report.runs.channels=Extract $channels 'M1 artifacts: ([^\r\n]+)'
    $waits=Check waits cargo @('xtask','test','--suite','m1','--case','waits')
    $report.runs.waits=Extract $waits 'M1 artifacts: ([^\r\n]+)'
    $supervisor=Check supervisor cargo @('xtask','test','--suite','m1','--case','supervisor')
    $report.runs.supervisor=Extract $supervisor 'M1 artifacts: ([^\r\n]+)'
    $report.status='passed'
} catch {
    $report.status='failed'
    $report['error']=$_.Exception.Message
    throw
} finally {
    $report | ConvertTo-Json -Depth 16 | Set-Content (Join-Path $run 'acceptance.json')
    Pop-Location
    "P3-09 acceptance: $run/acceptance.json"
}
