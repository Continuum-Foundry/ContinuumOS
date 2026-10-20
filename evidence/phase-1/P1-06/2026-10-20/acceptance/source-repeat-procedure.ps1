param([Parameter(Mandatory)][string]$AcceptancePath)
$ErrorActionPreference='Stop'
$root=(Get-Location).Path
. ./tools/Enter-Dev.ps1
$original=Get-Content (Join-Path $AcceptancePath 'acceptance-results.json') -Raw | ConvertFrom-Json -AsHashtable
if ($original.status -ne 'passed') {throw 'Original acceptance must finish successfully'}
$final=Join-Path $root ('build/m1/acceptance-p1-06-final-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory $final | Out-Null
Copy-Item -LiteralPath (Join-Path $AcceptancePath 'acceptance-results.json') -Destination (Join-Path $final 'original-acceptance-results.json')
Get-ChildItem -LiteralPath $AcceptancePath -File -Filter '*.log' | Copy-Item -Destination $final
$original.original_acceptance=$AcceptancePath
$original.superseded_runs=@{}
$original.status='running'
try {
    foreach ($case in @($original.runs.Keys)) {
        if ($case -in 'm0','dependencies','prerequisites') {continue}
        $stale=$false
        foreach ($file in Get-ChildItem -LiteralPath $original.runs[$case] -Recurse -Filter build-inputs.json | Where-Object FullName -NotMatch '[\\/]cargo[\\/]') {
            $inputs=Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json -AsHashtable
            foreach ($name in $inputs.sources.Keys) {
                if ((Get-FileHash -LiteralPath (Join-Path $root $name)).Hash.ToLowerInvariant() -ne $inputs.sources[$name]) {$stale=$true;break}
            }
        }
        if (-not $stale) {continue}
        $log=Join-Path $final "repeat-$case.log"
        $arguments=if ($case -eq 'run') {@('xtask','run','--profile','vm-baseline')} else {@('xtask','test','--suite','m1','--case',$case)}
        & cargo @arguments *> $log
        $code=$LASTEXITCODE
        $original.commands+=@{name="repeat-$case";executable='cargo';arguments=$arguments;exit_code=$code;log="repeat-$case.log"}
        if ($code -ne 0) {throw "Repeat failed: $case"}
        $path=[regex]::Match([IO.File]::ReadAllText($log),'M1 artifacts: ([^\r\n]+)').Groups[1].Value
        if (-not $path -or (Get-Content (Join-Path $path 'result.json') -Raw | ConvertFrom-Json).status -ne 'passed') {throw "Missing repeat result: $case"}
        $original.superseded_runs[$case]=$original.runs[$case]
        $original.runs[$case]=$path
    }
    $original.status='passed'
} catch {$original.status='failed';$original.error=$_.Exception.Message;throw}
finally {
    $original | ConvertTo-Json -Depth 20 | Set-Content (Join-Path $final 'acceptance-results.json')
    "Final P1-06 selection: $final"
}
