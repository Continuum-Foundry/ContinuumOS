# Registry declarations are an inventory input, not a legal approval or shipped SBOM.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$graphs = @(
    @{name='workspace';manifest='Cargo.toml';features=@()},
    @{name='boot-bindings-probe';manifest='experiments/boot-bindings/Cargo.toml';features=@()},
    @{name='candidate-experiments';manifest='experiments/dependencies/Cargo.toml';features=@('--no-default-features','--features','architecture,raster,shaping,font,toolkit')}
)
$records = @{}
$locks = @()
$firstPartyManifests = @('Cargo.toml','tools/xtask/Cargo.toml','kernel/Cargo.toml','crates/editor-model/Cargo.toml',
    'experiments/kernel-link/Cargo.toml','experiments/userland-link/Cargo.toml',
    'experiments/dependencies/Cargo.toml','experiments/boot-bindings/Cargo.toml')
foreach ($graph in $graphs) {
    $manifest = Join-Path $root $graph.manifest
    $lockPath = Join-Path (Split-Path $manifest) 'Cargo.lock'
    $lockRelative = [IO.Path]::GetRelativePath($root,$lockPath).Replace('\','/')
    $locks += [ordered]@{path=$lockRelative;sha256=(Get-FileHash -LiteralPath $lockPath).Hash.ToLowerInvariant()}
    $checksums = @{}
    foreach ($block in [regex]::Matches((Get-Content $lockPath -Raw),'(?ms)^\[\[package\]\]\r?\n(.*?)(?=^\[\[package\]\]|\z)')) {
        $name = [regex]::Match($block.Value,'(?m)^name = "([^"]+)"').Groups[1].Value
        $version = [regex]::Match($block.Value,'(?m)^version = "([^"]+)"').Groups[1].Value
        $checksum = [regex]::Match($block.Value,'(?m)^checksum = "([^"]+)"').Groups[1].Value
        $checksums["$name@$version"] = $checksum
    }
    $arguments = @('metadata','--locked','--offline','--format-version','1','--filter-platform','x86_64-unknown-none','--manifest-path',$manifest) + $graph.features
    $raw = & cargo @arguments
    if ($LASTEXITCODE) { throw "Offline metadata failed: $($graph.name); supply reviewed locked inputs explicitly first" }
    $metadata = ($raw | Out-String) | ConvertFrom-Json
    foreach ($package in $metadata.packages) {
        $packageManifest = [IO.Path]::GetRelativePath($root,$package.manifest_path).Replace('\','/')
        $identity = "$($package.name)@$($package.version)|$($package.source)"
        if (-not $records.ContainsKey($identity)) {
            $records[$identity] = [ordered]@{
                name=$package.name;version=$package.version;source=$package.source
                declared_license=$package.license
                declared_license_file=$(if($package.license_file){[IO.Path]::GetFileName($package.license_file)}else{$null})
                registry_checksum=$(if($package.source){$checksums["$($package.name)@$($package.version)"]}else{$null})
                repository=$package.repository
                graphs=[Collections.Generic.List[string]]::new()
                first_party=($null -eq $package.source -and $packageManifest -in $firstPartyManifests)
            }
        }
        $records[$identity].graphs.Add($graph.name)
    }
}
$destination = Join-Path $root 'legal/dependency-inventory.json'
New-Item -ItemType Directory -Force (Split-Path $destination) | Out-Null
$report = [ordered]@{
    schema_version=1
    scope='Resolved Cargo graphs filtered for x86_64-unknown-none; includes host/candidate packages, not an artifact-level SBOM'
    first_party_rights='All rights reserved; LICENSE applies only to original material'
    distribution_cleared=$false
    method='cargo metadata --locked --offline; declared upstream terms and Cargo.lock registry checksums; no independent legal or signature verification'
    lockfiles=@($locks)
    packages=@($records.GetEnumerator() | Sort-Object Name | ForEach-Object {$_.Value})
}
[IO.File]::WriteAllText($destination,(($report | ConvertTo-Json -Depth 12).Replace("`r`n","`n") + "`n"))
"License declaration inventory: $($report.packages.Count) packages -> legal/dependency-inventory.json"
