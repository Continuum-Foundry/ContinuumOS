param([string]$Prerequisites='')
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$run=Join-Path $root ('build/m2/P2-10/acceptance-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
Push-Location $root
try {
    if (-not $Prerequisites) {
        & pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1 *> (Join-Path $run 'prerequisites.log')
        if ($LASTEXITCODE) {throw 'Prerequisite recheck failed'}
        $match=Select-String (Join-Path $run 'prerequisites.log') -Pattern '^Prerequisite evidence: (.+)$' | Select-Object -Last 1
        if (-not $match) {throw 'Missing prerequisite result'}
        $Prerequisites=$match.Matches[0].Groups[1].Value.Trim()
    }
    $prereq=Get-Content (Join-Path $Prerequisites 'prerequisites.json') -Raw | ConvertFrom-Json
    if ($prereq.status -ne 'passed' -or $prereq.commands.Count -ne 10 -or @($prereq.commands | Where-Object exit_code -ne 0).Count) {throw 'All ten prerequisites must pass'}
    $testedPaths=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | Where-Object {$_ -match '^(crates/|tools/|experiments/|kernel/|config/|fixtures/)' -and $_ -match '\.(rs|toml|lock|ps1|py|json)$'})
    $testedInputs=@($testedPaths | ForEach-Object {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}})
    $testedInputs | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $run 'tested-inputs.json')
    $commands=@(
        @{name='format';exe='cargo';args=@('fmt','--all','--check')},
        @{name='tests';exe='cargo';args=@('test','--locked','--offline','-p','custom-ui','-p','text-core','-p','editor-model','-p','xtask')},
        @{name='paint-failure';exe='cargo';args=@('test','--locked','--offline','-p','xtask','custom::tests::failed_paint_abandons_unpublished_buffer_and_keeps_document','--','--ignored')},
        @{name='behavior-v2';exe='cargo';args=@('test','--locked','--offline','-p','custom-ui','--features','behavior-v2')},
        @{name='portable';exe='cargo';args=@('check','--locked','--offline','-p','custom-ui','--target','x86_64-unknown-none')},
        @{name='text';exe='pwsh';args=@('-NoProfile','-File','tools/Test-P2-04.ps1')},
        @{name='build';exe='cargo';args=@('build','--locked','--offline','-p','xtask','--profile','m2-comparison')},
        @{name='corpus';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',(Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'),'test','--suite','custom-ui')},
        @{name='selected-a';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',(Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'),'test','--suite','selected-ui')},
        @{name='selected-b';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',(Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'),'test','--suite','selected-ui')},
        @{name='native';exe='pwsh';args=@('-NoProfile','-File','tools/Preview-Custom.ps1','-Replay','-Selected')}
    )
    $rows=@()
    foreach ($command in $commands) {
        $arguments=$command.args;$timer=[Diagnostics.Stopwatch]::StartNew()
        & $command.exe @arguments *> (Join-Path $run ($command.name+'.log'))
        $code=$LASTEXITCODE
        $rows+=@{name=$command.name;program=$command.exe;arguments=$arguments;exit_code=$code;elapsed_ms=$timer.ElapsedMilliseconds}
        $rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'commands.json')
        if ($code) {throw "P2-10 command failed: $($command.name); retained at $run"}
    }
    $replays=@()
    foreach ($name in @('selected-a','selected-b')) {
        $match=Select-String (Join-Path $run ($name+'.log')) -Pattern '^Selected UI checks passed: (.+)$' | Select-Object -Last 1
        if (-not $match) {throw 'Missing selected output'}
        $path=$match.Matches[0].Groups[1].Value.Trim()
        & python tools/Review-Selected.py $path *> (Join-Path $run ($name+'-review.log'))
        if ($LASTEXITCODE) {throw "Independent selected review failed: $name"}
        $replays+=@{path=[IO.Path]::GetRelativePath($root,$path).Replace('\','/');result_sha256=(Get-FileHash (Join-Path $path 'result.json')).Hash.ToLowerInvariant();matrix_sha256=(Get-FileHash (Join-Path $path 'matrix.json')).Hash.ToLowerInvariant();workflows_sha256=(Get-FileHash (Join-Path $path 'workflows.json')).Hash.ToLowerInvariant();visual_text_sha256=(Get-FileHash (Join-Path $path 'visual-text.json')).Hash.ToLowerInvariant()}
    }
    foreach ($key in @('result_sha256','matrix_sha256','workflows_sha256','visual_text_sha256')) {if ($replays[0][$key] -ne $replays[1][$key]) {throw "Selected determinism mismatch: $key"}}
    $match=Select-String (Join-Path $run 'corpus.log') -Pattern '^Custom UI checks passed: (.+)$' | Select-Object -Last 1
    if (-not $match) {throw 'Missing common corpus output'}
    $corpus=$match.Matches[0].Groups[1].Value.Trim()
    & python tools/Review-Custom.py $corpus (Join-Path $corpus 'review') *> (Join-Path $run 'corpus-review.log')
    if ($LASTEXITCODE) {throw 'Independent common corpus review failed'}
    foreach ($inputFile in $testedInputs) {if ((Get-FileHash -LiteralPath $inputFile.path).Hash.ToLowerInvariant() -ne $inputFile.sha256) {throw "Tested input changed during acceptance: $($inputFile.path)"}}
    $inputs=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | ForEach-Object {if (Test-Path -LiteralPath $_ -PathType Leaf) {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}}})
    $inputs | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    git diff --binary | Set-Content (Join-Path $run 'source.patch')
    @{status='passed';source_base=(git rev-parse HEAD);utc=[DateTime]::UtcNow.ToString('o');prerequisites=[IO.Path]::GetRelativePath($root,$Prerequisites).Replace('\','/');commands=$rows;replays=$replays;corpus=[IO.Path]::GetRelativePath($root,$corpus).Replace('\','/');compiler=(& rustc -vV);os=[Environment]::OSVersion.VersionString;cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name);cargo_lock_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant();signed_executable_sha256=(Get-FileHash (Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe')).Hash.ToLowerInvariant();profile='m2-comparison';native_ime=$false;native_assistive=$false;guest=$false;reload=$false;performance_gate=$false} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-10 acceptance: $run"
} finally {Pop-Location}
