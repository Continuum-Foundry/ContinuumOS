$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$inventory = Join-Path $PSScriptRoot 'Inventory.ps1'
$cases = @()
$variables = 'CONTINUUM_RUSTC','CONTINUUM_QEMU_SYSTEM_X86_64','CONTINUUM_OVMF_CODE'
$previous = @{}
foreach ($variable in $variables) { $previous[$variable] = [Environment]::GetEnvironmentVariable($variable) }
try {
    $env:CONTINUUM_RUSTC = Join-Path $root 'work/nonexistent-rustc.exe'
    $report = (& pwsh -NoProfile -File $inventory -Json | Out-String) | ConvertFrom-Json
    if ($LASTEXITCODE -ne 1 -or ($report.tools | Where-Object name -eq rustc).status -ne 'blocked') { throw 'Invalid required override was not rejected' }
    $cases += 'invalid required override returns failure without fallback'
    $env:CONTINUUM_RUSTC = (Get-Command rustc).Source
    $env:CONTINUUM_QEMU_SYSTEM_X86_64 = Join-Path $root 'work/nonexistent-qemu.exe'
    $env:CONTINUUM_OVMF_CODE = Join-Path $root 'work/nonexistent-firmware.fd'
    $report = (& pwsh -NoProfile -File $inventory -Json | Out-String) | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Valid Rust override should allow compile-tool checks' }
    if (($report.tools | Where-Object name -eq rustc).status -ne 'available') { throw 'Valid executable override not honored' }
    if (($report.tools | Where-Object name -eq qemu-system-x86_64).status -ne 'blocked' -or $report.firmware.status -ne 'blocked') { throw 'Optional invalid paths not reported' }
    $cases += 'valid required executable override honored'
    $cases += 'invalid optional tool and firmware paths remain visible without blocking compilation'
} finally {
    foreach ($variable in $variables) { [Environment]::SetEnvironmentVariable($variable,$previous[$variable]) }
}
@{status='passed';cases=$cases} | ConvertTo-Json
