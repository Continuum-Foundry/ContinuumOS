param()
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$run = Join-Path $root ('build/m2/P2-03/acceptance-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
Push-Location $root
try {
    $lock=Get-Content fixtures/m2/actions-v1/lock.json -Raw | ConvertFrom-Json
    if ($lock.schema_version -ne 1 -or $lock.files.Count -ne 3) { throw 'Unsupported action fixture lock' }
    foreach ($file in $lock.files) {
        if ($file.path -notin @('fixtures/m2/actions-v1/replay.json','fixtures/m2/actions-v1/initial.hex','docs/06-contracts/schemas/hosted-action.schema.json') -or
            (Get-FileHash -LiteralPath $file.path).Hash.ToLowerInvariant() -ne $file.sha256) { throw 'Action fixture lock mismatch' }
    }
    $fixture='fixtures/m2/actions-v1/replay.json'
    $schema='docs/06-contracts/schemas/hosted-action.schema.json'
    $data=Get-Content $fixture -Raw | ConvertFrom-Json -AsHashtable
    $shape=@()
    function Check-Shape($Name,$Value,[bool]$Expected) {
        $actual=Test-Json -Json (ConvertTo-Json -InputObject $Value -Depth 30 -Compress) -SchemaFile $schema -ErrorAction SilentlyContinue
        if ($actual -ne $Expected) { throw "Action schema mismatch: $Name" }
        $script:shape += @{name=$Name;expected=$Expected;actual=[bool]$actual}
    }
    foreach ($step in $data.steps) { Check-Shape "request-$($step.request.id)" $step.request $true }
    foreach ($mutation in @('version','unknown-key','wrong-payload','unknown-action','negative-id','overflow-id','missing-revision')) {
        $request=($data.steps[0].request | ConvertTo-Json -Depth 30 | ConvertFrom-Json -AsHashtable)
        switch ($mutation) {
            'version' {$request.version=2}
            'unknown-key' {$request['rights']='all'}
            'wrong-payload' {$request.action.value=5}
            'unknown-action' {$request.action.name='files.save'}
            'negative-id' {$request.id=-1}
            'overflow-id' {$request.id=9007199254740992}
            'missing-revision' {$request.Remove('expected_document_revision') | Out-Null}
        }
        Check-Shape $mutation $request $false
    }
    $shape | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'schemas.json')
    $commands=@(
        @{name='format';args=@('fmt','--all','--check')},
        @{name='tests';args=@('test','--locked','--offline','-p','editor-model','-p','xtask')},
        @{name='portable-check';args=@('check','--locked','--offline','-p','editor-model','--target','x86_64-unknown-none')},
        @{name='replay-a';args=@('xtask','test','--suite','editor-model')},
        @{name='replay-b';args=@('xtask','test','--suite','editor-model')},
        @{name='ui-fixtures';args=@('xtask','test','--suite','ui-fixtures')}
    )
    $rows=@()
    foreach ($command in $commands) {
        $arguments=$command.args; & cargo @arguments *> (Join-Path $run ($command.name+'.log'))
        $rows += @{name=$command.name;arguments=$arguments;exit_code=$LASTEXITCODE}
        $rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'commands.json')
        if ($LASTEXITCODE -ne 0) { throw "P2-03 command failed: $($command.name); retained at $run" }
    }
    $replays=@(); $independent=@()
    foreach ($name in @('replay-a','replay-b')) {
        $match=Select-String -LiteralPath (Join-Path $run ($name+'.log')) -Pattern '^Document action checks passed: (.+)$' | Select-Object -Last 1
        if (-not $match) { throw "Missing replay output: $name" }
        $path=$match.Matches[0].Groups[1].Value.Trim(); $result=Get-Content (Join-Path $path 'result.json') -Raw | ConvertFrom-Json
        if ($result.status -ne 'passed' -or $result.entries.Count -ne 4 -or $result.negative_cases.Count -ne 56) { throw 'Incomplete replay' }
        foreach ($entry in $result.entries) { $independent += & (Join-Path $PSScriptRoot 'Test-EditorCheckpoint.ps1') -Path (Join-Path $path ($entry.entry+'.checkpoint')) -Fixture $fixture }
        $replays += @{path=[IO.Path]::GetRelativePath($root,$path).Replace('\','/');result_sha256=(Get-FileHash (Join-Path $path 'result.json')).Hash.ToLowerInvariant();final_sha256=$result.entries[0].final_sha256}
    }
    if ($replays[0].result_sha256 -ne $replays[1].result_sha256) { throw 'Separate replay commands differ' }
    $independent | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'independent-codec.json')
    $inputs=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | ForEach-Object { if (Test-Path -LiteralPath $_ -PathType Leaf) {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}} })
    $inputs | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    @{schema_version=1;status='passed';source_base=(git rev-parse HEAD);utc=[DateTime]::UtcNow.ToString('o');commands=$rows;schema_cases=$shape.Count;replays=$replays;independent_decodes=$independent.Count;compiler=(& rustc -vV);os=[Environment]::OSVersion.VersionString;cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name);cargo_lock_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant();fixture_lock_sha256=(Get-FileHash fixtures/m2/actions-v1/lock.json).Hash.ToLowerInvariant();signed_executable_sha256=(Get-FileHash (Join-Path $env:CARGO_TARGET_DIR 'debug/xtask.exe')).Hash.ToLowerInvariant();harness_sha256=(Get-FileHash tools/xtask/src/editor.rs).Hash.ToLowerInvariant();guest_tested=$false;render_tested=$false;provider_replacement_tested=$false} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-03 acceptance: $run"
} finally { Pop-Location }
