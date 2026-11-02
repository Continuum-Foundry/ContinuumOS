param([switch]$Json, [switch]$M1)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot '../Host.ps1')
$root=Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$sysroot=''
$rustc=if ($env:CONTINUUM_RUSTC) {$env:CONTINUUM_RUSTC} else {(Get-Command rustc -ErrorAction SilentlyContinue).Source}
if ($rustc -and (Test-Path -LiteralPath $rustc -PathType Leaf)) {
    $start=[Diagnostics.ProcessStartInfo]::new($rustc)
    $start.UseShellExecute=$false;$start.RedirectStandardOutput=$true;$start.RedirectStandardError=$true
    $start.ArgumentList.Add('--print');$start.ArgumentList.Add('sysroot')
    $child=[Diagnostics.Process]::Start($start)
    $stdout=$child.StandardOutput.ReadToEndAsync();$stderr=$child.StandardError.ReadToEndAsync()
    if (-not $child.WaitForExit(15000)) {$child.Kill($true);$child.WaitForExit();throw 'Compiler sysroot probe timed out'}
    if ($child.ExitCode -eq 0) {$sysroot=$stdout.Result.Trim()}
}
$llvm=Join-Path $(if ($sysroot) {$sysroot} else {'/nonexistent-compiler'}) ('lib/rustlib/'+$script:ContinuumHost+'/bin')
$tools=@(foreach ($name in @('rustc','cargo','cc','rust-lld','llvm-readobj','llvm-nm','qemu-system-x86_64','xorriso')) {
    $key='CONTINUUM_'+$name.ToUpperInvariant().Replace('-','_')
    $explicit=[Environment]::GetEnvironmentVariable($key)
    $required=$name -in @('rustc','cargo','cc','rust-lld')
    $path=if ($explicit) {$explicit} elseif ($name -eq 'rust-lld') {Join-Path $(if ($sysroot) {$sysroot} else {'/nonexistent-compiler'}) ('lib/rustlib/'+$script:ContinuumHost+'/bin/rust-lld')} elseif ($name -like 'llvm-*') {Join-Path $llvm $name} else {(Get-Command $name -ErrorAction SilentlyContinue).Source}
    $status=if ($path -and (Test-Path -LiteralPath $path -PathType Leaf)) {'available'} elseif ($explicit) {'blocked'} else {'missing'}
    $detail='Not found'
    if ($status -eq 'available') {
        [string[]]$arguments=if ($name -eq 'rust-lld') {@('-flavor','gnu','--version')} elseif ($name -eq 'xorriso') {@('-version')} else {@('--version')}
        try {
            $start=[Diagnostics.ProcessStartInfo]::new($path)
            $start.UseShellExecute=$false
            $start.RedirectStandardOutput=$true
            $start.RedirectStandardError=$true
            foreach ($argument in $arguments) {$start.ArgumentList.Add($argument)}
            $child=[Diagnostics.Process]::Start($start)
            $stdout=$child.StandardOutput.ReadToEndAsync()
            $stderr=$child.StandardError.ReadToEndAsync()
            if (-not $child.WaitForExit(15000)) {$child.Kill($true);$child.WaitForExit();throw 'Version probe timed out'}
            $detail=($stdout.Result+$stderr.Result).Trim()
            if ($child.ExitCode -ne 0) {$status='blocked'}
        } catch {$status='blocked';$detail=$_.Exception.Message}
    }
    @{name=$name;path=$path;status=$status;detail=$detail;required_for_compile=$required;override=$key}
})
$firmware=$env:CONTINUUM_OVMF_CODE
if (-not $firmware -and (Test-Path (Get-M1LockPath))) {$lock=Get-Content (Get-M1LockPath) -Raw | ConvertFrom-Json;$firmware=Join-Path $root ($lock.files | Where-Object id -eq firmware_code).path}
$report=@{schema_version=1;host_target=$script:ContinuumHost;tools=$tools;firmware=@{status=$(if ($firmware -and (Test-Path -LiteralPath $firmware)) {'available'} elseif ($env:CONTINUUM_OVMF_CODE) {'blocked'} else {'missing'})};guest_core_status=$(if (Test-Path "$sysroot/lib/rustlib/x86_64-unknown-none/lib/libcore-*.rlib") {'available'} else {'missing'});boot_tested=$false;windows_signing='not-applicable'}
if ($M1) {$report['m1']=& (Join-Path $PSScriptRoot '../Test-M1Prerequisites.ps1')}
$report | ConvertTo-Json -Depth 12
if (@($tools | Where-Object {$_.required_for_compile -and $_.status -ne 'available'}).Count -or $report.guest_core_status -ne 'available' -or ($M1 -and $report.m1.status -ne 'passed')) {exit 1}
exit 0
