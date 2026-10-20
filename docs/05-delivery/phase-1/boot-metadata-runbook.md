# Checked boot metadata and identity (P1-03)

Implementation acceptance passed on 2026-10-18; see the
[retained review](../../../evidence/phase-1/P1-03/2026-10-18/README.md).
This slice extends the original Rust kernel and standalone UEFI optical route;
it does not complete M1. The current [P1-04 exception runbook](exception-diagnostics-runbook.md)
extends diagnostics to version 3, adds an emergency flag and requires an exception
setup checkpoint before completion. Version-2 descriptions below document P1-03's
historical contract; its retained evidence is unchanged.
P1-05's [memory integration](memory-ownership-runbook.md) uses version 4 and owned
page tables/guarded stacks. The reservation and loader-lifetime rules below are
preserved; check that story's acceptance review before proceeding.

## Run

Use PowerShell 7 and the existing workspace-local, pinned tools:

```powershell
. ./tools/Enter-Dev.ps1
cargo test --locked -p continuum-kernel --lib
cargo test --locked -p xtask
cargo xtask doctor --profile m1 --json
cargo xtask test --suite m1 --case boot-metadata
cargo xtask test --suite m1 --case boot-reject
pwsh -NoProfile -File tools/Test-P1-03.ps1
```

The acceptance script also reruns Phase 0, dependency probes, P1-01 prerequisite
rejections, normal observation and both existing ISO cases. Allocation-dependent
dependency link probes remain expected blockers, not guest implementation. Run
xorriso in the narrowly scoped execution context documented in the
[minimal ISO runbook](minimal-iso-runbook.md); do not change system settings or
replace pinned tools to work around its MSYS2 named-object restriction.

## Owned boundary and bootstrap trust

`BootInfoV1` is an internal Rust value, never an IPC or persisted layout. It uses
static kernel storage initialized in place: 512 regions, 16 modules, four
framebuffers, 256 bytes per descriptor string and 1,024 bytes of arguments,
including terminators. Exhaustion, missing terminators and arithmetic errors
reject boot. Strings are bounded printable ASCII; none are silently truncated.

The adapter requires supported base revision 3, a memory map, HHDM and executable
address/file responses. It explicitly requests four-level paging. Module,
framebuffer, ACPI and loader-name responses may be absent. Supplied descriptors
must validate; missing graphics is an explicit serial-only mode.

Pinned Limine/firmware remain trusted to provide readable initial HHDM/map headers,
pointer arrays and entries. Canonicality, alignment and bounded arithmetic cannot
prove arbitrary pointers are mapped. After validating the map, the adapter checks
bootstrap spans retrospectively and checks subsequent metadata spans against
loader-reclaimable memory before reading. Private C ABI prefixes are anchored to
the pinned `limine.h`, with compile-time size/offset checks. Binding convenience
accessors that create unchecked slices or scan C strings are not used.

Loaded kernel bounds use executable-address information and linker image symbols,
including BSS. The raw ELF buffer is tracked separately and may occupy retained
loader memory. Module payloads are independently described and reserved. Usable
and loader-reclaimable regions must be page aligned and disjoint from all other
entries; overlapping non-usable descriptors are legal. Unknown types remain
reserved. Addresses are checked against the architectural 52-bit physical ceiling;
this is not discovery of CPU-specific physical-address capabilities.

The link script places GOT inputs before the image-end symbol. Every build
cross-checks LLVM's image-boundary symbols against every ELF load segment's
memory extent, including BSS, rejecting any omitted orphan section.

Framebuffers support packed RGB 24/32-bit formats, padded pitch and disjoint valid
channel masks. Arithmetic, HHDM translation and framebuffer-map backing are checked.
No pixel is written. EDID and optional mode arrays are neither consumed nor
validated. Existing loader cache mappings remain in effect; no cache policy is
changed or inferred from memory-map type.

Base revision 3 returns a **physical** ACPI root locator. P1-03 checks its nonzero
address and arithmetic but never dereferences it or claims its checksum is valid.
The protocol does not promise locator alignment; OVMF's extended locator can be
20 bytes after its legacy locator. Keep a byte address, not an aligned reference.
P1-06 must establish mappings and validate table contents before discovery.

## Reservations and lifetime

The immutable handoff enumerates kernel image, raw ELF, module and framebuffer
ranges, all non-usable memory-map ranges, and the ACPI locator. Reservations may
overlap. Loader stack, GDT, page tables, response structures and other loader
allocations remain live, so **all** loader-reclaimable memory stays reserved.
P1-03 reclaims zero bytes. The owned descriptors need no loader pointer lifetime;
referenced payloads and bootstrap execution structures still require their backing.
P1-05 must preserve the union of reservations and establish allocation ownership.
Virtual addresses and the HHDM offset describe the current loader page tables;
P1-05 must preserve or explicitly replace those mappings before using these
values after a page-table switch. Owned descriptors do not extend mapping lifetime.

## Records and launch identity

Every supported xtask launch packages a fresh ISO configuration containing
`session=<64 lowercase hex> fixture=<name>`. Session IDs come from host launch
context, are copied once and are not secure random IDs or reset detectors. Kernel
build identity is fixed before configuration and excludes the session value.
Fixture names are limited to 64 characters so records fit their fixed buffer.
Two launches therefore retain identical kernel bytes/build IDs but have different
session IDs, configuration hashes and ISO hashes. The `iso` command packages an
unlaunched image with a prospective session; manual reuse is outside the fresh
session guarantee of the runner.

Version-2 package/launch manifests map these identities to the exact ISO digest.
No ISO hashes itself. Each record is at most 512 bytes including newline:

```text
m1.<event> version=2 build=<id> session=<id> session_origin=host seq=<n> stage=<stage> outcome=<outcome> <event fields>
```

Sequence starts at zero, and a session is immutable across entry, validation,
summaries and completion/rejection. Failures before session parsing explicitly
use `session=unavailable`; acceptance rejects these as unsuccessful boots.
`m1.boot_info_validated` precedes resource/reservation summaries;
`m1.boot_complete` closes successful diagnostics. No uptime is reported.

Host assertions require matching identities, ordered sequences, exactly one entry,
consistent resource counts and a complete terminal outcome. Success tests require
exit 33; expected metadata rejections require the exact field/reason and exit 35.
Normal observation waits for completion, observes for one host second, then reaps
QEMU as host cleanup. The 30-second deadline and 2 MiB log limit remain host limits.

## Fixtures and evidence

`boot-metadata` runs two fresh ordinary boots, a separately packaged real module,
and a serial-only owned-input fixture. The baseline asserts an actual framebuffer
and ACPI locator. The serial-only case exercises absence normalization after safe
capture; it does not claim to test a different physical display configuration.

`boot-reject` covers missing HHDM prerequisite, unsupported memory-response revision,
range overflow, excessive region count, unterminated path, and framebuffer
pitch/format/range errors. The `boot-inject` feature implies `emulator-test`;
ordinary binaries reject fixture requests. Injection operates only after safe
handoff on owned inputs or the shared count/revision/string validators. It never
forges an arbitrary raw pointer or demonstrates resilience to malicious firmware.
Fixture configuration, build flavor and ISO digest are retained separately.

Generated artifacts remain under ignored `build/m1/`. Retain sanitized logs,
`guest-records.json`, source-input inventories, package/launch manifests, identity
comparisons, failed experiments and an acceptance review under
`evidence/phase-1/P1-03/<run-id>/`. Never modify P1-02's historical evidence.
Only mark the story Done after all criteria and the review pass. Exceptions,
paging ownership, interrupts, timekeeping, rendering and full M1 remain later work.
