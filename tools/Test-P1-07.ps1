# Complete local acceptance; fixed deadlines, offline launches, no automatic retry.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    & cargo xtask test --suite m1
    if ($LASTEXITCODE -ne 0) { throw 'P1-07 failed; retain the reported M1 artifacts' }
} finally { Pop-Location }
