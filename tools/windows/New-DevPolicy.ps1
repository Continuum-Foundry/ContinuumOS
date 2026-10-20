# ConfigCI runs in built-in Windows PowerShell 5.1 (not pwsh).
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSEdition -ne 'Desktop') { throw 'Run this script with powershell.exe, not pwsh.' }
Import-Module ConfigCI
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$directory = Join-Path $root 'work/windows-dev-signing'
$statePath = Join-Path $directory 'state.json'
$state = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
if (Test-Path -LiteralPath (Join-Path $directory 'policy-manifest.json')) {
    throw 'Policy already prepared. Do not silently replace its identity; use the lifecycle runbook for updates.'
}
$publicPath = Join-Path $directory 'development-signer.cer'
if ((Get-FileHash $publicPath -Algorithm SHA256).Hash -ne $state.public_certificate_sha256) { throw 'Public signer certificate changed.' }
$template = Join-Path $env:windir 'schemas/CodeIntegrity/ExamplePolicies/SmartAppControl.xml'
$audit = Join-Path $directory 'audit.xml'
Copy-Item -LiteralPath $template -Destination $audit
Set-CIPolicyIdInfo -FilePath $audit -PolicyName 'ContinuumOS Windows Development' -ResetPolicyID | Out-Null
Set-CIPolicyVersion -FilePath $audit -Version '1.0.0.0'
# Conditional lockdown is reserved for Windows Smart App Control, not custom bases.
[xml]$xml = Get-Content -LiteralPath $audit
$ns = New-Object Xml.XmlNamespaceManager($xml.NameTable)
$ns.AddNamespace('ci','urn:schemas-microsoft-com:sipolicy')
$conditional = $xml.SelectSingleNode('//ci:Rule[ci:Option="Enabled:Conditional Windows Lockdown Policy"]', $ns)
if (-not $conditional) { throw 'Installed SAC template changed; review before adapting.' }
$null = $conditional.ParentNode.RemoveChild($conditional)
$xml.Save($audit)
Add-SignerRule -FilePath $audit -CertificatePath $publicPath -User
foreach ($option in 3,6,9,10,16) { Set-RuleOption -FilePath $audit -Option $option }
ConvertFrom-CIPolicy -XmlFilePath $audit -BinaryFilePath (Join-Path $directory 'audit.cip') | Out-Null
$enforced = Join-Path $directory 'enforced.xml'
Copy-Item -LiteralPath $audit -Destination $enforced
Set-RuleOption -FilePath $enforced -Option 3 -Delete
Set-CIPolicyVersion -FilePath $enforced -Version '1.0.0.1'
ConvertFrom-CIPolicy -XmlFilePath $enforced -BinaryFilePath (Join-Path $directory 'enforced.cip') | Out-Null
[xml]$xml = Get-Content -LiteralPath $enforced
$state.policy_id = [string]$xml.SiPolicy.PolicyID
$state | ConvertTo-Json | Set-Content -LiteralPath $statePath
@{template_sha256=(Get-FileHash $template -Algorithm SHA256).Hash;audit_sha256=(Get-FileHash (Join-Path $directory 'audit.cip') -Algorithm SHA256).Hash;enforced_sha256=(Get-FileHash (Join-Path $directory 'enforced.cip') -Algorithm SHA256).Hash;policy_id=$state.policy_id;prepared_utc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $directory 'policy-manifest.json')
Write-Output "Prepared audit and enforced policy $($state.policy_id); nothing deployed."
