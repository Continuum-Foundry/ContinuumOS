param()
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$run=Join-Path $root ('build/m2/P2-06/acceptance-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
Push-Location $root
try {
    & (Join-Path $PSScriptRoot 'Supply-TextInputs.ps1')
    $exe=Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'
    $runner=Join-Path $PSScriptRoot 'windows/Invoke-DevExecutable.ps1'
    # The interpreter is deliberately a separate offline resolution attempt. It is
    # neither enabled in the compiled candidate nor required by the guest proposal.
    $interpreter=Join-Path $run 'interpreter'
    New-Item -ItemType Directory -Force $interpreter | Out-Null
    @('[workspace]','[package]','name = "p2-06-interpreter-probe"','version = "0.1.0"','edition = "2024"','[dependencies]','slint-interpreter = "=1.18.1"') | Set-Content (Join-Path $interpreter 'Cargo.toml')
    'fn main() {}' | Set-Content (Join-Path $interpreter 'main.rs')
    Add-Content (Join-Path $interpreter 'Cargo.toml') @('[[bin]]','name = "interpreter-probe"','path = "main.rs"')
    $commands=@(
        @{name='prerequisites';exe='pwsh';args=@('-NoProfile','-File','tools/Recheck-P2Prerequisites.ps1')},
        @{name='format';exe='cargo';args=@('fmt','--all','--check')},
        @{name='inputs';exe='python';args=@('tools/Review-SlintInputs.py',(Join-Path $run 'inputs'))},
        @{name='tests';exe='cargo';args=@('test','--locked','--offline','-p','custom-ui','-p','editor-model','-p','text-core','-p','xtask')},
        @{name='text';exe='pwsh';args=@('-NoProfile','-File','tools/Test-P2-04.ps1')},
        @{name='behavior-build';exe='cargo';args=@('build','--locked','--offline','-p','xtask','--features','slint-candidate,behavior-v2','--profile','m2-comparison')},
        @{name='behavior-v2';exe='pwsh';args=@('-NoProfile','-File',$runner,$exe,'slint-behavior')},
        @{name='build';exe='cargo';args=@('build','--locked','--offline','-p','xtask','--features','slint-candidate','--profile','m2-comparison')},
        @{name='behavior-v1';exe='pwsh';args=@('-NoProfile','-File',$runner,$exe,'slint-behavior')},
        @{name='native-text';exe='pwsh';args=@('-NoProfile','-File',$runner,$exe,'slint-native-text-probe');expected='No font fallback found'},
        @{name='interpreter';exe='cargo';args=@('metadata','--offline','--format-version','1','--manifest-path',(Join-Path $interpreter 'Cargo.toml'));expected='no matching package named `slint-interpreter`'},
        @{name='slint-a';exe='pwsh';args=@('-NoProfile','-File',$runner,$exe,'test','--suite','slint-ui')},
        @{name='slint-b';exe='pwsh';args=@('-NoProfile','-File',$runner,$exe,'test','--suite','slint-ui')},
        @{name='custom-regression';exe='pwsh';args=@('-NoProfile','-File',$runner,$exe,'test','--suite','custom-ui')},
        @{name='native';exe='pwsh';args=@('-NoProfile','-File','tools/Preview-Slint.ps1','-Replay')}
    )
    $order=@('inputs','format','behavior-build','behavior-v2','build','behavior-v1','native-text','interpreter','slint-a','slint-b','custom-regression','native','prerequisites','tests','text')
    $commands=@($order | ForEach-Object {$name=$_;$commands | Where-Object name -eq $name})
    $rows=@()
    foreach ($command in $commands) {
        $arguments=$command.args;$timer=[Diagnostics.Stopwatch]::StartNew()
        & $command.exe @arguments *> (Join-Path $run ($command.name+'.log'))
        $code=$LASTEXITCODE
        $passed=$code -eq 0
        if ($command.ContainsKey('expected')) {$passed=$code -ne 0 -and (Select-String -LiteralPath (Join-Path $run ($command.name+'.log')) -SimpleMatch $command.expected -Quiet)}
        $row=@{name=$command.name;program=$command.exe;arguments=$arguments;exit_code=$code;expected_failure=$command.ContainsKey('expected');passed=[bool]$passed;elapsed_ms=$timer.ElapsedMilliseconds}
        if ($command.name -in 'behavior-v2','behavior-v1') {$row['signed_executable_sha256']=(Get-FileHash $exe).Hash.ToLowerInvariant()}
        $rows+=$row
        $rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'commands.json')
        if (-not $passed) {throw "P2-06 command failed: $($command.name); retained at $run"}
    }
    $replays=@()
    foreach ($name in @('slint-a','slint-b')) {
        $match=Select-String -LiteralPath (Join-Path $run ($name+'.log')) -Pattern '^Slint UI checks passed: (.+)$' | Select-Object -Last 1
        if (-not $match) {throw 'Missing Slint run output'}
        $path=$match.Matches[0].Groups[1].Value.Trim()
        $result=Get-Content (Join-Path $path 'result.json') -Raw | ConvertFrom-Json
        if ($result.status -ne 'passed' -or -not $result.goldens_checked -or $result.capture_count -ne 150 -or $result.workflow_steps -ne 300 -or $result.text.Count -ne 45 -or $result.text_nodes.Count -ne 63) {throw 'Incomplete candidate matrix or missing reviewed goldens'}
        & python tools/Review-Slint.py $path (Join-Path $path 'review') *> (Join-Path $run ($name+'-pixels.log'))
        if ($LASTEXITCODE) {throw 'Independent pixel checks failed'}
        $replays+=@{path=[IO.Path]::GetRelativePath($root,$path).Replace('\','/');raster_manifest_sha256=(Get-FileHash (Join-Path $path 'raster-manifest.json')).Hash.ToLowerInvariant();result_sha256=(Get-FileHash (Join-Path $path 'result.json')).Hash.ToLowerInvariant();traces_sha256=(Get-FileHash (Join-Path $path 'traces.json')).Hash.ToLowerInvariant()}
    }
    if ($replays[0].result_sha256 -ne $replays[1].result_sha256 -or $replays[0].raster_manifest_sha256 -ne $replays[1].raster_manifest_sha256 -or $replays[0].traces_sha256 -ne $replays[1].traces_sha256) {throw 'Separate semantic/raster runs differ'}
    $inputs=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | ForEach-Object {if (Test-Path -LiteralPath $_ -PathType Leaf) {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}}})
    $inputs | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    @{schema_version=1;status='passed';source_base=(git rev-parse HEAD);utc=[DateTime]::UtcNow.ToString('o');commands=$rows;replays=$replays;compiler=(& rustc -vV);os=[Environment]::OSVersion.VersionString;cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name);cargo_lock_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant();signed_executable_sha256=(Get-FileHash $exe).Hash.ToLowerInvariant();profile='m2-comparison';candidate_rendered=$true;renderer='Slint 1.18.1 AOT software; P2-04 text Image adapter';native_text_passed=$false;rich_effect_passed=$false;rounded_ancestor_clip_passed=$false;interpreter_available=$false;guest_tested=$false;native_ime_tested=$false;native_accessibility_tested=$false;reload_tested=$false;performance_gate_tested=$false} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-06 acceptance: $run"
}finally{Pop-Location}
