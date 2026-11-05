# Host selection shared by portable orchestration; no Windows policy changes.
$script:ContinuumHost = if ($IsWindows) {'x86_64-pc-windows-msvc'} elseif ($IsLinux) {'x86_64-unknown-linux-gnu'} else {throw 'Unsupported host'}
$script:ContinuumExeSuffix = if ($IsWindows) {'.exe'} else {''}
function Get-M1LockPath { Join-Path (Split-Path $PSScriptRoot -Parent) $(if ($IsLinux) {'config/m1-inputs.linux.lock.json'} else {'config/m1-inputs.lock.json'}) }
function Set-HostExecutableSignature([string]$Path, [string]$CompilerSource = '') {
    if ($IsWindows) {
        . (Join-Path $PSScriptRoot 'windows/DevSigning.ps1')
        Set-DevExecutableSignature $Path $CompilerSource
    } else {
        # ELF execution has no Authenticode step. Record its identity explicitly.
        @{platform='linux';signing='not-applicable';sha256=(Get-FileHash -LiteralPath $Path).Hash.ToLowerInvariant()}
    }
}
