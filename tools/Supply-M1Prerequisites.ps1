# Explicit workspace-only acquisition. Does not install packages or launch a VM.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$lock = Get-Content (Join-Path $root 'config/m1-inputs.lock.json') -Raw | ConvertFrom-Json
foreach ($inputFile in $lock.acquisitions) {
    $destination = Join-Path $root $inputFile.path
    Write-Host "$($inputFile.url) ($($inputFile.bytes) bytes) -> $destination"
}
Write-Host 'Extraction stays under work/tools; no installer is run or Windows feature changed.'
foreach ($inputFile in $lock.acquisitions) {
    $destination = Join-Path $root $inputFile.path
    New-Item -ItemType Directory -Force (Split-Path $destination) | Out-Null
    if (-not (Test-Path -LiteralPath $destination)) { Invoke-WebRequest $inputFile.url -OutFile $destination }
    if ((Get-Item -LiteralPath $destination).Length -ne $inputFile.bytes -or (Get-FileHash -LiteralPath $destination).Hash.ToLowerInvariant() -ne $inputFile.sha256) { throw "Acquisition hash/size mismatch: $destination; retained for inspection" }
}
Push-Location $root
try {
    foreach ($dir in '7zip','qemu','iso','limine','bindings','limine-protocol') { New-Item -ItemType Directory -Force "work/tools/$dir" | Out-Null }
    & work/downloads/p1-01/7zr.exe x work/downloads/p1-01/7z2604-x64.exe -owork/tools/7zip -y
    if ($LASTEXITCODE) { throw '7-Zip extraction failed' }
    & work/tools/7zip/7z.exe x work/downloads/p1-01/qemu-w64-setup-20260811.exe -owork/tools/qemu -y
    if ($LASTEXITCODE) { throw 'QEMU extraction failed' }
    foreach ($package in $lock.acquisitions | Where-Object path -like '*.pkg.tar.zst') {
        & tar -xf (Join-Path $root $package.path) -C work/tools/iso
        if ($LASTEXITCODE) { throw "Runtime extraction failed: $($package.path)" }
    }
    Expand-Archive work/downloads/p1-01/limine-binary.zip work/tools/limine -Force
    & tar -xf work/downloads/p1-01/limine-0.5.0.crate -C work/tools/bindings
    if ($LASTEXITCODE) { throw 'Binding extraction failed' }
    foreach ($name in 'limine.h','PROTOCOL.md','LICENSE') {
        $sourcePath = if ($name -eq 'limine.h') { 'include/limine.h' } else { $name }
        $destination = "work/tools/limine-protocol/$name"
        Invoke-WebRequest "https://raw.githubusercontent.com/Limine-Bootloader/limine-protocol/$($lock.protocol_commit)/$sourcePath" -OutFile $destination
        $pin = $lock.files | Where-Object path -eq $destination
        if (-not $pin -or (Get-FileHash $destination).Hash.ToLowerInvariant() -ne $pin.sha256) { throw "Protocol hash mismatch: $name" }
    }
    'Inputs supplied. Run cargo xtask doctor --profile m1 before packaging or launching.'
} finally { Pop-Location }
