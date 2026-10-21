$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE = 'true'
$run = Join-Path $root ('build/m2/P2-01/acceptance-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force (Join-Path $run 'config with spaces') | Out-Null
Push-Location $root
try {
    & cargo test --locked -p xtask *> (Join-Path $run 'unit-tests.log')
    if ($LASTEXITCODE -ne 0) { throw 'Host unit tests failed.' }
    $rows = [Collections.Generic.List[object]]::new()
    function Invoke-Case($executable, $label, $config, $profile, $scale, $case, $expected, $launched) {
        $output = & $executable hosted --config $config --profile $profile --scale $scale --case $case 2> (Join-Path $run ($label + '.stderr.log'))
        $code = $LASTEXITCODE
        $output | Set-Content (Join-Path $run ($label + '.stdout.log'))
        $line = @($output | Where-Object { $_ -like 'Hosted lifecycle evidence: *' })
        if ($line.Count -ne 1) { throw "$label did not record its manifest." }
        $directory = $line[0].Substring('Hosted lifecycle evidence: '.Length)
        $record = Get-Content (Join-Path $directory 'result.json') -Raw | ConvertFrom-Json
        if ($record.outcome -ne $expected -or $record.launched -ne $launched -or
            ($launched -and $record.cleanup -ne 'reaped') -or
            (-not $launched -and $record.cleanup -ne 'not-launched') -or
            (($code -eq 0) -ne ($expected -eq 'exited-zero'))) { throw "$label result mismatch: $($record | ConvertTo-Json -Compress -Depth 8)" }
        if ($launched -and (Get-Process -Id $record.child_pid -ErrorAction SilentlyContinue)) { throw "$label child is still running." }
        $rows.Add(@{case=$label;outcome=$record.outcome;exit_code=$code;cleanup=$record.cleanup;result=([IO.Path]::GetRelativePath($root,(Join-Path $directory 'result.json')).Replace('\','/'));sha256=(Get-FileHash (Join-Path $directory 'result.json')).Hash.ToLowerInvariant()})
    }
    foreach ($mode in 'm2-comparison','m2-diagnostic') {
        & cargo build --locked -p xtask --profile $mode *> (Join-Path $run ($mode + '.build.log'))
        if ($LASTEXITCODE -ne 0) { throw "$mode build failed." }
        $executable = Join-Path $env:CARGO_TARGET_DIR "$mode/xtask.exe"
        # The existing runner verifies the same name/root/key used by Cargo.
        . (Join-Path $PSScriptRoot 'windows/DevSigning.ps1')
        $null = Set-DevExecutableSignature $executable
        foreach ($profile in 'hd','full-hd') { foreach ($scale in '1.0','1.5','2.0') {
            Invoke-Case $executable "$mode-$profile-$scale" 'config/m2-profiles.json' $profile $scale 'success' 'exited-zero' $true
        } }
    }
    foreach ($entry in @(@('nonzero','exited-nonzero'),@('timeout','timeout'))) {
        Invoke-Case $executable $entry[0] 'config/m2-profiles.json' 'hd' '1.0' $entry[0] $entry[1] $true
    }
    $spacedInput = Join-Path $run 'config with spaces/input with spaces.txt'
    Copy-Item -LiteralPath 'fixtures/m2/runner-input.txt' -Destination $spacedInput
    $spacedConfig = Get-Content config/m2-profiles.json -Raw | ConvertFrom-Json -AsHashtable
    $spacedConfig.input.path = [IO.Path]::GetRelativePath($root, $spacedInput).Replace('\','/')
    $spacedPath = Join-Path $run 'config with spaces/valid.json'
    $spacedConfig | ConvertTo-Json -Depth 10 | Set-Content $spacedPath
    Invoke-Case $executable 'paths-with-spaces' $spacedPath 'hd' '1.0' 'success' 'exited-zero' $true
    Invoke-Case $executable 'invalid-scale' 'config/m2-profiles.json' 'hd' '0.0' 'success' 'validation-rejected' $false
    Invoke-Case $executable 'invalid-profile' 'config/m2-profiles.json' '4k' '1.0' 'success' 'validation-rejected' $false
    Invoke-Case $executable 'missing-config' 'work/p2-01/nonexistent-config.json' 'hd' '1.0' 'success' 'validation-rejected' $false
    foreach ($variant in 'invalid-size','missing-input','checksum-mismatch','unknown-field','invalid-timeout') {
        $config = Get-Content config/m2-profiles.json -Raw | ConvertFrom-Json -AsHashtable
        switch ($variant) {
            'invalid-size' { $config.profiles.hd.width = 0 }
            'missing-input' { $config.input.path = 'work/p2-01/nonexistent-input.txt' }
            'checksum-mismatch' { $config.input.sha256 = '0' * 64 }
            'unknown-field' { $config['unknown'] = $true }
            'invalid-timeout' { $config.timeout_ms = 10001 }
        }
        $path = Join-Path $run "config with spaces/$variant.json"
        $config | ConvertTo-Json -Depth 10 | Set-Content $path
        Invoke-Case $executable $variant $path 'hd' '1.0' 'success' 'validation-rejected' $false
    }
    @{schema_version=1;status='passed';cases=$rows.ToArray();rendered=$false;guest_metrics=$false} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-01 lifecycle acceptance passed: $run"
} finally { Pop-Location }
