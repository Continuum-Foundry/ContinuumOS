param(
    [Parameter(Mandatory)][string]$AcceptancePath,
    [string[]]$FailedRunPaths = @(),
    [string]$BaselineM0Path,
    [string]$DependencyPath
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$destination = Join-Path $root 'evidence/phase-1/P1-02/2026-10-18'
$accepted = Get-Content (Join-Path $AcceptancePath 'acceptance-results.json') -Raw | ConvertFrom-Json
if ($accepted.status -ne 'passed') { throw 'A passing complete acceptance result is required' }
New-Item -ItemType Directory -Force $destination | Out-Null
function Save-Text([string]$source,[string]$target) {
    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
    # This slice's serial stream contains only firmware/loader/kernel diagnostics
    # and internal ISO paths. Keep its exact bytes, including CR/LF and ANSI.
    if ((Split-Path $source -Leaf) -eq 'serial.log') {
        [IO.File]::WriteAllBytes($target,[IO.File]::ReadAllBytes($source))
        return
    }
    $text = [IO.File]::ReadAllText($source)
    foreach ($pair in @(@($root,'${WORKSPACE}'),@($env:USERPROFILE,'${USERPROFILE}'))) {
        $prefix = $pair[0]
        foreach ($variant in @($prefix.Replace('\','\\'),$prefix,$prefix.Replace('\','/'),('/cygdrive/' + $prefix.Substring(0,1).ToLowerInvariant() + '/' + $prefix.Substring(3).Replace('\','/')))) {
            $text = $text.Replace($variant,$pair[1])
        }
    }
    [IO.File]::WriteAllText($target,$text.Replace("`r`n","`n"))
}
function Save-Run([string]$source,[string]$target) {
    foreach ($file in Get-ChildItem -LiteralPath $source -File) {
        if ($file.Extension -in '.json','.log','.txt') { Save-Text $file.FullName (Join-Path $target $file.Name) }
    }
    $inputs = Join-Path $source 'build-inputs.json'
    if (Test-Path -LiteralPath $inputs) {
        $identity = Get-Content $inputs -Raw | ConvertFrom-Json
        foreach ($property in $identity.sources.psobject.Properties) {
            $actual = (Get-FileHash -LiteralPath (Join-Path $root $property.Name) -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($actual -ne $property.Value) { throw "Stale build source: $($property.Name); rerun acceptance before retaining evidence" }
        }
    }
    $config = Join-Path $source 'iso root with spaces/boot/limine/limine.conf'
    if (Test-Path -LiteralPath $config) { Save-Text $config (Join-Path $target 'staged-limine.conf') }
    foreach ($child in Get-ChildItem -LiteralPath $source -Directory) {
        if ($child.Name -in 'cold-boot-a','cold-boot-b','no-entry-run','bad-config-package','bad-config-run','bad-config-kernel') {
            Save-Run $child.FullName (Join-Path $target $child.Name)
        }
    }
}
Save-Run $AcceptancePath (Join-Path $destination 'acceptance')
foreach ($property in $accepted.runs.psobject.Properties) {
    Save-Run $property.Value (Join-Path $destination $property.Name)
}
Save-Text (Join-Path $root 'work/p1-02-baseline-doctor.json') (Join-Path $destination 'baseline-doctor-sandbox.json')
Save-Text (Join-Path $root 'work/p1-02-host-tests-final.log') (Join-Path $destination 'host-tests.log')
# Preserve both genuine failed M1 integration experiments, with their original
# input identities. They intentionally predate the final source revision.
foreach ($source in $FailedRunPaths) {
    $name = Split-Path $source -Leaf
    $target = Join-Path $destination "failed-experiments/$name"
    foreach ($file in Get-ChildItem $source -File | Where-Object { $_.Extension -in '.json','.log' }) {
        Save-Text $file.FullName (Join-Path $target $file.Name)
    }
    $launch = Join-Path $source 'cold-boot-a'
    if (Test-Path $launch) {
        foreach ($file in Get-ChildItem $launch -File | Where-Object { $_.Extension -in '.json','.log' }) {
            Save-Text $file.FullName (Join-Path $target "cold-boot-a/$($file.Name)")
        }
    }
}
if ($BaselineM0Path) { Save-Run $BaselineM0Path (Join-Path $destination 'baseline-m0') }
if ($DependencyPath) { Save-Run $DependencyPath (Join-Path $destination 'baseline-dependencies') }
$m0Log = [IO.File]::ReadAllText((Join-Path $accepted.runs.m0 'compile-tests.log'))
$m0Verification = [regex]::Match($m0Log,'Evidence: ([^\r\n]+)').Groups[1].Value
if ($m0Verification) { Save-Run $m0Verification (Join-Path $destination 'm0-verification') }
$artifacts = foreach ($property in $accepted.runs.psobject.Properties) {
    foreach ($file in Get-ChildItem $property.Value -Recurse -File | Where-Object { $_.Name -in 'kernel.elf','continuum.iso' -and $_.FullName -notmatch '[\\/]cargo[\\/]|extracted payload|iso root with spaces' }) {
        @{path=[IO.Path]::GetRelativePath($root,$file.FullName).Replace('\','/');bytes=$file.Length;sha256=(Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
    }
}
@{schema_version=1;source_commit=(& git -C $root rev-parse HEAD);branch=(& git -C $root branch --show-current);source_dirty=$true;source_identity='Per-build build-inputs.json hashes verified against the current source; docs and evidence are working-tree changes';artifacts=@($artifacts);limits='Binaries stay in ignored build/m1; retained text is sanitized. This is not a release or ISO reproducibility result.'} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $destination 'artifact-locations.json')
$checksums = foreach ($file in Get-ChildItem $destination -Recurse -File | Where-Object Name -ne 'SHA256SUMS.json' | Sort-Object FullName) {
    @{path=[IO.Path]::GetRelativePath($destination,$file.FullName).Replace('\','/');bytes=$file.Length;sha256=(Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
}
@{algorithm='SHA-256';files=@($checksums)} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $destination 'SHA256SUMS.json')
"Retained evidence: $destination"
