param([string]$PrerequisiteEvidence, [switch]$SkipMeasurements)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
. (Join-Path $PSScriptRoot 'windows/DevSigning.ps1')
$env:CARGO_NET_OFFLINE='true'
$env:CONTINUUM_FONT_DIR=Join-Path $root 'work/text-inputs'
$run=Join-Path $root ('build/m2/P2-08/acceptance-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
$rows=@()
function Invoke-Step($Name,$Program,$Arguments,$TimeoutMs=120000,$ExpectedFailure=$false) {
    $info=[Diagnostics.ProcessStartInfo]::new()
    $info.FileName=$Program;$info.WorkingDirectory=$root;$info.UseShellExecute=$false;$info.CreateNoWindow=$true
    $info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    foreach ($argument in $Arguments) {$info.ArgumentList.Add([string]$argument)}
    $process=[Diagnostics.Process]::new();$process.StartInfo=$info
    $timer=[Diagnostics.Stopwatch]::StartNew()
    $out=[IO.File]::Create((Join-Path $run ($Name+'.stdout.log')))
    $err=[IO.File]::Create((Join-Path $run ($Name+'.stderr.log')))
    $timedOut=$false
    try {
        if (-not $process.Start()) {throw "Could not start $Name"}
        $stdout=$process.StandardOutput.BaseStream.CopyToAsync($out)
        $stderr=$process.StandardError.BaseStream.CopyToAsync($err)
        if (-not $process.WaitForExit($TimeoutMs)) {$timedOut=$true;$process.Kill($true);$process.WaitForExit()}
        $stdout.GetAwaiter().GetResult();$stderr.GetAwaiter().GetResult()
        $passed=-not $timedOut -and (($ExpectedFailure -and $process.ExitCode -ne 0) -or (-not $ExpectedFailure -and $process.ExitCode -eq 0))
        $script:rows+=@{name=$Name;program=$Program;arguments=$Arguments;timeout_ms=$TimeoutMs;elapsed_ms=$timer.ElapsedMilliseconds;exit=$process.ExitCode;timed_out=$timedOut;cleanup='reaped';expected_failure=$ExpectedFailure;passed=$passed}
        $script:rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'commands.json')
        Write-Output "$Name : passed=$passed elapsed_ms=$($timer.ElapsedMilliseconds)"
        if (-not $passed) {throw "Failed $Name; retained at $run"}
    } finally {
        try {if (-not $process.HasExited) {$process.Kill($true);$process.WaitForExit()}} catch {}
        $out.Dispose();$err.Dispose();$process.Dispose()
    }
}
Push-Location $root
try {
    Write-Output "P2-08 evidence: $run"
    if (-not $PrerequisiteEvidence) {
        Invoke-Step 'prerequisites' 'pwsh' @('-NoProfile','-File','tools/Recheck-P2Prerequisites.ps1') 900000
        $match=Select-String -LiteralPath (Join-Path $run 'prerequisites.stdout.log') -Pattern '^Prerequisite evidence: (.+)$' | Select-Object -Last 1
        if (-not $match) {throw 'Missing prerequisite path'}
        $PrerequisiteEvidence=Join-Path $match.Matches[0].Groups[1].Value.Trim() 'prerequisites.json'
    }
    $prereq=Get-Content -LiteralPath $PrerequisiteEvidence -Raw | ConvertFrom-Json
    if ($prereq.status -ne 'passed' -or $prereq.commands.Count -ne 10 -or @($prereq.commands | Where-Object exit_code -ne 0).Count) {throw 'All ten prerequisites must pass'}
    & (Join-Path $PSScriptRoot 'Supply-TextInputs.ps1')
    Invoke-Step 'format' 'cargo' @('fmt','--all','--check')
    Invoke-Step 'unit-tests' 'cargo' @('test','--locked','--offline','-p','xtask','-p','custom-ui','-p','text-core','-p','editor-model') 900000
    $binaries=@{}
    foreach ($variant in 'v1','v2') {
        $features=if ($variant -eq 'v1') {'slint-candidate'}else{'slint-candidate,behavior-v2'}
        Invoke-Step ('build-'+$variant) 'cargo' @('build','--locked','--offline','-p','xtask','--features',$features,'--profile','m2-comparison') 900000
        $directory=Join-Path $env:CARGO_TARGET_DIR ('p2-08/'+$variant)
        New-Item -ItemType Directory -Force $directory | Out-Null
        $exe=Join-Path $directory 'xtask.exe'
        Copy-Item -LiteralPath (Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe') -Destination $exe
        $null=Set-DevExecutableSignature $exe
        $binaries[$variant]=@{path=$exe;sha256=(Get-FileHash -LiteralPath $exe).Hash.ToLowerInvariant();features=$features;bytes=(Get-Item -LiteralPath $exe).Length}
    }
    # A controlled invalid first-party source proves a failed build cannot activate.
    $bad=Join-Path $run 'invalid-behavior.rs'
    'fn main() { compile_error!("p2-08 injected behavior build failure"); }' | Set-Content $bad
    Invoke-Step 'failed-behavior-build' 'rustc' @('--crate-name','p2_08_bad',$bad,'--out-dir',$run) 120000 $true
    if (-not (Select-String -LiteralPath (Join-Path $run 'failed-behavior-build.stderr.log') -SimpleMatch 'p2-08 injected behavior build failure' -Quiet)) {throw 'Unexpected build failure'}
    foreach ($candidate in 'custom','slint') {
        Invoke-Step ($candidate+'-regression') $binaries.v1.path @('test','--suite',($candidate+'-ui')) 300000
        Invoke-Step ($candidate+'-reload') $binaries.v1.path @('ui-reload',$candidate,$binaries.v2.path,(Join-Path $run ($candidate+'-reload'))) 120000
    }
    if (-not $SkipMeasurements) {
        for ($repetition=0;$repetition -lt 5;$repetition++) {
            # Reverse candidate and trace order on alternate repetitions.
            $candidates=if ($repetition % 2 -eq 0) {@('custom','slint')}else{@('slint','custom')}
            $traces=if ($repetition % 2 -eq 0) {@('off','on')}else{@('on','off')}
            foreach ($trace in $traces) {foreach ($candidate in $candidates) {
                $name="$candidate-$trace-$repetition"
                Invoke-Step $name $binaries.v1.path @('ui-compare',$candidate,$trace,$repetition,(Join-Path $run $name)) 120000
            }}
        }
    }
    $inputs=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | ForEach-Object {if ((Test-Path -LiteralPath $_ -PathType Leaf) -and ($_ -match '^(tools/xtask/|crates/|fixtures/m2/|config/|Cargo|rust-toolchain|tools/(Test-P2-08|Review-P2Comparison))')) {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}}})
    $inputs | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $run 'source-inputs.json')
    @{schema_version=1;status='passed';source_base=(git rev-parse HEAD);prerequisite=@{path=$PrerequisiteEvidence;sha256=(Get-FileHash -LiteralPath $PrerequisiteEvidence).Hash.ToLowerInvariant()};binaries=$binaries;measurements_skipped=[bool]$SkipMeasurements;cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name);os=[Environment]::OSVersion.VersionString;logical_processors=[Environment]::ProcessorCount;compiler=(& rustc -vV);protocol_sha256=(Get-FileHash config/p2-08-comparison.json).Hash.ToLowerInvariant();commands=$rows} | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-08 acceptance: $run"
} finally {Pop-Location}
