param([Parameter(Mandatory)][string]$AcceptancePath)
$ErrorActionPreference = 'Stop'
$projectRoot = (Get-Location).Path
$acceptance = Get-Content (Join-Path $AcceptancePath 'acceptance-results.json') -Raw | ConvertFrom-Json -AsHashtable
if ($acceptance.status -ne 'passed') { throw 'Acceptance must pass' }
$reviews = @()
$builds = @()
$sessions = @()
$allCases = 'run','iso-smoke','iso-negative','boot-metadata','boot-reject','exception-invalid-opcode','exception-page-fault','panic','exception-emergency'
foreach ($case in $allCases) {
    $path = $acceptance.runs[$case]
    $result = Get-Content (Join-Path $path 'result.json') -Raw | ConvertFrom-Json -AsHashtable
    if ($result.status -ne 'passed') { throw "Failed case $case" }
    foreach ($file in Get-ChildItem $path -Recurse -File -Filter 'build-manifest.json' | Where-Object FullName -NotMatch '[\\/]cargo[\\/]') {
        $manifest = Get-Content $file.FullName -Raw | ConvertFrom-Json -AsHashtable
        $inputs = Get-Content (Join-Path $file.DirectoryName 'build-inputs.json') -Raw | ConvertFrom-Json -AsHashtable
        foreach ($name in $inputs.sources.Keys) {
            if ((Get-FileHash (Join-Path $projectRoot $name) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $inputs.sources[$name]) { throw "Stale source $name" }
        }
        if ((Get-FileHash (Join-Path $file.DirectoryName 'kernel.elf') -Algorithm SHA256).Hash.ToLowerInvariant() -ne $manifest.kernel_sha256) { throw 'Kernel hash mismatch' }
        if (-not $manifest.inspection.kernel_image_bounds.all_load_memory_including_bss) { throw 'Incomplete image bounds' }
        $disassembly = [IO.File]::ReadAllText((Join-Path $file.DirectoryName 'llvm-objdump.stdout.log'))
        $errorVectors = 8,10,11,12,13,14,17,21,29,30
        foreach ($vector in 0..31) {
            $block = [regex]::Match($disassembly, "(?m)^[0-9a-f]+ <exception_$vector>:\r?\n(?<block>(?:[^\r\n]+\r?\n)+)").Groups['block'].Value
            $pushes = [regex]::Matches($block, 'pushq\s+\$0x([0-9a-f]+)')
            $want = if ($vector -in $errorVectors) { 1 } else { 2 }
            if ($pushes.Count -ne $want -or [Convert]::ToInt32($pushes[-1].Groups[1].Value,16) -ne $vector -or $block -notmatch '<exception_common>') { throw "Wrong entry code for $vector" }
            if ($want -eq 2 -and $pushes[0].Groups[1].Value -ne '0') { throw 'Missing synthetic error zero' }
        }
        $common = [regex]::Match($disassembly, '(?m)^[0-9a-f]+ <exception_common>:\r?\n(?<block>(?:[^\r\n]+\r?\n)+)').Groups['block'].Value
        foreach ($instruction in 'cli','cld','movq\s+%rsp, %rdi','movq\s+%cr2, %rsi','andq\s+\$-0x10, %rsp','callq') {
            if ($common -notmatch $instruction) { throw "Missing common entry instruction $instruction" }
        }
        if ($common -match 'iret|retq|sti') { throw 'Unexpected return or interrupts enabled' }
        $builds += @{case=$case;build_id=$manifest.build_id;kernel_sha256=$manifest.kernel_sha256;flavor=$manifest.flavor;diagnostic_sites=$manifest.inspection.diagnostic_sites;all_load_memory_including_bss=$true;entry_disassembly_reviewed=$true}
    }
    foreach ($file in Get-ChildItem $path -Recurse -File -Filter 'run-result.json') {
        $run = Get-Content $file.FullName -Raw | ConvertFrom-Json -AsHashtable
        $launch = Get-Content (Join-Path $file.DirectoryName 'launch-manifest.json') -Raw | ConvertFrom-Json -AsHashtable
        $packageDir = $file.DirectoryName -replace '-run$','-package'
        $package = Get-Content (Join-Path $packageDir 'iso-manifest.json') -Raw | ConvertFrom-Json -AsHashtable
        if ((Get-FileHash (Join-Path $packageDir 'continuum.iso') -Algorithm SHA256).Hash.ToLowerInvariant() -ne $package.iso_sha256 -or $run.iso_sha256 -ne $package.iso_sha256 -or $launch.iso_sha256 -ne $package.iso_sha256) { throw 'ISO digest mismatch' }
        if (-not $run.cleanup_reaped -or $run.build_id -ne $launch.build_id -or $run.host_session_id -ne $launch.host_session_id) { throw 'Lifecycle/identity mismatch' }
        $sessions += $run.host_session_id
        $serial = [IO.File]::ReadAllText((Join-Path $file.DirectoryName 'serial.log'))
        $entries = [regex]::Matches($serial,'(?m)^m1.kernel_entry ').Count
        if ($case -eq 'iso-negative') {
            if ($run.status -ne 'failed' -or $entries -ne 0) { throw 'Negative boot counted as success' }
        } else {
            if ($run.status -ne 'passed' -or $entries -ne 1 -or $run.timed_out) { throw 'Run failed or reset' }
            $records = Get-Content (Join-Path $file.DirectoryName 'guest-records.json') -Raw | ConvertFrom-Json -AsHashtable
            if ($records.records.Count -ne [regex]::Matches($serial,'(?m)^m1\.').Count) { throw 'Unparsed guest record' }
            $seq = 0
            foreach ($record in $records.records) {
                if ($record.fields.build -ne $run.build_id -or $record.fields.session -ne $run.host_session_id -or [int]$record.fields.seq -ne $seq -or $record.fields.version -ne '3') { throw 'Uncorrelated guest checkpoint' }
                $seq++
            }
            $fault = $case -in 'exception-invalid-opcode','exception-page-fault','panic','exception-emergency'
            if ($fault -and (-not $records.fatal -or $records.complete -or $run.emulator_test_exit -ne 37 -or -not $records.exceptions_ready)) { throw 'Expected fault misclassified' }
            if ($case -eq 'boot-reject' -and ($records.complete -or $records.exceptions_ready -or $run.emulator_test_exit -ne 35)) { throw 'Rejection misclassified' }
        }
        $reviews += @{case=$case;fixture=$run.fixture;build_id=$run.build_id;session=$run.host_session_id;iso_sha256=$run.iso_sha256;status=$run.status;entries=$entries;test_exit=$run.emulator_test_exit;cleanup_reaped=$run.cleanup_reaped;deadline_ms=$run.deadline_ms;arguments=$launch.arguments}
    }
}
if ($reviews.Count -ne 21 -or ($sessions | Select-Object -Unique).Count -ne 21) { throw "Expected 21 distinct launches, got $($reviews.Count)" }
foreach ($group in $builds | Group-Object flavor) {
    if (($group.Group.build_id | Select-Object -Unique).Count -ne 1 -or ($group.Group.kernel_sha256 | Select-Object -Unique).Count -ne 1) { throw "Unexplained identity/ELF difference within flavor $($group.Name)" }
}
@{status='passed';review='Local Codex self-review; not independent approval';acceptance=(Split-Path $AcceptancePath -Leaf);launch_count=$reviews.Count;all_sessions_distinct=$true;all_source_hashes_current=$true;all_entry_stubs_reviewed=$true;builds=$builds;runs=$reviews;deferred='Stack guards and actual double-fault/IST integration remain P1-05; M1 is incomplete'} | ConvertTo-Json -Depth 20 | Set-Content work/p1-04-review.json
'P1-04 review passed'
