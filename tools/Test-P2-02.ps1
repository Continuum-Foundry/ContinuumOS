param()
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE = 'true'
Push-Location $root
try {
    $run = Join-Path $root ('build/m2/P2-02/acceptance-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
    New-Item -ItemType Directory -Path $run -Force | Out-Null
    $base = 'fixtures/m2/ui-v1'
    $results = @()
    function Check-Shape([string]$Name, [string]$File, $Data, [bool]$Expected) {
        $schema = Join-Path $root "docs/06-contracts/schemas/hosted-$File.schema.json"
        $errors = @()
        $actual = Test-Json -Json (ConvertTo-Json -InputObject $Data -Depth 100 -Compress) -SchemaFile $schema -ErrorAction SilentlyContinue -ErrorVariable errors
        if ($actual -ne $Expected) {
            @{name=$Name;file=$File;expected=$Expected;actual=[bool]$actual;errors=@($errors | ForEach-Object ToString);status='failed'} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'schema-failure.json')
            throw "Independent schema mismatch: $Name expected=$Expected actual=$actual; retained at $run"
        }
        $script:results += @{name=$Name;file=$File;expected=$Expected;actual=[bool]$actual;passed=$true}
    }
    foreach ($name in @('surface','tokens','layout','corpus','workflow','variants','paint')) {
        Check-Shape $name $name (Get-Content "$base/$name.json" -Raw | ConvertFrom-Json -AsHashtable) $true
    }
    $negative = Get-Content "$base/negative.json" -Raw | ConvertFrom-Json -AsHashtable
    foreach ($case in $negative.cases) {
        $data = Get-Content "$base/$($case.file).json" -Raw | ConvertFrom-Json -AsHashtable
        if ($case.path -eq '') { $data = $case.value }
        else {
            $segments = $case.path.TrimStart('/').Split('/')
            $parent = $data
            foreach ($segment in $segments[0..($segments.Length-2)]) {
                # The single-component path has no parent traversal.
                if ($segments.Length -eq 1) { break }
                if ($parent -is [System.Collections.IDictionary]) { $parent = $parent[$segment] }
                else { $parent = $parent[[int]$segment] }
            }
            $key = $segments[-1]
            $value = $case.value
            if ($case.ContainsKey('repeat')) { $value = $parent[$key] * [int]$case.repeat }
            if ($parent -is [System.Collections.IDictionary]) { $parent[$key] = $value }
            else { $parent[[int]$key] = $value }
        }
        Check-Shape $case.name $case.file $data ($case.error -ne 'shape')
    }
    $results | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'schemas.json')
    $commands = @(
        @{name='format';args=@('fmt','--all','--check')},
        @{name='tests';args=@('test','--locked','--offline','-p','xtask')},
        @{name='fixtures-a';args=@('xtask','test','--suite','ui-fixtures')},
        @{name='fixtures-b';args=@('xtask','test','--suite','ui-fixtures')}
    )
    $rows = @()
    foreach ($command in $commands) {
        $arguments = $command.args
        & cargo @arguments *> (Join-Path $run ($command.name + '.log'))
        $rows += @{name=$command.name;arguments=$arguments;exit_code=$LASTEXITCODE}
        $rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'command-results.json')
        if ($LASTEXITCODE -ne 0) { throw "P2-02 command failed: $($command.name); retained at $run" }
    }
    $fixtureRuns = @()
    foreach ($name in @('fixtures-a','fixtures-b')) {
        $match = Select-String -LiteralPath (Join-Path $run ($name + '.log')) -Pattern '^Shared UI fixture checks passed: (.+)$' | Select-Object -Last 1
        if (-not $match) { throw "Missing fixture run path: $name" }
        $fixturePath = $match.Matches[0].Groups[1].Value.Trim()
        $resultPath = Join-Path $fixturePath 'result.json'
        $result = Get-Content $resultPath -Raw | ConvertFrom-Json
        $dataHash = (Get-FileHash (Join-Path $fixturePath 'files.jsonl')).Hash.ToLowerInvariant()
        if ($result.status -ne 'passed' -or $result.generated_file_list_sha256 -ne $dataHash) { throw "Invalid generated fixture result: $name" }
        $fixtureRuns += @{path=[IO.Path]::GetRelativePath($root,$fixturePath).Replace('\','/');result_sha256=(Get-FileHash $resultPath).Hash.ToLowerInvariant();data_sha256=$dataHash;data_bytes=(Get-Item (Join-Path $fixturePath 'files.jsonl')).Length}
    }
    if ($fixtureRuns[0].data_sha256 -ne $fixtureRuns[1].data_sha256) { throw 'Independent fixture commands differ' }
    $inputs = @(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | ForEach-Object {
        if (Test-Path -LiteralPath $_ -PathType Leaf) { @{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()} }
    })
    $inputs | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $run 'source-inputs.json')
    $hostBinary = Join-Path $env:CARGO_TARGET_DIR 'debug/xtask.exe'
    @{schema_version=1;status='passed';source_base=(git rev-parse HEAD);utc=[DateTime]::UtcNow.ToString('o');commands=$rows;independent_schema_cases=$results.Count;fixture_runs=$fixtureRuns;corpus_lock_sha256=(Get-FileHash "$base/lock.json").Hash.ToLowerInvariant();compiler=(& rustc -vV);os=[Environment]::OSVersion.VersionString;cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name);harness_sha256=(Get-FileHash tools/xtask/src/ui_fixtures.rs).Hash.ToLowerInvariant();signed_executable_sha256=(Get-FileHash $hostBinary).Hash.ToLowerInvariant();cargo_lock_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant();guest_tested=$false;render_tested=$false} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-02 acceptance: $run"
} finally { Pop-Location }
