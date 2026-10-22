# Implementation milestones and gates

Status: gated execution plan. Phase 0 compile/tooling implementation and evidence
are recorded in [the runbook](phase-0.md). P1-02 demonstrates minimal UEFI ISO
kernel entry; M1's declared experimental baseline now passes local acceptance.
Later milestones remain uncompleted.
Stages are evidence gates, not promised calendar dates. First build feasibility
determines credible estimates. Do not attempt the whole OS in one coding session.
Revision 2 prioritizes M2/M4/M5 state-preserving reload and an AI-free usable desktop.
The [reconciled plan](../00-overview/reconciled-plan.md) records user-approved direction.
Its 2026-10-21 installer amendment makes M7 account/installation readiness a
required dependency of M8. Existing M0–M2 acceptance and P2 story boundaries remain
unchanged. The approved desktop/platform expansion adds M9–M21 without extending
M8's release exit; see the [feature matrix](desktop-feature-matrix.md).
Detailed P3–P21 stories are decomposed at the owner's request before
each milestone; the broad backlog does not substitute for that decomposition.

## M0 — Feasibility and locked toolchain

Inputs: this library and a clean workspace. Record actual Windows/Linux build options,
Rust/linker/ISO/QEMU/firmware availability and host accelerator. Pin tested versions
and checksums; inspect licenses and guest dependency features. Select explicit
kernel and userland targets, panic/allocation policies and build directory layout.

Experiments: separate freestanding kernel/userland Rust links; candidate
UI/raster/font no-OS feature builds; QEMU/firmware/ISO prerequisite inventory.
The user selected Windows native first and compile/tooling-only Phase 0: actual
ISO boot and VM launch are M1. Local dev transport execution remains M4.
Add Slint/custom target feasibility, x86_64 crate review, and a Limine/Rust OSDev
bootloader comparison tied to an actual ISO recipe.
Do not install OS features, alter virtualization settings or download large toolchains
without reporting the concrete need and respecting environment approval rules.

Exit: two clean probe builds, artifact inspection, source path with spaces and
doctor override checks; locked toolchain, dependency outcomes, and an explicit
packaging/VM prerequisite report with blockers. ISO recipes remain not boot-tested.
Artifacts: environment/build/toolchain manifests and retained logs. Supports R03,
early R16 and R18; no release reproducibility or ABI/isolation claim.

## M1 — Original kernel boots from ISO

Execution is decomposed into [one epic and seven ordered stories](phase-1/README.md).
P1-01 supplies pinned host inputs; P1-02 adds the minimal original-kernel ISO path.
See its [runbook](phase-1/minimal-iso-runbook.md) for evidence boundaries.
P1-03 adds [checked owned metadata and correlated boot records](phase-1/boot-metadata-runbook.md)
with retained acceptance evidence. P1-04 adds
[fatal exception diagnostics](phase-1/exception-diagnostics-runbook.md).
P1-05 implements [owned frames/mappings and guards](phase-1/memory-ownership-runbook.md);
check its acceptance review. P1-06 [interrupt/timekeeping acceptance](../../evidence/phase-1/P1-06/2026-10-20/README.md) passes on the pinned single-core profile.
P1-07's [complete local review](../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
passes 33 rows and 42 launches, including both clean artifacts and six final normal
boots. This closes the declared experimental M1 gate, not later product/runtime gates.

Implement boot validation, serial output, framebuffer metadata, owned paging,
exception handling, physical allocator and interrupt/timer baseline. Build UEFI ISO.
Initially avoid heap/SIMD until supported; emit boot/readiness checkpoints.

Exit: cold boot from the exact ISO with no host share/network, stable boot ID/uptime,
usable diagnostics on intentional exception and repeatable shutdown/test exit.
Archive config, hash and serial transcript. Supports R01–R03.
Require the packaged ISO's firmware/optical boot path; a working disk image or
direct-kernel emulator launch cannot substitute for it.

## M2 — Hosted UI core and resource reload

Execution from the accepted M1 state is decomposed into
[one epic and thirteen ordered local stories](phase-2/README.md). P2-01's
[local acceptance](../../evidence/phase-2/P2-01/2026-10-20/README.md) passes prerequisite,
profile and headless lifecycle checks. P2-02's
[accepted shared corpus](../../evidence/phase-2/P2-02/2026-10-21/README.md) adds
boundary/token/layout drafts and deterministic negative/semantic fixture checks.
P2-03's [accepted hosted document/action model](../../evidence/phase-2/P2-03/2026-10-21/README.md)
adds shared execution, bounded history/checkpoints and modeled rights/context tests.
P2-04's [accepted bounded text experiment](../../evidence/phase-2/P2-04/2026-10-21/README.md)
adds pinned fonts, shaping/bidi/fallback/caret/composition tests and text-only rasters.
P2-05's [accepted custom candidate](../../evidence/phase-2/P2-05/2026-10-21/README.md)
adds rendered scenes, shared interactions/semantics, bounded lists/caches and a
native Windows presentation adapter, with explicit quality/cost limits.
P2-06 [adds bounded Slint evidence](../../evidence/phase-2/P2-06/2026-10-21/README.md):
AOT/software primitives and shared adapters, with native feature failures preserved.
P2-07–P2-13 remain Not started; no toolkit is selected, guest UI/reload demonstrated
or Jira issue created. P2-02 through P2-09 establish shared fixtures, candidates,
target feasibility and the provisional decision. P2-10 through P2-13 complete
selected behavior, hosted reload, measurements and the acceptance review.

After M0, compare Slint and a minimal custom UI with identical editor/file-list
scenes, tokens, text, semantics and input. Audit guest compile/platform hooks,
reload and distribution terms. Select provisionally before expanding a full toolkit;
confirm in M4. Define typed edit/settings actions shared by UI and automation.
Prove hosted token/layout reload preserving document/focus state.

Exit: deterministic renderer fixtures, keyboard workflow, text/semantic test subset,
idle/damage instrumentation and measured hosted baseline. No guest performance claim.
Supports R04, R08, R09, R15 and R19 interface design. Hosted preview does not prove
guest interpreter support or compiled behavior replacement.

## M3 — Isolated processes and stable interfaces

Add static user ELF loader, user stacks, syscall entry/return, handles, channels,
waits, preemptive task scheduling, quotas and supervisor. Implement proxy routes
and epochs. Establish FP/SIMD context policy before running UI math in user tasks.

Establish the [identity, ownership and authorization foundations](../02-architecture/identity-and-authorization.md)
in B11–B13: distinguish user/service principals, sessions, process instances and
resource owners. Trusted launch/IPC metadata supplies the caller context; a payload
cannot select another owner or role. Define default-deny policy that maps scoped
role assignments to explicit capability grants without making ownership or a role
name ambient authority. Exercise two synthetic users and a service principal while
retaining one local development user in the actual product. Principal records and
role assignments are ephemeral fixtures here; no login or credential store is required.
Reserve narrowly scoped disk-installation and account-administration authority:
ordinary apps, development-session claims and caller-supplied IDs cannot acquire
it. Test unauthorized disk/account requests without requiring an installer or
credential implementation at M3.
Reserve scoped file-operation, process/service-control and diagnostic interfaces
for later clients. Distinguish monotonic deadlines from wall-clock values in SDK
types; these interface seeds do not implement M9/M11 administration or time settings.

Exit: process isolation/forged-handle/malformed-IPC/deadlock/resource tests, independent
SDK client/service build, user fault containment and no stale-writer bypass.
Also require forged caller/role rejection, cross-owner denial, authorized sharing,
same-user app least authority, child-process authority ceilings and brokered
session-expiry/policy-revocation tests. Define reauthorization at service replacement
for M5 to exercise; do not claim universal revocation of direct handles.
Supports R10, R14 and prerequisites for R05/R06/R19. Full accounts and operational
RBAC belong to the separate M7 accounts gate after M6, not the M3 exit.

## M4 — Guest desktop and live assets

Bring the same UI core to guest framebuffer/surfaces; add input driver, minimal
compositor/shell and controlled host→guest dev transport. Keep serial recovery.
Transmit validated resource/layout updates and publish at scene boundaries.
Bind development authority to its authenticated development session; it cannot
mint a desktop user identity or imply account-administrator privileges. B15 must
use M3's trusted caller context and reject forged session/role claims.
Establish trusted setup/login input prerequisites: protected focus, restricted
capture/clipboard access and exclusion of credential events from ordinary logs,
traces and UI inspectors. Exercise routing/capture denial with synthetic sensitive
input; real authentication and its review remain M7 work. The selected UI's later
installer/setup consumers must support keyboard operation and visible focus.
Reserve grant-bearing clipboard/drag offers, picker results and semantic notification
actions for M9's integrated workflows; a string path or offered object ID conveys
no authority. Do not add those later desktop features to the M4 exit.

Exit: live resource demo inside running guest, kernel boot ID/uptime unchanged,
correct focus/gesture handling, rejected corrupt update, disconnect/retry tests and
guest rendering metrics. Persistence can remain explicitly ephemeral.
Confirm the selected toolkit's actual guest backend and surface/input/semantics.
Raster buffer submission is sufficient; custom scene export is not mandatory.
Supports R04, R08, R09, R13, R17.

## M5 — Stateful app behavior replacement

Implement coordinator, quiescence, restricted candidate startup, identity-schema
checkpoint, route/lease handover, health test and safe retirement. Editor is the
first meaningful case. Add deliberate incompatible schema rejection and migration fixture.
Exclude credentials and privileged installation handles from ordinary app
checkpoints. Define cancellation and replacement boundaries for future installer
operations; do not replace a service while it owns an active disk write/flush
transaction until a separate handover test establishes a supported path. M5 tests
the restriction with modeled operations, not an implemented installer.
Give later file/update/backup operations explicit request identity, bounded progress
and cancellation outcomes. Document committed effects rather than promising generic
undo; these are interface foundations, not completion of M9–M11 workflows.

Exit: edit unsaved text → change behavior → new instance takes over with text,
selection and window intact; at least 100 cycles without unexplained growth. Inject
failure in every in-memory phase. Demonstrate old code is not freed while reachable.
Show UI/layout and compiled behavior changes, unchanged boot ID/uptime, no VM reset,
and equivalent typed UI/automation actions through replacement. This first meaningful
product demo precedes optional inference.
Supports R05, R07, R12, R14, R17, R19. Not yet a power-loss durability claim.

## M6 — Durable state and selected service updates

Add VirtIO block, real flush semantics, tested record store or revised filesystem
choice, immutable packages/generations and activation journal. Implement durable
settings/file operations, idempotency records and recovery selection. Replace a
simple settings/service process under load before attempting complex storage drivers.
Separately implement boot-attempt/health/known-good selection (B40), with bounded
failed boots and schema-aware fallback. A/B slots remain a later packaging option.

Seed the [VM installer effort](installer-preparation.md) within B22–B24/B40:
bounded disk geometry/target validation, separate boot/system/user-data lifecycles,
an explicit firmware-to-installed-generation boot design, immutable offline payload
manifests and interrupted-initialization recovery cases. Produce a non-writing
installation-plan prototype for disposable file-backed VM disks; it identifies
the exact target, proposed writes and destructive effects without formatting it.
These are M6 prerequisites, not a bootable installer or permission to access physical disks.

Exit: fault matrix proves durable outcomes; full disk/flush errors are honest; accepted
mutations are preserved; old/new writer overlap never occurs. Rescue ISO can recover
the last good coherent generation. Supports R06, R11, R12, R14.
The installer-preparation exit requires selection and experimental validation of
the boot/storage layout and offline payload contract, rejected invalid/ambiguous
targets and passing initial-generation fault cases in the storage model. Validate
the boot-readable format, early driver dependencies and firmware discovery route
on disposable VM images; this narrow layout probe does not establish installation.
Review exact-target entropy and credential-storage feasibility before M7 account
implementation; absent evidence blocks readiness instead of allowing mock secrets.
Define the later install/reinstall/recovery matrix; full installed-disk lifecycle
acceptance remains M7/M8 work.
Seed portable settings schemas, consistent backup/read views and per-owner resource/
disk instrumentation for later desktop clients. M11 backup is independent storage,
not another name for this recovery selector or preserving reinstall.

## M7 — Installation and accounts

The account/session/RBAC gate (B41) and installer lifecycle gates (B42–B44) below
are mandatory prerequisites of M8. Their graphical flows reuse the UI selected
earlier; they do not require a second toolkit. Networking, audio/assistive integration
and physical qualification now have explicit M12/M13/M15 gates; optional runtime,
compatibility and acceleration experiments live in M16–M21. They do not become M8
prerequisites through this expansion. Exit M7 only when B41–B44 meet their gates.

### VM-disk installer — required for M8

Build on M6's preparation to add an explicit Install action to the live ISO for
the supported file-backed VM disk profile. Keep live boot and rescue available.
Require target selection/confirmation, revalidation before writes, bounded disk
initialization, verified offline system/boot payloads and honest failure/rescue
status. B42 supplies the privileged backend; B43 supplies the unprivileged graphical,
keyboard-accessible installer and first-run setup; B44 accepts fresh install,
boot/system repair and data-preserving reinstall. Use one package/generation
mechanism for installation, updates and repair. Pass the
[installer acceptance matrix](installer-preparation.md#later-installer-gate):
cold-boot the installed OS with ISO detached and fresh firmware variables, test
interruption/retry and data-preserving reinstall/update behavior, and verify rescue.
No physical-PC, dual-boot, partition-resize or encryption support is implied.
This gate is required for M8. Earlier live-only previews have separate scoped
release reviews and cannot satisfy M8. Publication remains independently gated.

### Accounts, sessions and RBAC — required for M8

This explicit later gate builds on M3's identity/authority tests, M4's input/session
boundaries and M6's durable storage/recovery. Implement local account lifecycle,
login/logout/lock, credential verification/recovery, persistent resource ownership,
scoped role administration and audited policy changes. Enforce role-derived grants
through the same capability brokers used by apps and typed actions. Require secure
input, a reviewed credential/entropy design, session revocation with declared direct-
handle limits, cross-user isolation and crash-consistent policy/recovery tests.
See the [accounts gate and acceptance cases](../02-architecture/identity-and-authorization.md#later-accounts-gate).
Implement B41 before accepting B43's first-run setup. Create the initial local
owner account on first installed boot, before ordinary desktop access; interrupted
setup must be recoverable without an unintended privileged account or bypass.
Select and validate the credential, entropy, recovery and trusted-input design
before accepting real credentials. Test role administration, cross-user isolation,
login/logout/lock, session revocation limits and durable account recovery.
This gate is required for M8. Encrypted homes retain a separate key-management/
recovery gate; login alone does not provide encryption.

## M8 — Coherent experimental release

Build standalone ISO and optional data image from a coherent locked generation.
Run requirement matrix, two clean-build comparison, offline boot, recovery, release
dev-authority checks, visual/accessibility review, license/SBOM and provenance audit.
The exact release ISO must provide graphical, keyboard-accessible Live/Install/
Rescue entry points and complete offline VM installation. Require B41–B44: boot
the installed disk with ISO detached and fresh firmware variables, complete local
account setup/login, preserve files and ownership across reboot, repair boot/system
content, and reinstall while preserving compatible user data. Include interruption,
invalid-target, unsupported-format and rescue outcomes. Both optical ISO boot and
installed-disk boot require their own evidence; neither substitutes for the other.

Exit: source/build instructions, exact tested binaries, checksums/symbols, VM profile,
known limitations and reproducible evidence bundle. Call it an experimental VM OS,
with declared support; do not imply daily-driver laptop readiness. Supports R02,
R08–R19 within the agreed matrix. An AI-free release can pass all core gates.
Live-only previews may be reviewed earlier but do not complete this milestone.

## M9 — Everyday desktop and files

B45 completes folder creation, copy/move/rename/delete/trash, name-conflict choices,
progress/cancellation and grant-bearing open/save pickers. B46 integrates clipboard,
drag/drop, file associations/default apps and keyboard-accessible notifications.
B47 persists keymap, locale, date/time and time-zone settings; monotonic deadlines
must survive wall-clock changes. Reuse the M8 installed system and SDK boundaries.

Exit: perform these workflows across real native apps by keyboard and pointer;
exercise denied access, conflicts, interrupted/cancelled operations and rebooted
settings. Never imply atomic cross-volume moves or erase a conflict without a
declared user choice. Required for M14, not added to M8.

## M10 — Applications and software management

B48 delivers the native SDK/package lifecycle: install, launch, update, uninstall,
requested-permission review and explicit app-data retention choices. B49 supplies
verified offline app/system updates, publisher/trust lifecycle, recoverable
activation and honest reboot-required handling for kernel/boot changes. Native
applications remain the baseline; an app store or Windows binary support is not assumed.

Exit: independently built SDK apps install and run with scoped grants; permission
increases require policy review; uninstall retention is explicit. Reject invalid,
unauthorized/incompatible updates, recover interrupted activation and keep ordinary
release updates separate from the development hot-reload endpoint. Required for M14.

## M11 — Backup and system administration

B50 backs up documents and portable settings to an explicitly selected separate
destination and restores into an authenticated fresh installation with explicit
ownership mapping. Exclude credentials, secrets and privileged policy by default.
B51 provides scoped app/service stop/restart, CPU/memory/disk reporting and bounded
diagnostic export. B52 completes orderly shutdown and compatible session recovery.

Exit: restore verified data/settings after losing the original guest disk, reject
corrupt/unauthorized restores and preserve current destination data on refusal.
Test hung apps, unsafe service-control denial, failed flush/forced-shutdown outcomes,
revoked grants and crash-loop suppression. A second VM image on the same host is
not proof against loss of that host; state the tested failure domain. Required for M14.

## M12 — Networking and online delivery

B28/B53 complete virtual Ethernet, addressing, DHCP/DNS, TCP and TLS on the declared
VM profile, including time/trust-store management and bounded network resources.
B54 adds connection settings and an online-update client that uses B49's verifier
and activation mechanism; transport success never grants package authority.

Exit: controlled network tests cover configuration changes, packet loss/malformed
traffic, certificate/time/entropy failures, disconnects and interrupted downloads.
Verification bypass is not acceptance. Demonstrate offline usability and no implicit
development listener in release images. IPv6/broader protocols remain separately
declared coverage; publishing an update service is independently authorized. Required for M14.

## M13 — Audio and assistive integration

B55 implements bounded VM playback/capture, mixing, device selection, permissions
and visible capture indicators. B31 integrates richer text and a native assistive
consumer with validated speech output. Select actual audio/synthesis dependencies
through exact-target probes, not assumed runtime ports.

Exit: test format/rate negotiation, underrun/overrun, device/service failure,
capture denial/revocation and resource cleanup. Keyboard/semantic workflows produce
correct announced focus/value/actions through the validated output path. Record
supported devices/languages and remaining accessibility limits. Required for M14.

## M14 — Coherent usable VM desktop release

B56 accepts M8 plus B45–B55/B31 on the exact packaged ISO and installed disk:
installation/accounts/repair/reinstall, everyday files/desktop, app/software
management, backup/restore, administration/shutdown, networking and audio/assistive
workflows. Re-run negative/fault, keyboard/accessibility, provenance and resource
gates. Preserve the 256 MiB baseline cold-idle target with complete service/cache
accounting; report active audio/network and optional-feature costs separately.

Exit: clean-build/package evidence, independent optical/installed boots, complete
feature-matrix acceptance and reviewed limitations/publication prerequisites.
Browser, EXE compatibility, GPU acceleration, SMP, Wasm and AI do not block this
release. A future browser-delivery commitment requires a separate decision.

## M15 — One physical-PC profile

B57 follows accepted B56 VM delivery and selects one named UEFI PC, firmware/device
inventory and explicit support matrix before driver implementation. Qualify boot
media, storage, input, software display, networking, audio and orderly poweroff.
No arbitrary-PC, laptop, Wi-Fi or suspend support is inferred from that profile.
B58 tests installation, ISO-detached boot, accounts, preserving repair/reinstall,
independent backup/restore and rescue on the named hardware.

Exit: actual profile-specific evidence, required driver/boot/flush dependencies,
performance/trust limits and recovery instructions. Physical-disk execution needs
its own explicit target/write scope; this roadmap edit authorizes none. Unknown
hardware is unsupported rather than silently substituted with VM evidence.

## M16 — Windows EXE compatibility research and prototypes

B59 compares a scoped first-party Rust Windows compatibility service with adapting
Wine while keeping the original kernel. Use the [dedicated study](../01-research/windows-compatibility-feasibility.md)
to inventory PE/ABI/DLL/API/runtime and host-platform dependencies. After B02/B11/
B12/B13/B16/B23/B41 prerequisites, exercise bounded x64 console, DLL, memory/thread/
exception and small-GUI probes, including denied access and malformed inputs.

Exit: reviewed dependency/security/maintenance costs and a supported-subset proposal
or evidence-backed stop/replan outcome. A failed experiment is not shipped support.
Exclude Windows kernel drivers, WOW64/32-bit, managed runtimes, DRM and general
application promises initially; no inherited foreign-app state-preserving reload.

## M17 — Browser feasibility and selection gate

B60 consumes B59's reviewed outcome and B53/B16/B09/B41 foundations. Compare a
native engine/browser port with a Windows-browser compatibility route in the
[browser study](../01-research/browser-feasibility.md). Audit rendering, platform
APIs, networking, isolation, updates, memory and licensing/dependency obligations.

Exit: bounded probe findings, missing hooks, credible sandbox/maintenance path and
an explicit subsequent product decision. Launching an executable, an HTML screenshot
or disabling the sandbox cannot establish a production browser. B59 may conclude
the EXE route is infeasible without preventing consideration of a native route.

## M18 — GPU acceleration experiment

Expand B29 after B16 and profiling evidence: integrate the exact guest/host backend,
command/resource ownership, fences/reset and measured workload benefit. Keep a
tested software fallback. Exit with supported features, fault/cleanup evidence and
actual memory/frame distributions; no generic GPU portability or benefit claim.

## M19 — SMP experiment

Expand B30 after B06/B12/B24: establish multicore context/scheduling, TLB shootdown,
ordering, ownership and concurrent update/recovery behavior. Exit with optimized
stress/fault tests and measured workload comparisons. Passing single-core tests or
adding vCPUs to a VM command line is insufficient.

## M20 — Bounded Wasm runtime

Expand B27 after B12/B18/B20: exact-target interpreter/platform integration,
restricted imports, fuel/memory/stack quotas, cancellation and state export/import.
Exit with adversarial tests and native-runtime cost comparison. Neither WASI,
general POSIX nor a guest compiler is assumed.

## M21 — Optional local AI service

Expand B39 after B06/B21/B37, hence after the first M5 product reload demo. Use an
isolated service, existing typed actions, scoped context, quotas, unload/crash tests
and separate cold/warm/active model-memory accounting. Exit with named model/backend
evidence, denied escalation and an unaffected desktop/reload path on worker failure.
It is never a core-release dependency and does not require M20 Wasm.

M16–M21 are separately gated tracks, not a universal serial chain: their declared
prerequisites permit work without completing M15 first. None blocks M8 or M14.
M15 alone requires the accepted VM desktop before physical qualification.

## Stop/replan criteria

A gate failure creates a bounded issue and evidence, not silent scope substitution.
If the host cannot build/run the selected path, complete portable work and report
the exact missing capability. If resource budgets fail, profile before dropping
correctness. If a user requires zero upstream boot code or full kernel live reload,
revise scope/ADRs explicitly before promising an implementation timeline.
