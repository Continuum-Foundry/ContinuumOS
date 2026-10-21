param([Parameter(Mandatory)][string]$Run, [string]$Destination, [switch]$Partial)
# Local self-review and compact promotion. Never copies binaries/source trees.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$Run = (Resolve-Path -LiteralPath $Run).Path
function Read-Json([string]$path) { Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -AsHashtable }
function Hash([string]$path) { (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() }
function Local([string]$path) { $path.Replace('${WORKSPACE}', $root) }
function Safe([string]$text) {
    $cygRoot = '/cygdrive/' + $root.Substring(0,1).ToLowerInvariant() + $root.Substring(2).Replace('\','/')
    $text.Replace($cygRoot,'${WORKSPACE_CYGWIN}').Replace($root.Replace('\','\\'),'${WORKSPACE}').Replace($root.Replace('\','/'),'${WORKSPACE}').Replace($root,'${WORKSPACE}').Replace($env:USERPROFILE.Replace('\','\\'),'${USERPROFILE}').Replace($env:USERPROFILE,'${USERPROFILE}')
}
function Require([bool]$condition, [string]$message) { if (-not $condition) { throw $message } }
if ($Partial) {
    $guest = Read-Json (Join-Path $Run 'guest-regressions.json')
    Require ($guest.status -eq 'passed' -and $guest.cases.Count -eq 23) 'Incomplete independent guest regressions'
    $inventory = @{builds=@();launches=@()}
    function Collect([string]$dir) {
        if (Test-Path (Join-Path $dir 'build-inputs.json')) {
            $inventory.builds += @{directory=$dir;inputs=(Read-Json (Join-Path $dir 'build-inputs.json'));manifest=(Read-Json (Join-Path $dir 'build-manifest.json'))}
        }
        if (Test-Path (Join-Path $dir 'run-result.json')) {
            $inventory.launches += @{directory=$dir;result=(Read-Json (Join-Path $dir 'run-result.json'));launch=(Read-Json (Join-Path $dir 'launch-manifest.json'))}
        }
        foreach ($child in Get-ChildItem -LiteralPath $dir -Directory) {
            if ($child.Name -notin 'cargo','iso root with spaces','extracted payload') { Collect $child.FullName }
        }
    }
    foreach ($case in $guest.cases) { Require ($case.exit_code -eq 0) 'Failed independent case'; Collect $case.artifacts }
    $result = @{status='passed';scope='Independent guest regressions only; complete M1 acceptance BLOCKED';stages=@{rows=@($guest.cases | ForEach-Object { @{case=$_.case;status='passed'} })}}
    $result | ConvertTo-Json -Depth 20 | Set-Content (Join-Path $Run 'result.json')
    $inventory.builds[0].inputs.sources | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $Run 'suite-source-inputs.json')
} else {
    $result = Read-Json (Join-Path $Run 'result.json')
    $inventory = Read-Json (Join-Path $Run 'suite-inventory.json')
    Require ($result.status -eq 'passed' -and $result.stages.rows.Count -eq 33) 'Incomplete suite'
}
$expectedLaunches = if ($Partial) { 36 } else { 42 }
Require ($inventory.launches.Count -eq $expectedLaunches) 'Wrong launch count'
$sessions = [Collections.Generic.HashSet[string]]::new()
$reviewed = @()
$builds = @{}
foreach ($b in $inventory.builds) {
    $dir = Local $b.directory
    Require ((Hash (Join-Path $dir 'kernel.elf')) -eq $b.manifest.kernel_sha256) 'Changed ELF'
    foreach ($p in $b.inputs.sources.Keys) {
        Require ((Hash (Join-Path $root $p)) -eq $b.inputs.sources[$p]) "Stale source: $p"
    }
    if ($builds.ContainsKey($b.manifest.flavor)) {
        Require ($builds[$b.manifest.flavor].manifest.build_id -eq $b.manifest.build_id) 'Build identity mismatch'
        Require ($builds[$b.manifest.flavor].manifest.kernel_sha256 -eq $b.manifest.kernel_sha256) 'ELF identity mismatch'
    } else { $builds[$b.manifest.flavor] = $b }
}
foreach ($l in $inventory.launches) {
    $dir = Local $l.directory
    $r = $l.result; $launch = $l.launch
    Require ($sessions.Add($r.host_session_id)) 'Duplicate session'
    Require ($r.cleanup_reaped -and $r.iso_unchanged_after_run -and $r.deadline_ms -eq 30000) 'Lifecycle failure'
    Require ($launch.vm.deadline_ms -eq 30000) 'Deadline differs from profile'
    Require ($launch.vars_template_sha256 -eq $launch.vars_copy_before_sha256) 'Firmware variables were not a fresh template copy'
    Require (($launch.arguments -contains '-no-reboot') -and ($launch.arguments -contains 'none')) 'Wrong launch policy'
    Require (@($launch.arguments | Where-Object { $_ -eq '-drive' }).Count -eq 3) 'Unexpected attached disk'
    foreach ($forbidden in '-kernel','-snapshot','-virtfs','-fsdev','-netdev','-incoming','-loadvm') {
        Require (-not ($launch.arguments -contains $forbidden)) "Forbidden launch argument: $forbidden"
    }
    $nicIndex = [Array]::IndexOf($launch.arguments,'-nic')
    Require ($nicIndex -ge 0 -and $launch.arguments[$nicIndex+1] -eq 'none') 'Network is enabled'
    $case = Read-Json (Join-Path $dir 'case-result.json')
    $iso = $launch.arguments | Where-Object { $_ -match '^file=.*media=cdrom' }
    Require (@($iso).Count -eq 1) 'Missing ISO launch argument'
    $isoPath = Local ([regex]::Match($iso,'^file=(.*),media=cdrom').Groups[1].Value)
    Require ((Hash $isoPath) -eq $r.iso_sha256 -and $r.iso_sha256 -eq $case.package.iso_sha256) 'Changed actual launched ISO'
    $serial = [IO.File]::ReadAllLines((Join-Path $dir 'serial.log'))
    $records = @(); $seq = 0
    foreach ($line in $serial) {
        if (-not $line.StartsWith('m1.')) { continue }
        $words = $line.Split(' ',[StringSplitOptions]::RemoveEmptyEntries)
        $fields = @{}
        foreach ($word in $words[1..($words.Count-1)]) {
            $kv = $word.Split('=',2)
            Require ($kv.Count -eq 2 -and -not $fields.ContainsKey($kv[0])) 'Malformed raw field'
            $fields[$kv[0]] = $kv[1]
        }
        Require ($fields.version -eq '5' -and $fields.build -eq $r.build_id -and $fields.session -eq $r.host_session_id) 'Raw identity mismatch'
        Require ([int]$fields.seq -eq $seq) 'Raw sequence mismatch'
        $seq++
        $records += @{event=$words[0];fields=$fields}
    }
    $events = @($records | ForEach-Object event)
    $positive = $events -contains 'm1.ready'
    if ($positive) {
        $last = -1
        foreach ($event in 'm1.kernel_entry','m1.boot_info_validated','m1.exceptions_ready','m1.memory_ready','m1.timer_ready','m1.timer_progress','m1.boot_complete','m1.ready') {
            Require (@($events | Where-Object { $_ -eq $event }).Count -eq 1) "Checkpoint count: $event"
            $index = [Array]::IndexOf($events,$event)
            Require ($index -gt $last) 'Checkpoint order'
            $last = $index
        }
        Require ($events[-1] -eq 'm1.ready') 'Readiness is not terminal'
        $samples = @($records | Where-Object event -eq 'm1.clock_sample')
        Require ($samples.Count -eq 100) 'Missing uptime samples'
        $resolution = [uint64]($records | Where-Object event -eq 'm1.timer_ready').fields.resolution_ns
        $previous = 0UL
        foreach ($sample in $samples) {
            $ns = [uint64]$sample.fields.uptime_ns
            Require ($ns -gt $previous -and $ns -eq ([uint64]$sample.fields.ticks * $resolution)) 'Nonmonotonic or incorrect delivered time'
            $previous = $ns
        }
        Require ($r.status -eq 'passed' -and -not $r.timed_out) 'False positive boot'
        if ($r.fixture -eq 'acceptance-exit') {
            Require ($case.package.build_flavor -eq 'normal' -and $r.emulator_test_exit -eq 33) 'Final normal boot was a test-feature kernel'
        }
    } else {
        Require (-not ($events -contains 'm1.boot_complete')) 'False completion on failure'
    }
    $outcome = if ($positive) { 'ready' }
        elseif ($events -contains 'm1.exception' -or $events -contains 'm1.panic') { 'expected-fatal' }
        elseif ($events -contains 'm1.boot_rejected' -or $events -contains 'm1.platform_rejected') { 'expected-rejection' }
        elseif ($r.status -eq 'failed' -and -not $r.kernel_entry_observed) { 'broken-before-entry' }
        else { throw 'Unclassified launch outcome' }
    $reviewed += @{directory=(Safe $dir);fixture=$r.fixture;outcome=$outcome;positive_ready=$positive;status=$r.status;exit=$r.emulator_test_exit;timed_out=$r.timed_out;session=$r.host_session_id;build=$r.build_id;iso_sha256=$r.iso_sha256;records=$seq}
}
Require ($sessions.Count -eq $expectedLaunches) 'Missing session'
$final = @($reviewed | Where-Object fixture -eq 'acceptance-exit')
if (-not $Partial) { Require ($final.Count -eq 6 -and @($final | Where-Object { -not $_.positive_ready }).Count -eq 0) 'Final clean boots incomplete' }
$positiveCount = if ($Partial) { 14 } else { 20 }
foreach ($pair in @(@('ready',$positiveCount),@('expected-fatal',9),@('expected-rejection',11),@('broken-before-entry',2))) {
    Require (@($reviewed | Where-Object outcome -eq $pair[0]).Count -eq $pair[1]) "Missing outcome class: $($pair[0])"
}
$report = @{schema_version=1;status='passed';review='local Codex self-review; not independent approval';review_tool_sha256=(Hash $PSCommandPath);run=(Safe $Run);rows=$result.stages.rows;build_flavors=@($builds.Keys | Sort-Object);launches=$reviewed;source_hashes_verified=$true;actual_artifacts_verified=$true;clean_comparison=$result.stages.clean_comparison}
if ($Partial) { $report.status='blocked'; $report['scope']='Guest regressions pass; this is NOT a passing full M1 review'; $report['blocker']='Smart App Control refuses current host test executable (4551); corrected full gate including clean comparison/five boots must be rerun' }
$report | ConvertTo-Json -Depth 30 | Set-Content (Join-Path $Run 'review.json')
if (-not $Destination) { $report | ConvertTo-Json -Depth 5; exit }
$Destination = [IO.Path]::GetFullPath((Join-Path $root $Destination))
$evidenceRoot = [IO.Path]::GetFullPath((Join-Path $root 'evidence/phase-1/P1-07'))
Require ($Destination.StartsWith($evidenceRoot + [IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) 'Evidence destination outside P1-07'
if (Test-Path -LiteralPath $Destination) {
    Require (-not (Test-Path (Join-Path $Destination 'SHA256SUMS.json'))) 'Do not overwrite frozen evidence'
    Require (@(Get-ChildItem -LiteralPath $Destination -File -Recurse | Where-Object Name -ne 'README.md').Count -eq 0) 'Destination contains existing evidence'
} else { New-Item -ItemType Directory -Path $Destination | Out-Null }
$index = [Collections.Generic.List[object]]::new()
function Promote([string]$source, [string]$relative) {
    $target = Join-Path $Destination $relative
    New-Item -ItemType Directory -Force -Path (Split-Path $target) | Out-Null
    $bytes = [IO.File]::ReadAllBytes($source)
    if ($relative.EndsWith('serial.log')) { [IO.File]::WriteAllBytes($target,$bytes) }
    else { [IO.File]::WriteAllText($target,(Safe ([IO.File]::ReadAllText($source))),[Text.UTF8Encoding]::new($false)) }
    $index.Add(@{path=$relative;source=(Safe $source);raw_sha256=(Hash $source);retained_sha256=(Hash $target);verbatim=((Hash $source) -eq (Hash $target))})
}
Promote (Join-Path $Run 'result.json') 'result.json'
Promote (Join-Path $Run 'review.json') 'review.json'
if (-not $Partial) { Promote (Join-Path $Run 'clean-comparison.json') 'clean-comparison.json' }
Promote (Join-Path $Run 'suite-source-inputs.json') 'source-inputs.json'
if ($Partial) {
# The original blocked bundle owns these historical diagnostics. A completed
# gate references that immutable bundle instead of copying its old approval state.
Promote (Join-Path $root 'work/p1-07-development-observations.txt') 'development/observations.txt'
Promote (Join-Path $root 'work/p1-07-doctor-precheck.log') 'development/sandboxed-doctor.log'
Promote (Join-Path $root 'work/p1-07-full-suite.log') 'development/first-suite.log'
Promote (Join-Path $root 'build/m1/suite-1792532218089714300-19176/result.json') 'development/first-suite-result.json'
Promote (Join-Path $root 'work/p1-07-iso-differences.json') 'development/first-iso-differences.json'
Promote (Join-Path $root 'work/p1-07-fixed-iso-comparison.json') 'development/fixed-guid-comparison.json'
Promote (Join-Path $root 'work/p1-07-comparison-test.log') 'development/application-control-test.log'
Promote (Join-Path $root 'work/p1-07-full-suite-final.log') 'development/blocked-suite.log'
Promote (Join-Path $root 'build/m1/suite-1792532726225181200-52940/result.json') 'development/blocked-suite-result.json'
Promote (Join-Path $root 'build/m1/suite-1792532726225181200-52940/m0/check.stderr.log') 'development/blocked-m0.stderr.log'
Promote (Join-Path $root 'work/p1-07-approval-request.txt') 'approval-request.txt'
Promote (Join-Path $root 'work/p1-07-code-integrity-summary.json') 'development/code-integrity-summary.json'
Promote (Join-Path $root 'work/inspect-p107-iso.cjs') 'development/iso-difference-procedure.cjs'
if ($Partial) { Promote (Join-Path $root 'work/Run-P1-07GuestRegressions.ps1') 'guest-regression-procedure.ps1' }
$firstSuite = Join-Path $root 'build/m1/suite-1792532218089714300-19176'
Promote (Join-Path $firstSuite 'clean-a/build-inputs.json') 'development/first-normal-inputs.json'
Promote (Join-Path $firstSuite 'clean-a/build-manifest.json') 'development/first-normal-manifest.json'
foreach ($label in 'final-1','final-2','final-3','final-4','final-5','second-clean') {
    $dir = Join-Path $firstSuite "$label-run"
    $old = Read-Json (Join-Path $dir 'case-result.json')
    Require ($old.status -eq 'passed' -and $old.emulator_test_exit -eq 33 -and $old.cleanup_reaped) 'First-suite normal launch did not pass'
    Require ((Hash (Join-Path $firstSuite "$label-package/continuum.iso")) -eq $old.iso_sha256) 'Changed first-suite ISO'
    Promote (Join-Path $dir 'serial.log') "development/$label-serial.log"
    $oldContext = @{scope='First suite only; generated GPT comparison subsequently failed';result=$old;launch=(Read-Json (Join-Path $dir 'launch-manifest.json'))}
    $oldPath = Join-Path $Run "first-$label-context.json"
    $oldContext | ConvertTo-Json -Depth 30 | Set-Content $oldPath
    Promote $oldPath "development/$label-context.json"
}
}
foreach ($flavor in $builds.Keys) {
    $dir = Local $builds[$flavor].directory
    Promote (Join-Path $dir 'build-inputs.json') "builds/$flavor-inputs.json"
    Promote (Join-Path $dir 'build-manifest.json') "builds/$flavor-manifest.json"
}
$disassembly = Join-Path (Local $builds.normal.directory) 'llvm-objdump.stdout.log'
$text = [IO.File]::ReadAllText($disassembly)
$excerpts = @{}
foreach ($symbol in 'timer_vector','spurious_vector','exception_common') {
    $match = [regex]::Match($text,"(?ms)^[^\r\n]*<$symbol>:\r?\n.*?(?=\r?\n\r?\n)")
    Require $match.Success "Missing disassembly excerpt: $symbol"
    $excerpts[$symbol] = $match.Value
}
@{scope='Selected verbatim LLVM function excerpts; full output stays local';source=(Safe $disassembly);raw_source_sha256=(Hash $disassembly);functions=$excerpts} | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $Run 'assembly-excerpts.json')
Promote (Join-Path $Run 'assembly-excerpts.json') 'builds/assembly-excerpts.json'
$launchNumber = 0
foreach ($l in $inventory.launches) {
    $dir = Local $l.directory
    $launchNumber++
    $name = '{0:D2}-{1}-{2}' -f $launchNumber,$l.result.fixture,$l.result.host_session_id.Substring(0,8)
    Promote (Join-Path $dir 'serial.log') "launches/$name-serial.log"
    $context = @{result=$l.result;launch=$l.launch;package=(Read-Json (Join-Path $dir 'case-result.json')).package;host_logs=@{};host_file_times=@{}}
    foreach ($file in 'qemu.stdout.log','qemu.stderr.log') {
        $path = Join-Path $dir $file
        $context.host_logs[$file] = @{raw_sha256=(Hash $path);text=(Safe ([IO.File]::ReadAllText($path)))}
    }
    foreach ($file in 'serial.log','run-result.json') {
        $item = Get-Item -LiteralPath (Join-Path $dir $file)
        $context.host_file_times[$file] = @{created_utc=$item.CreationTimeUtc.ToString('o');last_write_utc=$item.LastWriteTimeUtc.ToString('o')}
    }
    $contextPath = Join-Path $Run "compact-$name.json"
    $context | ConvertTo-Json -Depth 30 | Set-Content $contextPath
    Promote $contextPath "launches/$name-context.json"
}
$checksRun = if ($Partial) { Join-Path $root 'build/m1/suite-1792532218089714300-19176' } else { $Run }
foreach ($name in 'format','m0','dependencies','kernel-host','doctor','prerequisites','docs','contracts','governance') {
    Promote (Join-Path $checksRun "$name/check.stdout.log") "checks/$name.stdout.log"
    Promote (Join-Path $checksRun "$name/check.stderr.log") "checks/$name.stderr.log"
    Promote (Join-Path $checksRun "$name/check.command.json") "checks/$name.command.json"
}
$m0 = [regex]::Match([IO.File]::ReadAllText((Join-Path $checksRun 'm0/check.stdout.log')),'Acceptance evidence: ([^\r\n]+)').Groups[1].Value
Promote (Join-Path $m0 'host-tests.log') 'checks/m0-host-tests.log'
Promote (Join-Path $m0 'acceptance-results.json') 'checks/m0-acceptance.json'
foreach ($file in 'compile-tests.log','doctor-tests.json','source-path-spaces.log') {
    Promote (Join-Path $m0 $file) "checks/m0-$file"
}
$pre = [regex]::Match([IO.File]::ReadAllText((Join-Path $checksRun 'prerequisites/check.stdout.log')),'P1-01 test evidence: ([^\r\n]+)').Groups[1].Value
foreach ($file in Get-ChildItem -LiteralPath $pre -File -Filter '*.json') {
    Promote $file.FullName "checks/prerequisites-$($file.Name)"
}
$dep = [regex]::Match([IO.File]::ReadAllText((Join-Path $checksRun 'dependencies/check.stdout.log')),'Evidence: ([^\r\n]+)').Groups[1].Value
Promote (Join-Path $dep 'dependency-report.json') 'checks/dependency-report.json'
@{scope='Selected diagnostic artifacts; original/retained hashes; README and frozen checksum manifest excluded';files=$index.ToArray()} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $Destination 'evidence-index.json')
$files = @(Get-ChildItem -LiteralPath $Destination -Recurse -File)
Require ($files.Count -lt 195 -and ($files | Measure-Object Length -Sum).Sum -lt 4MB -and @($files | Where-Object Length -gt 256KB).Count -eq 0) 'Compact evidence budget exceeded'
"Reviewed and promoted compact evidence: $Destination; $($files.Count) files. Add review README and freeze SHA256SUMS after inspection."
