$ErrorActionPreference='Stop'
$projectRoot=(Get-Location).Path
$destination=Join-Path $projectRoot 'evidence/phase-1/P1-05/2026-10-19'
$indexPath=Join-Path $destination 'evidence-index.json'
$index=Get-Content $indexPath -Raw | ConvertFrom-Json -AsHashtable
foreach ($pair in @(
    @('work/p1-05-docs.log','validation/documentation.json'),
    @('work/p1-05-governance-final.log','validation/governance.json'),
    @('work/p1-05-contracts.log','validation/contracts.log'),
    @('work/contract-validation-results.json','validation/contract-fixtures.json'),
    @('work/Freeze-P1-05Evidence.ps1','validation/freeze-procedure.ps1')
)) {
    $source=Join-Path $projectRoot $pair[0]
    $target=Join-Path $destination $pair[1]
    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
    Copy-Item -LiteralPath $source -Destination $target
    $hash=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
    $index.files+=@{path=$pair[1];source=('${WORKSPACE}/'+$pair[0]);raw_sha256=$hash;retained_sha256=$hash;verbatim=$true}
}
$index.scope='Captured diagnostics, validation and exact source snapshots; README, checksum/index files and final verification summary are excluded'
$index | ConvertTo-Json -Depth 16 | Set-Content $indexPath
foreach ($file in $index.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $destination $file.path) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.retained_sha256) { throw "Retained evidence changed: $($file.path)" }
}
$inputs=Get-Content (Join-Path $destination 'run/build-inputs.json') -Raw | ConvertFrom-Json -AsHashtable
foreach ($name in $inputs.sources.Keys) {
    foreach ($path in @((Join-Path $projectRoot $name),(Join-Path $destination "source/$name"))) {
        if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $inputs.sources[$name]) { throw "Source input changed: $name" }
    }
}
# Check local links in the new bundle summary and the hashed kernel README,
# which are outside audit/check-docs.cjs's ordinary documentation inventory.
$links=0
foreach ($document in @((Join-Path $destination 'README.md'),(Join-Path $projectRoot 'kernel/README.md'))) {
    $text=[IO.File]::ReadAllText($document)
    foreach ($match in [regex]::Matches($text,'\[[^\]\r\n]*\]\(([^)\r\n]+)\)')) {
        $reference=$match.Groups[1].Value
        if ($reference -match '^(https?:|mailto:|#)') { continue }
        $path=$reference.Split('#')[0]
        if (-not (Test-Path -LiteralPath (Join-Path (Split-Path $document) $path))) { throw "Missing local link $reference" }
        $links++
    }
}
$review=Get-Content (Join-Path $destination 'review.json') -Raw | ConvertFrom-Json -AsHashtable
if ($review.status -ne 'passed' -or $review.launch_count -ne 27) { throw 'Review not passed' }
@{status='passed';indexed_files_verified=$index.files.Count;source_inputs_verified=$inputs.sources.Count;additional_local_links_checked=$links;selected_launches=27;host_tests=23;documentation='558 local links, 19 requirements, 16 Proposed ADRs';contract_scope='Synthetic schema fixtures only';review='Local Codex self-review, not independent approval'} | ConvertTo-Json | Set-Content (Join-Path $destination 'validation/final-verification.json')
$sums=@(Get-ChildItem -LiteralPath $destination -File -Recurse | Where-Object Name -ne 'SHA256SUMS.json' | Sort-Object FullName | ForEach-Object { @{path=[IO.Path]::GetRelativePath($destination,$_.FullName).Replace('\','/');sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()} })
@{algorithm='SHA256';scope='All retained files except SHA256SUMS.json itself';files=$sums} | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $destination 'SHA256SUMS.json')
foreach ($file in $sums) {
    if ((Get-FileHash -LiteralPath (Join-Path $destination $file.path) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256) { throw "Freeze verification failed: $($file.path)" }
}
"P1-05 evidence frozen and verified: $($sums.Count) files"
