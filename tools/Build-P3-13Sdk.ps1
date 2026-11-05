param([Parameter(Mandatory)][string]$Output,[switch]$CacheBuild)
# Isolated public-only SDK stress consumer. No kernel source is copied or linked.
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$Output=[IO.Path]::GetFullPath($Output)
if (-not $Output.StartsWith($root+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {throw 'Output must stay in workspace.'}
if (-not $CacheBuild) {
    if (Test-Path -LiteralPath (Join-Path $Output 'public-stress')) {throw 'Public stress output already exists.'}
    # One exact public ELF per measured source/compiler set. Fresh Cargo roots
    # can vary non-semantic crate identities and linker layout; paired guests
    # must consume the identical bytes, not merely equivalent source.
    $paths=@('tools/Build-P3-13Sdk.ps1','tools/Enter-Dev.ps1','.cargo/config.toml','LICENSE','experiments/sdk-service/src/policy.rs','experiments/sdk-service/src/routes.rs')
    foreach ($directory in @('sdk/continuum-rt','experiments/sdk-stress')) {
        $paths+=@(Get-ChildItem -LiteralPath (Join-Path $root $directory) -Recurse -File | ForEach-Object {[IO.Path]::GetRelativePath($root,$_.FullName).Replace('\','/')})
    }
    $keyInputs=[ordered]@{}
    foreach ($path in @($paths | Sort-Object -Unique)) {$keyInputs[$path]=(Get-FileHash -LiteralPath (Join-Path $root $path)).Hash.ToLowerInvariant()}
    $compiler=(Get-Command rustc).Source
    $keyInputs['compiler-executable']=(Get-FileHash -LiteralPath $compiler).Hash.ToLowerInvariant()
    $keyInputs['compiler-version']=(& rustc -vV | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {throw 'Compiler identity failed.'}
    $compilerRoot=(& rustc --print sysroot | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {throw 'Compiler backend identity failed.'}
    $keyInputs['compiler-backend']=(Get-FileHash -LiteralPath (Join-Path $compilerRoot ('bin/rustc'+$script:ContinuumExeSuffix))).Hash.ToLowerInvariant()
    $keyBytes=[Text.Encoding]::UTF8.GetBytes(($keyInputs | ConvertTo-Json -Compress))
    $key=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($keyBytes)).ToLowerInvariant()
    $cache=Join-Path $root ('build/m3/P3-13/public-cache/'+$key)
    $reused=Test-Path -LiteralPath (Join-Path $cache 'stress-builds.json')
    if (-not $reused) {
        & $PSCommandPath -Output $cache -CacheBuild
        if ($LASTEXITCODE -ne 0) {throw 'Canonical public build failed.'}
    }
    $cached=Get-Content (Join-Path $cache 'stress-builds.json') -Raw | ConvertFrom-Json
    foreach ($inputFile in $cached.inputs) {
        $source=Join-Path $root $inputFile.path
        $snapshot=Join-Path $cache ('public-stress/'+$inputFile.path)
        if ((Get-FileHash -LiteralPath $snapshot).Hash.ToLowerInvariant() -ne $inputFile.sha256 -or (Get-FileHash -LiteralPath $source).Hash.ToLowerInvariant() -ne $inputFile.sha256) {throw 'Changed cached public source.'}
    }
    foreach ($name in @('user_a.elf','user_b.elf','service.elf')) {
        if ((Get-FileHash -LiteralPath (Join-Path $cache $name)).Hash.ToLowerInvariant() -ne $cached.sha256) {throw 'Changed cached public ELF.'}
    }
    New-Item -ItemType Directory -Force $Output | Out-Null
    foreach ($item in @(Get-ChildItem -LiteralPath $cache)) {Copy-Item -LiteralPath $item.FullName -Destination (Join-Path $Output $item.Name) -Recurse}
    $cached | Add-Member -NotePropertyName canonical_root -NotePropertyValue (Join-Path $cache 'public-stress')
    $cached | Add-Member -NotePropertyName cache_key -NotePropertyValue $key
    $cached | Add-Member -NotePropertyName cache_inputs -NotePropertyValue $keyInputs
    $cached | Add-Member -NotePropertyName reused_artifact -NotePropertyValue $reused
    $cached | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $Output 'stress-builds.json')
    "Stress SDK artifacts: $Output"
    return
}
$clean=Join-Path $Output 'public-stress'
if (Test-Path -LiteralPath $clean) {throw 'Public stress source root already exists.'}
New-Item -ItemType Directory -Force $clean | Out-Null
foreach ($relative in @('sdk/continuum-rt','experiments/sdk-stress')) {
    $destination=Join-Path $clean $relative
    New-Item -ItemType Directory -Force (Split-Path $destination -Parent) | Out-Null
    Copy-Item -LiteralPath (Join-Path $root $relative) -Destination $destination -Recurse
}
New-Item -ItemType Directory -Force (Join-Path $clean 'experiments/sdk-service/src') | Out-Null
foreach ($name in @('policy.rs','routes.rs')) {Copy-Item -LiteralPath (Join-Path $root ('experiments/sdk-service/src/'+$name)) -Destination (Join-Path $clean ('experiments/sdk-service/src/'+$name))}
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination (Join-Path $clean 'LICENSE')
$manifest=Join-Path $clean 'experiments/sdk-stress/Cargo.toml'
$env:CARGO_TARGET_DIR=Join-Path $clean 'out'
$env:CARGO_NET_OFFLINE='true'
$env:RUSTFLAGS='-C code-model=small -C relocation-model=static -C no-redzone=yes -C link-arg=--build-id=none -C link-arg=-T'+(Join-Path $clean 'sdk/continuum-rt/user.ld')+' --remap-path-prefix='+$clean+'=/public-stress'
$arguments=@('build','--locked','--offline','--release','--target','x86_64-unknown-none','--manifest-path',$manifest)
& cargo @arguments *> (Join-Path $Output 'stress-public-build.log')
if ($LASTEXITCODE -ne 0) {throw "Public stress build failed: $Output"}
& cargo metadata --locked --offline --format-version 1 --manifest-path $manifest > (Join-Path $Output 'stress-public-metadata.json')
if ($LASTEXITCODE -ne 0) {throw 'Public stress dependency graph failed.'}
$graph=Get-Content (Join-Path $Output 'stress-public-metadata.json') -Raw | ConvertFrom-Json
foreach ($package in $graph.packages) {
    if ($package.name -notin @('sdk-stress','continuum-rt') -or $package.source -or -not $package.manifest_path.StartsWith($clean+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {throw 'Private/unreviewed stress dependency.'}
}
$image=Join-Path $clean 'out/x86_64-unknown-none/release/sdk-image'
foreach ($name in @('user_a','user_b','service')) {Copy-Item -LiteralPath $image -Destination (Join-Path $Output ($name+'.elf'))}
$inputs=@(Get-ChildItem -LiteralPath $clean -Recurse -File | Where-Object {-not $_.FullName.StartsWith((Join-Path $clean 'out')+[IO.Path]::DirectorySeparatorChar)} | ForEach-Object {@{path=[IO.Path]::GetRelativePath($clean,$_.FullName).Replace('\','/');sha256=(Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant()}})
@{schema_version=1;status='passed';public_sdk=$true;arguments=$arguments;inputs=$inputs;sha256=(Get-FileHash $image).Hash.ToLowerInvariant();roles='same reviewed public consumer ELF, trusted distinct startup roles'} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $Output 'stress-builds.json')
"Stress SDK artifacts: $Output"
