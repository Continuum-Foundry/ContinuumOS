$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot '../Enter-Dev.ps1')
$version=(& rustc -vV | Out-String)
if ($LASTEXITCODE -ne 0 -or $version -notmatch '(?m)^release: 1\.99\.1$' -or $version -notmatch 'host: x86_64-unknown-linux-gnu') {throw 'Unexpected compiler version/host'}
$sysroot=(& rustc --print sysroot | Out-String).Trim()
@{status='passed';platform='linux';authenticode='not-applicable';compiler_sha256=(Get-FileHash (Join-Path $sysroot 'bin/rustc')).Hash.ToLowerInvariant();compiler=$version} | ConvertTo-Json
