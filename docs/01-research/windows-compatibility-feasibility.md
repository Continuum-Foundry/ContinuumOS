# Windows EXE compatibility feasibility

Research review: 2026-10-21. Status: source-backed findings plus proposed M16/B59
experiments; no PE guest loader, Wine port or Windows application execution has
been demonstrated. The baseline still uses original-kernel, static native ELF
processes. See [runtime architecture](../02-architecture/applications-and-runtime.md),
[M16](../05-delivery/milestones.md#m16--windows-exe-compatibility-research-and-prototypes)
and the [feature matrix](../05-delivery/desktop-feature-matrix.md).

## What the sources establish

PE describes image sections and metadata such as imports, relocation information,
TLS and machine/subsystem declarations. Parsing/mapping an image is only one
requirement. Windows x64 function calls and unwinding have a specified ABI; a
ContinuumOS syscall boundary does not automatically implement it. Modern imports
can name API-set contracts resolved by OS loader policy rather than literal DLL
files. [S85](sources.md#s85), [S86](sources.md#s86), [S87](sources.md#s87)

Wine supplies a loader and Windows API implementation on supported Unix-like hosts.
Its README identifies host/build dependencies. Source inspection separately shows
Unix virtual-memory/filesystem/library hooks, pthread/signal/architecture handling,
and server-side process/object coordination. This is evidence of adaptation work,
not proof that a POSIX shim alone would be sufficient or that Wine cannot be ported.
[S88](sources.md#s88), [S89](sources.md#s89), [S90](sources.md#s90), [S91](sources.md#s91)

Architecture/runtime support is another boundary: Windows documents WOW64 as an
additional subsystem; managed deployments can require their runtime and associated
platform dependencies. They are not conferred by support for one x64 native image.
[S93](sources.md#s93), [S94](sources.md#s94)

ReactOS documents a Windows-oriented architecture and Wine cooperation. It is a
comparison/reference, not an approved kernel replacement. This project retains its
original kernel. [S97](sources.md#s97)

## Candidate approaches and required inventory

| Approach | Required investigation | Boundary/cost |
| --- | --- | --- |
| Scoped native compatibility service | First-party Rust PE loader/runtime adapters and only an explicit Windows API subset mapped to native brokers | Small initial corpus can be precise, but the project owns API semantics and compatibility maintenance; no generic Win32 claim |
| Adapt Wine | Pin candidate source, inventory Unix/libc/thread/signal/server/graphics dependencies, compare replacing the platform backend versus supplying a bounded host interface | Reuses upstream API work but imports major port/build/runtime/maintenance and provenance obligations; not a drop-in crate |
| Different kernel or guest Windows VM | Compare only to make scope differences explicit | Replacing the kernel violates the project requirement; running another OS is not native ContinuumOS EXE compatibility |

Inventory memory reservation/commit/protection, file/path and sharing semantics,
handles/waits, threads/TLS, exception dispatch/unwind, process environment, DLL/API
resolution and required C/runtime behavior. A GUI tier additionally needs windows,
messages, drawing, input/focus, clipboard and font integration. Network/audio/GPU,
COM/services/registry, installers and runtime dependencies must be scoped per app;
they cannot be treated as automatically available because an image loads.

Keep first-party kernel/bridge code Rust with explicit architecture assembly.
Third-party C/C++ from a candidate is a separate integration decision: exact sources,
licenses/notices, toolchain hooks and distribution review precede adoption. Wine's
license text is a review input, not this project's distribution clearance.
[S92](sources.md#s92)

## Proposed M16 evidence ladder

B59 follows B02/B11/B12/B13/B16/B23/B41. Source research may precede those gates;
guest execution evidence cannot. Compile fixtures on the host; no guest compiler
or downloaded proprietary Windows system DLL is assumed.

1. **Inspection:** bounded PE32+ x64 validation, sections/imports/relocations and
   permissions. Reject overflow, overlap, unsupported machine/subsystem and malformed
   input. Inspection does not demonstrate execution.
2. **Minimal guest entry:** an authored Rust x64 Windows-target console fixture
   with a declared minimal import surface enters, produces deterministic output
   and exits under guest supervision. Record actual image/loader/bridge identities.
3. **Runtime semantics:** explicit DLL/import cases, memory protection, thread/TLS,
   waits and exception/unwind probes. Unknown APIs fail precisely; a stub returning
   success cannot stand in for behavior. Check ABI registers/stack/FP preservation.
4. **Small GUI:** authored bounded window/message/input/software drawing fixture,
   routed through native surface/input brokers. Record missing behavior, cleanup,
   crash containment and memory/CPU costs against the same native workload.
5. **Decision:** compare native-service/Wine results, maintenance surface and
   resource/security cost. Propose a narrow supported app/API set or retain failed
   probes and issue an explicit stop/replan. No unnamed existing app is promised.

Required negatives include forged compatibility handles, denied file/device/network
access, excessive allocations/threads, malformed DLLs, unsupported imports, process
death and leaked resources. Map compatibility IDs to process-local native grants;
Windows-looking names cannot create ambient authority. Exception and executable-
memory/JIT needs must be reviewed against native isolation/W^X rather than weakening
it for a demo. Apply cross-user scope checks and resource quotas to the bridge too.

## Claims deliberately outside the initial gate

Start with x64 native user applications. Windows kernel drivers, 32-bit/WOW64,
managed runtimes, DRM/anti-cheat, broad installer/application support and arbitrary
Windows-version behavior require separate evidence and decisions. A filename or
host execution result does not establish guest support. Ordinary Windows apps have
no assumed checkpoint/typed-action protocol, so state-preserving foreign-app reload
is a separate capability, not inherited from native M5 replacement.

A browser uses the [separate M17 study](browser-feasibility.md). A source review,
tiny EXE, successful DLL binding or GUI fixture cannot establish safe browser use.
No runtime integration, source pin, license selection or publication is authorized
by this research document.
