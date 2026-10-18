# Dot-source this script. Changes apply only to the calling PowerShell process.
$continuumRoot = Split-Path $PSScriptRoot -Parent
$continuumCargo = Join-Path $continuumRoot 'work/toolchains/cargo'
$continuumRustup = Join-Path $continuumRoot 'work/toolchains/rustup'
if (Test-Path (Join-Path $continuumCargo 'bin/cargo.exe')) {
    $env:CARGO_HOME = $continuumCargo
    $env:RUSTUP_HOME = $continuumRustup
    $env:PATH = "$(Join-Path $continuumCargo 'bin');$env:PATH"
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Rust is missing. See docs/05-delivery/phase-0.md for explicit setup; no automatic installation is performed.'
}
$env:CARGO_TARGET_DIR = Join-Path $continuumRoot 'build/cargo'
$env:RUSTUP_AUTO_INSTALL = '0'
