param()
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$run=Join-Path $root ('build/m2/P2-04/acceptance-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
Push-Location $root
try {
    & (Join-Path $PSScriptRoot 'Supply-TextInputs.ps1')
    $lock=Get-Content fixtures/m2/text-v1/lock.json -Raw | ConvertFrom-Json
    $allowed=@('corpus.json','composition.json','fonts.lock.json','data.lock.json','shaping-reference.json')
    if ($lock.schema_version -ne 1 -or $lock.files.Count -ne $allowed.Count) {throw 'Unsupported text fixture lock'}
    foreach ($file in $lock.files) {
        if ($file.path -notin $allowed -or (Get-FileHash -LiteralPath ('fixtures/m2/text-v1/'+$file.path)).Hash.ToLowerInvariant() -ne $file.sha256) {throw 'Text fixture lock mismatch'}
    }
    $commands=@(
        @{name='format';exe='cargo';args=@('fmt','--all','--check')},
        @{name='tests';exe='cargo';args=@('test','--locked','--offline','-p','editor-model','-p','text-core','-p','xtask')},
        @{name='portable';exe='cargo';args=@('check','--locked','--offline','-p','text-core','--target','x86_64-unknown-none')},
        @{name='text-a';exe='cargo';args=@('xtask','test','--suite','text')},
        @{name='text-b';exe='cargo';args=@('xtask','test','--suite','text')},
        @{name='oracle';exe='python';args=@('tools/Reference-TextShape.py','--verify','fixtures/m2/text-v1/shaping-reference.json')},
        @{name='editor';exe='cargo';args=@('xtask','test','--suite','editor-model')},
        @{name='ui-fixtures';exe='cargo';args=@('xtask','test','--suite','ui-fixtures')}
    )
    $rows=@()
    foreach ($command in $commands) {
        $arguments=$command.args; & $command.exe @arguments *> (Join-Path $run ($command.name+'.log'))
        $rows+=@{name=$command.name;program=$command.exe;arguments=$arguments;exit_code=$LASTEXITCODE}
        $rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'commands.json')
        if ($LASTEXITCODE -ne 0) {throw "P2-04 command failed: $($command.name); retained at $run"}
    }
    $replays=@()
    foreach ($name in @('text-a','text-b')) {
        $match=Select-String -LiteralPath (Join-Path $run ($name+'.log')) -Pattern '^Text conformance checks passed: (.+)$' | Select-Object -Last 1
        if (-not $match) {throw 'Missing text run output'}
        $path=$match.Matches[0].Groups[1].Value.Trim()
        $result=Get-Content (Join-Path $path 'result.json') -Raw | ConvertFrom-Json
        if ($result.status -ne 'passed' -or $result.observations.Count -ne 45 -or $result.composition.Count -ne 6 -or $result.negative.Count -ne 14) {throw 'Incomplete text run'}
        $replays+=@{path=[IO.Path]::GetRelativePath($root,$path).Replace('\','/');result_sha256=(Get-FileHash (Join-Path $path 'result.json')).Hash.ToLowerInvariant()}
    }
    if ($replays[0].result_sha256 -ne $replays[1].result_sha256) {throw 'Separate text runs differ (semantic/raster hashes)'}
    $inputs=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | ForEach-Object {if (Test-Path -LiteralPath $_ -PathType Leaf) {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}}})
    $inputs | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    & cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-unknown-none > (Join-Path $run 'metadata.json')
    if ($LASTEXITCODE) {throw 'Cannot record exact dependency metadata'}
    @{schema_version=1;status='passed';source_base=(git rev-parse HEAD);utc=[DateTime]::UtcNow.ToString('o');commands=$rows;replays=$replays;compiler=(& rustc -vV);os=[Environment]::OSVersion.VersionString;cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name);cargo_lock_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant();fixture_lock_sha256=(Get-FileHash fixtures/m2/text-v1/lock.json).Hash.ToLowerInvariant();signed_executable_sha256=(Get-FileHash (Join-Path $env:CARGO_TARGET_DIR 'debug/xtask.exe')).Hash.ToLowerInvariant();harness_sha256=(Get-FileHash tools/xtask/src/text.rs).Hash.ToLowerInvariant();guest_tested=$false;candidate_rendered=$false;text_raster_rendered=$true;native_ime_tested=$false;full_unicode_conformance=$false} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-04 acceptance: $run"
} finally {Pop-Location}
