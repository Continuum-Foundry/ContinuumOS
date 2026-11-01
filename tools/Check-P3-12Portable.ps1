param([Parameter(Mandatory)][string]$Output)
# Affected selected portable build only; this does not execute a UI in a guest.
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
if ($env:RUSTFLAGS -or $env:CARGO_ENCODED_RUSTFLAGS -or $env:CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS) {throw 'Unexpected inherited target flags.'}
New-Item -ItemType Directory -Force $Output | Out-Null
$env:CONTINUUM_FONT_DIR=Join-Path $root 'work/text-inputs'
$llvm=Join-Path (& rustc --print sysroot) 'lib/rustlib/x86_64-pc-windows-msvc/bin'
& cargo test --locked --offline -p editor-model --lib *> (Join-Path $Output 'model-tests.log')
if ($LASTEXITCODE -ne 0) {throw 'Portable model host regression failed.'}
& cargo rustc --locked --offline --manifest-path experiments/ui-link/Cargo.toml --target x86_64-unknown-none --release --bin ui-link -- -C code-model=small -C relocation-model=static -C ('link-arg=-T'+(Join-Path $root 'experiments/ui-link/link.ld')) -C link-arg=--no-undefined *> (Join-Path $Output 'selected-link.log')
if ($LASTEXITCODE -ne 0) {throw 'Selected portable final link failed.'}
$elf=Join-Path $Output 'selected-ui.elf'
Copy-Item -LiteralPath (Join-Path $env:CARGO_TARGET_DIR 'x86_64-unknown-none/release/ui-link') -Destination $elf
& python tools/Review-UiLink.py $elf $Output --llvm $llvm *> (Join-Path $Output 'selected-audit.log')
if ($LASTEXITCODE -ne 0) {throw 'Selected portable ELF audit failed.'}
@{schema_version=1;status='passed';scope='selected custom portable final link and model host tests';guest_ui_executed=$false;elf_sha256=(Get-FileHash -LiteralPath $elf).Hash.ToLowerInvariant();lock_sha256=(Get-FileHash experiments/ui-link/Cargo.lock).Hash.ToLowerInvariant();audit=Get-Content (Join-Path $Output 'audit.json') -Raw | ConvertFrom-Json} | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $Output 'portable.json')
