# Clean, offline integration test: Cargo build helper, proc macro, test and run.
. (Join-Path $PSScriptRoot 'DevSigning.ps1')
. (Join-Path (Split-Path $PSScriptRoot -Parent) 'Enter-Dev.ps1')
if (-not $env:RUSTC_WRAPPER) { throw 'Initialize-DevCompiler.ps1 must complete first.' }
$run = Join-Path $script:DevRoot ('work/acceptance/signing-' + [guid]::NewGuid().ToString('N'))
$source = Join-Path $run 'source workspace with spaces'
$target = Join-Path $run 'host cargo with spaces'
New-Item -ItemType Directory -Force (Join-Path $source 'src'),(Join-Path $source 'meaning/src') | Out-Null
@'
[package]
name = "xtask"
version = "0.0.0"
edition = "2024"
[dependencies]
meaning = { path = "meaning" }
[workspace]
'@ | Set-Content (Join-Path $source 'Cargo.toml')
@'
fn main() { println!("cargo::rustc-env=DEV_BUILD_HELPER=passed"); }
'@ | Set-Content (Join-Path $source 'build.rs')
@'
[package]
name = "meaning"
version = "0.0.0"
edition = "2024"
[lib]
proc-macro = true
'@ | Set-Content (Join-Path $source 'meaning/Cargo.toml')
@'
extern crate proc_macro;
#[proc_macro]
pub fn answer(_: proc_macro::TokenStream) -> proc_macro::TokenStream { "42".parse().unwrap() }
'@ | Set-Content (Join-Path $source 'meaning/src/lib.rs')
@'
fn main() {
    assert_eq!(meaning::answer!(), 42);
    assert_eq!(env!("DEV_BUILD_HELPER"), "passed");
    assert_eq!(std::env::args().skip(1).collect::<Vec<_>>(), vec!["space argument", "--flag", ""]);
    println!("compiler-signing-passed");
    std::process::exit(23);
}
#[test]
fn helper_and_macro() { assert_eq!(meaning::answer!(), 42); assert_eq!(env!("DEV_BUILD_HELPER"), "passed"); }
'@ | Set-Content (Join-Path $source 'src/main.rs')
Push-Location $source
try {
    & cargo test --offline --manifest-path (Join-Path $source 'Cargo.toml') --target-dir $target *> (Join-Path $run 'tests.log')
    if ($LASTEXITCODE -ne 0) { throw "Clean compiler signing integration failed: $run/tests.log" }
    & cargo run --offline --manifest-path (Join-Path $source 'Cargo.toml') --target-dir $target -- 'space argument' '--flag' '' *> (Join-Path $run 'run.log')
    if ($LASTEXITCODE -ne 23 -or (Get-Content (Join-Path $run 'run.log') -Raw) -notmatch 'compiler-signing-passed') { throw "Cargo runner argument/exit integration failed: $run/run.log" }
} finally { Pop-Location }
$binaries = @(Get-ChildItem -LiteralPath $target -Recurse -File | Where-Object { $_.Extension -in '.exe','.dll' })
foreach ($binary in $binaries) {
    $signature = Get-AuthenticodeSignature -LiteralPath $binary.FullName
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ne (Get-DevSigningState).thumbprint) { throw "Unsigned/invalid host compiler output: $($binary.FullName)" }
}
if (@($binaries | Where-Object Name -like 'build-script-build.exe').Count -lt 1 -or @($binaries | Where-Object Extension -eq '.dll').Count -lt 1) { throw 'Integration test did not exercise a build script and proc macro.' }
@{status='passed';build_script=$true;proc_macro=$true;cargo_test=$true;cargo_run=$true;argument_and_exit=$true;binaries=@($binaries | ForEach-Object { @{path=$_.FullName;sha256=(Get-FileHash $_.FullName -Algorithm SHA256).Hash} })} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $run 'result.json')
Write-Output "Clean compiler signing integration passed: $run"
