# P3-13 bounded stress experiment

Status: declared bounded P3-13 subset passes local Codex self-review; M3 exit remains open.
See the [reviewed measurements](../../../evidence/phase-3/P3-13/2026-11-01/README.md).
The [story](13-adversarial-lifecycle-and-resource-accounting.md) and
[fixed inputs](../../../config/p3-13-stress.json) govern the experiment.
P3-12's exact SDK freeze remains unchanged.

Stress images use the explicitly recorded `kernel-stress` profile and public
fixture release build at optimization level 3. Earlier level-1 development
attempts remain retained as failures; their timings do not describe this profile.
Ordinary/prior-story kernel builds retain `kernel-dev`. No deadline, resource
ceiling, test count or zero-growth tolerance changes with optimization.

The adapter verifies all three immutable RO/NX boot modules once and retains
their typed launch plans. Verification duration and fixed cache bytes are
reported; the cache is charged within the kernel image. Every launch retains
the original commitment, generation, staging and publication checks. Ordinary
launch paths still hash/parse their input. This does not support mutable module
replacement or reload. The retained cache is constant across all 100 waves.

Each instrumentation profile has five fresh one-vCPU guests. Each guest keeps
one healthy control process alive across 100 two-worker admission/work/exit/reap
waves and 100 synthetic provider route/fence cycles. Thus each profile executes
1,000 complete worker lifecycles. The first wave adds 1,000 warm-up round trips;
each wave measures 100, yielding 10,000 measured round trips per repetition.
These are runtime cycles, not state-preserving reloads. A restart or snapshot
cannot substitute for any of the 100 continuous waves.

The throughput workers use the P3-12 trusted bootstrap profile. They do not
claim P3-09's much smaller prepaid supervision envelope: a unit there has four
lifetime messages and twelve handle slots. A separate public-wrapper port of
the complete four-epoch/29-admission supervision fixture preserves those exact
limits, including ancestor quotas, failed admissions, retry quarantine, parked
owner/orphan termination and supervisor death. Experimental syscall families
remain outside the SDK freeze. No throughput measurement raises their ceilings.

The source declares scenario IDs, seed, finite limits and exact expected errors
before running. The inherited 30-second guest deadline, 64-quantum fairness ceiling,
resource capacities and zero unexplained retained-object/frame tolerances remain
unchanged. A missing completion, host preflight failure or guest timeout fails;
retain it and diagnose without increasing a bound to obtain a pass.

All guest intervals use elapsed HPET time. IPC samples exclude warm-up and use
nearest-rank quantiles. Dispatch/wake sampling retains every 32nd observation in fixed-capacity buffers,
with complete observation counts and maxima; overflow rejects the run. It cannot
describe an unsampled tail.

The interrupt-off observer reports a conservative upper bound: each observed gap
contains a real resume/STI boundary, so the maximum sum of adjacent gaps plus
clock resolution covers a masked interval crossing an observation. It includes
user/HLT time rather than subtracting it. Dispatcher body timing remains a
separate observation. The complete 64-quantum settle spans at least 320 ms and
64 delivered ticks; an extra halt may be needed for initial phase alignment.
After all owners/waits/borrows settle, sample export runs with interrupts enabled
and retains its own elapsed duration. None of this raises the 320 ms bound.
Paired tracing adds read-only inventory observation, retaining mandatory timing
and outcome instrumentation in both profiles. Report each profile separately;
do not subtract host and guest clocks or call emulator elapsed time CPU time.

The public-only fixture imports the frozen SDK and the packaged provider policy
and route implementation. Trusted bootstrap pages supply process bindings and
endpoints; they are fixture addresses, not a new SDK discovery interface. The
healthy process uses its own private control channel and completes when the
trusted coordinator closes the other endpoint. Public operations do not select
return IP, selectors or privileged flags.

Paired ISO runs consume one exact public ELF for each source/compiler input set.
The isolated locked/offline Cargo build lives under an ignored source-keyed host
cache; each ISO gets a source snapshot and rechecked ELF copy. Every source and
compiler backend hash participates in its key. This avoids Cargo/linker identity
or layout variation between fresh roots affecting the comparison. This host build
artifact cache is separate from the guest's three immutable launch plans.
Raw IPC values use sixteen-sample records with five fixed-width base62 digits
per nanosecond value (alphabet `0-9 A-Z a-z`), within the existing 96-byte log
fragment and its alphanumeric/hyphen vocabulary. The fixed 320 ms ceiling fits
without quantization or discarded bits;
scheduler latency values use batches of up to sixteen in direct kernel records.
The reviewer requires every sample and batch index, with no downsampling of IPC.

The [inventory schema](../../../config/p3-13-inventory.json) declares all original
memory, object, backing and owner fields in explicit wire order. Each snapshot
uses canonical unsigned LEB128 integers and bounded hex chunks, reducing repeated
serial headers while retaining every numeric value. The independent reviewer
rejects noncanonical/overflowing integers, missing/reordered/interleaved chunks,
wrong field counts and retargeted owners, then expands complete resource CSVs.
Schema 2 distinguishes physical resident backing from per-owner quota charges:
foreign mappings add commitment credits without duplicating physical bytes.
Both global totals reconcile with their corresponding owner/orphan rows.
Seven route decisions use seven fixed four-byte status/revision/value/queued
tuples in one bounded diagnostic token. No Rust layout crosses either boundary.

The private fixture protocol is four explicitly little-endian u64 words (32
bytes): sequence, operation/revision, wave and value. The operation occupies the
low 32 bits; the high 32 bits are revision zero. Echo=0 carries seed 20261025;
route-check=1 uses sequence/value zero and replies with seven checked outcomes;
finish=2 uses sequence/value zero; immutable-share=3 carries one attenuated read
handle and replies with its 4096-byte size. Unknown operation/revision and wrong
byte/handle counts fail the fixture. This is test traffic, outside the SDK freeze.

## Commands

Use the existing signer/xorriso execution context, keep the host awake, and run
acceptance serially. These commands now exist; their presence is not acceptance.

```powershell
. tools/Enter-Dev.ps1
$env:RUST_MIN_STACK='16777216'
$env:RUST_TEST_THREADS='1'
cargo xtask test --suite m1 --case sdk-stress
cargo xtask test --suite m1 --case sdk-stress-trace
cargo xtask test --suite m1 --case sdk-stress-supervisor
python tools/Review-P3-13.py --run <stress-artifacts> --output <local-review>
python tools/Review-P3-13.py --supervisor --run <supervision-artifacts> --output <local-review>
pwsh -NoProfile -File tools/Test-P3-13.ps1
python tools/Review-P3-13.py --acceptance <acceptance.json>
```

The final script runs the complete current P3-12 prerequisite/M1/runtime/codec/
portable regression gate and its independent review, the public supervision
fixture, fourteen separate metadata build flavors and five repetitions of each
stress instrumentation profile. A failed command stops it without retrying.

The final stress/supervision commands use `cargo run --release --locked --offline
-p xtask -- ...`: the identical native observer source is optimized because it
revalidates the complete growing serial tree every 20 ms. Validation, polling and
guest deadlines are unchanged. Direct `cargo xtask` commands above use the debug
observer and remain useful for debugging, not the declared measurement profile.
`-RuntimeAcceptance <passed-P3-12-acceptance.json>` may reuse an existing complete
regression gate only after the entire measured source set and compiler match;
the independent raw review runs again. Any changed/new/missing input rejects reuse.

The final independent review reopens all ten raw guest runs and the separate
public supervision run, checks ISO payload/source identity against the fresh
runtime regressions, rejects corrupted evidence and computes paired overhead.
Optional `--destination evidence/phase-3/P3-13/<date>` promotes a compact bundle:
all IPC samples and cycle rows, complete resource series for the first repetition
of each profile, explicit selected serial records and full local trace hashes.

Measured results, source identities, accounting detail and unsafe-code review
are retained in the compact evidence linked above. Full generated traces and images
remain local under ignored build/work directories.
The 256 MiB cold-idle desktop, guest UI/reload and M3 exit remain open.
