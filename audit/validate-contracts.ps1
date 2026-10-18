$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Path 'work' -Force | Out-Null
$results = [System.Collections.Generic.List[object]]::new()
function Check-Fixture {
    param([string]$Name, [string]$Schema, [string]$Json, [bool]$Expected)
    $schemaErrors = @()
    $actual = Test-Json -Json $Json -SchemaFile $Schema -ErrorAction SilentlyContinue -ErrorVariable schemaErrors
    $passed = ($actual -eq $Expected)
    $results.Add([pscustomobject]@{ name = $Name; expected = $Expected; actual = [bool]$actual; passed = $passed })
    if (-not $passed) { throw "Fixture failed: $Name" }
}
$pkgSchema = (Resolve-Path 'docs/06-contracts/schemas/package-manifest.schema.json').Path
$updSchema = (Resolve-Path 'docs/06-contracts/schemas/update-manifest.schema.json').Path
$pkgRaw = Get-Content -LiteralPath 'docs/06-contracts/examples/package-manifest.json' -Raw
$updRaw = Get-Content -LiteralPath 'docs/06-contracts/examples/update-manifest.json' -Raw
Check-Fixture 'package positive synthetic fixture' $pkgSchema $pkgRaw $true
Check-Fixture 'update positive synthetic fixture' $updSchema $updRaw $true
$pkg = $pkgRaw | ConvertFrom-Json -AsHashtable
$pkg.files[0].sha256 = 'bad'
Check-Fixture 'reject invalid package digest shape' $pkgSchema ($pkg | ConvertTo-Json -Depth 30) $false
$pkg = $pkgRaw | ConvertFrom-Json -AsHashtable
$pkg.Remove('format_version') | Out-Null
Check-Fixture 'reject missing package format version' $pkgSchema ($pkg | ConvertTo-Json -Depth 30) $false
$pkg = $pkgRaw | ConvertFrom-Json -AsHashtable
$pkg['unexpected_authority'] = $true
Check-Fixture 'reject unknown package property' $pkgSchema ($pkg | ConvertTo-Json -Depth 30) $false
$pkg = $pkgRaw | ConvertFrom-Json -AsHashtable
$pkg.limits.memory_bytes = 2147483649
Check-Fixture 'reject excessive declared package memory' $pkgSchema ($pkg | ConvertTo-Json -Depth 30) $false
$pkg = $pkgRaw | ConvertFrom-Json -AsHashtable
$pkg.capabilities.required = @('surface.own', 'surface.own')
Check-Fixture 'reject duplicate capability request' $pkgSchema ($pkg | ConvertTo-Json -Depth 30) $false
$upd = $updRaw | ConvertFrom-Json -AsHashtable
$upd.format_version = 2
Check-Fixture 'reject unsupported update format version' $updSchema ($upd | ConvertTo-Json -Depth 30) $false
$upd = $updRaw | ConvertFrom-Json -AsHashtable
$upd.components = @()
Check-Fixture 'reject empty update component set' $updSchema ($upd | ConvertTo-Json -Depth 30) $false
$upd = $updRaw | ConvertFrom-Json -AsHashtable
$upd.components[0].new_epoch = -1
Check-Fixture 'reject negative update epoch' $updSchema ($upd | ConvertTo-Json -Depth 30) $false
$upd = $updRaw | ConvertFrom-Json -AsHashtable
$upd.max_checkpoint_bytes = 16777217
Check-Fixture 'reject oversized checkpoint declaration' $updSchema ($upd | ConvertTo-Json -Depth 30) $false
$report = [ordered]@{
    validator = 'PowerShell Test-Json'; powershell = $PSVersionTable.PSVersion.ToString()
    dialect = 'JSON Schema 2020-12'; positive = 2; negative = 9
    scope = 'Schema structure and synthetic fixtures only. No payload integrity, semantic compatibility, authentication or OS behavior tested.'
    results = $results
}
$report | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath 'work/contract-validation-results.json' -Encoding utf8
Write-Output ($report | ConvertTo-Json -Depth 10)
