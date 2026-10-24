param([switch]$Replay, [switch]$Selected, [switch]$ResourceReload, [string]$ResourcePath="", [int]$TimeoutSeconds=120, [ValidateSet("Custom","Slint")][string]$Candidate="Custom")
# Host-only adapter. Pixel generation, input routing and state remain in Rust.
$ErrorActionPreference='Stop'
if ($ResourceReload) {$Selected=$true}
if ($Selected -and $Candidate -ne 'Custom') {throw 'Selected behavior is custom only; Slint is frozen comparison material.'}
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
. (Join-Path $PSScriptRoot 'windows/DevSigning.ps1')
$exe=Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'
if (-not (Test-Path -LiteralPath $exe)) {throw 'Build the comparison profile first: cargo build --locked --offline -p xtask --profile m2-comparison'}
$null=Set-DevExecutableSignature $exe
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
[Windows.Forms.Application]::SetHighDpiMode([Windows.Forms.HighDpiMode]::PerMonitorV2) | Out-Null
$start=[Diagnostics.ProcessStartInfo]::new($exe)
$start.ArgumentList.Add($Candidate.ToLowerInvariant()+'-host')
$start.WorkingDirectory=$root
$start.UseShellExecute=$false; $start.CreateNoWindow=$true
$start.RedirectStandardInput=$true; $start.RedirectStandardOutput=$true; $start.RedirectStandardError=$true
$script:customProcess=[Diagnostics.Process]::Start($start)
$stderr=$script:customProcess.StandardError.ReadToEndAsync()
$script:customForm=$null; $script:customImage=$null; $script:customResponse=$null
$script:customSequence=0; $script:customHighSurrogate=''; $script:customFrames=0
$script:customAnimation=$null
$script:customFailure=$null
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
    if ($ResourceReload -and $answer.resources) {
        $status=$answer.resources.status
        $reason=if ($status.PSObject.Properties["reason"]) {$status.reason} else {""}
        $script:customForm.Text="ContinuumOS - ephemeral hosted L0/L1 - generation $($answer.resources.active_generation) - $($status.outcome) $reason"
        if ($script:customResponse) {$script:customResponse.resources=$answer.resources}
    }
    if ($answer.PSObject.Properties["unchanged"] -and $answer.unchanged) {return}
    $script:customResponse=$answer
    $stream=[IO.MemoryStream]::new([IO.File]::ReadAllBytes($answer.frame))
    try {$temporary=[Drawing.Bitmap]::new($stream);try {$next=[Drawing.Bitmap]::new($temporary)}finally{$temporary.Dispose()}}finally{$stream.Dispose()}
    if ($script:customImage) {$script:customImage.Dispose()}
    $script:customImage=$next
    $script:customForm.BackgroundImage=$next
    $script:customForm.Invalidate()
    $script:customForm.Update()
    $script:customFrames++
    if ($script:customAnimation) {
        $script:customAnimation.Stop()
        if ($null -ne $answer.next_delay_ms -and -not $Replay) {
            $script:customAnimation.Interval=[Math]::Clamp([int]$answer.next_delay_ms,1,1000)
            $script:customAnimation.Start()
        }
    }
}
try {
    $ready=Read-CustomResponse
    if (-not $ready.ready) {throw 'Rust host not ready'}
    $script:customForm=[Windows.Forms.Form]::new()
    $script:customForm.Text='ContinuumOS · hosted '+$Candidate.ToLowerInvariant()+' candidate'
    $script:customForm.ClientSize=[Drawing.Size]::new(1280,720)
    $script:customForm.FormBorderStyle=$(if($Selected){'Sizable'}else{'FixedSingle'});$script:customForm.MaximizeBox=[bool]$Selected
    if ($Selected) {$script:customForm.MinimumSize=[Drawing.Size]::new(680,420)}
    $script:customForm.BackgroundImageLayout='None';$script:customForm.KeyPreview=$true
    $script:customForm.AutoScaleMode='None'
    $script:customForm.Add_KeyDown({param($sender,$e)
        $usage=switch ([int]$e.KeyCode) {8 {42} 9 {43} 13 {40} 27 {41} 36 {74} 35 {77} 32 {44} 37 {80} 39 {79} 46 {76} 16 {225} 17 {224} 18 {226} 38 {82} 40 {81} default {if ([int]$e.KeyCode -ge 65 -and [int]$e.KeyCode -le 90) {[int]$e.KeyCode-61}else{0}}}
        if ($usage -and -not ($script:customPressed -contains $usage)) {$script:customPressed+=@($usage);Send-Custom @{kind='key';usage=$usage;pressed=$true}}
        if ($e.KeyCode -eq 'Tab') {$e.SuppressKeyPress=$true}
    })
    $script:customPressed=@()
    $script:customForm.Add_KeyUp({param($sender,$e)
        $usage=switch ([int]$e.KeyCode) {8 {42} 9 {43} 13 {40} 27 {41} 36 {74} 35 {77} 32 {44} 37 {80} 39 {79} 46 {76} 16 {225} 17 {224} 18 {226} 38 {82} 40 {81} default {if ([int]$e.KeyCode -ge 65 -and [int]$e.KeyCode -le 90) {[int]$e.KeyCode-61}else{0}}}
        if ($script:customPressed -contains $usage) {$script:customPressed=@($script:customPressed | Where-Object {$_ -ne $usage});Send-Custom @{kind='key';usage=$usage;pressed=$false}}
    })
    $script:customForm.Add_KeyPress({param($sender,$e)
        if ($script:customPressed -contains 224 -or $script:customPressed -contains 226) {return}
        if ($e.KeyChar -eq [char]13 -and $script:customResponse.state.focus -eq 'editor') {Send-Custom @{kind='text';text=[string][char]10};return}
        if ($script:customResponse.state.focus -notin @('editor','search') -or [char]::IsControl($e.KeyChar)) {return}
        if ([char]::IsHighSurrogate($e.KeyChar)) {$script:customHighSurrogate=[string]$e.KeyChar;return}
        $text=$script:customHighSurrogate+[string]$e.KeyChar;$script:customHighSurrogate=''
        Send-Custom @{kind='text';text=$text}
    })
    $script:customForm.Add_MouseDown({param($sender,$e) $script:customForm.Capture=$true;Send-Custom @{kind='pointer';x=$e.X;y=$e.Y;button=1;phase='down'}})
    $script:customForm.Add_MouseUp({param($sender,$e) Send-Custom @{kind='pointer';x=$e.X;y=$e.Y;button=1;phase='up'};$script:customForm.Capture=$false})
    if ($Selected) {
        $script:customForm.Add_MouseMove({param($sender,$e) if ($script:customResponse.state.capture -eq 'editor') {Send-Custom @{kind='pointer-move';x=$e.X;y=$e.Y}}})
        $script:customForm.Add_ResizeEnd({Send-Custom @{resize=@($script:customForm.ClientSize.Width,$script:customForm.ClientSize.Height,100)}})
        $script:customAnimation=[Windows.Forms.Timer]::new()
        $script:customAnimation.Add_Tick({$script:customAnimation.Stop();Send-Custom @{tick=$true}})
    }
    $script:customForm.Add_MouseWheel({param($sender,$e) Send-Custom @{kind='scroll';target='files-window';y=(-$e.Delta/120*72)}})
    $script:customForm.Add_Deactivate({if ($ResourceReload -and $Replay) {return};$script:customPressed=@();$script:customHighSurrogate='';Send-Custom @{kind='reset';target=''}})
    $timer=[Windows.Forms.Timer]::new();$timer.Interval=1000
    $script:customDeadline=[DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    $timer.Add_Tick({if ([DateTime]::UtcNow -ge $script:customDeadline) {$script:customForm.Close()}})
    $script:customForm.Add_Shown({
        try {
        if ($Replay) {
            if ($ResourceReload) {
                . (Join-Path $PSScriptRoot 'Replay-ResourceReload.ps1')
            } elseif ($Selected) {
                Send-Custom @{selected=$true;resize=@(1280,720,100)}
                Send-Custom @{kind='text';target='editor';text='notes'}
                $saved=$script:customResponse.state.document_sha256
                $commands=@(
                    @{resize=@(1440,900,150)},
                    @{kind='pointer';x=175;y=125;button=1;phase='down'},
                    @{kind='pointer-move';x=245;y=125},
                    @{kind='pointer';x=245;y=125;button=1;phase='up'},
                    @{modal='settings'},
                    @{kind='key';usage=43;pressed=$true},
                    @{kind='key';usage=43;pressed=$false},
                    @{kind='key';usage=41;pressed=$true},
                    @{kind='key';usage=41;pressed=$false},
                    @{transition=@('launcher',0.5)},
                    @{tick=$true},
                    @{resize=@(1280,720,100)}
                )
                foreach ($command in $commands) {Send-Custom $command;[Windows.Forms.Application]::DoEvents()}
                if ($script:customResponse.state.document_sha256 -ne $saved -or $script:customResponse.state.pressed_keys -ne 0 -or $script:customResponse.state.pressed_buttons -ne 0) {throw 'Selected native replay lost accepted text or retained input'}
            } else {
                $steps=Get-Content (Join-Path $root 'fixtures/m2/ui-v1/workflow.json') -Raw | ConvertFrom-Json
                for ($index=0;$index -lt $steps.steps.Count;$index++) {Send-Custom @{step=$index};[Windows.Forms.Application]::DoEvents()}
            }
            $script:customForm.Close()
        }else{if($Selected){Send-Custom @{selected=$true;resize=@(1280,720,100)}}else{Send-Custom @{frame=$true}}}
        } catch { $script:customFailure=$_;$_ | Format-List * -Force | Out-String | Set-Content (Join-Path $ready.run 'native-failure.txt');$script:customForm.Close() }
    })
    $script:resourceTimer=$null
    if ($ResourceReload -and -not $Replay) {
        if (-not $ResourcePath) {$ResourcePath=$ready.resource_file}
        $script:resourceTimer=[Windows.Forms.Timer]::new();$script:resourceTimer.Interval=100
        $script:resourceTimer.Add_Tick({Send-Custom @{resource_poll=$ResourcePath}})
        $script:resourceTimer.Start()
        Write-Output "Edit resource generation: $ResourcePath"
    }
    $timer.Start()
    [Windows.Forms.Application]::Run($script:customForm)
    $timer.Stop();$timer.Dispose()
    $script:customProcess.StandardInput.WriteLine('{"quit":true}')
    $script:customProcess.StandardInput.Flush()
    if (-not $script:customProcess.WaitForExit(10000)) {throw 'Rust host did not stop after quit'}
    if ($script:customProcess.ExitCode -ne 0) {throw ($stderr.Result)}
    if ($script:customFailure) {throw $script:customFailure}
    if ($ResourceReload -and $Replay -and -not (Test-Path (Join-Path $ready.run 'native-reload.json'))) {throw 'Missing completed native reload proof'}
    @{schema_version=1;candidate=$Candidate;status='passed';selected=[bool]$Selected;replay=[bool]$Replay;pid=$ready.pid;submitted_frames=$script:customFrames;host_pixel_conversion='Rust RGBA to BGR24 BMP; WinForms owned bitmap clone';dpi=$(if($Selected){'Explicit shell resize/scale; replay 100/150 percent; native automatic DPI changes unaccepted'}else{'PerMonitorV2; fixed 1280x720 physical client, scale 1; no stretch'});native_ime=$false;native_accessibility=$false;final=$script:customResponse.state} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $ready.run 'native-preview.json')
    Write-Output "Native preview evidence: $($ready.run)"
}finally{
    if ($script:resourceTimer) {$script:resourceTimer.Stop();$script:resourceTimer.Dispose()}
    if ($script:customAnimation) {$script:customAnimation.Stop();$script:customAnimation.Dispose()}
    if ($script:customProcess -and -not $script:customProcess.HasExited) {$script:customProcess.Kill($true);$script:customProcess.WaitForExit()}
    if ($script:customForm) {$script:customForm.BackgroundImage=$null;$script:customForm.Dispose()}
    if ($script:customImage) {$script:customImage.Dispose()}
    $script:customProcess.Dispose()
}
