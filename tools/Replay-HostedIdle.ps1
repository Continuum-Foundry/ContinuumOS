# Invoked within Preview-Custom's existing native window and worker lifetime.
function Get-IdleProcess($process) {
    $process.Refresh()
    return @{pid=$process.Id;cpu_ms=$process.TotalProcessorTime.TotalMilliseconds;working_set=$process.WorkingSet64;private_commit=$process.PrivateMemorySize64;peak_working_set=$process.PeakWorkingSet64;handles=$process.HandleCount}
}
Send-Custom @{selected=$true;resize=@(1280,720,100)}
Send-Custom @{kind='focus';target='search'}
$shellProcess=[Diagnostics.Process]::GetCurrentProcess()
$observations=@()
foreach ($phase in @(@{name='static';ms=10000},@{name='caret';ms=10000},@{name='hidden';ms=5000},@{name='minimized';ms=5000})) {
    switch ($phase.name) {
        'caret' {Send-Custom @{kind='focus';target='editor'}}
        'hidden' {$script:customForm.Hide()}
        'minimized' {$script:customForm.WindowState='Minimized';$script:customForm.Show()}
    }
    [Windows.Forms.Application]::DoEvents()
    $frames=$script:customFrames
    $before=@{worker=Get-IdleProcess $script:customProcess;presenter=Get-IdleProcess $shellProcess}
    $clock=[Diagnostics.Stopwatch]::StartNew()
    $wakes=0
    while ($clock.ElapsedMilliseconds -lt $phase.ms) {
        Start-Sleep -Milliseconds 100
        $wakes++
        [Windows.Forms.Application]::DoEvents()
    }
    $row=@{phase=$phase.name;elapsed_ms=$clock.Elapsed.TotalMilliseconds;submitted_frames=$script:customFrames-$frames;observer_wakes=$wakes;before=$before;after=@{worker=Get-IdleProcess $script:customProcess;presenter=Get-IdleProcess $shellProcess};window_handle=$script:customForm.Handle.ToInt64();os_scheduler_wakes=$null}
    $observations+=@($row)
    if ($phase.name -ne 'caret' -and $row.submitted_frames -ne 0) {throw "Native $($phase.name) perpetually repainted"}
    if ($phase.name -eq 'caret' -and $row.submitted_frames -eq 0) {throw 'Native caret timer did not paint'}
}
@{status='passed';geometry=@(1280,720,100);observations=$observations;clock_timer='none in native adapter; separately measured in Rust RAM scheduler';caret_period_ms=500;accessibility_timer='none';resource_polling='disabled for this idle baseline; resource mode has an additional 100ms file poll';deadline_timer_ms=1000;bitmap_ownership='one disposed/replaced bitmap per submitted frame; GDI/WinForms storage charged to presenter process';scanout_measured=$false;guest=$false} | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $ready.run 'native-idle.json')
