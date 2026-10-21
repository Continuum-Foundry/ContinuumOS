# Cargo passes the executable followed by its original argument vector.
# Use $args rather than named parameter binding so --flags and their values survive.
. (Join-Path $PSScriptRoot 'DevSigning.ps1')
if ($args.Count -lt 1) { throw 'Expected a first-party host executable.' }
$executable = [string]$args[0]
$forward = @()
if ($args.Count -gt 1) { $forward = @($args[1..($args.Count - 1)]) }
$null = Set-DevExecutableSignature $executable
try {
    & $executable @forward
    exit $LASTEXITCODE
} catch {
    throw "Signed host executable could not run. Check tools/windows/Test-DevSigning.ps1 and the active App Control policy. $($_.Exception.Message)"
}
