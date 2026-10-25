# Fresh hashes exercise signer-based trust after rebuilds. Two probes per run.
. (Join-Path $PSScriptRoot 'DevSigning.ps1')
. (Join-Path (Split-Path $PSScriptRoot -Parent) 'Enter-Dev.ps1')
$null = Get-DevCertificate
$directory = Join-Path $script:DevRoot ('build/windows-dev-signing/' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $directory | Out-Null
$results = @()
$rejected = $false
try { $null = Assert-DevExecutablePath (Join-Path $env:windir 'System32/cmd.exe') } catch { $rejected = $true }
if (-not $rejected) { throw 'Runner accepted an executable outside declared build roots.' }
$unknown = Join-Path $directory 'undeclared.exe'
'not an executable' | Set-Content -LiteralPath $unknown
$rejected = $false
try { $null = Assert-DevExecutablePath $unknown } catch { $rejected = $true }
if (-not $rejected) { throw 'Runner accepted an undeclared executable name.' }
foreach ($iteration in 1,2) {
    $nonce = [guid]::NewGuid().ToString('N')
    $source = Join-Path $directory "dev-probe-$nonce.rs"
    $executable = Join-Path $directory "dev-probe-$nonce.exe"
    # Verify argument boundaries and exit-code propagation, including spaces.
    'fn main() { let a: Vec<_> = std::env::args().skip(1).collect(); assert_eq!(a, vec!["space argument", "--flag", "quote\"value", ""]); println!("' + $nonce + '"); std::process::exit(23); }' | Set-Content -LiteralPath $source
    & rustc --edition=2024 --crate-name dev_signing_probe $source -o $executable
    if ($LASTEXITCODE -ne 0) { throw 'Probe compilation failed.' }
    $record = Set-DevExecutableSignature $executable
    # Windows may retain an image mapping after process exit. Mutate a separate
    # never-executed copy, preserving the signed/actually executed artifact.
    $tampered = Join-Path $directory "dev-probe-$nonce-tampered.exe"
    Copy-Item -LiteralPath $executable -Destination $tampered
    $output = & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'Invoke-DevExecutable.ps1') $executable 'space argument' '--flag' 'quote"value' ''
    $code = $LASTEXITCODE
    if ($code -ne 23 -or $output -ne $nonce) { throw "Signed execution/argument/exit probe failed: exit=$code output=$output. Active policy may still reject the development signer." }
    # Verify digest validation actually detects changed signed content. Never run
    # the corrupted probe. Change a section's data, preserving valid PE headers.
    $bytes = [IO.File]::ReadAllBytes($tampered)
    $pe = [BitConverter]::ToInt32($bytes, 0x3c)
    $section = $pe + 24 + [BitConverter]::ToUInt16($bytes, $pe + 20)
    $raw = [BitConverter]::ToInt32($bytes, $section + 20)
    if ($raw -le 0 -or $raw -ge $bytes.Length) { throw 'Unexpected PE section layout.' }
    $bytes[$raw] = $bytes[$raw] -bxor 1
    [IO.File]::WriteAllBytes($tampered, $bytes)
    if ((Get-AuthenticodeSignature -LiteralPath $tampered).Status -ne 'HashMismatch') { throw 'Changed signed probe was not rejected by Authenticode verification.' }
    $results += @{unsigned_sha256=$record.before_sha256;signed_sha256=$record.signed_sha256;exit_code=$code;nonce=$nonce}
}
if ($results[0].signed_sha256 -eq $results[1].signed_sha256) { throw 'Fresh-build probes unexpectedly share a hash.' }
@{status='passed';utc=[DateTime]::UtcNow.ToString('o');probes=$results;path_and_name_guards=$true;tamper_detection=$true;thumbprint=(Get-DevSigningState).thumbprint} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $directory 'result.json')
Write-Output "Development signing preflight passed: $directory"
