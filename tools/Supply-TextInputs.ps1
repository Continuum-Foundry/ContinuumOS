param([switch]$Download)
# No host font discovery or Cargo network override. Only public immutable inputs.
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$locks=@('fixtures/m2/text-v1/fonts.lock.json','fixtures/m2/text-v1/data.lock.json')
foreach ($lockPath in $locks) {
    $lock=Get-Content (Join-Path $root $lockPath) -Raw | ConvertFrom-Json
    if ($lock.schema_version -ne 1) { throw 'Unsupported text input lock' }
    $files=if ($lockPath -like '*fonts*') {$lock.fonts} else {$lock.files}
    foreach ($file in $files) {
        if ($file.path -notmatch '^work/text-inputs/[A-Za-z0-9._-]+$' -or $file.url -notmatch '^https://(raw.githubusercontent.com|www.unicode.org)/') { throw 'Invalid pinned input path/source' }
        $path=Join-Path $root $file.path
        if (-not (Test-Path -LiteralPath $path)) {
            if (-not $Download) { throw "Missing pinned input: $($file.path). Supply exact locked bytes or explicitly use -Download." }
            New-Item -ItemType Directory -Force (Split-Path $path) | Out-Null
            Invoke-WebRequest -Uri $file.url -OutFile $path
        }
        if ((Get-Item -LiteralPath $path).Length -ne $file.bytes -or (Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant() -ne $file.sha256) { throw "Text input identity mismatch: $($file.path)" }
    }
}
Write-Output 'Pinned text assets verified; no system font used.'
