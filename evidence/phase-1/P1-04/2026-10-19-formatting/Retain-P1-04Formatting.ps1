$ErrorActionPreference='Stop'
$projectRoot=(Get-Location).Path
$source=Join-Path $projectRoot 'build/m1/exception-invalid-opcode-1792453369322739500-30180'
$destination=Join-Path $projectRoot 'evidence/phase-1/P1-04/2026-10-19-formatting'
if (Test-Path $destination) { throw 'Refusing to overwrite evidence' }
New-Item -ItemType Directory $destination | Out-Null
foreach ($file in Get-ChildItem $source -Recurse -File | Where-Object { $_.Extension -in '.json','.log','.txt' -and $_.FullName -notmatch '[\\/](cargo|iso root with spaces|extracted payload)[\\/]' }) {
    $target=Join-Path $destination ([IO.Path]::GetRelativePath($source,$file.FullName))
    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
    if ($file.Name -eq 'serial.log') { Copy-Item -LiteralPath $file.FullName -Destination $target }
    else {
        $text=[IO.File]::ReadAllText($file.FullName)
        foreach ($prefix in @($projectRoot,$env:USERPROFILE)) {
            $label=if ($prefix -eq $projectRoot) { '${WORKSPACE}' } else { '${USERPROFILE}' }
            $text=$text.Replace($prefix.Replace('\','\\'),$label).Replace($prefix.Replace('\','/'),$label).Replace($prefix,$label)
        }
        [IO.File]::WriteAllText($target,$text)
    }
}
$inputs=Get-Content (Join-Path $source 'build-inputs.json') -Raw | ConvertFrom-Json -AsHashtable
foreach ($name in $inputs.sources.Keys) {
    if ((Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash.ToLowerInvariant() -ne $inputs.sources[$name]) { throw "Stale source $name" }
}
New-Item -ItemType Directory -Force (Join-Path $destination 'source/kernel/src') | Out-Null
Copy-Item -LiteralPath kernel/src/exception-entry.S -Destination (Join-Path $destination 'source/kernel/src/exception-entry.S')
foreach ($name in 'Compare-P1-04Formatting.cjs','Compare-P1-04Formatting-initial.cjs','p1-04-formatting-review.json','Retain-P1-04Formatting.ps1') { Copy-Item -LiteralPath (Join-Path $projectRoot "work/$name") -Destination $destination }
'Formatting follow-up retained'
