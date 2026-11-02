param([string]$RuntimeAcceptance)
# Final-source serial acceptance. Fixed deadlines and counts; no automatic retries.
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$env:RUST_TEST_THREADS='1'
$env:RUST_MIN_STACK='16777216'
$run=Join-Path $root ('build/m3/P3-13/acceptance-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
$report=[ordered]@{schema_version=1;story='P3-13';status='running';source_commit=(git rev-parse HEAD);native_observer_profile='release';commands=@();runs=@{};stress=@()}
function Check([string]$name,[string]$program,[string[]]$arguments) {
    $timer=[Diagnostics.Stopwatch]::StartNew()
    & $program @arguments *> (Join-Path $run ($name+'.log'))
    $report.commands+=@{name=$name;program=$program;arguments=$arguments;exit_code=$LASTEXITCODE;elapsed_ms=$timer.ElapsedMilliseconds}
    if ($LASTEXITCODE -ne 0) {throw "Failed $name; see $run"}
    [IO.File]::ReadAllText((Join-Path $run ($name+'.log')))
}
Push-Location $root
try {
    $profile=Get-Content config/p3-13-stress.json -Raw | ConvertFrom-Json
    if ($profile.seed -ne 20261025 -or $profile.quantum_ns -ne 5000000 -or $profile.fairness_dispatch_wake_irq_off_ceiling_ns -ne 320000000 -or $profile.ipc_round_trip_deadline_ns -ne 320000000 -or $profile.repetitions -ne 5 -or $profile.waves_per_continuous_guest -ne 100 -or $profile.measured_round_trips_per_guest -ne 10000 -or $profile.warmup_round_trips_per_guest -ne 1000 -or $profile.guest_deadline_ms -ne 30000 -or $profile.retained_frame_tolerance -ne 0 -or $profile.retained_object_tolerance -ne 0) {throw 'Stress profile diverged from fixed experiment/reviewer.'}
    git status --porcelain=v1 --untracked-files=all | Set-Content (Join-Path $run 'source-status.txt')
    git diff --binary | Set-Content (Join-Path $run 'source.patch')
    $inputs=@(foreach ($path in @(git ls-files --cached --others --exclude-standard | Sort-Object -Unique)) {if (Test-Path -LiteralPath $path -PathType Leaf) {@{path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant()}}})
    ConvertTo-Json -InputObject $inputs -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    if ($RuntimeAcceptance) {
        $priorPath=(Resolve-Path -LiteralPath $RuntimeAcceptance).Path
        if (-not $priorPath.StartsWith($root+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {throw 'Regression reference must stay in workspace.'}
        $priorData=Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json
        if ($priorData.story -ne 'P3-12' -or $priorData.status -ne 'passed' -or $priorData.commands.Count -ne 19 -or @($priorData.commands | Where-Object exit_code -ne 0).Count -ne 0) {throw 'Incomplete regression reference.'}
        $sourceManifest=Join-Path $priorData.runs.m1 'suite-source-inputs.json'
        $measured=Get-Content -LiteralPath $sourceManifest -Raw | ConvertFrom-Json -AsHashtable
        $sourcePaths=@{}
        foreach ($directory in @('kernel','tools/xtask','tools/windows','tools/linux','experiments/user-images','sdk/continuum-rt','experiments/sdk-client','experiments/sdk-service','experiments/sdk-stress')) {
            foreach ($file in @(Get-ChildItem -LiteralPath (Join-Path $root $directory) -Recurse -File -Force)) {$sourcePaths[[IO.Path]::GetRelativePath($root,$file.FullName).Replace('\','/')]=$file.FullName}
        }
        foreach ($path in $measured.Keys) {if (-not $sourcePaths.ContainsKey($path)) {$sourcePaths[$path]=Join-Path $root $path}}
        if ($sourcePaths.Count -ne $measured.Count) {throw 'Regression source set changed.'}
        foreach ($path in $sourcePaths.Keys) {
            if (-not $measured.ContainsKey($path) -or (Get-FileHash -LiteralPath $sourcePaths[$path]).Hash.ToLowerInvariant() -ne $measured[$path]) {throw "Changed regression input: $path"}
        }
        $priorSdk=Get-Content (Join-Path $priorData.runs.sdk 'build-inputs.json') -Raw | ConvertFrom-Json
        $compilerRoot=(& rustc --print sysroot | Out-String).Trim()
        if ($LASTEXITCODE -ne 0 -or (Get-FileHash (Join-Path $compilerRoot ('bin/rustc'+$script:ContinuumExeSuffix))).Hash.ToLowerInvariant() -ne $priorSdk.rustc_sha256) {throw 'Regression compiler changed.'}
        $report['regression_reuse']=@{acceptance_sha256=(Get-FileHash -LiteralPath $priorPath).Hash.ToLowerInvariant();source_manifest=$sourceManifest;source_manifest_sha256=(Get-FileHash -LiteralPath $sourceManifest).Hash.ToLowerInvariant();validated_files=$measured.Count;reason='identical complete measured source set and compiler; native observer build optimized without changing validation policy or guest inputs'}
        $report.commands+=@{name='runtime-regressions';program='validated-local-reference';arguments=@($priorPath,$sourceManifest);exit_code=0;elapsed_ms=0}
    } else {
        $prior=Check runtime-regressions pwsh @('-NoProfile','-File','tools/Test-P3-12.ps1')
        $match=[regex]::Match($prior,'P3-12 acceptance: ([^\r\n]+)')
        if (-not $match.Success) {throw 'Missing prior runtime acceptance path.'}
        $priorPath=$match.Groups[1].Value.Trim()
    }
    $previous=Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json
    if ($previous.status -ne 'passed') {throw 'Incomplete prior runtime regression gate.'}
    $report.runs.prior_acceptance=$priorPath
    Check runtime-regression-review python @('tools/Review-P3-12.py','--acceptance',$priorPath) | Out-Null
    $supervisor=Check public-supervision cargo @('run','--release','--locked','--offline','-p','xtask','--','test','--suite','m1','--case','sdk-stress-supervisor')
    $match=[regex]::Match($supervisor,'M1 artifacts: ([^\r\n]+)')
    if (-not $match.Success) {throw 'Missing public supervision artifact path.'}
    $report.runs.public_supervision=$match.Groups[1].Value.Trim()
    Check public-supervision-review python @('tools/Review-P3-13.py','--supervisor','--run',$report.runs.public_supervision,'--output',(Join-Path $run 'public-supervision-review')) | Out-Null
    $observer=Join-Path $env:CARGO_TARGET_DIR ('release/xtask'+$script:ContinuumExeSuffix)
    $report['native_observer']=@{path=$observer;sha256=(Get-FileHash -LiteralPath $observer).Hash.ToLowerInvariant()}
    foreach ($repetition in 0..4) {
        foreach ($trace in @('off','on')) {
            $name="stress-$trace-$repetition"
            $case=if ($trace -eq 'on') {'sdk-stress-trace'} else {'sdk-stress'}
            $text=Check $name cargo @('run','--release','--locked','--offline','-p','xtask','--','test','--suite','m1','--case',$case)
            $match=[regex]::Match($text,'M1 artifacts: ([^\r\n]+)')
            if (-not $match.Success) {throw 'Missing stress artifact path.'}
            $artifacts=$match.Groups[1].Value.Trim()
            $review=Join-Path $run ($name+'-review')
            Check ($name+'-review') python @('tools/Review-P3-13.py','--run',$artifacts,'--output',$review) | Out-Null
            $report.stress+=@{trace=$trace;repetition=$repetition;artifacts=$artifacts;review=$review;status='passed'}
            if ($trace -eq 'off' -and $repetition -eq 0) {
                Check stress-flavors pwsh @('-NoProfile','-File','tools/Check-P3-13Flavors.ps1','-Descriptors',$artifacts,'-Output',(Join-Path $run 'flavors')) | Out-Null
                $report.runs.flavors=Join-Path $run 'flavors'
            }
        }
    }
    foreach ($inputFile in $inputs) {
        if ($inputFile.path -match '^(kernel/|tools/xtask/|sdk/|experiments/sdk-|config/|contracts/|tools/Build-P3-1[23]Sdk.ps1$)') {
            if ((Get-FileHash -LiteralPath $inputFile.path).Hash.ToLowerInvariant() -ne $inputFile.sha256) {throw "Source changed during acceptance: $($inputFile.path)"}
        }
    }
    if ((Get-FileHash -LiteralPath $observer).Hash.ToLowerInvariant() -ne $report.native_observer.sha256) {throw 'Native observer executable changed during acceptance.'}
    $report.status='passed'
} catch {$report.status='failed';$report['error']=$_.Exception.Message;throw}
finally {$report | ConvertTo-Json -Depth 14 | Set-Content (Join-Path $run 'acceptance.json');Pop-Location;"P3-13 acceptance: $run/acceptance.json"}
