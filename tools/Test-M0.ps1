$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$run = Join-Path $root ("work/acceptance/" + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
Push-Location $root
try {
    & cargo test --locked -p xtask *> (Join-Path $run 'host-tests.log')
    if ($LASTEXITCODE -ne 0) { throw 'Host tests failed' }
    & cargo xtask test --suite m0 *> (Join-Path $run 'compile-tests.log')
    if ($LASTEXITCODE -ne 0) { throw 'Clean build / artifact checks failed' }
    & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'Test-Doctor.ps1') *> (Join-Path $run 'doctor-tests.json')
    if ($LASTEXITCODE -ne 0) { throw 'Doctor tests failed' }
    $space = Join-Path $run 'source workspace with spaces'
    New-Item -ItemType Directory -Force $space | Out-Null
    foreach ($part in 'Cargo.toml','Cargo.lock','rust-toolchain.toml','.cargo','tools','kernel','experiments/kernel-link','experiments/userland-link') {
        $destination = Join-Path $space $part
        New-Item -ItemType Directory -Force (Split-Path $destination) | Out-Null
        Copy-Item -LiteralPath (Join-Path $root $part) -Destination $destination -Recurse
    }
    $oldTarget = $env:CARGO_TARGET_DIR
    Push-Location $space
    try {
        $env:CARGO_TARGET_DIR = Join-Path $run 'host cargo with spaces'
        & cargo xtask build --profile feasibility *> (Join-Path $run 'source-path-spaces.log')
        if ($LASTEXITCODE -ne 0) { throw 'Build from source path containing spaces failed' }
    } finally { Pop-Location; $env:CARGO_TARGET_DIR=$oldTarget }
    @{status='passed';host_tests=$true;two_clean_builds=$true;doctor_override_tests=$true;source_path_with_spaces=$true;boot_tested=$false} | ConvertTo-Json | Set-Content (Join-Path $run 'acceptance-results.json')
    "Acceptance evidence: $run"
} finally { Pop-Location }
