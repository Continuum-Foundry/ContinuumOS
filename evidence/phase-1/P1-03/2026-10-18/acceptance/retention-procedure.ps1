$ErrorActionPreference = 'Stop'
$projectRoot = (Get-Location).Path
$acceptanceRoot = Join-Path $projectRoot 'build/m1/acceptance-p1-03-1792346763755'
$acceptance = Get-Content (Join-Path $acceptanceRoot 'acceptance-results.json') -Raw | ConvertFrom-Json -AsHashtable
if ($acceptance.status -ne 'passed') { throw 'Acceptance must pass before retention/review' }
$destination = Join-Path $projectRoot 'evidence/phase-1/P1-03/2026-10-18'
if (Test-Path -LiteralPath $destination) { throw 'Refusing to overwrite retained evidence' }
New-Item -ItemType Directory -Path $destination -Force | Out-Null
$index = [System.Collections.Generic.List[object]]::new()
function Clean-Text([string]$value) {
    foreach ($prefix in @($projectRoot, $env:USERPROFILE)) {
        $label = if ($prefix -eq $projectRoot) { '${WORKSPACE}' } else { '${USERPROFILE}' }
        $value = $value.Replace($prefix.Replace('\','\\'),$label).Replace($prefix.Replace('\','/'),$label).Replace($prefix,$label)
    }
    return $value
}
function Retain-File([string]$source, [string]$target, [bool]$verbatim = $false) {
    New-Item -ItemType Directory -Path (Split-Path $target -Parent) -Force | Out-Null
    $rawHash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($verbatim) { Copy-Item -LiteralPath $source -Destination $target }
    else { [IO.File]::WriteAllText($target, (Clean-Text ([IO.File]::ReadAllText($source)))) }
    $index.Add(@{path=$target.Substring($destination.Length+1).Replace('\','/');source=(Clean-Text $source);raw_sha256=$rawHash;retained_sha256=(Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant();verbatim=$verbatim})
}
function Retain-Top([string]$source, [string]$target) {
    foreach ($file in Get-ChildItem -LiteralPath $source -File) {
        if ($file.Extension -in '.json','.log','.txt','.conf') { Retain-File $file.FullName (Join-Path $target $file.Name) }
    }
}
function Retain-Tree([string]$source, [string]$target) {
    Retain-Top $source $target
    $configuration = Join-Path $source 'iso root with spaces/boot/limine/limine.conf'
    if (Test-Path -LiteralPath $configuration) { Retain-File $configuration (Join-Path $target 'staged-limine.conf') }
    $module = Join-Path $source 'iso root with spaces/boot/fixture.bin'
    if (Test-Path -LiteralPath $module) { Retain-File $module (Join-Path $target 'module-fixture.bin') $true }
    foreach ($directory in Get-ChildItem -LiteralPath $source -Directory) {
        if ($directory.Name -notin 'cargo','iso root with spaces','extracted payload') {
            Retain-Tree $directory.FullName (Join-Path $target $directory.Name)
        }
    }
}
Retain-Top $acceptanceRoot (Join-Path $destination 'acceptance')
foreach ($case in 'run','iso-smoke','iso-negative','boot-metadata','boot-reject') {
    Retain-Tree $acceptance.runs[$case] (Join-Path $destination $case)
}
foreach ($gate in 'm0','dependencies','prerequisites') {
    Retain-Top $acceptance.runs[$gate] (Join-Path $destination $gate)
}
foreach ($trial in 'boot-metadata-1792345390047482600-26852','boot-metadata-1792345493499238800-34296') {
    Retain-Tree (Join-Path $projectRoot "build/m1/$trial") (Join-Path $destination "failed-experiments/$trial")
}
Retain-Tree (Join-Path $projectRoot 'build/m1/run-1792345904537412600-33412') (Join-Path $destination 'failed-experiments/image-boundary-review')
Retain-Top (Join-Path $projectRoot 'build/m1/acceptance-p1-03-1792345817176') (Join-Path $destination 'failed-experiments/superseded-acceptance')
Retain-File (Join-Path $projectRoot 'work/p1-03-image-boundary-review.json') (Join-Path $destination 'failed-experiments/image-boundary-review/review.json')
$inputs = Get-Content (Join-Path $acceptance.runs['run'] 'build-inputs.json') -Raw | ConvertFrom-Json -AsHashtable
foreach ($name in $inputs.sources.Keys) {
    $source = Join-Path $projectRoot $name
    if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $inputs.sources[$name]) { throw "Source changed after acceptance build: $name" }
    Retain-File $source (Join-Path $destination "source/$name") $true
}
Retain-File (Join-Path $projectRoot 'work/Review-P1-03.ps1') (Join-Path $destination 'acceptance/review-procedure.ps1') $true
Retain-File (Join-Path $projectRoot 'work/Retain-P1-03.ps1') (Join-Path $destination 'acceptance/retention-procedure.ps1') $true
@{schema_version=1;scope='Captured artifacts and exact source snapshots; review files and this index are excluded';files=$index.ToArray()} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $destination 'evidence-index.json')
"Retained $($index.Count) artifacts: $destination"
