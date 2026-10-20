# Exact hashes for the existing reviewed tool bundles; never signs/mutates tools.
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSEdition -ne 'Desktop') { throw 'Use Windows PowerShell for ConfigCI.' }
Import-Module ConfigCI
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$directory = Join-Path $root 'work/windows-dev-signing'
$state = Get-Content (Join-Path $directory 'state.json') -Raw | ConvertFrom-Json
if (Test-Path (Join-Path $directory 'tools-manifest.json')) { throw 'Tool policy already prepared; review a versioned update rather than silently reenrolling files.' }
$version = (Get-Content (Join-Path $root 'rust-toolchain.toml') | Select-String '^channel\s*=\s*"([^"]+)"').Matches.Groups[1].Value
if ($version -ne '1.99.1') { throw 'Review the changed pinned Rust version before enrollment.' }
$files = @()
foreach ($part in @('work/toolchains/cargo/bin', "work/toolchains/rustup/toolchains/$version-x86_64-pc-windows-msvc")) {
    $files += Get-ChildItem -LiteralPath (Join-Path $root $part) -Recurse -File | Where-Object { $_.Extension -in '.exe','.dll' }
}
$files += Get-Item (Join-Path $root 'work/tools/qemu/qemu-system-x86_64.exe')
$files += Get-ChildItem -LiteralPath (Join-Path $root 'work/tools/qemu') -Recurse -File -Filter '*.dll'
$lock = Get-Content (Join-Path $root 'config/m1-inputs.lock.json') -Raw | ConvertFrom-Json
$isoTool = @($lock.files | Where-Object id -eq 'xorriso')[0]
$files += Get-Item (Join-Path $root $isoTool.path)
$files += Get-ChildItem -LiteralPath (Split-Path (Join-Path $root $isoTool.path) -Parent) -Recurse -File -Filter '*.dll'
$files = @($files | Sort-Object FullName -Unique)
foreach ($pin in @($lock.files | Where-Object { $_.id -in 'qemu','xorriso' })) {
    if ((Get-FileHash (Join-Path $root $pin.path) -Algorithm SHA256).Hash -ne $pin.sha256) { throw "Locked tool changed: $($pin.id)" }
}
$inventory = @($files | ForEach-Object {
    if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Refusing reparse-point tool enrollment.' }
    @{path=$_.FullName.Substring($root.Length + 1).Replace('\','/');sha256=(Get-FileHash $_.FullName -Algorithm SHA256).Hash;bytes=$_.Length}
})
$rules = New-CIPolicyRule -DriverFilePath $files.FullName -Level Hash
$xmlPath = Join-Path $directory 'tools.xml'
New-CIPolicy -FilePath $xmlPath -Rules $rules -UserPEs -MultiplePolicyFormat | Out-Null
Set-CIPolicyIdInfo -FilePath $xmlPath -SupplementsBasePolicyID $state.policy_id -PolicyName 'ContinuumOS Pinned Development Tools' -ResetPolicyID | Out-Null
Set-CIPolicyVersion -FilePath $xmlPath -Version '1.0.0.0'
Set-RuleOption -FilePath $xmlPath -Option 3 -Delete
[xml]$xml = Get-Content $xmlPath
$ns = New-Object Xml.XmlNamespaceManager($xml.NameTable)
$ns.AddNamespace('ci','urn:schemas-microsoft-com:sipolicy')
# User-mode file hashes only; no development-tool kernel authorization.
foreach ($node in @($xml.SelectNodes('//ci:SigningScenario[@Value="131"]',$ns))) { $null = $node.ParentNode.RemoveChild($node) }
if ($xml.SelectNodes('//ci:FileRules/ci:Allow[not(@Hash)]',$ns).Count -ne 0) { throw 'Unexpected non-hash tool authorization.' }
$xml.Save($xmlPath)
$binary = Join-Path $directory 'tools.cip'
ConvertFrom-CIPolicy -XmlFilePath $xmlPath -BinaryFilePath $binary | Out-Null
@{policy_id=[string]$xml.SiPolicy.PolicyID;base_policy_id=$state.policy_id;binary_sha256=(Get-FileHash $binary -Algorithm SHA256).Hash;files=$inventory} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $directory 'tools-manifest.json')
Write-Output "Prepared exact-hash supplemental policy for $($files.Count) local tool files; nothing deployed."
