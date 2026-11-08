# P4-02 bounded runtime admission acceptance

Status: passed local Codex self-review for the exact finite runtime subset.
Scope: story closeout with selected affected regressions, not a fresh M1/M3
milestone pass or external human approval. All original milestone exits remain.

Source: freshly pulled `main` at `7959d1a6230cee72842b802a6f3afad5b39aa673`, with
implementation hashes in [source-lock.json](source-lock.json) and
[provenance](provenance.json). Three preexisting Windows signing/policy edits were
preserved byte for byte and excluded from the P4-02 staged changes.
Full logs, source patches, binaries, disassemblies and ISOs remain local. No P4-02
commit, remote transfer, power or security-policy change is included.

## Reviewed coverage and owner direction

The owner initially directed original full validation and deferred the approved
selection policy to the next story. On 2026-11-08 the later request, “how many tests
are we running!?!? can we not slim this down???”, superseded that timing. The
verified owned full-run process tree was stopped; its aggregate remains
interrupted, never passed. The [selection](selection.json) records affected
consumers and reasons for omissions. No deadline or new-feature count was reduced.

[Acceptance](acceptance.json) distinguishes reused completed components from five
focused regression cases. [Review](review.json) reopens the actual local
raw serial, ISO, kernel/user ELF and source bytes using the existing independent
oracles. The source-lock deltas when carrying components forward are the selection JSON
and two host-reviewer files correcting the boundary adapter's obsolete assembly
expectation. These are not guest build inputs; guest code and runtime profiles
remain unchanged.
The measured implementation, runtime profile and per-artifact compiler checks
remain unchanged. This one current-campaign carry-forward is explicit; it is not
a generic exemption from the selection runner's conservative resume checks.

| Coverage | Result |
| --- | --- |
| Host suites | 101 ordinary and 101 desktop-runtime tests; exact original coverage plus all four graph/heap tests rechecked |
| Desktop guests | Five completed, separately packaged 30-second cases; 900 heap cycles each; every raw artifact re-reviewed |
| Negative evidence | 248 criterion mutations reject per desktop guest, including extra assertion diagnostics |
| Separate builds | All 16 metadata flavors; exact independent arguments and outcomes checked |
| Ordinary boot | Seven actual CPL3 owners; no acceptance progress observer or injected return/fault behavior |
| Fresh focused cases | Boundary (four optical guests), scheduler (traced/untraced plus real CPU denial), capabilities, waits and independent SDK runtime |
| Final audits | Documentation, governance, staged repository hygiene and whitespace checks |

The complete historical M3 exit remains separate. Fresh full M1/M3 attempts during
this work are failed or interrupted; none is relabeled as a new milestone pass.
Unchanged IPC/policy/route protocols and hosted UI do not receive another aggregate
rerun. Relevant desktop-only feature extensions are covered by the narrow builds
and selected actual guest consumers. Full suites remain required at their own exits.

## Guest observations and bounded resources

Ready and cleanup limits remain 320 ms; local completion remains below 25 seconds.
These HPET observations establish the bounded experiment, not desktop performance.

| Guest | Maximum ready delay (ms) | Case runtime (s) | Healthy control dispatches | Rejected mutations |
| --- | ---: | ---: | ---: | ---: |
| 1 | 102.9616 | 4.4067964 | 219 | 248 |
| 2 | 61.6393 | 4.0770604 | 281 | 248 |
| 3 | 83.9792 | 3.5836193 | 199 | 248 |
| 4 | 104.4059 | 4.9092329 | 233 | 248 |
| 5 | 71.9487 | 3.3743650 | 192 | 248 |

Each desktop guest executes nine owners, 100 heap cycles per owner, three
prepared-owner abort reaps, two editor replacements and CPU-hog quarantine. Every
guest settles spaces, allocations, objects, handles and frames exactly to zero;
12 lifetime units remain spent with no refunds or hidden helper launches.
Ordinary boot is a readiness checkpoint stopped by the host, not a shutdown/soak.

The [experimental profile](runtime-profile.json) admits 168 MiB of fixed private
heaps in a 182 MiB dynamic graph envelope, preserves 16 MiB of physical recovery
reserve and counts images, BSS, tables, stacks and metadata. The 384 MiB active
planning envelope does not accept staging implementation. Actual bootstrap
inventory is 21 handles/objects: CONTROL, CONTROL and QUERY|CONTROL per role
(256/256/257), without queues or device grants. The SDK hash is unchanged.

## Criterion and unsafe review

Local Codex self-review, with a separate raw-record/ELF oracle; no external review or security certification is claimed.

Trusted bootstrap owns the finite seven-role graph, immutable image bindings, synthetic principals/session, fixed per-role allocations and restart/watchdog policy. Payloads cannot select identities, return IP/selectors/flags, executable mappings or public launch parameters. Each owner receives attenuated self-process CONTROL, self-thread CONTROL and timer QUERY|CONTROL. Unknown/legacy IDs retain Unsupported; known omitted operations deny.

Unsafe boundaries: verified module references remain backed by reserved RO/NX pages without writable aliases. The three cached VerifiedImage values contain only immutable boot data and plans; every owner still passes launch/context/commitment checks. Image/root/heap/config grants are completed before queue publication; every prepublication error destroys registry and launcher state. Reaping runs after the user continuation returns and restores kernel CR3, clears the task slot, terminates waits, removes handles and destroys mappings/backing. No memory/registry borrow or copy pin survives user dispatch; shared boundary assertions remain active. Single-vCPU local globals rely on disabled IRQs and no IRQ/NMI access. The allocator has one owning thread and no callback/signal reentry; offset metadata stays private, allocation scans are finite, realloc preserves the old block on failure, and string-memory assembly is integer-only. The existing FXSAVE/FXRSTOR and checked return machinery is reused; no broader CPU/exception support is claimed.

The independent reviewer checks actual ISO/kernel/user ELF hashes, load segments/pages/instructions, source hashes and profile values; derives role generations and prepaid counts from raw launches/reaps; reconciles initial/peak/settled backing and actual handle inventory; requires actual healthy control resumes during the CPU hog; and mutates criterion-closing records. Guest assertions execute allocator alignment, zero-reuse, fragmentation, coalescing, byte/descriptor OOM, failed realloc preservation and complete-capacity reuse, repeated 100 times per executed owner. Host tests separately exercise the 21-unit exhaustion/recovery reserve and all-role quarantine.

Limits: role executables are nonrendering runtime scaffolds; the kernel bootstrap/watchdog is not a general desktop service manager. UI-sized buffers are synthetic and do not validate actual toolkit/font/raster code. 100 heap cycles do not accept live resource reload. Watchdog residence includes kernel elapsed time and is not CPU billing. Private direct mappings retain authority until full owner teardown. No shared heaps/threads, data retirement protocol, device authority, authentication, universal revocation, desktop performance or 256 MiB cold-idle result is accepted. P4-03 remains the next gated story after acceptance; P4-07 must exercise real UI dependencies.

Acceptance control termination applies only to an owner without an existing result; panic, exit and assertion failures retain their original outcome. The raw reviewer rejects every unexpected user diagnostic, including an injected assertion failure alongside otherwise intact completion markers. These checks close the observed failure-masking review gap.


The first selected closeout's four boundary guests all passed, but its raw review
failed on an obsolete P3-04 20-push/15-pop syscall expectation. Current P3-05/P3-12
already require the balanced RAX/CR0 guard's exact 21/16 layout. The focused adapter
now requests that exact layout explicitly; historical P3-04 keeps its default.
The current real artifact passes, while missing push/pop/CR0, surplus push and
historical-default checks reject. The failed first closeout remains failed; its
completed boundary executions are carried forward and freshly re-reviewed. Only
the remaining four cases are executed again, without repeating boundary guests.

## Preserved failures and limits

The compact findings retain the failed development boots, omitted wait-resume
repair, real timeout handling, allocator-cap correction, 414.333 ms ready-bound
failure, restart-starvation and verbose-log failures, unsupported-operation ABI
repair, the inherited 97-versus-101 host-oracle failure and its 15 negative checks,
source/index drift, and the acceptance-controller failure-masking repair. The
controller now preserves an existing task result, and raw review rejects surplus
assertion diagnostics. Earlier mixed-source/agent-stopped runs are interrupted.

Host startup failures remain failed at their original 1000 ms bounds. Moving child
dispatch before parent-only imports did not eliminate timing variation; diagnostic
interpreter flags were not adopted and cause remains unproven. A later full run
reached 32/33 M1 rows before host compilation timed out; selected Windows events
record lid-triggered Modern Standby overlapping that attempt only. Its later
standalone timer pass does not replace the failed aggregate.

Another full run passed five desktop guests, ordinary boot and five stress guests,
then its sixth stress guest hit the unchanged 60000 ms bound after 91 completed
waves. It was reaped. An identical-kernel diagnostic completed all 100 waves in
34.940 seconds and passed raw review; the variation's cause is unproven. The next
complete run was stopped at the owner's request after its completed new-behavior
components; no complete M3 pass is claimed from it. Selected components were
individually re-reviewed for this story, with explicit provenance and current-source
checks. The failed/interrupted aggregate records are retained unchanged in status.

Only the two previously approved long M3 stress flavors use revision-2 60000 ms;
this is not an original-limit M3 pass. Ordinary/desktop guest deadlines remain
30000 ms. No sample count, host power or security setting was changed.

## Reproduction and retention

For a fresh selected story run, use the approved signer/xorriso context serially:

```powershell
pwsh -NoProfile -File tools/Test-P4-02.ps1 -Manifest config/p4-02-validation.json
python tools/p4_runtime_acceptance.py review --acceptance <reported-acceptance.json>
```

The [closeout method](closeout-method.json) identifies the exact local campaign
driver and reviewer source hashes; full source stays local with the raw artifacts.
The acceptance records carry every command, origin, selected review and source
delta. Full local artifacts are required to reopen this particular report; this
explicit campaign carry-forward is not a generic resume interface.

See the [runtime runbook](../../../../docs/05-delivery/phase-4/runtime-admission-runbook.md)
and [retention policy](../../../../docs/05-delivery/evidence-retention.md). Workspace
paths are sanitized to `${WORKSPACE}`; [normalization](normalization.json) records
original hashes, and [trace selection](trace-selection.json) declares each selected
line. Selected text is not a full log. [Checksums](SHA256SUMS.json) cover actual
retained members; source copies and full logs/ELFs/ISOs remain local-only.

P4-03 remains Not started and is next. Actual toolkit/font rendering belongs to
P4-07; device input, authenticated dev transport, live reload, safe old-data/code
retirement and the 256 MiB desktop efficiency target remain open. No new ABI freezes.
