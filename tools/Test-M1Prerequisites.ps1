param([string]$LockPath = (Join-Path $PSScriptRoot '../config/m1-inputs.lock.json'), [switch]$Json)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$failures = [Collections.Generic.List[string]]::new()
$queries = [Collections.Generic.List[object]]::new()
function Probe([string]$exe, [string[]]$arguments) {
    $start = [Diagnostics.ProcessStartInfo]::new($exe)
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    foreach ($argument in $arguments) { $start.ArgumentList.Add($argument) }
    $p = [Diagnostics.Process]::Start($start)
    $stdout = $p.StandardOutput.ReadToEndAsync()
    $stderr = $p.StandardError.ReadToEndAsync()
    if (-not $p.WaitForExit(15000)) { $p.Kill($true); $p.WaitForExit(); throw "Probe timed out: $exe" }
    $text = ($stdout.Result + $stderr.Result).Replace($root,'${WORKSPACE}').Replace($env:USERPROFILE,'${USERPROFILE}')
    $queries.Add(@{arguments=$arguments;exit_code=$p.ExitCode;output=$text})
    if ($p.ExitCode -ne 0) { throw "Probe failed ($($p.ExitCode)): $text" }
    return $text
}
try {
    if (-not (Test-Path -LiteralPath $LockPath -PathType Leaf)) { throw 'M1 input lock missing; supply tools and config/m1-inputs.lock.json before M1 work' }
    $lock = Get-Content -LiteralPath $LockPath -Raw | ConvertFrom-Json
    if ($lock.schema_version -ne 1 -or -not $lock.files.Count) { throw 'Unsupported or empty M1 input lock' }
    $paths = @{}
    foreach ($file in $lock.files) {
        $path = Join-Path $root $file.path
        if ($file.override) {
            $explicit = [Environment]::GetEnvironmentVariable($file.override)
            if ($explicit) { $path = $explicit }
        }
        if ($file.id -in @('qemu','xorriso') -and (Test-Path -LiteralPath $path -PathType Leaf)) {
            $lockedDirectory = [IO.Path]::GetFullPath((Split-Path (Join-Path $root $file.path)))
            if ([IO.Path]::GetFullPath((Split-Path $path)) -ne $lockedDirectory) {
                $failures.Add("$($file.id) override must use its locked runtime directory; review a new tool-tree lock to relocate it")
            }
        }
        $paths[$file.id] = $path
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { $failures.Add("Missing $($file.id): $path; restore the locked input (no fallback)"); continue }
        $item = Get-Item -LiteralPath $path
        if ($item.Length -ne $file.bytes -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256) {
            $failures.Add("Hash/size mismatch: $($file.id); restore the locked input; overrides must match the pin")
        }
    }
    # Never execute a mismatched or partially supplied tool tree.
    if (-not $failures.Count) {
        $version = Probe $paths.qemu @('--version')
        if (-not $version.Contains($lock.qemu_version)) { throw 'QEMU version differs from lock' }
        $machines = Probe $paths.qemu @('-machine','help')
        $cpus = Probe $paths.qemu @('-cpu','help')
        $accels = Probe $paths.qemu @('-accel','help')
        $properties = Probe $paths.qemu @('-machine',"$($lock.vm.machine),help")
        if ($machines -notmatch "(?m)^$([regex]::Escape($lock.vm.machine))\s" -or $cpus -notmatch "(?m)^\s+$([regex]::Escape($lock.vm.cpu))\s" -or $accels -notmatch '(?m)^tcg\s*$' -or $properties -notmatch 'hpet=<bool>') { throw 'Pinned machine/CPU/TCG/HPET not reported by QEMU' }
        $isoVersion = Probe $paths.xorriso @('-version')
        if (-not $isoVersion.Contains($lock.xorriso_version)) { throw 'xorriso version differs from lock' }
    }
} catch { $failures.Add($_.Exception.Message.Replace($root,'${WORKSPACE}').Replace($env:USERPROFILE,'${USERPROFILE}')) }
$report = [ordered]@{status=$(if ($failures.Count) {'blocked'} else {'passed'});failures=@($failures);queries=@($queries);boot_tested=$false;policy='Read-only probes; no VM launch, install, network or host feature change'}
if ($Json) { $report | ConvertTo-Json -Depth 12; if ($failures.Count) { exit 1 } } else { $report }
