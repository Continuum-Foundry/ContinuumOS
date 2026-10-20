# Run once as the normal developer account in PowerShell 7; no admin needed here.
. (Join-Path $PSScriptRoot 'DevSigning.ps1')
New-Item -ItemType Directory -Force $script:DevDirectory | Out-Null
$statePath = Join-Path $script:DevDirectory 'state.json'
if (Test-Path -LiteralPath $statePath) {
    $state = Get-DevSigningState
    $null = Get-DevCertificate
    Write-Output "Existing signer retained: $($state.thumbprint)"
    exit 0
}
$certificate = New-SelfSignedCertificate -Type CodeSigningCert -Subject ("CN=ContinuumOS Local Development " + [guid]::NewGuid().ToString('N')) -CertStoreLocation Cert:\CurrentUser\My -KeyAlgorithm RSA -KeyLength 3072 -HashAlgorithm SHA256 -KeyExportPolicy NonExportable -Provider 'Microsoft Software Key Storage Provider' -NotAfter (Get-Date).AddYears(1)
$publicPath = Join-Path $script:DevDirectory 'development-signer.cer'
Export-Certificate -Cert $certificate -FilePath $publicPath | Out-Null
# Current-user root trust enables local Authenticode verification; policy trust is
# separate and uses the exact certificate's TBS identity. No machine root is added.
Import-Certificate -FilePath $publicPath -CertStoreLocation Cert:\CurrentUser\Root | Out-Null
$state = [ordered]@{schema=1;workspace=$script:DevRoot;thumbprint=$certificate.Thumbprint;public_certificate_sha256=(Get-FileHash $publicPath -Algorithm SHA256).Hash;created_utc=[DateTime]::UtcNow.ToString('o');expires_utc=$certificate.NotAfter.ToUniversalTime().ToString('o');key_exportable=$false;policy_id=$null}
$state | ConvertTo-Json | Set-Content -LiteralPath $statePath
Write-Output "Created non-exportable local RSA signer: $($certificate.Thumbprint)"
