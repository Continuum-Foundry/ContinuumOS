# Elevated Windows PowerShell helper. No download, private key, task or reboot.
param([Parameter(Mandatory=$true)][ValidateSet('Audit','Tools','Enforce','Rollback','Status')][string]$Action)
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'This machine-wide action needs an administrator/UAC elevation.' }
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$directory = Join-Path $root 'work/windows-dev-signing'
$state = Get-Content (Join-Path $directory 'state.json') -Raw | ConvertFrom-Json
$manifest = Get-Content (Join-Path $directory 'policy-manifest.json') -Raw | ConvertFrom-Json
$ci = Join-Path $env:windir 'System32/CiTool.exe'
$registry = 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy'
$resultPath = Join-Path $directory ("admin-$($Action.ToLower()).json")

function Invoke-Ci([string[]]$Arguments) {
    $output = & $ci @Arguments -json
    if ($LASTEXITCODE -ne 0) { throw "CiTool failed ($LASTEXITCODE): $output" }
    $value = ($output -join "`n") | ConvertFrom-Json
    if ($value.OperationResult -ne 0) { throw "CiTool rejected operation: $output" }
    return $value
}
function Get-Policies { return (Invoke-Ci @('-lp')).Policies }
function Test-ActivePolicy($Policies, [string]$PolicyId) {
    return @($Policies | Where-Object { $_.PolicyID.Trim('{}') -eq $PolicyId.Trim('{}') -and $_.IsEnforced -and $_.IsAuthorized }).Count -gt 0
}
function Assert-DevelopmentPolicyMode($Policies, [bool]$Audit) {
    $policy = @($Policies | Where-Object { $_.PolicyID.Trim('{}') -eq $state.policy_id.Trim('{}') })
    if ($policy.Count -ne 1 -or -not (Test-ActivePolicy $Policies $state.policy_id)) { throw 'Expected development policy is not active/authorized.' }
    $isAudit = $policy[0].PolicyOptions -contains 'Enabled:Audit Mode'
    $version = if ($Audit) { '1.0.0.0' } else { '1.0.0.1' }
    if ($isAudit -ne $Audit -or $policy[0].VersionString -ne $version) { throw 'Live development policy mode/version does not match the requested deployment; SAC retained where possible.' }
}

try {
    if ($state.policy_id -ne $manifest.policy_id -or $state.policy_id -notmatch '^\{[A-F0-9-]{36}\}$') { throw 'Invalid prepared policy identity.' }
    foreach ($mode in 'audit','enforced') {
        $expected = $manifest.($mode + '_sha256')
        if ((Get-FileHash (Join-Path $directory ($mode + '.cip')) -Algorithm SHA256).Hash -ne $expected) { throw "Prepared $mode binary changed." }
    }
    $before = @(Get-Policies)
    if ($Action -eq 'Status') {
        $before | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $directory 'active-policies.json')
    } elseif ($Action -eq 'Audit') {
        $backup = Join-Path $directory 'before-policy-change'
        if (Test-Path -LiteralPath $backup) { throw 'Initial policy snapshot already exists; inspect state instead of overwriting recovery data.' }
        New-Item -ItemType Directory $backup | Out-Null
        $before | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $backup 'policies.json')
        @{sac_state=(Get-ItemProperty $registry).VerifiedAndReputablePolicyState;utc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $backup 'sac.json')
        Copy-Item -LiteralPath (Join-Path $env:windir 'System32/CodeIntegrity/CiPolicies/Active') -Destination $backup -Recurse
        Get-MpComputerStatus | Select-Object AntivirusEnabled,RealTimeProtectionEnabled,BehaviorMonitorEnabled | ConvertTo-Json | Set-Content (Join-Path $backup 'antivirus.json')
        $null = Invoke-Ci @('-up', (Join-Path $directory 'audit.cip'))
        Assert-DevelopmentPolicyMode @(Get-Policies) $true
    } elseif ($Action -eq 'Tools') {
        # Non-MDM deployment requires Windows ISG service initialization.
        Get-Service AppIDSvc | Select-Object Name,Status,StartType | ConvertTo-Json | Set-Content (Join-Path $directory 'before-isg-service.json')
        & (Join-Path $env:windir 'System32/appidtel.exe') start
        if ($LASTEXITCODE -ne 0) { throw 'Windows ISG service initialization failed.' }
        $toolsManifest = Get-Content (Join-Path $directory 'tools-manifest.json') -Raw | ConvertFrom-Json
        $toolsBinary = Join-Path $directory 'tools.cip'
        if ($toolsManifest.base_policy_id -ne $state.policy_id -or (Get-FileHash $toolsBinary -Algorithm SHA256).Hash -ne $toolsManifest.binary_sha256) { throw 'Pinned tools policy changed or targets another base.' }
        foreach ($file in $toolsManifest.files) {
            $path = [IO.Path]::GetFullPath((Join-Path $root $file.path))
            if (-not $path.StartsWith($root + '\', [StringComparison]::OrdinalIgnoreCase) -or (Get-FileHash $path -Algorithm SHA256).Hash -ne $file.sha256) { throw 'Reviewed tool inventory changed.' }
        }
        $null = Invoke-Ci @('-up', $toolsBinary)
        if (-not (Test-ActivePolicy @(Get-Policies) $toolsManifest.policy_id)) { throw 'Pinned tools policy not active.' }
    } elseif ($Action -eq 'Enforce') {
        if (-not (Test-Path (Join-Path $directory 'admin-audit.json'))) { throw 'Deploy and inspect the audit policy first.' }
        $auditResult = Get-Content (Join-Path $directory 'admin-audit.json') -Raw | ConvertFrom-Json
        if ($auditResult.status -ne 'passed') { throw 'Audit deployment did not succeed.' }
        $toolsResult = Get-Content (Join-Path $directory 'admin-tools.json') -Raw | ConvertFrom-Json
        if ($toolsResult.status -ne 'passed') { throw 'Enroll reviewed pinned tool hashes before enforcement.' }
        $toolsManifest = Get-Content (Join-Path $directory 'tools-manifest.json') -Raw | ConvertFrom-Json
        if (-not (Test-ActivePolicy $before $toolsManifest.policy_id)) { throw 'Reviewed pinned tools policy is not active.' }
        Assert-DevelopmentPolicyMode $before $true
        # Activate the replacement BEFORE retiring consumer SAC. Other policies stay.
        $null = Invoke-Ci @('-up', (Join-Path $directory 'enforced.cip'))
        Assert-DevelopmentPolicyMode @(Get-Policies) $false
        Set-ItemProperty -LiteralPath $registry -Name VerifiedAndReputablePolicyState -Type DWord -Value 0
        $null = Invoke-Ci @('-r')
        $after = @(Get-Policies)
        Assert-DevelopmentPolicyMode $after $false
        if (Test-ActivePolicy $after '{0283ac0f-fff1-49ae-ada1-8a933130cad6}') { throw 'SAC enforcement still active after refresh; restart may be required. No restart performed.' }
    } elseif ($Action -eq 'Rollback') {
        $saved = Get-Content (Join-Path $directory 'before-policy-change/sac.json') -Raw | ConvertFrom-Json
        if ($saved.sac_state -ne 1) { throw 'Automatic rollback only supports the captured enforced SAC state; review other configurations manually.' }
        Set-ItemProperty -LiteralPath $registry -Name VerifiedAndReputablePolicyState -Type DWord -Value $saved.sac_state
        $null = Invoke-Ci @('-r')
        if (-not (Test-ActivePolicy @(Get-Policies) '{0283ac0f-fff1-49ae-ada1-8a933130cad6}')) { throw 'SAC did not reactivate. Development policy retained to preserve enforcement; use recovery runbook.' }
        if (Test-Path (Join-Path $directory 'tools-manifest.json')) {
            $toolsManifest = Get-Content (Join-Path $directory 'tools-manifest.json') -Raw | ConvertFrom-Json
            if (Test-ActivePolicy @(Get-Policies) $toolsManifest.policy_id) { $null = Invoke-Ci @('-rp', $toolsManifest.policy_id) }
        }
        $null = Invoke-Ci @('-rp', $state.policy_id)
    }
    @{status='passed';action=$Action;utc=[DateTime]::UtcNow.ToString('o');policy_id=$state.policy_id;policies=@(Get-Policies)} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $resultPath
} catch {
    @{status='failed';action=$Action;utc=[DateTime]::UtcNow.ToString('o');message=$_.Exception.Message} | ConvertTo-Json | Set-Content -LiteralPath $resultPath
    throw
}
