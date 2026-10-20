param([Parameter(Mandatory)][string]$AcceptancePath)
$ErrorActionPreference = 'Stop'
$projectRoot = (Get-Location).Path
$acceptance = Get-Content (Join-Path $AcceptancePath 'acceptance-results.json') -Raw | ConvertFrom-Json -AsHashtable
if ($acceptance.status -ne 'passed') { throw 'Acceptance must pass before retention' }
$destination = Join-Path $projectRoot 'evidence/phase-1/P1-05/2026-10-19'
if (Test-Path -LiteralPath $destination) { throw 'Refusing to overwrite retained evidence' }
New-Item -ItemType Directory $destination | Out-Null
$index = [System.Collections.Generic.List[object]]::new()
function Clean-Text([string]$value) {
    foreach ($prefix in @($projectRoot, $env:USERPROFILE)) {
        $label = if ($prefix -eq $projectRoot) { '${WORKSPACE}' } else { '${USERPROFILE}' }
        foreach ($variant in @($prefix.Replace('\','\\'),$prefix.Replace('\','/'),$prefix,('/cygdrive/' + $prefix.Substring(0,1).ToLowerInvariant() + '/' + $prefix.Substring(3).Replace('\','/')))) { $value = $value.Replace($variant,$label) }
    }
    return $value
}
function Save-File([string]$source,[string]$target,[bool]$verbatim=$false) {
    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
    if ($verbatim) { Copy-Item -LiteralPath $source -Destination $target }
    else { [IO.File]::WriteAllText($target,(Clean-Text ([IO.File]::ReadAllText($source)))) }
    $index.Add(@{path=[IO.Path]::GetRelativePath($destination,$target).Replace('\','/');source=(Clean-Text $source);raw_sha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant();retained_sha256=(Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant();verbatim=$verbatim})
}
function Save-Top([string]$source,[string]$target) {
    foreach ($file in Get-ChildItem -LiteralPath $source -File) {
        if ($file.Extension -in '.json','.log','.txt','.conf') { Save-File $file.FullName (Join-Path $target $file.Name) ($file.Name -eq 'serial.log') }
    }
}
function Save-Tree([string]$source,[string]$target) {
    Save-Top $source $target
    $config = Join-Path $source 'iso root with spaces/boot/limine/limine.conf'
    if (Test-Path $config) { Save-File $config (Join-Path $target 'staged-limine.conf') }
    foreach ($child in Get-ChildItem -LiteralPath $source -Directory) {
        if ($child.Name -notin 'cargo','iso root with spaces','extracted payload') { Save-Tree $child.FullName (Join-Path $target $child.Name) }
    }
}
Save-Top $AcceptancePath (Join-Path $destination 'acceptance')
Save-Top $acceptance.original_acceptance (Join-Path $destination 'superseded/acceptance')
foreach ($case in $acceptance.superseded_runs.Keys) { Save-Tree $acceptance.superseded_runs[$case] (Join-Path $destination "superseded/$case") }
Save-File (Join-Path $projectRoot 'evidence/phase-1/P1-04/2026-10-19/source/kernel/README.md') (Join-Path $destination 'superseded/source/kernel/README.md') $true
foreach ($case in 'run','iso-smoke','iso-negative','boot-metadata','boot-reject','exception-invalid-opcode','exception-page-fault','panic','exception-emergency','memory-ownership','memory-exhaustion','guard-fault','write-protect-fault','execute-disable-fault','emergency-stack') { Save-Tree $acceptance.runs[$case] (Join-Path $destination $case) }
foreach ($gate in 'm0','dependencies','prerequisites') { Save-Top $acceptance.runs[$gate] (Join-Path $destination $gate) }
$inputs = Get-Content (Join-Path $acceptance.runs['run'] 'build-inputs.json') -Raw | ConvertFrom-Json -AsHashtable
foreach ($name in $inputs.sources.Keys) {
    if ((Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash.ToLowerInvariant() -ne $inputs.sources[$name]) { throw "Source changed after acceptance: $name" }
    Save-File (Join-Path $projectRoot $name) (Join-Path $destination "source/$name") $true
}
foreach ($failure in 'build-1792454033627736200-28776','acceptance-p1-05-1792454399054') { Save-Top (Join-Path $projectRoot "build/m1/$failure") (Join-Path $destination "failed-experiments/$failure") }
Save-Top (Join-Path $projectRoot 'work/acceptance/1792454400322') (Join-Path $destination 'failed-experiments/application-control')
Save-File (Join-Path $projectRoot 'work/p1-05-development-observations.txt') (Join-Path $destination 'failed-experiments/development-observations.txt')
Save-File (Join-Path $projectRoot 'work/p1-05-review.json') (Join-Path $destination 'review.json')
Save-File (Join-Path $projectRoot 'work/Review-P1-05.ps1') (Join-Path $destination 'acceptance/review-procedure.ps1') $true
Save-File (Join-Path $projectRoot 'work/Retain-P1-05.ps1') (Join-Path $destination 'acceptance/retention-procedure.ps1') $true
Save-File (Join-Path $projectRoot 'work/Correct-P1-05Inputs.ps1') (Join-Path $destination 'acceptance/input-correction-procedure.ps1') $true
Save-File (Join-Path $projectRoot 'work/Repeat-P1-05Sources.ps1') (Join-Path $destination 'acceptance/source-repeat-procedure.ps1') $true
@{scope='Captured diagnostic artifacts and exact source snapshots; README, checksums and index are excluded';files=$index.ToArray()} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $destination 'evidence-index.json')
"Retained $($index.Count) files: $destination"
