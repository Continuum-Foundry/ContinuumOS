param([switch]$SkipCompilerSigning)
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
# This process only; no global Cargo configuration or guest signing changes.
$continuumEnrollment = Join-Path $continuumRoot 'work/windows-dev-signing/admin-enforce.json'
if ((Test-Path -LiteralPath $continuumEnrollment) -and ((Get-Content -LiteralPath $continuumEnrollment -Raw | ConvertFrom-Json).status -eq 'passed')) {
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER = 'pwsh -NoProfile -File "' + (Join-Path $continuumRoot 'tools/windows/Invoke-DevExecutable.ps1') + '"'
    $continuumCompilerState = Join-Path $continuumRoot 'work/windows-dev-signing/compiler.json'
    if (-not $SkipCompilerSigning -and (Test-Path -LiteralPath $continuumCompilerState)) {
        $continuumCompiler = Get-Content -LiteralPath $continuumCompilerState -Raw | ConvertFrom-Json
        if ((Get-FileHash -LiteralPath (Join-Path $continuumRoot 'tools/windows/compiler-wrapper.rs') -Algorithm SHA256).Hash -ne $continuumCompiler.wrapper_source_sha256 -or
            (Get-FileHash -LiteralPath $continuumCompiler.executable -Algorithm SHA256).Hash.ToLowerInvariant() -ne $continuumCompiler.wrapper_signed_sha256) {
            throw 'Compiler signing wrapper changed or is missing. Run tools/windows/Initialize-DevCompiler.ps1 explicitly.'
        }
        $env:RUSTC_WRAPPER = $continuumCompiler.executable
        $env:CONTINUUM_COMPILER_SIGNER = Join-Path $continuumRoot 'tools/windows/Sign-DevCompilerOutput.ps1'
    } elseif (-not $SkipCompilerSigning) {
        throw 'Run tools/windows/Initialize-DevCompiler.ps1 to prepare compiler signing before development.'
    }
}
