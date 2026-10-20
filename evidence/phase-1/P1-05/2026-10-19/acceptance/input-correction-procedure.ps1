param([Parameter(Mandatory)][string]$AcceptancePath,[string]$RepeatPath)
$ErrorActionPreference='Stop'
. ./tools/Enter-Dev.ps1
$original=Get-Content (Join-Path $AcceptancePath 'acceptance-results.json') -Raw | ConvertFrom-Json -AsHashtable
if ($original.status -ne 'passed') { throw 'Original matrix must finish first' }
$run=Join-Path (Get-Location) ('build/m1/acceptance-p1-05-final-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory $run | Out-Null
Get-ChildItem -LiteralPath $AcceptancePath -File -Filter '*.log' | Copy-Item -Destination $run
$result=$original
$result.status='running'
$result.original_acceptance=$AcceptancePath
$result.superseded_runs=@{}
$result.correction='Kernel README is included in the input hash inventory. Updated documentation changed that input during early cases; repeat the affected cases without changing executable source. Original observations remain retained and do not count toward the final selected matrix.'
try {
    if ($RepeatPath) {
        $repeated=Get-Content (Join-Path $RepeatPath 'repeat-results.json') -Raw | ConvertFrom-Json -AsHashtable
        if ($repeated.status -ne 'passed') { throw 'Source repeats must pass' }
        foreach ($case in 'run','iso-smoke','iso-negative') {
            $result.superseded_runs[$case]=$result.runs[$case]
            $result.runs[$case]=$repeated.runs[$case]
        }
        $result.commands+=$repeated.commands
        Get-ChildItem -LiteralPath $RepeatPath -File -Filter '*.log' | Copy-Item -Destination $run
        $result.status='passed'
        return
    }
    foreach ($case in 'run','iso-smoke','iso-negative') {
        $result.superseded_runs[$case]=$result.runs[$case]
        $arguments=if ($case -eq 'run') { @('xtask','run','--profile','vm-baseline') } else { @('xtask','test','--suite','m1','--case',$case) }
        $log=Join-Path $run "repeat-$case.log"
        & cargo @arguments *> $log
        $code=$LASTEXITCODE
        $result.commands+=@{name="repeat-$case";executable='cargo';arguments=$arguments;exit_code=$code;log="repeat-$case.log"}
        if ($code -ne 0) { throw "Repeat $case failed" }
        $path=[regex]::Match([IO.File]::ReadAllText($log),'M1 artifacts: ([^\r\n]+)').Groups[1].Value
        if (-not $path -or (Get-Content (Join-Path $path 'result.json') -Raw | ConvertFrom-Json).status -ne 'passed') { throw "Missing passed result for $case" }
        $result.runs[$case]=$path
    }
    $result.status='passed'
} catch { $result.status='failed'; $result.error=$_.Exception.Message; throw }
finally {
    $result | ConvertTo-Json -Depth 20 | Set-Content (Join-Path $run 'acceptance-results.json')
    "Final P1-05 acceptance evidence: $run"
}
