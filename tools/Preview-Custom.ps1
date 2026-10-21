param([switch]$Replay, [int]$TimeoutSeconds=120)
# Host-only adapter. Pixel generation, input routing and state remain in Rust.
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
. (Join-Path $PSScriptRoot 'windows/DevSigning.ps1')
$exe=Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'
if (-not (Test-Path -LiteralPath $exe)) {throw 'Build the comparison profile first: cargo build --locked --offline -p xtask --profile m2-comparison'}
$null=Set-DevExecutableSignature $exe
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
[Windows.Forms.Application]::SetHighDpiMode([Windows.Forms.HighDpiMode]::PerMonitorV2) | Out-Null
$start=[Diagnostics.ProcessStartInfo]::new($exe)
$start.ArgumentList.Add('custom-host')
$start.WorkingDirectory=$root
$start.UseShellExecute=$false; $start.CreateNoWindow=$true
$start.RedirectStandardInput=$true; $start.RedirectStandardOutput=$true; $start.RedirectStandardError=$true
$script:customProcess=[Diagnostics.Process]::Start($start)
$stderr=$script:customProcess.StandardError.ReadToEndAsync()
$script:customForm=$null; $script:customImage=$null; $script:customResponse=$null
$script:customSequence=0; $script:customHighSurrogate=''; $script:customFrames=0
function Read-CustomResponse {
    $pending=$script:customProcess.StandardOutput.ReadLineAsync()
    if (-not $pending.Wait(30000)) {throw 'Rust host response timed out'}
    if (-not $pending.Result) {throw 'Rust host exited without a response'}
    return ($pending.Result | ConvertFrom-Json)
}
function Send-Custom($command) {
    $script:customProcess.StandardInput.WriteLine(($command | ConvertTo-Json -Compress -Depth 10))
    $script:customProcess.StandardInput.Flush()
    $answer=Read-CustomResponse
    if (-not $answer.ok) {throw "Rust host rejected input: $($answer.reason)"}
    $script:customResponse=$answer
    $stream=[IO.MemoryStream]::new([IO.File]::ReadAllBytes($answer.frame))
    try {$temporary=[Drawing.Bitmap]::new($stream);try {$next=[Drawing.Bitmap]::new($temporary)}finally{$temporary.Dispose()}}finally{$stream.Dispose()}
    if ($script:customImage) {$script:customImage.Dispose()}
    $script:customImage=$next
    $script:customForm.BackgroundImage=$next
    $script:customForm.Invalidate()
    $script:customForm.Update()
    $script:customFrames++
}
try {
    $ready=Read-CustomResponse
    if (-not $ready.ready) {throw 'Rust host not ready'}
    $script:customForm=[Windows.Forms.Form]::new()
    $script:customForm.Text='ContinuumOS · P2-05 hosted custom candidate'
    $script:customForm.ClientSize=[Drawing.Size]::new(1280,720)
    $script:customForm.FormBorderStyle='FixedSingle';$script:customForm.MaximizeBox=$false
    $script:customForm.BackgroundImageLayout='None';$script:customForm.KeyPreview=$true
    $script:customForm.AutoScaleMode='None'
    $script:customForm.Add_KeyDown({param($sender,$e)
        $usage=switch ([int]$e.KeyCode) {8 {42} 9 {43} 13 {40} 32 {44} 37 {80} 39 {79} 46 {76} 16 {225} 17 {224} 18 {226} 38 {82} 40 {81} default {if ([int]$e.KeyCode -ge 65 -and [int]$e.KeyCode -le 90) {[int]$e.KeyCode-61}else{0}}}
        if ($usage -and -not ($script:customPressed -contains $usage)) {$script:customPressed+=@($usage);Send-Custom @{kind='key';usage=$usage;pressed=$true}}
        if ($e.KeyCode -eq 'Tab') {$e.SuppressKeyPress=$true}
    })
    $script:customPressed=@()
    $script:customForm.Add_KeyUp({param($sender,$e)
        $usage=switch ([int]$e.KeyCode) {8 {42} 9 {43} 13 {40} 32 {44} 37 {80} 39 {79} 46 {76} 16 {225} 17 {224} 18 {226} 38 {82} 40 {81} default {if ([int]$e.KeyCode -ge 65 -and [int]$e.KeyCode -le 90) {[int]$e.KeyCode-61}else{0}}}
        if ($script:customPressed -contains $usage) {$script:customPressed=@($script:customPressed | Where-Object {$_ -ne $usage});Send-Custom @{kind='key';usage=$usage;pressed=$false}}
    })
    $script:customForm.Add_KeyPress({param($sender,$e)
        if ($e.KeyChar -eq [char]13 -and $script:customResponse.state.focus -eq 'editor') {Send-Custom @{kind='text';text=[string][char]10};return}
        if ($script:customResponse.state.focus -notin @('editor','search') -or [char]::IsControl($e.KeyChar)) {return}
        if ([char]::IsHighSurrogate($e.KeyChar)) {$script:customHighSurrogate=[string]$e.KeyChar;return}
        $text=$script:customHighSurrogate+[string]$e.KeyChar;$script:customHighSurrogate=''
        Send-Custom @{kind='text';text=$text}
    })
    $script:customForm.Add_MouseDown({param($sender,$e) $script:customForm.Capture=$true;Send-Custom @{kind='pointer';x=$e.X;y=$e.Y;button=1;phase='down'}})
    $script:customForm.Add_MouseUp({param($sender,$e) Send-Custom @{kind='pointer';x=$e.X;y=$e.Y;button=1;phase='up'};$script:customForm.Capture=$false})
    $script:customForm.Add_MouseWheel({param($sender,$e) Send-Custom @{kind='scroll';target='files-window';y=(-$e.Delta/120*72)}})
    $script:customForm.Add_Deactivate({$script:customPressed=@();$script:customHighSurrogate='';Send-Custom @{kind='reset';target=''}})
    $timer=[Windows.Forms.Timer]::new();$timer.Interval=1000
    $script:customDeadline=[DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    $timer.Add_Tick({if ([DateTime]::UtcNow -ge $script:customDeadline) {$script:customForm.Close()}})
    $script:customForm.Add_Shown({
        if ($Replay) {
            $steps=Get-Content (Join-Path $root 'fixtures/m2/ui-v1/workflow.json') -Raw | ConvertFrom-Json
            for ($index=0;$index -lt $steps.steps.Count;$index++) {Send-Custom @{step=$index};[Windows.Forms.Application]::DoEvents()}
            $script:customForm.Close()
        }else{Send-Custom @{frame=$true}}
    })
    $timer.Start()
    [Windows.Forms.Application]::Run($script:customForm)
    $timer.Stop();$timer.Dispose()
    $script:customProcess.StandardInput.WriteLine('{"quit":true}')
    $script:customProcess.StandardInput.Flush()
    if (-not $script:customProcess.WaitForExit(10000)) {throw 'Rust host did not stop after quit'}
    if ($script:customProcess.ExitCode -ne 0) {throw ($stderr.Result)}
    @{schema_version=1;status='passed';replay=[bool]$Replay;pid=$ready.pid;submitted_frames=$script:customFrames;host_pixel_conversion='Rust RGBA to BGR24 BMP; WinForms owned bitmap clone';dpi='PerMonitorV2; fixed 1280x720 physical client, scale 1; no stretch';native_ime=$false;native_accessibility=$false;final=$script:customResponse.state} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $ready.run 'native-preview.json')
    Write-Output "Native preview evidence: $($ready.run)"
}finally{
    if ($script:customProcess -and -not $script:customProcess.HasExited) {$script:customProcess.Kill($true);$script:customProcess.WaitForExit()}
    if ($script:customForm) {$script:customForm.BackgroundImage=$null;$script:customForm.Dispose()}
    if ($script:customImage) {$script:customImage.Dispose()}
    $script:customProcess.Dispose()
}
