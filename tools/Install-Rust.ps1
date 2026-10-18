# Explicit opt-in setup. No profile, global PATH, Windows feature or SDK changes.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$download = Join-Path $root 'work/downloads'
New-Item -ItemType Directory -Force $download | Out-Null
$installer = Join-Path $download 'rustup-init-1.29.1.exe'
$url = 'https://static.rust-lang.org/rustup/archive/1.29.1/x86_64-pc-windows-msvc/rustup-init.exe'
Invoke-WebRequest $url -OutFile $installer
$expected = '6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e'
if ((Get-FileHash $installer -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) { throw 'Rust installer checksum mismatch' }
$env:CARGO_HOME = Join-Path $root 'work/toolchains/cargo'
$env:RUSTUP_HOME = Join-Path $root 'work/toolchains/rustup'
& $installer -y --no-modify-path --profile minimal --default-host x86_64-pc-windows-msvc --default-toolchain 1.99.1 --component llvm-tools-preview --component rustfmt --target x86_64-unknown-none
if ($LASTEXITCODE -ne 0) { throw 'Rust setup failed; preserve installer output and inspect permissions' }
