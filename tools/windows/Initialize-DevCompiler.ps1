. (Join-Path $PSScriptRoot 'DevSigning.ps1')
. (Join-Path (Split-Path $PSScriptRoot -Parent) 'Enter-Dev.ps1') -SkipCompilerSigning
$null = Get-DevCertificate
$directory = Join-Path $script:DevRoot 'build/windows-dev-signing/bootstrap'
New-Item -ItemType Directory -Force $directory | Out-Null
$source = Join-Path $PSScriptRoot 'compiler-wrapper.rs'
$executable = Join-Path $directory 'dev-compiler-wrapper.exe'
& rustc --edition=2024 --crate-name dev_compiler_wrapper $source -o $executable
if ($LASTEXITCODE -ne 0) { throw 'Compiler wrapper bootstrap failed.' }
$record = Set-DevExecutableSignature $executable
$locks = @('Cargo.lock') + @(Get-ChildItem (Join-Path $script:DevRoot 'experiments') -Recurse -File -Filter Cargo.lock | ForEach-Object { [IO.Path]::GetRelativePath($script:DevRoot, $_.FullName).Replace('\','/') })
$inventory = @($locks | ForEach-Object { @{path=$_;sha256=(Get-FileHash (Join-Path $script:DevRoot $_) -Algorithm SHA256).Hash} })
@{schema=1;locks=$inventory;wrapper_source_sha256=(Get-FileHash $source -Algorithm SHA256).Hash;wrapper_signed_sha256=$record.signed_sha256;executable=$executable} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $script:DevDirectory 'compiler.json')
Write-Output 'Cargo compiler signing wrapper prepared. Re-enter the development environment.'
