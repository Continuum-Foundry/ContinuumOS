# Shared by setup, preflight and the Cargo runner. Never exports a private key.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:DevRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$script:DevDirectory = Join-Path $script:DevRoot 'work/windows-dev-signing'

function Get-DevSigningState {
    $path = Join-Path $script:DevDirectory 'state.json'
    if (-not (Test-Path -LiteralPath $path)) { throw 'Run tools/windows/Initialize-DevSigning.ps1 first.' }
    $state = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    if ($state.schema -ne 1 -or $state.workspace -ne $script:DevRoot -or $state.thumbprint -notmatch '^[A-F0-9]{40}$') {
        throw 'Invalid or relocated development signing state; rerun setup explicitly.'
    }
    return $state
}

function Get-DevCertificate {
    $state = Get-DevSigningState
    $certificate = Get-Item -LiteralPath ("Cert:\CurrentUser\My\" + $state.thumbprint)
    if (-not $certificate.HasPrivateKey -or $certificate.NotAfter -lt (Get-Date).AddDays(30) -or $certificate.NotBefore -gt (Get-Date)) {
        throw 'Development signer missing its key, not yet valid, or expires within 30 days. Rotate using the runbook before continuing.'
    }
    if (@($certificate.EnhancedKeyUsageList | ForEach-Object { [string]$_.ObjectId }) -notcontains '1.3.6.1.5.5.7.3.3') { throw 'Signer lacks code-signing EKU.' }
    return $certificate
}

function Assert-DevExecutablePath([string]$Path, [string]$CompilerSource = '') {
    $full = (Resolve-Path -LiteralPath $Path).Path
    $relative = [IO.Path]::GetRelativePath($script:DevRoot, $full).Replace('\','/')
    # Deliberately limited to this project's two first-party Cargo packages and probes.
    # Compiler outputs use a separate path with local source and lockfile checks.
    $allowedRoot = $relative -match '^build/cargo/' -or $relative -match '^work/acceptance/[^/]+/host cargo with spaces/' -or $relative -match '^build/windows-dev-signing/'
    $allowedName = [IO.Path]::GetFileName($full) -match '^(xtask(-[0-9a-f]+)?|continuum_kernel-[0-9a-f]+|dev-probe-[0-9a-f]+|dev-compiler-wrapper)\.exe$'
    if ($CompilerSource) {
        $compiler = Get-Content (Join-Path $script:DevDirectory 'compiler.json') -Raw | ConvertFrom-Json
        foreach ($lock in $compiler.locks) {
            if ((Get-FileHash (Join-Path $script:DevRoot $lock.path) -Algorithm SHA256).Hash -ne $lock.sha256) { throw 'A Cargo.lock changed; review dependency changes before reenrolling compiler signing.' }
        }
        $source = (Resolve-Path -LiteralPath $CompilerSource).Path
        if (-not $source.StartsWith($script:DevRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetExtension($source) -ne '.rs') { throw 'Compiler source must be in this offline workspace or its local dependency cache.' }
        $allowedRoot = $relative -match '^build/' -or $relative -match '^work/(acceptance|dependency-probes)/'
        $allowedName = [IO.Path]::GetExtension($full) -in '.exe','.dll'
    }
    if (-not $allowedRoot -or -not $allowedName) { throw "Refusing to sign an undeclared host artifact: $full" }
    $item = Get-Item -LiteralPath $full
    while ($item -and $item.FullName -ne $script:DevRoot) {
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Signing through reparse points is prohibited.' }
        if ($item -is [IO.FileInfo]) { $item = $item.Directory } else { $item = $item.Parent }
    }
    return (Resolve-Path -LiteralPath $Path).Path
}

function Set-DevExecutableSignature([string]$Path, [string]$CompilerSource = '') {
    $full = Assert-DevExecutablePath $Path $CompilerSource
    $state = Get-DevSigningState
    $certificate = Get-DevCertificate
    $before = (Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash.ToLowerInvariant()
    $signature = Get-AuthenticodeSignature -LiteralPath $full
    $alreadySigned = $signature.Status -eq 'Valid' -and $signature.SignerCertificate.Thumbprint -eq $certificate.Thumbprint
    if (-not $alreadySigned) {
        # No timestamp server or cloud signing: private project bytes stay local.
        $signature = Set-AuthenticodeSignature -LiteralPath $full -Certificate $certificate -HashAlgorithm SHA256
    }
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ne $state.thumbprint) {
        throw "Authenticode verification failed: $($signature.Status): $($signature.StatusMessage)"
    }
    $after = (Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash.ToLowerInvariant()
    $record = [ordered]@{utc=[DateTime]::UtcNow.ToString('o');path=$full;before_sha256=$before;signed_sha256=$after;thumbprint=$certificate.Thumbprint;already_signed=$alreadySigned;expires_utc=$certificate.NotAfter.ToUniversalTime().ToString('o')}
    # Separate files allow concurrent Cargo processes without interleaved JSON records.
    $log = Join-Path $script:DevDirectory 'signatures'
    New-Item -ItemType Directory -Force $log | Out-Null
    $record | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $log (([guid]::NewGuid().ToString()) + '.json'))
    return $record
}
