param([switch]$Json, [switch]$M1)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
function Sanitize([string]$value) {
    $value.Replace($root, '${WORKSPACE}').Replace($env:USERPROFILE, '${USERPROFILE}')
}
function Capture([string]$exe, [string[]]$arguments) {
    try {
        $start = [Diagnostics.ProcessStartInfo]::new($exe)
        $start.UseShellExecute = $false
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        $start.Environment['RUSTUP_AUTO_INSTALL'] = '0'
        foreach ($argument in $arguments) { $start.ArgumentList.Add($argument) }
        $process = [Diagnostics.Process]::Start($start)
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(15000)) { $process.Kill($true); return @{ status='blocked'; detail='Version probe timed out' } }
        $detail = (($stdout.Result + $stderr.Result) -replace "`0", '').Trim()
        @{ status=$(if ($process.ExitCode -eq 0) {'available'} else {'blocked'}); exit_code=$process.ExitCode; detail=(Sanitize $detail);raw_stdout=$stdout.Result.Trim() }
    } catch { @{status='blocked';detail=(Sanitize $_.Exception.Message)} }
}
$vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$vs = @()
if (Test-Path $vswhere) { $vs = @((& $vswhere -products '*' -format json | ConvertFrom-Json).installationPath) }
$msvc = @($vs | ForEach-Object { Get-ChildItem "$_/VC/Tools/MSVC/*/bin/Hostx64/x64/link.exe" -ErrorAction SilentlyContinue } | ForEach-Object FullName)
$sdkLibs = @(Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/Lib/*/um/x64/kernel32.lib" -ErrorAction SilentlyContinue | ForEach-Object FullName)
$ucrtLibs = @(Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/Lib/*/ucrt/x64/ucrt.lib" -ErrorAction SilentlyContinue | ForEach-Object FullName)
$rustcCommand = if ($env:CONTINUUM_RUSTC) { $env:CONTINUUM_RUSTC } else { (Get-Command rustc -ErrorAction SilentlyContinue).Source }
$sysrootProbe = if ($rustcCommand -and (Test-Path -LiteralPath $rustcCommand -PathType Leaf)) { Capture $rustcCommand @('--print','sysroot') } else { $null }
$sysroot = if ($sysrootProbe.status -eq 'available') { $sysrootProbe.raw_stdout } else { '' }
$llvm = if ($sysroot) { "$sysroot/lib/rustlib/x86_64-pc-windows-msvc/bin" } else { '' }
$guestCore = @($(if ($sysroot) { Get-ChildItem "$sysroot/lib/rustlib/x86_64-unknown-none/lib/libcore-*.rlib" -ErrorAction SilentlyContinue }))
$definitions = @(
    @{name='git';args=@('--version');paths=@("$env:ProgramFiles/Git/cmd/git.exe");required=$false},
    @{name='gh';args=@('--version');paths=@("$env:ProgramFiles/GitHub CLI/gh.exe");required=$false},
    @{name='node';args=@('--version');paths=@();required=$false},
    @{name='rustc';args=@('-vV');paths=@("$root/work/toolchains/cargo/bin/rustc.exe", "$env:USERPROFILE/.cargo/bin/rustc.exe");required=$true},
    @{name='cargo';args=@('-V');paths=@("$root/work/toolchains/cargo/bin/cargo.exe", "$env:USERPROFILE/.cargo/bin/cargo.exe");required=$true},
    @{name='link';args=@();paths=$msvc;required=$true;fileVersion=$true},
    @{name='rust-lld';args=@('-flavor','gnu','--version');paths=@("$llvm/rust-lld.exe");required=$true},
    @{name='llvm-readobj';args=@('--version');paths=@("$llvm/llvm-readobj.exe");required=$false},
    @{name='llvm-nm';args=@('--version');paths=@("$llvm/llvm-nm.exe");required=$false},
    @{name='qemu-system-x86_64';args=@('--version');paths=@("$root/work/tools/qemu/qemu-system-x86_64.exe", "$env:ProgramFiles/qemu/qemu-system-x86_64.exe");required=$false},
    @{name='xorriso';args=@('-version');paths=@("$root/work/tools/iso/usr/bin/xorriso.exe", 'C:/msys64/usr/bin/xorriso.exe');required=$false},
    @{name='oscdimg';args=@('-help');paths=@("${env:ProgramFiles(x86)}/Windows Kits/10/Assessment and Deployment Kit/Deployment Tools/amd64/Oscdimg/oscdimg.exe");required=$false}
)
$tools = foreach ($definition in $definitions) {
    $overrideName = 'CONTINUUM_' + ($definition.name.ToUpperInvariant() -replace '-', '_')
    $override = [Environment]::GetEnvironmentVariable($overrideName)
    $found = $null
    if ($override) {
        if (Test-Path -LiteralPath $override -PathType Leaf) { $found = $override }
    } else {
        $command = Get-Command ($definition.name + '.exe') -ErrorAction SilentlyContinue
        if ($command) { $found = $command.Source }
        if (-not $found) { $found = $definition.paths | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Leaf) } | Select-Object -First 1 }
    }
    $result = if ($found -and $definition.fileVersion) {
        @{status='available';detail="File version: $((Get-Item -LiteralPath $found).VersionInfo.FileVersion); host invocation is tested by the xtask build"}
    } elseif ($found) { Capture $found $definition.args } else { @{status=$(if ($override) {'blocked'} else {'missing'});detail=$(if ($override) {'Invalid explicit override; no fallback used'} else {'Not found on PATH or checked standard locations'})} }
    [ordered]@{name=$definition.name;status=$result.status;path=$(if ($found) {Sanitize $found} else {$null});override=$overrideName;required_for_compile=$definition.required;detail=$result.detail}
}
$firmwareOverride = $env:CONTINUUM_OVMF_CODE
$firmwareCandidates = @("$root/work/tools/qemu/share/edk2-x86_64-code.fd", "$env:ProgramFiles/qemu/share/edk2-x86_64-code.fd", "$env:ProgramFiles/qemu/share/OVMF_CODE.fd")
$firmware = if ($firmwareOverride) { $firmwareOverride } else { $firmwareCandidates | Where-Object {Test-Path -LiteralPath $_} | Select-Object -First 1 }
$firmwareReport = if ($firmware -and (Test-Path -LiteralPath $firmware -PathType Leaf)) {
    @{status='available';path=(Sanitize $firmware);sha256=(Get-FileHash -LiteralPath $firmware -Algorithm SHA256).Hash.ToLowerInvariant();boot_tested=$false}
} else { @{status=$(if ($firmwareOverride) {'blocked'} else {'missing'});detail='OVMF_CODE not found; set CONTINUUM_OVMF_CODE to an existing file';boot_tested=$false} }
try {
    $processor = Get-CimInstance Win32_Processor | Select-Object -First 1
    $computer = Get-CimInstance Win32_ComputerSystem
    $virtualization = @{status='available';cpu=$processor.Name;firmware_virtualization=$processor.VirtualizationFirmwareEnabled;slat=$processor.SecondLevelAddressTranslationExtensions;hypervisor_present=$computer.HypervisorPresent;whpx='untested'}
} catch { $virtualization = @{status='blocked';detail='CIM access unavailable; virtualization capability is unknown';whpx='untested'} }
$drive = Get-PSDrive -Name ([IO.Path]::GetPathRoot($root).TrimEnd('\',':'))
$report = [ordered]@{
    schema_version=1;platform=[Environment]::OSVersion.VersionString;architecture=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString();powershell=$PSVersionTable.PSVersion.ToString();free_bytes=$drive.Free
    tools=@($tools);windows_sdk=@{status=$(if ($sdkLibs.Count -and $ucrtLibs.Count) {'available'} else {'missing'});libraries=@(($sdkLibs + $ucrtLibs) | ForEach-Object {Sanitize $_})}
    firmware=$firmwareReport;virtualization=$virtualization;wsl=@{status='untested';policy='Not used or installed by Phase 0'}
    guest_target='x86_64-unknown-none';guest_core_status=$(if ($guestCore.Count) {'available'} else {'missing'});host_target='x86_64-pc-windows-msvc';boot_tested=$false
}
$m1Failed = $false
if ($M1) {
    $report['m1'] = & (Join-Path $PSScriptRoot 'Test-M1Prerequisites.ps1')
    $m1Failed = $report.m1.status -ne 'passed'
}
if ($Json) { $report | ConvertTo-Json -Depth 12 } else {
    $tools | ForEach-Object { [pscustomobject]$_ } | Select-Object name,status,path | Format-Table -AutoSize
    "Windows SDK: $($report.windows_sdk.status); firmware: $($firmwareReport.status); VM boot: not tested"
    if ($M1) { $report.m1 | ConvertTo-Json -Depth 8 }
}
if ($m1Failed) { exit 1 }
if (@($tools | Where-Object { $_.required_for_compile -and $_.status -ne 'available' }).Count -or -not $sdkLibs.Count -or -not $ucrtLibs.Count -or -not $guestCore.Count) { exit 1 }
