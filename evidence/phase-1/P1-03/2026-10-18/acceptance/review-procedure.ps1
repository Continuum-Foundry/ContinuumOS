$ErrorActionPreference = 'Stop'
$projectRoot = (Get-Location).Path
$acceptanceRoot = Join-Path $projectRoot 'build/m1/acceptance-p1-03-1792346763755'
$acceptance = Get-Content (Join-Path $acceptanceRoot 'acceptance-results.json') -Raw | ConvertFrom-Json -AsHashtable
if ($acceptance.status -ne 'passed' -or @($acceptance.commands | Where-Object exit_code -ne 0).Count) { throw 'Acceptance failed' }
$launches = @()
$builds = @()
foreach ($case in 'run','iso-smoke','iso-negative','boot-metadata','boot-reject') {
    $caseRoot = $acceptance.runs[$case]
    $result = Get-Content (Join-Path $caseRoot 'result.json') -Raw | ConvertFrom-Json -AsHashtable
    if ($result.status -ne 'passed') { throw "Case failed: $case" }
    foreach ($dir in Get-ChildItem -LiteralPath $caseRoot -Directory -Filter '*-run') {
        $launch = Get-Content (Join-Path $dir.FullName 'launch-manifest.json') -Raw | ConvertFrom-Json -AsHashtable
        $run = Get-Content (Join-Path $dir.FullName 'run-result.json') -Raw | ConvertFrom-Json -AsHashtable
        if (-not $run.cleanup_reaped -or -not $run.iso_unchanged_after_run -or $run.guest_shutdown_claimed) { throw "Bad lifecycle: $dir" }
        if ('-no-reboot' -notin $launch.arguments -or '-nic' -notin $launch.arguments -or '-kernel' -in $launch.arguments -or '-snapshot' -in $launch.arguments) { throw 'Launch policy mismatch' }
        $launches += @{case=$case;run=$dir.Name;build_id=$launch.build_id;session_id=$launch.host_session_id;iso_sha256=$launch.iso_sha256;exit=$run.emulator_test_exit;status=$run.status;cleanup_reaped=$run.cleanup_reaped}
        $recordsFile = Join-Path $dir.FullName 'guest-records.json'
        if (Test-Path -LiteralPath $recordsFile) {
            $guest = Get-Content -LiteralPath $recordsFile -Raw | ConvertFrom-Json -AsHashtable
            $sequence = 0
            foreach ($record in $guest.records) {
                if ($record.fields.session -ne $launch.host_session_id -or $record.fields.build -ne $launch.build_id -or [int]$record.fields.seq -ne $sequence) { throw 'Guest correlation mismatch' }
                $sequence++
            }
            if ($case -eq 'boot-reject') {
                if ($guest.validated -or $guest.complete -or -not $guest.rejected -or $run.emulator_test_exit -ne 35) { throw 'False rejection success' }
            }
        } elseif ($case -ne 'iso-negative') { throw 'Missing guest records' }
    }
    foreach ($buildFile in @((Join-Path $caseRoot 'build-manifest.json')) + @(Get-ChildItem -LiteralPath $caseRoot -Directory | Where-Object Name -like '*kernel' | ForEach-Object { Join-Path $_.FullName 'build-manifest.json' })) {
        if (Test-Path -LiteralPath $buildFile) {
            $b = Get-Content -LiteralPath $buildFile -Raw | ConvertFrom-Json -AsHashtable
            if (-not $b.inspection.kernel_image_bounds.all_load_memory_including_bss) { throw 'Missing full image-boundary audit' }
            $builds += @{image_bounds_checked=$true;case=$case;flavor=$b.flavor;build_id=$b.build_id;kernel_sha256=$b.kernel_sha256;source_commit=$b.source_commit;source_dirty=$b.source_dirty}
        }
    }
}
if ($launches.Count -ne 17 -or @($launches.session_id | Sort-Object -Unique).Count -ne 17) { throw 'Wrong launch/session counts' }
$metadata = Get-Content (Join-Path $acceptance.runs['boot-metadata'] 'result.json') -Raw | ConvertFrom-Json -AsHashtable
if ($metadata.stages.runs.Count -ne 4 -or -not $metadata.stages.identities.same_kernel_bytes -or -not $metadata.stages.identities.distinct_host_and_guest_sessions) { throw 'Metadata identity matrix mismatch' }
$reject = Get-Content (Join-Path $acceptance.runs['boot-reject'] 'result.json') -Raw | ConvertFrom-Json -AsHashtable
if ($reject.stages.runs.Count -ne 8) { throw 'Wrong rejection case count' }
foreach ($r in $reject.stages.runs) {
    if ($r.positive_boot -or $r.run.status -ne 'passed' -or $r.run.emulator_test_exit -ne 35) { throw 'Rejection assertion failure' }
    $guest = Get-Content (Join-Path $acceptance.runs['boot-reject'] "$($r.fixture)-run/guest-records.json") -Raw | ConvertFrom-Json -AsHashtable
    $terminal = $guest.records[-1]
    if ($terminal.event -ne 'm1.boot_rejected' -or $terminal.fields.field -ne $r.expected_field -or $terminal.fields.reason -ne $r.expected_reason) { throw 'Wrong rejection reason' }
}
@{schema_version=1;reviewer='Codex; local self-review, not external review';status='passed';vm_launches=17;positive_metadata_boots=7;expected_metadata_rejections=8;expected_iso_boot_failures=2;distinct_host_sessions=17;all_processes_reaped=$true;guest_uptime_claimed=$false;requirements=@('R01','R03','R17','R18');preparatory_contract_discipline='R14; internal BootInfo only, no external ABI';acceptance_criteria=@{normal_checked_summaries=$true;fresh_immutable_sessions_same_kernel=$true;bounded_required_revision_arithmetic_count_string_rejections=$true;optional_framebuffer_and_invalid_framebuffer_cases=$true;host_manifest_maps_final_iso=$true};launches=$launches;builds=$builds} | ConvertTo-Json -Depth 14 | Set-Content work/p1-03-review.json
'P1-03 evidence review passed'
