$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$acceptanceRoot = Join-Path $projectRoot 'build/m1/acceptance-p1-03-1792348291638'
$acceptance = Get-Content (Join-Path $acceptanceRoot 'acceptance-results.json') -Raw | ConvertFrom-Json -AsHashtable
if ($acceptance.status -ne 'passed') { throw 'Acceptance must pass before retention' }
$destination = Join-Path $projectRoot 'evidence/phase-1/m1-module-refactor/2026-10-18'
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
function Retain([string]$source, [string]$relative) {
    $target = Join-Path $destination $relative
    New-Item -ItemType Directory -Path (Split-Path $target -Parent) -Force | Out-Null
    [IO.File]::WriteAllText($target, (Clean-Text ([IO.File]::ReadAllText($source))))
    $index.Add(@{path=$relative.Replace('\','/');source=(Clean-Text $source);raw_sha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant();retained_sha256=(Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()})
}
$inputs = Get-Content (Join-Path $acceptance.runs.run 'build-inputs.json') -Raw | ConvertFrom-Json -AsHashtable
foreach ($name in $inputs.sources.Keys) {
    if ((Get-FileHash -LiteralPath (Join-Path $projectRoot $name) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $inputs.sources[$name]) { throw "Source changed after acceptance: $name" }
}
foreach ($module in Get-ChildItem (Join-Path $projectRoot 'tools/xtask/src/m1') -File) {
    if (-not $inputs.sources.ContainsKey("tools/xtask/src/m1/$($module.Name)")) { throw "Missing source identity: $module" }
}
Retain (Join-Path $acceptanceRoot 'acceptance-results.json') 'acceptance-results.json'
Retain (Join-Path $projectRoot 'work/m1-module-refactor-review.json') 'review.json'
Retain (Join-Path $acceptance.runs.run 'build-inputs.json') 'build-inputs.json'
Retain (Join-Path $acceptance.runs.m0 'host-tests.log') 'host-tests.log'
Retain (Join-Path $acceptance.runs.m0 'acceptance-results.json') 'phase-0-results.json'
Retain (Join-Path $acceptanceRoot 'kernel-host.log') 'kernel-host.log'
Retain (Join-Path $acceptanceRoot 'dependencies.log') 'dependency-probes.log'
Retain (Join-Path $projectRoot 'work/acceptance/1792348073355/source-path-spaces.log') 'baseline-sandbox-failure.log'
Retain (Join-Path $projectRoot 'work/acceptance/1792348130289/acceptance-results.json') 'baseline-phase-0-results.json'
foreach ($case in 'run','iso-smoke','iso-negative','boot-metadata','boot-reject') {
    Retain (Join-Path $acceptance.runs[$case] 'result.json') "$case/result.json"
    Retain (Join-Path $acceptance.runs[$case] 'build-manifest.json') "$case/build-manifest.json"
}
Retain (Join-Path $projectRoot 'work/Review-M1ModuleRefactor.ps1') 'review-procedure.ps1'
Retain $PSCommandPath 'retention-procedure.ps1'
@{schema_version=1;scope='Sanitized verification summaries and source-input inventory; raw run artifacts remain in the indexed workspace paths. README and this index are excluded.';files=$index.ToArray()} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $destination 'evidence-index.json')
"Retained $($index.Count) verification artifacts: $destination"
