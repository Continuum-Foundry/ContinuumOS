param([string]$Prerequisites='')
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
$run=Join-Path $root ('build/m2/P2-12/acceptance-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
Push-Location $root
try {
    if (-not $Prerequisites) {
        & pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1 *> (Join-Path $run 'prerequisites.log')
        if ($LASTEXITCODE) {throw "Prerequisite recheck failed: $run"}
        $match=Select-String (Join-Path $run 'prerequisites.log') -Pattern '^Prerequisite evidence: (.+)$' | Select-Object -Last 1
        if (-not $match) {throw 'Missing prerequisite output'}
        $Prerequisites=$match.Matches[0].Groups[1].Value.Trim()
    }
    $prereq=Get-Content (Join-Path $Prerequisites 'prerequisites.json') -Raw | ConvertFrom-Json
    if ($prereq.status -ne 'passed' -or $prereq.commands.Count -ne 10 -or @($prereq.commands | Where-Object exit_code -ne 0).Count) {throw 'All ten prerequisites must pass'}
    $paths=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | Where-Object {$_ -match '^(crates/|tools/|experiments/|kernel/|config/|fixtures/|Cargo\.|rust-toolchain\.)' -and $_ -match '\.(rs|toml|lock|ps1|py|json)$'})
    $tested=@($paths | ForEach-Object {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}})
    $tested | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $run 'tested-inputs.json')
    $exe=Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'
    $commands=@(
        @{name='format';exe='cargo';args=@('fmt','--all','--check')},
        @{name='tests';exe='cargo';args=@('test','--locked','--offline','-p','custom-ui','-p','editor-model','-p','text-core','-p','xtask')},
        @{name='behavior-v2';exe='cargo';args=@('test','--locked','--offline','-p','custom-ui','--features','behavior-v2')},
        @{name='portable';exe='cargo';args=@('check','--locked','--offline','-p','custom-ui','--target','x86_64-unknown-none')},
        @{name='comparison-feature-check';exe='cargo';args=@('check','--locked','--offline','-p','xtask','--features','slint-candidate')},
        @{name='build';exe='cargo';args=@('build','--locked','--offline','-p','xtask','--profile','m2-comparison')},
        @{name='damage';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',$exe,'test','--suite','ui-damage')},
        @{name='stability';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',$exe,'test','--suite','ui-stability')},
        @{name='reload';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',$exe,'test','--suite','resource-reload')},
        @{name='corpus';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',$exe,'test','--suite','custom-ui')},
        @{name='selected';exe='pwsh';args=@('-NoProfile','-File','tools/windows/Invoke-DevExecutable.ps1',$exe,'test','--suite','selected-ui')},
        @{name='native-reload';exe='pwsh';args=@('-NoProfile','-File','tools/Preview-Custom.ps1','-ResourceReload','-Replay')},
        @{name='native-idle';exe='pwsh';args=@('-NoProfile','-File','tools/Preview-Custom.ps1','-IdleReplay')}
    )
    $environment=@{power_scheme=(& powercfg /getactivescheme);video=@(Get-CimInstance Win32_VideoController | Select-Object Name,CurrentHorizontalResolution,CurrentVerticalResolution,CurrentRefreshRate);cpu_load_start=(Get-CimInstance Win32_Processor | Select-Object Name,LoadPercentage);free_host_memory_start_kib=(Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory;actual_display_cadence='unmeasured; RAM submission is not VSync'}
    $environment | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'host-environment.json')
    $rows=@();$outputs=@{}
    foreach ($command in $commands) {
        $arguments=$command.args;$clock=[Diagnostics.Stopwatch]::StartNew()
        & $command.exe @arguments *> (Join-Path $run ($command.name+'.log'))
        $code=$LASTEXITCODE
        $rows+=@{name=$command.name;program=$command.exe;arguments=$arguments;exit_code=$code;elapsed_ms=$clock.ElapsedMilliseconds}
        $rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'commands.json')
        if ($code) {throw "P2-12 command failed: $($command.name); retained at $run"}
    }
    foreach ($pair in @(@('damage','UI damage checks passed: '),@('stability','UI stability checks passed: '),@('reload','Resource reload checks passed: '),@('corpus','Custom UI checks passed: '),@('selected','Selected UI checks passed: '),@('native-reload','Native preview evidence: '),@('native-idle','Native preview evidence: '))) {
        $match=Select-String (Join-Path $run ($pair[0]+'.log')) -Pattern ('^'+[regex]::Escape($pair[1])+'(.+)$') | Select-Object -Last 1
        if (-not $match) {throw "Missing output for $($pair[0])"}
        $outputs[$pair[0]]=$match.Matches[0].Groups[1].Value.Trim()
    }
    . (Join-Path $PSScriptRoot 'windows/DevSigning.ps1')
    $null=Set-DevExecutableSignature $exe
    $measurement=@()
    for ($rep=0;$rep -lt 5;$rep++) {
        $modes=if ($rep%2) {@('on','off')} else {@('off','on')}
        foreach ($mode in $modes) {
            $directory=Join-Path $run ("measure-$rep-$mode")
            New-Item -ItemType Directory -Force $directory | Out-Null
            $start=[Diagnostics.ProcessStartInfo]::new($exe)
            foreach ($arg in @('ui-performance',$mode,[string]$rep,$directory)) {$start.ArgumentList.Add($arg)}
            $start.WorkingDirectory=$root;$start.UseShellExecute=$false;$start.CreateNoWindow=$true
            $start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
            $process=[Diagnostics.Process]::Start($start)
            $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
            try {
                if (-not $process.WaitForExit(180000)) {$process.Kill($true);$process.WaitForExit();throw "Measurement timed out: $directory"}
                $stdout.Result | Set-Content (Join-Path $directory 'stdout.log');$stderr.Result | Set-Content (Join-Path $directory 'stderr.log')
                if ($process.ExitCode) {throw "Measurement failed: $directory"}
                $measurement+=@{repetition=$rep;trace=$mode;directory=$directory;exit_code=$process.ExitCode;reaped=$true}
                $measurement | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $run 'measurement-index.json')
            } finally {if (-not $process.HasExited) {$process.Kill($true);$process.WaitForExit()};$process.Dispose()}
        }
    }
    & python tools/Review-Custom.py $outputs.corpus (Join-Path $outputs.corpus 'review') *> (Join-Path $run 'corpus-review.log')
    if ($LASTEXITCODE) {throw 'Common corpus independent review failed'}
    & python tools/Review-Selected.py $outputs.selected *> (Join-Path $run 'selected-review.log')
    if ($LASTEXITCODE) {throw 'Selected independent review failed'}
    & python tools/Review-ResourceReload.py $outputs.reload $outputs.'native-reload' *> (Join-Path $run 'reload-review.log')
    if ($LASTEXITCODE) {throw 'Resource reload independent review failed'}
    foreach ($inputFile in $tested) {if ((Get-FileHash -LiteralPath $inputFile.path).Hash.ToLowerInvariant() -ne $inputFile.sha256) {throw "Tested input changed during acceptance: $($inputFile.path)"}}
    git diff --binary | Set-Content (Join-Path $run 'source.patch')
    @{status='passed';source_base=(git rev-parse HEAD);utc=[DateTime]::UtcNow.ToString('o');prerequisites=$Prerequisites;commands=$rows;outputs=$outputs;measurements=$measurement;compiler=(& rustc -vV);os=[Environment]::OSVersion.VersionString;cpu=(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name);cargo_lock_sha256=(Get-FileHash Cargo.lock).Hash.ToLowerInvariant();signed_executable_sha256=(Get-FileHash $exe).Hash.ToLowerInvariant();profile='m2-comparison';guest=$false} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance.json')
    & python tools/Review-P2Performance.py $run *> (Join-Path $run 'performance-review.log')
    if ($LASTEXITCODE) {throw 'Independent P2-12 review failed'}
    Write-Output "P2-12 acceptance: $run"
} finally {Pop-Location}
