param()
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$run=Join-Path $root ('build/m2/P2-05/acceptance-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
Push-Location $root
try {
    & (Join-Path $PSScriptRoot 'Supply-TextInputs.ps1')
    $commands=@(
        @{name='prerequisites';exe='pwsh';args=@('-NoProfile','-File','tools/Recheck-P2Prerequisites.ps1')},
        @{name='format';exe='cargo';args=@('fmt','--all','--check')},
        @{name='tests';exe='cargo';args=@('test','--locked','--offline','-p','custom-ui','-p','editor-model','-p','text-core','-p','xtask')},
        @{name='paint-failure';exe='cargo';args=@('test','--locked','--offline','-p','xtask','custom::tests::failed_paint_abandons_unpublished_buffer_and_keeps_document','--','--ignored')},
        @{name='behavior-v2';exe='cargo';args=@('test','--locked','--offline','-p','custom-ui','--features','behavior-v2')},
        @{name='portable';exe='cargo';args=@('check','--locked','--offline','-p','custom-ui','--target','x86_64-unknown-none')},
        @{name='text';exe='pwsh';args=@('-NoProfile','-File','tools/Test-P2-04.ps1')},
        @{name='build';exe='cargo';args=@('build','--locked','--offline','-p','xtask','--profile','m2-comparison')},
        @{name='custom-a';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',(Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'),'test','--suite','custom-ui')},
        @{name='custom-b';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',(Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'),'test','--suite','custom-ui')},
        @{name='native';exe='pwsh';args=@('-NoProfile','-File','tools/Preview-Custom.ps1','-Replay')}
    )
    $rows=@()
    foreach ($command in $commands) {
        $arguments=$command.args;$timer=[Diagnostics.Stopwatch]::StartNew()
        & $command.exe @arguments *> (Join-Path $run ($command.name+'.log'))
        $rows+=@{name=$command.name;program=$command.exe;arguments=$arguments;exit_code=$LASTEXITCODE;elapsed_ms=$timer.ElapsedMilliseconds}
        $rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'commands.json')
        if ($LASTEXITCODE -ne 0) {throw "P2-05 command failed: $($command.name); retained at $run"}
    }
    $replays=@()
    foreach ($name in @('custom-a','custom-b')) {
        $match=Select-String -LiteralPath (Join-Path $run ($name+'.log')) -Pattern '^Custom UI checks passed: (.+)$' | Select-Object -Last 1
        if (-not $match) {throw 'Missing custom run output'}
        $path=$match.Matches[0].Groups[1].Value.Trim()
        $result=Get-Content (Join-Path $path 'result.json') -Raw | ConvertFrom-Json
        if ($result.status -ne 'passed' -or -not $result.goldens_checked -or $result.capture_count -ne 150 -or $result.workflow_steps -ne 300 -or $result.text.Count -ne 45 -or $result.text_nodes.Count -ne 63) {throw 'Incomplete candidate matrix or missing reviewed goldens'}
        & python tools/Review-Custom.py $path (Join-Path $path 'review') *> (Join-Path $run ($name+'-pixels.log'))
        if ($LASTEXITCODE) {throw 'Independent pixel checks failed'}
        $replays+=@{path=[IO.Path]::GetRelativePath($root,$path).Replace('\','/');raster_manifest_sha256=(Get-FileHash (Join-Path $path 'raster-manifest.json')).Hash.ToLowerInvariant();result_sha256=(Get-FileHash (Join-Path $path 'result.json')).Hash.ToLowerInvariant();traces_sha256=(Get-FileHash (Join-Path $path 'traces.json')).Hash.ToLowerInvariant()}
    }
    if ($replays[0].result_sha256 -ne $replays[1].result_sha256 -or $replays[0].raster_manifest_sha256 -ne $replays[1].raster_manifest_sha256 -or $replays[0].traces_sha256 -ne $replays[1].traces_sha256) {throw 'Separate semantic/raster runs differ'}
    $inputs=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | ForEach-Object {if (Test-Path -LiteralPath $_ -PathType Leaf) {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}}})
    $inputs | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    & cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-unknown-none > (Join-Path $run 'metadata.json')
    if ($LASTEXITCODE) {throw 'Dependency metadata missing'}
    @{schema_version=1;status='passed';source_base=(git rev-parse HEAD);utc=[DateTime]::UtcNow.ToString('o');commands=$rows;replays=$replays;compiler=(& rustc -vV);os=[Environment]::OSVersion.VersionString;cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name);cargo_lock_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant();signed_executable_sha256=(Get-FileHash (Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe')).Hash.ToLowerInvariant();profile='m2-comparison';candidate_rendered=$true;guest_tested=$false;native_ime_tested=$false;native_accessibility_tested=$false;reload_tested=$false;performance_gate_tested=$false} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-05 acceptance: $run"
}finally{Pop-Location}
