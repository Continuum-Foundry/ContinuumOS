$ErrorActionPreference='Stop'
. ./tools/Enter-Dev.ps1
$run=Join-Path (Get-Location) ('build/m1/p1-05-source-repeats-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory $run | Out-Null
$result=@{status='running';commands=@();runs=@{}}
try {
    foreach ($case in 'run','iso-smoke','iso-negative') {
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
finally { $result | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'repeat-results.json'); "Source repeat evidence: $run" }
