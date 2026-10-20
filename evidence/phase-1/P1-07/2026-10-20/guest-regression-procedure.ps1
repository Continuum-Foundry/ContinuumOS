# Independent guest regression evidence while M0 host execution is blocked.
# This does NOT satisfy the complete M1 gate or waive its failed host row.
$ErrorActionPreference = 'Stop'
. ./tools/Enter-Dev.ps1
$run = Join-Path (Get-Location) ('build/m1/p1-07-guest-regressions-' + [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Path $run | Out-Null
$report = @{scope='Independent guest regressions only; full M1 acceptance remains blocked by host Application Control';status='running';cases=@()}
try {
    foreach ($case in 'run','iso-smoke','iso-negative','boot-metadata','boot-reject','exception-invalid-opcode','exception-page-fault','panic','exception-emergency','memory-ownership','memory-exhaustion','guard-fault','write-protect-fault','execute-disable-fault','emergency-stack','timer-progress','timer-masked','clock-read','idle-wakeup','timer-exception','acpi-corrupt','timer-unsupported','timer-calibration') {
        $arguments = if ($case -eq 'run') { @('xtask','run','--profile','vm-baseline') } else { @('xtask','test','--suite','m1','--case',$case) }
        & cargo @arguments *> (Join-Path $run "$case.log")
        $exitCode = $LASTEXITCODE
        $text = Get-Content (Join-Path $run "$case.log") -Raw
        $artifacts = [regex]::Match($text,'M1 artifacts: ([^\r\n]+)').Groups[1].Value
        $report.cases += @{case=$case;exit_code=$exitCode;artifacts=$artifacts}
        $report | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $run 'guest-regressions.json')
        if ($exitCode -ne 0) { throw "$case failed; see $run" }
    }
    $report.status = 'passed'
} catch { $report.status='failed'; $report['error']=$_.Exception.Message; throw }
finally {
    $report | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $run 'guest-regressions.json')
    "Independent guest evidence: $run"
}
