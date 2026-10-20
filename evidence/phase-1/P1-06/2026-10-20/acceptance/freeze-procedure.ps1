$ErrorActionPreference='Stop'
$root=(Get-Location).Path
$destination=Join-Path $root 'evidence/phase-1/P1-06/2026-10-20'
$indexPath=Join-Path $destination 'evidence-index.json'
$index=Get-Content -LiteralPath $indexPath -Raw | ConvertFrom-Json -AsHashtable
$additional=@{
    'work/p1-06-final-audits/docs.log'='acceptance/docs-audit.log'
    'work/p1-06-final-audits/governance.log'='acceptance/governance-before-freeze.log'
    'work/p1-06-final-audits/contracts.log'='acceptance/contracts.log'
    'work/Freeze-P1-06.ps1'='acceptance/freeze-procedure.ps1'
}
foreach ($source in $additional.Keys) {
    $target=Join-Path $destination $additional[$source]
    if (Test-Path -LiteralPath $target) {throw "Already retained: $target"}
    Copy-Item -LiteralPath (Join-Path $root $source) -Destination $target
    $hash=(Get-FileHash -LiteralPath $target).Hash.ToLowerInvariant()
    $index.files+=@{path=$additional[$source];source=$source;raw_sha256=$hash;retained_sha256=$hash;verbatim=$true}
}
$index | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $indexPath
foreach ($entry in $index.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $destination $entry.path)).Hash.ToLowerInvariant() -ne $entry.retained_sha256) {throw "Retained bytes changed: $($entry.path)"}
}
$checksumPath=Join-Path $destination 'SHA256SUMS.json'
if (Test-Path -LiteralPath $checksumPath) {throw 'Refusing to rewrite frozen evidence'}
$files=@(Get-ChildItem -LiteralPath $destination -Recurse -File | Sort-Object FullName | ForEach-Object {
    @{path=[IO.Path]::GetRelativePath($destination,$_.FullName).Replace('\','/');bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant()}
})
@{algorithm='SHA-256';review_date='2026-10-20';scope='P1-06 local acceptance; all retained files except this manifest';files=$files} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $checksumPath
"Frozen $($files.Count) files; $([math]::Round(($files.bytes | Measure-Object -Sum).Sum/1MB,2)) MiB"
