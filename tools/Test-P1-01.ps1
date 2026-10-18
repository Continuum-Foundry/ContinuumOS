$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$run = Join-Path $root ('build/m1/p1-01-tests-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
$probe = Join-Path $PSScriptRoot 'Test-M1Prerequisites.ps1'
$lockPath = Join-Path $root 'config/m1-inputs.lock.json'
$saved = @{}
foreach ($name in 'CONTINUUM_QEMU_SYSTEM_X86_64','CONTINUUM_XORRISO','CONTINUUM_OVMF_CODE','CONTINUUM_OVMF_VARS') {
    $saved[$name] = [Environment]::GetEnvironmentVariable($name)
    [Environment]::SetEnvironmentVariable($name,$null)
}
$results = [Collections.Generic.List[object]]::new()
function Check($name,$report,$expected) {
    $report | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run "$name.json")
    if ($report.status -ne $expected) { throw "$name expected $expected, got $($report.status); see $run" }
    $results.Add(@{case=$name;status='passed';observed=$report.status})
}
try {
    Check baseline (& $probe) passed
    $lock = Get-Content $lockPath -Raw | ConvertFrom-Json
    $env:CONTINUUM_QEMU_SYSTEM_X86_64 = Join-Path $root ($lock.files | Where-Object id -eq qemu).path
    Check valid_override (& $probe) passed
    $env:CONTINUUM_QEMU_SYSTEM_X86_64 = Join-Path $run 'missing.exe'
    $report = & $probe
    Check invalid_override $report blocked
    if ($report.queries.Count) { throw 'Invalid input executed a tool' }
    $env:CONTINUUM_QEMU_SYSTEM_X86_64 = $null
    $code = Join-Path $run 'corrupted-code.fd'
    Copy-Item (Join-Path $root ($lock.files | Where-Object id -eq firmware_code).path) $code
    $stream = [IO.File]::OpenWrite($code)
    try { $stream.WriteByte(0x42) } finally { $stream.Dispose() }
    $env:CONTINUUM_OVMF_CODE = $code
    $report = & $probe
    Check corrupted_firmware $report blocked
    if ($report.queries.Count -or -not ($report.failures -match 'Hash/size mismatch')) { throw 'Corruption was not rejected before execution' }
    $env:CONTINUUM_OVMF_CODE = $null
    $runtime = $lock.files | Where-Object id -eq 'msys-2.0.dll'
    $runtime.path = [IO.Path]::GetRelativePath($root,(Join-Path $run 'missing-msys-2.0.dll'))
    $missingLock = Join-Path $run 'missing-runtime.lock.json'
    $lock | ConvertTo-Json -Depth 12 | Set-Content $missingLock
    $report = & $probe -LockPath $missingLock
    Check missing_runtime $report blocked
    if ($report.queries.Count) { throw 'Incomplete runtime executed a tool' }
    Check missing_lock (& $probe -LockPath (Join-Path $run 'missing.lock.json')) blocked
    @{status='passed';cases=@($results);boot_tested=$false} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'results.json')
    "P1-01 test evidence: $run"
} finally {
    foreach ($name in $saved.Keys) { [Environment]::SetEnvironmentVariable($name,$saved[$name]) }
}
