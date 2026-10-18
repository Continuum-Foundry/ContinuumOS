param([string]$RecordedDate)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
if (-not $RecordedDate) {
    $zone = [TimeZoneInfo]::FindSystemTimeZoneById('Eastern Standard Time')
    $RecordedDate = [TimeZoneInfo]::ConvertTimeFromUtc([DateTime]::UtcNow,$zone).ToString('yyyy-MM-dd')
}
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$destination = Join-Path $root 'evidence/phase-0'
$logs = Join-Path $destination 'logs'
New-Item -ItemType Directory -Force $logs | Out-Null
function Save-Json($name,$value) {
    $text = ($value | ConvertTo-Json -Depth 30).Replace("`r`n","`n") + "`n"
    [IO.File]::WriteAllText((Join-Path $destination $name),$text)
}
function Read-Json($path) { Get-Content -LiteralPath $path -Raw | ConvertFrom-Json }
function Latest($path) { (Get-ChildItem -LiteralPath $path -Directory | Sort-Object Name -Descending | Select-Object -First 1).FullName }
function Assert-Inputs($inputs) {
    foreach ($inputFile in $inputs) {
        if ((Get-FileHash (Join-Path $root $inputFile.path) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $inputFile.sha256) { throw "Stale evidence input: $($inputFile.path). Re-run checks." }
    }
}
function Copy-Log($source,$name) {
    $content = [IO.File]::ReadAllText($source).Replace($root,'${WORKSPACE}').Replace($env:USERPROFILE,'${USERPROFILE}')
    [IO.File]::WriteAllText((Join-Path $logs $name),$content.Replace("`r`n","`n"))
}
$verification = (Get-ChildItem (Join-Path $root 'build/m0') -Directory -Filter 'verification-*' | Sort-Object Name -Descending | Select-Object -First 1).FullName
$m0 = Read-Json (Join-Path $verification 'm0-results.json')
if ($m0.status -ne 'passed') { throw 'No passing M0 result' }
Assert-Inputs $m0.build.inputs
$acceptance = Latest (Join-Path $root 'work/acceptance')
$accepted = Read-Json (Join-Path $acceptance 'acceptance-results.json')
if ($accepted.status -ne 'passed') { throw 'No passing acceptance result' }
$dependencyRun = Latest (Join-Path $root 'work/dependency-probes')
$dependencies = Read-Json (Join-Path $dependencyRun 'dependency-report.json')
Assert-Inputs $dependencies.inputs
if (@($dependencies.results | Where-Object compile -ne 'passed').Count) { throw 'Unresolved dependency compile failure' }
Save-Json 'm0-results.json' $m0
Save-Json 'acceptance-results.json' $accepted
Save-Json 'doctor-tests.json' (Read-Json (Join-Path $acceptance 'doctor-tests.json'))
Save-Json 'dependency-report.json' $dependencies
$environment = (& pwsh -NoProfile -File (Join-Path $root 'tools/Inventory.ps1') -Json | Out-String) | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Environment inventory failed' }
Save-Json 'environment-report.json' $environment
foreach ($log in Get-ChildItem $verification -File -Filter '*.txt') { Copy-Log $log.FullName $log.Name }
foreach ($log in Get-ChildItem (Join-Path $verification 'clean-a') -File -Filter '*.log') { Copy-Log $log.FullName $log.Name }
foreach ($log in Get-ChildItem $dependencyRun -File -Filter '*.log') { Copy-Log $log.FullName $log.Name }
foreach ($name in 'host-tests.log','compile-tests.log','source-path-spaces.log') { Copy-Log (Join-Path $acceptance $name) $name }
$initialFailure = Join-Path $root 'work/dependency-first-build.log'
if (Test-Path $initialFailure) { Copy-Log $initialFailure 'initial-slint-pixel-type-failure.log' }
foreach ($name in 'doc-audit-results.json','contract-validation-results.json') {
    $source = Join-Path $root "work/$name"
    if (Test-Path $source) { Save-Json $name (Read-Json $source) }
}
$distributionPath = Join-Path $root 'work/downloads/channel-rust-1.99.1.toml'
$distributionText = Get-Content $distributionPath -Raw
$distributions = foreach ($block in ($distributionText -split '\r?\n\r?\n')) {
    if ($block -match '^\[pkg\.(rustc|cargo|rust-std|llvm-tools-preview|rustfmt-preview)\.target\.(x86_64-pc-windows-msvc|x86_64-unknown-none)\]' -and $block -match 'available = true') {
        @{package=([regex]::Match($block,'^\[([^\]]+)\]').Groups[1].Value);url=([regex]::Match($block,'(?m)^xz_url = "([^"]+)"').Groups[1].Value);sha256=([regex]::Match($block,'(?m)^xz_hash = "([^"]+)"').Groups[1].Value)}
    }
}
$sysroot = & rustc --print sysroot
$executables = foreach ($relative in 'bin/rustc.exe','bin/cargo.exe','lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe','lib/rustlib/x86_64-pc-windows-msvc/bin/llvm-readobj.exe','lib/rustlib/x86_64-pc-windows-msvc/bin/llvm-nm.exe') {
    @{path=$relative;sha256=(Get-FileHash (Join-Path $sysroot $relative) -Algorithm SHA256).Hash.ToLowerInvariant()}
}
$manifest = @{
    schema_version=1;recorded_date=$RecordedDate;timezone='America/New_York';source_base_commit=(& git -C $root rev-parse HEAD);dirty_source=$true;source_identity='Build input hashes in m0-results.json and dependency-report.json; not a committed build'
    compiler=(& rustc -vV | Out-String).Trim();cargo=(& cargo -V);host='x86_64-pc-windows-msvc';guest='x86_64-unknown-none';installation_scope='workspace-local; no persistent PATH or Windows feature changes'
    bootstrap=@{version='1.29.1';url='https://static.rust-lang.org/rustup/archive/1.29.1/x86_64-pc-windows-msvc/rustup-init.exe';sha256='6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e';license='MIT OR Apache-2.0'}
    rust_distribution_manifest=@{url='https://static.rust-lang.org/dist/channel-rust-1.99.1.toml';sha256=(Get-FileHash $distributionPath -Algorithm SHA256).Hash.ToLowerInvariant()};distributions=@($distributions);installed_executables=@($executables)
    boot_candidates=@{limine=@{version='12.9.3';url='https://github.com/Limine-Bootloader/Limine/releases/download/v12.9.3/limine-binary.zip';upstream_reported_sha256='1bdbbcda650c5dba9b1eca5833d598137bf11bd6289bc89b6bcc4f8f112fa676';downloaded=$false;boot_tested=$false;license='BSD-2-Clause; redistribution notices must be verified from archive'};rust_osdev=@{version='0.11.17';source='https://crates.io/crates/bootloader/0.11.17';license='MIT OR Apache-2.0';source_reviewed=$true;build_tested=$false;needs_nightly=$true;iso_path='unverified'}}
    vm=@{qemu_version=$null;firmware_sha256=$null;machine='q35; pin versioned machine identifier after tool supply';vcpus=1;ram_mib=2048;accelerator='tcg planned; not executed';network=$false;host_shares=$false;boot_tested=$false}
}
Save-Json 'toolchain-manifest.json' $manifest
$checksums = foreach ($file in Get-ChildItem $destination -Recurse -File | Where-Object Name -ne 'SHA256SUMS.json' | Sort-Object FullName) {
    @{path=[IO.Path]::GetRelativePath($destination,$file.FullName).Replace('\','/');bytes=$file.Length;sha256=(Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
}
Save-Json 'SHA256SUMS.json' @{algorithm='SHA-256';files=@($checksums)}
"Evidence recorded: $destination"
