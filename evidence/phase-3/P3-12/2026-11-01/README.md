# P3-12 — Independent SDK and bounded runtime closure pass locally

**P3-12 is Done under local Codex self-review for its declared bounded subset.**
This is not external approval or the M3 exit. See the six criteria and unsafe-code
review in [review.json](review.json), the [SDK contract](../../../../docs/06-contracts/sdk-v1.md),
[runbook](../../../../docs/05-delivery/phase-3/sdk-runbook.md) and
[ADR-0028](../../../../docs/04-decisions/0028-bounded-independent-sdk-and-portable-runtime-subset.md), which remains Proposed.

P3-12 passes local Codex self-review for the bounded independent SDK 0.1.0 subset. Three clean public-only build roots per ISO preserve one unchanged client across minor-0/minor-1 service builds. The route guest completes 20 version exchanges, two accepted immutable moves, two incompatible-message handle closures and the full nine-incarnation writer-fence matrix. A separate six-process ISO passes reusable 64 KiB heap/32-slot limits, zero/copy/reallocation checks, selected model/grapheme/libm work, preempted FP state, normal return, panic, fallible OOM and exact teardown. The independent reference passes 4871 byte cases and 2411 separately supplied producer cases, including paired field-swap rejection. All 93 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches), ten prior runtime regressions, eleven separate metadata build flavors and affected selected portable link/model checks pass. Only the exact tested contract hash is frozen; supervision and later desktop families remain experimental. No guest display/font/shaping/raster backend, dev transport, state migration, safe old-code retirement, M3 exit or desktop efficiency target is accepted. P3-13 is next.

## Exact identities

Source base: `5e412e8196db893a828a7ec4ef23597a42e42ae8` plus
[measured dirty-input hashes](source-provenance.json). The three pre-existing owner
signing-policy edits are preserved and excluded from staging; their measured hashes
are recorded. Runtime/build inputs are unchanged across prerequisites, all regressions
and both SDK builds. Final documentation is hashed separately.

Frozen contract SHA-256: `debe57866ed308f46ff2f48f5065e739c7457d0f28d0e8dccc93d42abd4c891b`.
SDK package version: 0.1.0. Register revision: 0. Service protocol: major 1,
minor 0/no optional features and minor 1/feature bit 0. These are distinct version spaces.

SDK route guest:

- Build ID: `51217abb298a602223f303b603a6c7f5faf25a2e9c9b383f7ce0e304b5d7a4fe`.
- Kernel SHA-256: `34d879b7204ecc6ae3bf4ae2c2741bc20c6a5be7226a29ba929f0df2817995fe`.
- Launched ISO SHA-256: `33b7d6d336217fd49f9d423fc5355c26d18bbf1ec83f5e1299fcdf8ae38d1fa5`.
- Complete raw serial SHA-256: `caabc3b3a1d4377bc049c09702944da67f15e7a30cdf57d57191423c4eb6ad89`.

Separate nonrendering runtime guest:

- Build ID: `6c4a6447a4136d7eceb9adfc8b6b958e729a55355dd318d9f74cf3cb17cb8516`.
- Kernel SHA-256: `f448147c4f2b096b8c50ba26d6baae8b992071af35ba2cab4744cc82b5112704`.
- Launched ISO SHA-256: `b0107ab15288332d43370f9bb9f7252333961515818213867a89bea6e54ef875`.
- Complete raw serial SHA-256: `3876cae5faaf45343fda0c5b26584d2989708cf66ac5d0db0af4e1153d77d1ab`.

[SDK builds](sdk-builds.json) and [runtime builds](runtime-builds.json) retain the
three independent public input graphs, compiler recipes and exact client/service hashes.
Each client is built once and remains byte-identical while the two services rebuild.
The runtime-feature client differs from the route client. No private kernel module
or native Rust layout is supplied to these clean roots. Packaged provider examples
derive from earlier policy/route logic; the Python expected outcomes and service
header implementation are separate from the production header codec.

## Criterion evidence

| Criterion | Retained result |
| --- | --- |
| Independent builds | Three clean locked/offline public-only roots per ISO; actual extracted ISO ELFs match audited inputs |
| Independent codec/semantics | 4871 classifications, 2411 malformed rejections, decoded field values and 2411 independent producer cases; paired route/policy field swap rejected |
| Compatibility and authority | 20 old/new version exchanges; accepted immutable moves and rejected-message handle closure; trusted sender metadata; full 131-decision route matrix with 25 authoritative attempts, 19 commits, 8 faults and 9 settlements |
| Runtime hooks | Separate 6-process ISO; full 64 KiB/32-slot exhaustion and reuse, zero/copy/realloc failure atomicity, selected model/grapheme/libm, distinct FP markers and nonzero timer preemptions ([10, 6]); normal return, panic 98, fallible OOM 2, later healthy process and exact settlement |
| Freeze | Exact sdk-v1 hash, 26 selected syscall IDs and 15 error IDs, explicit 64-byte startup/48-byte header, synchronized contract/bindings/fixtures; remaining families experimental |
| M4/M5 handoff | Separate process/session/policy/route and monotonic/wall-time domains, manual handle ownership, use-time provider checks, fresh replacement authorization and checkpoint authority exclusion |

The SDK evidence consumer rejects 22 focused mutations; the runtime
consumer rejects 19. The aggregate also runs the prior runtime regression
reviewers. Eleven metadata compilations supplement the actual separately built
ordinary, boundary, scheduler, capability and later runtime regression guests.
The affected selected portable final link and editor-model host tests pass; their
ELF includes the pre-existing link arena and is not executed as a guest UI.

## Unsafe boundary and limits

The kernel retains registered fixture observers, common dispatch and return-state
validation. Public operations cannot choose return IP, selectors or privileged flags.
The wrapper explicitly declares register/memory clobbers; pointer-bearing raw calls
remain unsafe. ELF/symbol/instruction review admits user SSE2 while limiting kernel
FP to its eager save/restore helpers. No AVX, XSAVE or CPU-dispatch path is admitted.

The allocator has one owning execution thread and no callback/signal reentry.
One private 64 KiB RW object stays charged until quiescent unmap or process death;
its source handle closes after mapping. Reused cells are zeroed, live-allocation
shutdown denies, and failed realloc retains the old allocation. Three smoke owners
complete at least 20 reuse cycles each. Normal paths explicitly unmap four heaps;
panic/OOM rely on measured process-local teardown. Lifetime kernel credits are not
refunded or increased. This is not a general allocator or multithreaded runtime.

Complete raw serial streams are retained as numbered `sdk-serial-*.txt` and
`runtime-serial-*.txt`; concatenate each family numerically to reproduce its recorded
SHA-256. Diagnostic suppression is the existing registered route fixture profile;
routine calls are omitted, while version, ownership, decision/commit, fault and
settlement records remain. Launch manifests retain the exact optical UEFI ISO,
one-vCPU Q35/TCG/OVMF arguments, original 30000-ms deadline and host-process reaping.
There is no deadline or log-limit increase. M1's 42 launches include its deliberate
broken-before-entry and rejection cases, not 42 successful normal boots.

Full logs, source copies, ELFs, ISOs, disassembly and failed development runs remain
local in ignored build/work. [Development findings](development-findings.json) retain
host child-start timeout, signer restrictions, image-size/channel-capacity/guest-deadline
failures and the allocator lifetime-credit finding; none counts as acceptance and
no root cause is inferred from timeout alone. [Source freeze check](runtime-source-freeze-check.json)
records unchanged final runtime/build inputs and the earlier deliberate interruption
for payload validation/correlation corrections. No power/security setting, ceiling or physical disk changed.

No native x87 debug metadata, unmasked SSE exceptions, shared-process threads/TLS,
SMP, full guest text/font/shaping/raster/display, development transport, accounts,
durable policy, state migration, safe old-code retirement or general SDK release is
accepted. M4/M5 obligations remain in the runbook. The 256 MiB cold-idle desktop and
guest UI/reload targets remain open; P3-13 sustained adversarial stress is next.
