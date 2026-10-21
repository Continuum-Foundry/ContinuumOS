param()
# Read/check only: no enrollment, installation, network override or policy changes.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE = 'true'
$run = Join-Path $root ('build/m2/P2-01/prerequisites-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
Push-Location $root
try {
    git rev-parse HEAD | Set-Content (Join-Path $run 'source-base.txt')
    git status --porcelain=v1 --untracked-files=all | Set-Content (Join-Path $run 'source-status.txt')
    git diff --binary | Set-Content (Join-Path $run 'source.patch')
    $paths = @(git ls-files --cached --others --exclude-standard | Sort-Object -Unique)
    $inputs = foreach ($path in $paths) {
        if (Test-Path -LiteralPath $path -PathType Leaf) {
            @{path=$path;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
        }
    }
    ConvertTo-Json -InputObject @($inputs) -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    $rows = @()
    $commands = @(
        @{name='signing';exe='pwsh';arguments=@('-NoProfile','-File','tools/windows/Test-DevSigning.ps1')},
        @{name='compiler';exe='pwsh';arguments=@('-NoProfile','-File','tools/windows/Test-DevCompiler.ps1')},
        @{name='host-metadata';exe='cargo';arguments=@('metadata','--locked','--offline','--format-version','1','--filter-platform','x86_64-pc-windows-msvc')},
        @{name='doctor';exe='cargo';arguments=@('xtask','doctor')},
        @{name='doctor-json';exe='cargo';arguments=@('xtask','doctor','--json')},
        @{name='build';exe='cargo';arguments=@('xtask','build','--profile','feasibility')},
        @{name='m0';exe='cargo';arguments=@('xtask','test','--suite','m0')},
        @{name='acceptance';exe='pwsh';arguments=@('-NoProfile','-File','tools/Test-M0.ps1')},
        @{name='dependencies';exe='pwsh';arguments=@('-NoProfile','-File','tools/Probe-Dependencies.ps1')},
        @{name='m1-doctor';exe='cargo';arguments=@('xtask','doctor','--profile','m1','--json')}
    )
    foreach ($command in $commands) {
        $timer = [Diagnostics.Stopwatch]::StartNew()
        $arguments = $command.arguments
        & $command.exe @arguments *> (Join-Path $run ($command.name + '.log'))
        $rows += @{name=$command.name;program=$command.exe;arguments=$arguments;exit_code=$LASTEXITCODE;elapsed_ms=$timer.ElapsedMilliseconds}
        # Signing failure makes every later executable check unreliable.
        if ($command.name -eq 'signing' -and $LASTEXITCODE -ne 0) { break }
    }
    . (Join-Path $PSScriptRoot 'windows/DevSigning.ps1')
    try {
        $cert = Get-DevCertificate
        $signing = @{status='passed';has_private_key=$cert.HasPrivateKey;expires_utc=$cert.NotAfter.ToUniversalTime().ToString('o');runner_config_sha256=(Get-FileHash (Join-Path $env:CARGO_HOME 'config.toml')).Hash.ToLowerInvariant()}
    } catch { $signing = @{status='blocked';reason=$_.Exception.Message} }
    $cpu = try { (Get-CimInstance Win32_Processor | Select-Object -First 1).Name } catch { 'unavailable' }
    $cache = Join-Path $env:CARGO_HOME 'registry/src'
    $facilities = @(Get-ChildItem $cache -Directory | Get-ChildItem -Directory | Where-Object Name -Match '^(winit-|softbuffer-|windows-|windows-sys-|image-|png-|tracing-|slint-|i-slint-)')
    $inventory = foreach ($facility in $facilities) {
        @{name=$facility.Name;cargo_manifest_sha256=(Get-FileHash (Join-Path $facility.FullName 'Cargo.toml')).Hash.ToLowerInvariant();notices=@(Get-ChildItem $facility.FullName -File | Where-Object Name -Match '^(LICENSE|COPYING|NOTICE)' | ForEach-Object { @{name=$_.Name;sha256=(Get-FileHash $_.FullName).Hash.ToLowerInvariant()} });scope='cached candidate only; not adopted or linked by P2-01'}
    }
    $hostPackages = @()
    $metadataPath = Join-Path $run 'host-metadata.log'
    if (Test-Path -LiteralPath $metadataPath) {
        try {
            $metadata = Get-Content $metadataPath -Raw | ConvertFrom-Json
            $hostPackages = @(foreach ($package in $metadata.packages) {
                $node = $metadata.resolve.nodes | Where-Object id -eq $package.id
                @{name=$package.name;version=$package.version;features=@($node.features);declared_license=$package.license;source=$package.source}
            })
        } catch { throw 'Cannot read offline host dependency metadata; prerequisite gate incomplete.' }
    }
    $report = @{schema_version=1;status=$(if ($rows.Count -eq $commands.Count -and @($rows | Where-Object exit_code -ne 0).Count -eq 0 -and $signing.status -eq 'passed') {'passed'} else {'blocked'});utc=[DateTime]::UtcNow.ToString('o');cpu=$cpu;os=[Environment]::OSVersion.VersionString;architecture=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString();signing=$signing;commands=$rows;cached_facilities=@($inventory);offline=$true;guest_boot_tested=$false}
    $report['host_packages'] = $hostPackages
    $report | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $run 'prerequisites.json')
    Write-Output "Prerequisite evidence: $run"
    if ($report.status -ne 'passed') { exit 1 }
} finally { Pop-Location }
