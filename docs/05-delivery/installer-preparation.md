# VM installer preparation and acceptance boundaries

Status: proposed plan, added at the owner's request on 2026-10-21. No installer,
disk initialization or installed-disk boot is implemented by this document.
The owner's approved 2026-10-21 amendment makes the graphical VM installer,
real local accounts, repair and data-preserving reinstall mandatory for M8.
M6 seeds the foundations; [M7 readiness](milestones.md#vm-disk-installer--required-for-m8)
implements B41–B44 in the [local backlog](backlog.md). Earlier live previews have
separate scoped release reviews and cannot complete M8. Detailed P3–P8 stories
are created at the owner's request before each milestone, not by this plan.

## M6 preparation

Reuse the durable [storage](../02-architecture/storage-and-data.md) and
[boot-recovery](../02-architecture/boot-recovery.md) work instead of creating an
independent installer storage/update mechanism. These are required preparation
deliverables and future tests, not results already obtained:

| Existing work | M6 preparation and acceptance |
| --- | --- |
| B22: block/flush | Expose bounded device geometry and a target identity with explicit lifetime; reject overflow, undersized/read-only/ambiguous targets and out-of-range writes; use only disposable file-backed VM disks |
| B23: layout/payload | Select and experimentally validate boot content/layout, immutable system generations, user data and operational metadata; produce a versioned offline payload manifest with exact hashes, sizes, layout requirements and compatibility; validate completeness before planning writes |
| B22/B23: installation plan | Produce a non-writing prototype that names the selected target, capacity, payload identity, byte ranges and destructive effects; validate target changes, overlapping/out-of-range regions, insufficient capacity and malformed/incomplete manifests |
| B24: initialization/recovery | Exercise empty storage, incomplete first generation, corrupt/missing blobs, disk full, write/flush failures and interruption around each initial activation step in the faulting storage model; report retry/rescue instead of claiming a valid installation |
| B40: first boot | Define attempt/health records and the no-known-good case; partial initialization cannot be selected as a complete boot generation; distinguish first installation from later fallback without rewinding user data |
| B02/B13/B16/B23/B24/B37: account feasibility | Review exact-target entropy, maintained credential-library options, durable account/policy records and trusted input/IPC dependencies; record unsupported hooks, resource costs and blocking experiments; neither deterministic VM IDs nor a hosted crypto test establishes a guest credential design |
| B33/B34 handoff | Specify installed-disk cold-boot/rescue tests and packaging variants; keep exact payload/boot identities and notices; execute these later before claiming an installable release |

The non-writing planner must not format, partition or initialize its target.
Future installer writes use the same validated plan/manifest semantics but require
fresh target checks and explicit confirmation; a successful dry run is not permission
to execute a stale plan. M6's normal storage tests may write only their explicitly
named disposable VM test images, as required by their own test scope.

## Boot and storage design to resolve at M6

Document the exact UEFI firmware → loader → kernel/initial services → selected
generation path on the installed disk. Firmware and the bootloader cannot be
assumed to read the proposed custom record store. Resolve the boot-readable
partition/filesystem, loader configuration, early storage driver availability,
generation matching and rescue path before committing to an installed layout.
GPT/EFI System Partition arrangements are candidates, not implemented choices.
M6 exits only after the chosen layout/format and discovery route have exact-target
probe evidence, including cold disk boot with fresh firmware variables. That probe
may use a separately prepared disposable VM image; it is not installer acceptance.
No new bootloader, filesystem or cryptography dependency is selected by this plan.

Keep boot files, system packages and current user data logically separate; fixed
partition counts and A/B slots are not prescribed. A mixed boot/kernel/service
generation must fail clearly. Record required capacity, alignment, sector sizes,
format versions, scan bounds and what can be inspected read-only by rescue.
Initial installation has no prior known-good system: interruption may require
retry or rescue, and must not promise rollback to a nonexistent installation.

## Service boundary and draft interfaces

These are proposed interface responsibilities, not executable commands or frozen
wire types. M6/B42 must specify versioned encodings and independently tested
schemas before freezing the installation protocol. Reuse the existing package/
generation format and durable activation/recovery mechanism for install, update,
repair and preserving reinstall; do not create a parallel mutable system copier.

The graphical installer is an unprivileged SDK client. A narrowly authorized
installation service owns disk writes, with grants restricted to the confirmed
target and operation. Neither an account role string nor development-channel
authority supplies that grant. Account creation and policy changes use B41's own
broker; UI code receives no credential database or unrestricted disk handles.

| Interface | Required behavior |
| --- | --- |
| Discover/plan | Bounded target discovery and non-writing validation of operation, exact target identity/capacity, source generation/payload, layout/version, write ranges, preservation rules and destructive effects; reject unsupported formats before confirmation |
| Confirm/apply | Bind explicit user confirmation to the plan identity, target, operation and payload; revalidate identity, geometry, source media exclusion and grant scope immediately before writes; a changed target/payload requires a new plan and confirmation |
| Progress/status | Correlate bounded phase/progress records to the operation and exact generation; distinguish staging, required flushes, activation, awaiting account setup and completed outcomes; copied bytes do not establish durable success |
| Cancel/recover | State whether cancellation was accepted and which writes already occurred; finish or abort at documented durable boundaries, reporting retry/rescue needs; no promise to undo destructive writes or cancel a committed effect |
| Outcome | Versioned success or typed validation/authority/capacity/IO/flush/compatibility/cancel/recovery failure with safe diagnostics; repeated requests follow the shared durable idempotency policy, not duplicate formatting |

Disconnect or UI failure cannot implicitly authorize another write or hide an
active transaction. The service retains authoritative operation status. M5's
quiescence rules prohibit replacing an active disk writer until a separately
validated handover exists. Ordinary UI checkpoints contain no credentials,
privileged installation handles or capability-grant assertions.

## Accounts and first installed boot

B41 is a dependency of B43 and M8. Before accepting real credentials, M7 selects
and validates the exact entropy source, credential verification/storage mechanism,
recovery design and trusted input/IPC path against M6's feasibility findings.
Record library versions/features, platform hooks and resource costs; no invented
cryptography, plaintext credential store or deterministic test secret may stand
in for that gate. No credential algorithm is chosen by this documentation edit.

The installed system first enters restricted graphical account setup. Create the
initial owner account and its authorized scoped role assignment through a durable,
idempotent broker operation before ordinary desktop access. Account metadata,
ownership, setup-complete state and policy must recover coherently after interruption;
rebooting incomplete setup cannot create duplicate privileged accounts, enable a
default password or bypass login. Do not bundle user passwords into the ISO payload.
Installation completion and first-user setup completion are distinct reported states.

Setup/login input has protected focus and restricted capture/clipboard access.
Credentials are excluded from ordinary event logs, traces, UI inspectors and app
checkpoints. Account administration, disable/removal, role changes and credential
recovery require explicit authority; rescue must not provide an undocumented login
bypass. Test login/logout/lock, cross-user access, session invalidation with declared
direct-handle limits, and preserved ownership through repair/reinstall.

## Later installer gate

After M6's prerequisite tests, B42 implements offline fresh installation, boot/system
repair, preserving reinstall and installed boot through the shared generation mechanism.
B43 supplies graphical, keyboard-complete Live/Install/Rescue choices, operation selection, confirmation,
progress, cancellation/error handling and first-user setup. Use the selected UI,
visible focus and accessible semantic controls. Never start destructive installation
automatically. The first supported target is a designated file-backed VM disk on
the declared QEMU Q35/OVMF UEFI profile. Tests use disposable targets; the host
harness attaches allowlisted image files only. A guest device name alone does not
establish safe host backing. B44 accepts the lifecycle on the exact packaged ISO.

M8 requires all of the following evidence:

- Select a target and show its identity, capacity and proposed destructive changes;
  refuse the running source/live medium and unsupported existing layouts. Revalidate
  the target, plan and payload before writes. Missing/changed/ambiguous targets fail
  closed without writes; fresh initialization erases only the confirmed VM target.
- Install the verified offline boot/system payload, initialize durable generation
  metadata, flush it and verify it. Report success only after required persistence
  completes. Network access, host shares or a host-prepared root must not be needed.
- Cold-boot the installed disk with the ISO detached and a fresh firmware-variable
  store. Prove the documented disk boot-discovery path works without a launcher's
  hidden firmware entry, and that editor/settings data survives a subsequent boot.
  Complete first-user setup and login with keyboard-only operation as well as the
  pointer path; verify account roles, file ownership and session isolation.
- Interrupt disk initialization, payload copy, metadata activation and first boot;
  test corruption, full disk, write/flush failure, retry and rescue. A half-installed
  target must not be labeled complete. Bound failed boots and report no-known-good
  situations honestly. Verify other attached sentinel disks remain unchanged.
- Implement boot/system repair and reinstall for the declared compatible installed
  format using the shared verified generation mechanism. Preserve current user files,
  settings, account/credential records and ownership; test their identities/content
  before and after. This support is mandatory for M8, not an optional fallback.
  Reject unknown/incompatible formats without modifying them or silently offering
  an erase as a repair. Boot fallback must respect current data schemas.
- Boot the independent rescue ISO, inspect the target read-only by default and test
  the declared repair path with explicit write authorization. Keep live ISO boot
  acceptance separate from installed-disk boot acceptance.
- Interrupt account creation, initial role assignment, setup completion and policy/
  credential writes; recover with coherent ownership and no unintended privileged
  access. Test authorized recovery, role escalation denial and logout/lock behavior.

Physical disks/PC support, dual boot, resizing/migrating existing partitions,
encryption, Secure Boot and unattended installation require further scoped gates.
Neither installer acceptance nor M6 preparation authorizes publication; the
[release review](../../RELEASE.md) and handling policy remain independent.
