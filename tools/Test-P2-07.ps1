param([string]$PrerequisiteEvidence)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'Enter-Dev.ps1')
$env:CARGO_NET_OFFLINE='true'
if ($env:RUSTFLAGS -or $env:CARGO_ENCODED_RUSTFLAGS -or $env:CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS) {throw 'Clear inherited Rust flags before the exact probe run.'}
$env:CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS='--emit=dep-info,llvm-ir,link'
$env:CONTINUUM_FONT_DIR=Join-Path $root 'work/text-inputs'
$run=Join-Path $root ('build/m2/P2-07/acceptance-'+[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
New-Item -ItemType Directory -Force $run | Out-Null
$llvm=Join-Path (& rustc --print sysroot) 'lib/rustlib/x86_64-pc-windows-msvc/bin'
$rows=@();$artifacts=@()
Push-Location $root
try {
    function Invoke-Step($Name,$Program,$Arguments,$Expected='') {
        $timer=[Diagnostics.Stopwatch]::StartNew()
        & $Program @Arguments *> (Join-Path $run ($Name+'.log'))
        $code=$LASTEXITCODE
        $pass=if ($Expected) {$code -ne 0 -and (Select-String -LiteralPath (Join-Path $run ($Name+'.log')) -SimpleMatch $Expected -Quiet)} else {$code -eq 0}
        $script:rows+=@{name=$Name;program=$Program;arguments=$Arguments;exit_code=$code;expected_failure=[bool]$Expected;passed=[bool]$pass;elapsed_ms=$timer.ElapsedMilliseconds}
        $script:rows | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $run 'commands.json')
        if (-not $pass) {throw "P2-07 step failed: $Name; logs at $run"}
    }
    & (Join-Path $PSScriptRoot 'Supply-TextInputs.ps1')
    if (-not $PrerequisiteEvidence) {
        Invoke-Step 'prerequisites' 'pwsh' @('-NoProfile','-File','tools/Recheck-P2Prerequisites.ps1')
        $line=Select-String -LiteralPath (Join-Path $run 'prerequisites.log') -Pattern '^Prerequisite evidence: (.+)$' | Select-Object -Last 1
        if (-not $line) {throw 'Missing prerequisite evidence path'}
        $PrerequisiteEvidence=Join-Path $line.Matches[0].Groups[1].Value.Trim() 'prerequisites.json'
    }
    $prereq=Get-Content -LiteralPath $PrerequisiteEvidence -Raw | ConvertFrom-Json
    if ($prereq.status -ne 'passed' -or $prereq.commands.Count -ne 10 -or @($prereq.commands | Where-Object exit_code -ne 0).Count) {throw 'Prerequisites must pass all ten checks'}
    $prerequisite=@{path=[IO.Path]::GetRelativePath($root,(Resolve-Path -LiteralPath $PrerequisiteEvidence).Path).Replace('\','/');sha256=(Get-FileHash -LiteralPath $PrerequisiteEvidence).Hash.ToLowerInvariant();status='passed';checks=10}

    Invoke-Step 'format' 'cargo' @('fmt','--all','--check')
    Invoke-Step 'probe-format' 'cargo' @('fmt','--manifest-path','experiments/ui-link/Cargo.toml','--check')
    Invoke-Step 'inputs' 'python' @('tools/Review-UiLinkInputs.py',(Join-Path $run 'inputs'))
    Invoke-Step 'unit-tests' 'cargo' @('test','--locked','--offline','-p','xtask','-p','custom-ui','-p','text-core','-p','editor-model')
    foreach ($candidate in 'custom','slint-aot') {
        $feature=if ($candidate -eq 'slint-aot') {@('--features','slint-aot')}else{@()}
        $base=@('rustc','--locked','--offline','--manifest-path','experiments/ui-link/Cargo.toml','--target','x86_64-unknown-none','--release','--bin','ui-link')
        $link=@('--','-C','code-model=small','-C','relocation-model=static','-C',('link-arg=-T'+(Join-Path $root 'experiments/ui-link/link.ld')),'-C','link-arg=--no-undefined','-C',('link-arg=-Map='+ (Join-Path $run ($candidate+'.map'))))
        Invoke-Step ($candidate+'-link') 'cargo' ($base+$feature+$link)
        $directory=Join-Path $run $candidate
        New-Item -ItemType Directory -Force $directory | Out-Null
        $elf=Join-Path $directory 'ui-link.elf'
        Copy-Item -LiteralPath (Join-Path $env:CARGO_TARGET_DIR 'x86_64-unknown-none/release/ui-link') -Destination $elf
        Invoke-Step ($candidate+'-audit') 'python' @('tools/Review-UiLink.py',$elf,$directory,'--llvm',$llvm)
        $audit=Get-Content (Join-Path $directory 'audit.json') -Raw | ConvertFrom-Json
        if ($candidate -eq 'slint-aot' -and (-not $audit.symbol_excerpts.'slint_ui::paint'.Count -or -not $audit.symbol_excerpts.i_slint_renderer_software.Count)) {throw 'Slint AOT renderer not retained'}
        $artifacts+=@{candidate=$candidate;sha256=$audit.sha256;file_bytes=$audit.file_bytes;final_link='passed';audit='passed';fp_simd_instruction_count=$audit.fp_simd_instruction_count;negative_elf_checks=$audit.negative_checks.Count;executed=$false}
        $badFeatures=if ($feature.Count) {@('--features','slint-aot,missing-hook')}else{@('--features','missing-hook')}
        Invoke-Step ($candidate+'-missing-hook') 'cargo' ($base+$badFeatures+$link) 'undefined symbol: p2_missing_present_hook'
        $badFeatures=if ($feature.Count) {@('--features','slint-aot,host-feature')}else{@('--features','host-feature')}
        Invoke-Step ($candidate+'-host-feature') 'cargo' ($base+$badFeatures+$link) "can't find crate for ``std``"
    }
    # --emit on target crates saves the actual linked AOT code's LLVM IR.
    $ir=@(Get-ChildItem (Join-Path $env:CARGO_TARGET_DIR 'x86_64-unknown-none/release/deps') -Filter 'slint_ui-*.ll' | ForEach-Object {@{path=[IO.Path]::GetRelativePath($root,$_.FullName).Replace('\','/');sha256=(Get-FileHash $_.FullName).Hash.ToLowerInvariant();bytes=$_.Length}})
    if (-not $ir.Count) {throw 'Missing generated AOT LLVM IR'}
    # No guest interpreter route is proposed: exact package availability is a separate failure.
    $interpreter=Join-Path $run 'interpreter';New-Item -ItemType Directory -Force $interpreter | Out-Null
    @('[workspace]','[package]','name="p2-07-interpreter"','version="0.1.0"','edition="2024"','[dependencies]','slint-interpreter = { version="=1.18.1", default-features=false }','[[bin]]','name="probe"','path="main.rs"') | Set-Content (Join-Path $interpreter 'Cargo.toml')
    'fn main() {}' | Set-Content (Join-Path $interpreter 'main.rs')
    Invoke-Step 'interpreter-offline' 'cargo' @('metadata','--offline','--format-version','1','--filter-platform','x86_64-unknown-none','--manifest-path',(Join-Path $interpreter 'Cargo.toml')) 'no matching package named `slint-interpreter`'
    Invoke-Step 'host-build' 'cargo' @('build','--locked','--offline','-p','xtask','--features','slint-candidate','--profile','m2-comparison')
    $exe=Join-Path $env:CARGO_TARGET_DIR 'm2-comparison/xtask.exe'
    $runner=Join-Path $PSScriptRoot 'windows/Invoke-DevExecutable.ps1'
    Invoke-Step 'slint-regression' 'pwsh' @('-NoProfile','-File',$runner,$exe,'test','--suite','slint-ui')
    Invoke-Step 'custom-regression' 'pwsh' @('-NoProfile','-File',$runner,$exe,'test','--suite','custom-ui')
    $sources=@(git ls-files --cached --others --exclude-standard | Sort-Object -Unique | ForEach-Object {if (Test-Path -LiteralPath $_ -PathType Leaf) {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}}})
    $sources | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $run 'source-inputs.json')
    $tools=@('rust-lld','llvm-readobj','llvm-nm','llvm-objdump') | ForEach-Object {@{name=$_;sha256=(Get-FileHash (Join-Path $llvm ($_+'.exe'))).Hash.ToLowerInvariant();version=$(if ($_ -eq 'rust-lld') {& (Join-Path $llvm ($_+'.exe')) -flavor gnu --version} else {& (Join-Path $llvm ($_+'.exe')) --version})}}
    @{schema_version=1;status='passed';source_base=(git rev-parse HEAD);compiler=(& rustc -vV);target='x86_64-unknown-none';profile='release opt-level=1 panic=abort codegen-units=1 no-LTO';commands=$rows;artifacts=$artifacts;generated_ir=$ir;tools=@($tools);fonts_lock_sha256=(Get-FileHash fixtures/m2/text-v1/fonts.lock.json).Hash.ToLowerInvariant();probe_lock_sha256=(Get-FileHash experiments/ui-link/Cargo.lock).Hash.ToLowerInvariant();host_executable_sha256=(Get-FileHash $exe).Hash.ToLowerInvariant();prerequisites=$prerequisite;target_rustflags=$env:CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS;target_cfg=(& rustc --print cfg --target x86_64-unknown-none);arena_bytes=67108864;arena_reclaims=$false;guest_executed=$false;guest_performance_tested=$false;toolkit_selected=$false} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $run 'acceptance.json')
    Write-Output "P2-07 acceptance: $run"
} finally {Remove-Item Env:CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS -ErrorAction SilentlyContinue;Pop-Location}
