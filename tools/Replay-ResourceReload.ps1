# Invoked inside Preview-Custom's shown event. Uses its actual native form and
# signed Rust child; no process restart, new window or mocked presenter.
$nativeClock=[Diagnostics.Stopwatch]::StartNew()
$processBefore=Get-Process -Id $ready.pid
$lifecycleBefore=@{worker_pid=$processBefore.Id;worker_start_utc=$processBefore.StartTime.ToUniversalTime().ToString('o');shell_pid=$PID;window_handle=$script:customForm.Handle.ToInt64();client=@($script:customForm.ClientSize.Width,$script:customForm.ClientSize.Height)}
Send-Custom @{selected=$true;resize=@(1280,720,100)}
Send-Custom @{kind='focus';target='reduced-motion'}
Send-Custom @{kind='key';usage=44;pressed=$true}
Send-Custom @{kind='key';usage=44;pressed=$false}
Send-Custom @{kind='focus';target='editor'}
Send-Custom @{kind='text';target='editor';text='unsaved native text'}
Send-Custom @{kind='key';usage=225;pressed=$true}
1..3 | ForEach-Object {Send-Custom @{kind='key';usage=80;pressed=$true};Send-Custom @{kind='key';usage=80;pressed=$false}}
Send-Custom @{kind='key';usage=225;pressed=$false}
$script:customForm.Location=[Drawing.Point]::new(90,70)
Send-Custom @{window=@($script:customForm.Left,$script:customForm.Top,$script:customForm.Width,$script:customForm.Height)}
$boundsBefore=@($script:customForm.Left,$script:customForm.Top,$script:customForm.Width,$script:customForm.Height)
$before=$script:customResponse | ConvertTo-Json -Depth 30 | ConvertFrom-Json
$script:customImage.Save((Join-Path $ready.run 'native-before.png'),[Drawing.Imaging.ImageFormat]::Png)
$path=$ready.resource_file
$generation=Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
'{' | Set-Content -LiteralPath $path
Send-Custom @{reload=$path}
$partial=$script:customResponse | ConvertTo-Json -Depth 30 | ConvertFrom-Json
if ($partial.resources.status.outcome -ne 'rejected' -or $partial.state.checkpoint_sha256 -ne $before.state.checkpoint_sha256 -or $partial.capture.rgba_sha256 -ne $before.capture.rgba_sha256) {throw 'Partial write changed visible generation or state'}
$generation.base_generation=$before.resources.active_generation
$generation.generation=$generation.base_generation+1
$generation.build_id='native-palette-structure'
$generation.tokens.generation=$generation.generation
$generation.layout.generation=$generation.generation
$generation.tokens.colors.panel=@(45,55,76,255)
$generation.variant.colors.panel=@(45,55,76,255)
$generation.tokens.spacing.small=8
$settingsShift=($generation.layout.nodes | Where-Object id -eq 'settings').rect[0]-340
foreach ($node in $generation.layout.nodes) {
    if ($node.id -in @('reduced-effects','reduced-motion')) {$node.rect[0]-=$settingsShift}
    if ($node.id -eq 'settings') {$node.rect[0]=340;$node.semantic.name='Reloaded settings'}
    if ($node.id -eq 'editor-window') {$node.children=@($node.children | Where-Object {$_ -ne 'status'})}
    if ($node.id -eq 'sidebar') {$node.children+=@('status');$node.text='Files'}
}
$buildClock=[Diagnostics.Stopwatch]::StartNew()
$generation | ConvertTo-Json -Depth 15 | Set-Content -LiteralPath $path
$buildNs=$buildClock.Elapsed.TotalNanoseconds
$completedBuild=$nativeClock.ElapsedTicks
Send-Custom @{resource_poll=$path}
Start-Sleep -Milliseconds 150
Send-Custom @{resource_poll=$path}
$after=$script:customResponse | ConvertTo-Json -Depth 30 | ConvertFrom-Json
$submitted=$nativeClock.ElapsedTicks
if ($after.resources.active_generation -ne 8 -or $after.resources.status.outcome -ne 'published') {throw "Native generation failed: $($after.resources.status | ConvertTo-Json -Compress)"}
if ($before.state.checkpoint_sha256 -ne $after.state.checkpoint_sha256 -or $before.capture.rgba_sha256 -eq $after.capture.rgba_sha256) {throw 'Native state continuity or visible change failed'}
$script:customImage.Save((Join-Path $ready.run 'native-after.png'),[Drawing.Imaging.ImageFormat]::Png)
$processAfter=Get-Process -Id $ready.pid
$lifecycleAfter=@{worker_pid=$processAfter.Id;worker_start_utc=$processAfter.StartTime.ToUniversalTime().ToString('o');shell_pid=$PID;window_handle=$script:customForm.Handle.ToInt64();client=@($script:customForm.ClientSize.Width,$script:customForm.ClientSize.Height)}
$boundsAfter=@($script:customForm.Left,$script:customForm.Top,$script:customForm.Width,$script:customForm.Height)
if (($lifecycleBefore | ConvertTo-Json -Compress) -ne ($lifecycleAfter | ConvertTo-Json -Compress) -or ($boundsBefore -join ',') -ne ($boundsAfter -join ',') -or $after.host_time_us -le $before.host_time_us) {throw 'Process/window lifecycle or monotonic host clock changed'}
Send-Custom @{reload=$path}
$retry=$script:customResponse | ConvertTo-Json -Depth 30 | ConvertFrom-Json
if ($retry.resources.active_generation -ne 8 -or $retry.resources.status.outcome -ne 'rejected') {throw 'Stale retry did not query actual active generation'}
Send-Custom @{kind='text';target='editor';text=' continued'}
$final=$script:customResponse | ConvertTo-Json -Depth 30 | ConvertFrom-Json
if ($final.state.revision -ne $before.state.revision+1 -or $final.state.pressed_keys -ne 0 -or $final.state.pressed_buttons -ne 0 -or $final.state.capture) {throw 'Continued typing failed or native input stuck'}
@{status='passed';session=$ready.session;lifecycle_before=$lifecycleBefore;lifecycle_after=$lifecycleAfter;bounds_before=$boundsBefore;bounds_after=$boundsAfter;before=$before;partial=$partial;after=$after;retry=$retry;final=$final;resource_file=$path;build_ns=$buildNs;completed_build_tick=$completedBuild;submitted_tick=$submitted;native_clock_frequency=[Diagnostics.Stopwatch]::Frequency;native_completed_build_to_submitted_ms=1000.0*($submitted-$completedBuild)/[Diagnostics.Stopwatch]::Frequency;status_title=$script:customForm.Text;scope='ephemeral hosted L0/L1; native bitmap submitted and Form.Update returned; no display scanout timestamp'} | ConvertTo-Json -Depth 25 | Set-Content (Join-Path $ready.run 'native-reload.json')
