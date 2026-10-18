# Explicit experiment command; downloads Cargo-locked inputs, never installs tools.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$manifest = Join-Path $root 'experiments/dependencies/Cargo.toml'
$run = Join-Path $root ("work/dependency-probes/" + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
$results = @()
foreach ($feature in 'architecture','raster','shaping','font','toolkit') {
    & cargo build --locked --manifest-path $manifest --target x86_64-unknown-none --release --no-default-features --features $feature --lib *> (Join-Path $run "$feature-compile.log")
    $compiled = $LASTEXITCODE -eq 0
    & cargo build --locked --manifest-path $manifest --target x86_64-unknown-none --release --no-default-features --features $feature --bin dependency-link *> (Join-Path $run "$feature-link.log")
    $linked = $LASTEXITCODE -eq 0
    $linkText = Get-Content (Join-Path $run "$feature-link.log") -Raw
    $blocker = if ($linked) { $null } elseif ($linkText -match 'no global memory allocator') { 'No guest allocator; linker symbol closure remains unverified' } else { 'See retained link log' }
    $results += [pscustomobject]@{feature=$feature;compile=$(if ($compiled) {'passed'} else {'failed'});link=$(if ($linked) {'passed'} else {'blocked'});blocker=$blocker;runtime='untested';compile_log="$feature-compile.log";link_log="$feature-link.log"}
}
$metadata = (& cargo metadata --locked --manifest-path $manifest --format-version 1 --filter-platform x86_64-unknown-none --no-default-features --features architecture,raster,shaping,font,toolkit | Out-String) | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Dependency metadata collection failed' }
$nodes = @{}
foreach ($node in $metadata.resolve.nodes) { $nodes[$node.id]=$node }
$lockChecksums = @{}
$lockText = Get-Content (Join-Path $root 'experiments/dependencies/Cargo.lock') -Raw
# Read only Cargo's generated package name/version/checksum fields, not general TOML.
foreach ($block in [regex]::Matches($lockText, '(?ms)^\[\[package\]\]\r?\n(.*?)(?=^\[\[package\]\]|\z)')) {
    $name = [regex]::Match($block.Value, '(?m)^name = "([^"]+)"').Groups[1].Value
    $version = [regex]::Match($block.Value, '(?m)^version = "([^"]+)"').Groups[1].Value
    $checksum = [regex]::Match($block.Value, '(?m)^checksum = "([a-f0-9]{64})"').Groups[1].Value
    if ($checksum) { $lockChecksums["$name@$version"]=$checksum }
}
$packages = foreach ($package in $metadata.packages) {
    if (-not $nodes.ContainsKey($package.id)) { continue }
    $checksum = $lockChecksums["$($package.name)@$($package.version)"]
    @{name=$package.name;version=$package.version;license=$package.license;rust_version=$package.rust_version;source=$package.source;checksum=$checksum;features=@($nodes[$package.id].features);dependencies=@($nodes[$package.id].deps | ForEach-Object pkg);build_script=(@($package.targets | Where-Object {$_.kind -contains 'custom-build'}).Count -gt 0);proc_macro=(@($package.targets | Where-Object {$_.kind -contains 'proc-macro'}).Count -gt 0)}
}
$inputs = foreach ($relative in 'experiments/dependencies/Cargo.toml','experiments/dependencies/Cargo.lock','experiments/dependencies/src/lib.rs','experiments/dependencies/src/link.rs') {
    @{path=$relative;sha256=(Get-FileHash (Join-Path $root $relative) -Algorithm SHA256).Hash.ToLowerInvariant()}
}
$report = @{schema_version=1;scope='Resolved Cargo graph includes host build tools; not a shipment inventory';target='x86_64-unknown-none';results=$results;packages=@($packages);inputs=@($inputs);lock_sha256=(Get-FileHash (Join-Path $root 'experiments/dependencies/Cargo.lock') -Algorithm SHA256).Hash.ToLowerInvariant();runtime_tested=$false;toolkit_selected=$false}
$report | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'dependency-report.json')
$workspacePrefix = $root
foreach ($file in Get-ChildItem $run -Filter '*.log') {
    $content = [IO.File]::ReadAllText($file.FullName).Replace($workspacePrefix,'${WORKSPACE}').Replace($env:USERPROFILE,'${USERPROFILE}')
    [IO.File]::WriteAllText($file.FullName,$content)
}
$results | Format-Table feature,compile,link,runtime
"Evidence: $run"
if (@($results | Where-Object compile -eq 'failed').Count) { exit 1 }
