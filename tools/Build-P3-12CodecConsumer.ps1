param([Parameter(Mandatory)][string]$Production,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
. (Join-Path $PSScriptRoot 'windows/DevSigning.ps1')
New-Item -ItemType Directory -Force $Output | Out-Null
$library=Get-ChildItem -LiteralPath (Join-Path (Split-Path $Production -Parent) 'deps') -Filter 'libcontinuum_rt-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $library) {throw 'Build the production SDK host consumer first.'}
$executable=Join-Path $Output 'semantic-consumer.exe'
$arguments=@((Join-Path $PSScriptRoot 'sdk_codec_consumer.rs'),'--edition=2024','--crate-name','sdk_codec_semantics','--extern',('continuum_rt='+$library.FullName),'-o',$executable)
& rustc @arguments *> (Join-Path $Output 'semantic-build.log')
if ($LASTEXITCODE -ne 0) {throw 'Production API semantic consumer failed to compile.'}
$null=Set-DevExecutableSignature $executable (Join-Path $PSScriptRoot 'sdk_codec_consumer.rs')
@{status='passed';arguments=$arguments;library_sha256=(Get-FileHash -LiteralPath $library.FullName).Hash.ToLowerInvariant();source_sha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'sdk_codec_consumer.rs')).Hash.ToLowerInvariant();consumer_sha256=(Get-FileHash -LiteralPath $executable).Hash.ToLowerInvariant();compiler=(& rustc -vV)} | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $Output 'semantic-build.json')
