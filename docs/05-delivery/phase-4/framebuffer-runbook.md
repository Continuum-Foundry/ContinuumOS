# P4-04 checked software scanout

Status: passes [local P4-04 self-review](../../../evidence/phase-4/P4-04/final-1794635215585403600/README.md) for the bounded software-copy backend.

The temporary kernel display mediator accepts one checked boot framebuffer, at
most 1920x1080, at most 8192 bytes per row, 24/32-bit pixels with three disjoint
byte-aligned eight-bit RGB channels. It checks the complete page-rounded mapping
against framebuffer-only reservations, kernel/module/ELF/ACPI ranges and the
selected CPU physical width. Unsupported or absent output emits a precise serial
`m1.display_unavailable` record. Serial and the boot-memory-timer baseline remain
independent; no desktop-ready or presentation success follows from that record.

Only the trusted compositor/display role (role 1 in the finite boot graph) may
acquire the resource. The mediator uses the P4-03 broker's checked MAP grant,
exact process incarnation and monotonically increasing output generation.
All scanout mappings are supervisor-only RW/NX/UC, without a WB direct alias.
No user scanout mapping or capture operation exists. Owner teardown revokes the
grant after synchronous copies have returned. The page window stays boot-reserved;
no DMA or asynchronous reader exists here. The mediator is temporary trusted
kernel code; P4-05 receives the copy backend, while future driver extraction must
preserve grant, mapping, copy lifetime and teardown rules.

The experimental internal operations 0x4100-0x4104 are installed by the desktop
runtime hook, outside the frozen SDK. Acquire returns an output generation;
query checks it and returns packed width/height. Row submission takes a source
pointer, packed signed 32-bit x/y placement, 1-1920 RGBX pixels and the current
generation. It first copies the complete row through the ordinary checked/pinned
user-copy boundary into at most 7680 private kernel bytes, then clips and converts
RGB to the validated scanout channel layout using volatile byte stores. Bad source,
capacity, role or generation causes no scanout writes. Completely clipped rows
still validate/copy the source. Alpha is ignored; this is an opaque sRGB byte copy,
not an alpha or color-management pipeline. Full initial zero clear includes padded
scanout rows. Conversion never writes beyond the admitted pitch/height extent.

Return follows an x86 MFENCE: all source reads and CPU scanout writes are complete,
and source memory can be changed or freed immediately. No pointer survives the
call. This is CPU-copy completion, not a page flip, VSync, tear-free scanout,
monitor acknowledgement or measured display latency. A failed later row may leave
previous rows visible; there is no atomic full-frame presentation promise.

## Validation selection and reproduction

Run serially in the approved signer/xorriso context; keep the host awake without
changing power or security settings.

```powershell
pwsh -NoProfile -File tools/Test-P4-04.ps1 -Plan
pwsh -NoProfile -File tools/Test-P4-04.ps1
```

The selection in `config/p4-04-validation.json` includes all staged, unstaged and
untracked inputs through the story runner's source snapshot. It declares 21 command
groups, 20 VM launches, 18 separate metadata builds and no full-suite opt-in.
New 720p and 1080p display guests exercise the trusted role, denied same-user app
mapping/capture, stale generation, invalid source/capacity, partial map rollback,
row-source reuse and full screen RGB/corner/edge patterns. Each exact ISO requests
its mode in Limine configuration. QMP is test-only stdio to the owned QEMU child;
no socket listener, network interface, host shares or guest-supplied host commands
are added. The observer matches each QMP reply before its next request and retains exact JSON request bytes. It captures the initially cleared output, then requests a stopped-VM screendump after the copy checkpoint,
then continues and reaps the guest within its unchanged 30000-ms deadline.

An independent Python oracle reconstructs all RGB pixels, checks the PPM dimensions
and compares every displayed pixel. Its CPU reference is a separate artifact from
the QEMU displayed capture and raw serial copy/mapping observations. Host tests
exercise both byte orders, 24/32-bit packing, padded rows, clipping and guard bytes,
invalid metadata arithmetic/backing/reservations, and broker incarnation/generation
lifetime. Selected regressions cover optical metadata/identity, owned pages and
memory, an actual guard exception, timer progress and separate boundary/scheduler/
capability/SDK-runtime consumers. IPC/policy/route and hosted UI are unchanged;
full milestone gates remain required at their exits.

Full logs, failures, ISOs, source snapshots and captures stay under ignored build
storage. The linked compact acceptance records exact identities and limitations.
No compositor, toolkit, input, live reload, 256 MiB cold idle, scanout timing or
VSync result is accepted by this implementation alone.

The [QMP specification](https://www.qemu.org/docs/master/interop/qmp-spec.html)
and [screendump reference](https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html#command-screendump)
define request IDs and capture arguments. A separately bounded no-ISO diagnostic
reproduced the Windows burst-request rejection and passed with response-matched
sequential requests. It is explicitly run outside the ordinary host test filter;
the normal filter leaves that VM-bearing diagnostic ignored.

The observer lifecycle host test now re-enters its already signed native test
binary for the four child behaviors instead of starting PowerShell scripts. The
three-second host bound and early/wrong exit, timeout, log-overflow and cleanup
assertions remain. Its child-only test entry is ignored by ordinary discovery and
explicitly invoked by the parent test. A retained PowerShell fixture completion
missed the bound; the underlying timing variation remains unresolved, and that
failed prerequisite attempt is not acceptance. Guest limits remain unchanged.

Desktop launch staging now maps the small read-only config page before the large
private heap. The transactional table builder reconstructs the inactive bank per
map, so the earlier order needlessly reconstructed the same large heap twice.
Image/grant/config/heap-allocation/heap-map cumulative timestamps are acceptance
diagnostics, not a performance guarantee. Heap sizes, allocation inventories, all
nine owner runs/900 heap cycles, two editor restarts, reserve and original timing
bounds remain. A retained failure exceeded the 320-ms ready bound during an editor
restart; it remains a failed run. The subsequent focused 720p probe passed with
140353800-ns maximum ready delay and exact displayed pixels; the fresh full selected story gate subsequently passed at both resolutions.
