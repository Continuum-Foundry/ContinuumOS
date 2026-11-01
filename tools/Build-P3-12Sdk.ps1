param([Parameter(Mandatory)][string]$Output,[switch]$Runtime)
# Three isolated source and Cargo output roots, locked/offline public inputs only.
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$Output=[IO.Path]::GetFullPath($Output)
if (-not $Output.StartsWith($root+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {throw 'Output must stay in workspace.'}
New-Item -ItemType Directory -Force $Output | Out-Null
$images=@();$clientHash=$null
foreach ($variant in @('client','service-old','service-new')) {
    $clean=Join-Path $Output $variant
    if (Test-Path -LiteralPath $clean) {throw "Clean source/output root already exists: $clean"}
    New-Item -ItemType Directory -Force $clean | Out-Null
    $consumer=if ($variant -eq 'client') {'client'} else {'service'}
    $public=@('sdk/continuum-rt',('experiments/sdk-'+$consumer))
    if ($consumer -eq 'client') {$public+=@('crates/editor-model')}
    foreach ($relative in $public) {
        $destination=Join-Path $clean $relative
        New-Item -ItemType Directory -Force (Split-Path $destination -Parent) | Out-Null
        Copy-Item -LiteralPath (Join-Path $root $relative) -Destination $destination -Recurse
    }
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination (Join-Path $clean 'LICENSE')
    # The portable model's inherited package metadata, with no kernel workspace.
    @'
[workspace]
members = ["crates/editor-model"]
resolver = "2"
[workspace.package]
edition = "2024"
publish = false
license-file = "LICENSE"
'@ | Set-Content (Join-Path $clean 'Cargo.toml')
    $manifest=Join-Path $clean ('experiments/sdk-'+$consumer+'/Cargo.toml')
    $env:CARGO_TARGET_DIR=Join-Path $clean 'out'
    $linker=Join-Path $clean 'sdk/continuum-rt/user.ld'
    $env:RUSTFLAGS='-C code-model=small -C relocation-model=static -C no-redzone=yes -C link-arg=--build-id=none -C link-arg=-T'+$linker+' --remap-path-prefix='+$clean+'=/public-sdk'
    $arguments=@('build','--locked','--offline','--release','--target','x86_64-unknown-none','--manifest-path',$manifest)
    if ($Runtime -and $variant -eq 'client') {$arguments+=@('--features','runtime-smoke')}
    if ($variant -eq 'service-new') {$arguments+=@('--features','minor-1')}
    & cargo @arguments *> (Join-Path $Output ($variant+'-build.log'))
    if ($LASTEXITCODE -ne 0) {throw "SDK $variant build failed; see $Output"}
    & cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-unknown-none --manifest-path $manifest > (Join-Path $Output ($variant+'-metadata.json'))
    if ($LASTEXITCODE -ne 0) {throw 'Public dependency graph failed.'}
    $graph=Get-Content (Join-Path $Output ($variant+'-metadata.json')) -Raw | ConvertFrom-Json
    $allowed=if ($consumer -eq 'client') {@('sdk-client','continuum-rt','editor-model','unicode-segmentation','libm')} else {@('sdk-service','continuum-rt')}
    foreach ($package in $graph.packages) {
        if ($package.name -notin $allowed) {throw "Private/unreviewed SDK dependency: $($package.name)"}
        if (-not $package.source -and -not $package.manifest_path.StartsWith($clean+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {throw 'Path dependency escaped clean public source root.'}
    }
    $image=Join-Path $clean 'out/x86_64-unknown-none/release/sdk-image'
    $filename=switch ($variant) {'client' {'user_a.elf'} 'service-old' {'service.elf'} 'service-new' {'user_b.elf'}}
    Copy-Item -LiteralPath $image -Destination (Join-Path $Output $filename)
    $hash=(Get-FileHash -LiteralPath $image).Hash.ToLowerInvariant()
    if ($variant -eq 'client') {$clientHash=$hash}
    elseif ((Get-FileHash -LiteralPath (Join-Path $Output 'user_a.elf')).Hash.ToLowerInvariant() -ne $clientHash) {throw 'Prior client artifact changed during service rebuild.'}
    $inputs=@(Get-ChildItem -LiteralPath $clean -Recurse -File | Where-Object { -not $_.FullName.StartsWith((Join-Path $clean 'out')+[IO.Path]::DirectorySeparatorChar) } | ForEach-Object { @{path=[IO.Path]::GetRelativePath($clean,$_.FullName).Replace('\','/');sha256=(Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant()} })
    $images+=@{variant=$variant;file=$filename;sha256=$hash;bytes=(Get-Item -LiteralPath $image).Length;sdk_version='0.1.0';wire_minor=$(if ($variant -eq 'service-new') {1} else {0});arguments=$arguments;inputs=$inputs;root=$clean;metadata_sha256=(Get-FileHash (Join-Path $Output ($variant+'-metadata.json'))).Hash.ToLowerInvariant()}
}
@{schema_version=1;status='passed';offline=$true;runtime=[bool]$Runtime;unchanged_client_sha256=$clientHash;images=$images} | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $Output 'sdk-builds.json')
"SDK artifacts: $Output"
