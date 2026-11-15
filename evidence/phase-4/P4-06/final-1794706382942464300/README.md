# P4-06 physical input and guest routing acceptance

Status: passed local Codex self-review, 2026-11-14, for the declared pinned-VM
keyboard/tablet and private routing subset. This is selected story acceptance;
M4 exit, protected input, toolkit/editing, reload and desktop efficiency remain open.

The checkout was safely switched to `main` and pulled with `git pull --ff-only`.
The final tested base is `9da91f4644d8e5492ff50b29bcbc948bd71a03e9` (P4-05).
Final checkout is `main` at merge `179629414c5d5ab24d4723eea8e28fbf1e937559`,
whose Git tree is identical to the tested base. The shared checkout had moved to
the P4-05 branch during implementation; main was safely fast-forwarded, restored
and pulled before final closure. [Closeout findings](final-closeout-findings.json)
record this source-equivalent reconciliation and an evidence-checksum schema fix.
[Input lock](input-lock.json), [source reconstruction](source-reconstruction.json)
and [verification](source-reconstruction-verification.json) identify all 531 tested
execution inputs. Base Git blobs plus exact raw dirty overrides reconstruct every
hash, including line endings and new files; patches supplement them. This is no
clean-checkout bit-reproducibility claim. Staging proof distinguishes Git-normalized
index bytes from exact tested working bytes. No commit or push is performed.

[Selection](selection.json) and [plan](plan.json) declare impact, consumers,
omissions, 24 command groups, at most 17 VM launches and 22 separate metadata
builds before execution. [Results](results.json) and [aggregate review](aggregate-review.json)
pass: 17 reviewed VM runs, 15 launched in the final attempt and two unchanged
input/ordinary runs reused with verified source/context/artifacts and fresh
independent review. The exact [reviewer repair](reviewer-repair.json) explains
why the original failed aggregate stays failed. Changed oracle/runner checks reran.
No full milestone suite or standing waiver is used. Original 30000-ms guest,
320-ms guest-local and inherited lifecycle/heap/retry limits remain unchanged.

## Criterion review

| Criterion | Reviewed evidence and practical limit |
| --- | --- |
| Native keyboard/tablet delivery | Host QMP enters the real VirtIO HID path, independently reconciled against all 104 acquired raw samples: 82 keyboard and 22 tablet, 47 acknowledged injections. Simultaneous Shift, duplicate-down repeat, release ordering and US ASCII text separation pass. Test clients decode explicit bytes independently; 41 publications and 38 actual app receipts are separately recorded, including 8 key, 2 text and 5 button events. |
| Current ownership and geometry | Trusted compositor snapshots select the visible owner and current input scene. Captured pointer movement outside the editor, secondary-button down under capture and per-button release ownership pass. Geometry change cancels; stale releases cannot activate moved controls. Real captured-owner exit clears routing and suppresses post-death release. Paint-only scene updates preserve focus in portable regression. Initial native output is 720p/scale 1; rational 3/2 transforms are portable arithmetic evidence only. |
| Overflow and missing release | Thirty-three native keyboard reports fill the 32-event broker queue. Reserved Cancel explicitly reports 32 discarded broker events and clears held modifiers/keys/buttons. The one-slot copied kernel inbox can separately be superseded by Cancel; publication is never claimed as receipt. A deliberately unreleased physical Shift is cleared by real device reset and app-consumed Cancel with zero modifiers. Per-held-key/button and incomplete-report two-second expiration has portable coverage. |
| Authority and malformed inputs | Current trusted process/incarnation/principal/session bindings guard private operations. Native forged app publication, invalid compositor-copy destination and wrong-session access deny; portable foreign capture/focus, stale geometry, malformed codec/raw events, sequence jumps and expiry reject or cancel. These are distinct portable/native observations, not a malicious-device or protected-login certification. |
| Device and unsafe lifetime | Two exclusive supervisor DMA runs total 64 KiB, explicitly admitted without spending the 16 MiB recovery reserve. Exact modern VirtIO capabilities and eight fixed event descriptors per device are checked. Whole visible batches validate unique heads, lengths and ownership before recycling; failed copies consume nothing. Portable 70000-cycle reuse and malformed completion checks supplement affected native device negotiation/IRQ/reset regressions. Actual reset completes before reclaim; timeout retains/quarantines backing and fails the bounded service path. No IOMMU confinement. |
| Rendering/control progress and settlement | Acquisition timestamps overlap frame-two row copying on the same guest elapsed clock. Input is sampled at bounded 16-row cooperation points; timer waits remain the idle path. Four complete frames and the final QEMU capture match the independent CPU reference pixel-for-pixel. All nine entered owner runs/900 heap cycles, legacy faults/restarts and exact process/object/frame/DMA settlement pass. This proves bounded progress, not input latency, VSync or idle CPU targets. |
| Independent checks | The reviewer reopens complete raw HID, QMP, client receipts, current authority, ELF/ISO identities, frame intervals and pixel bytes; it does not import the broker/parser/codec. Twenty-three criterion-removal mutations reject. Four oracle/repair tests, 29 runner tests and 11 retention tests pass. |
| Affected regressions | Ordinary real-device runtime, existing devices, 720p surface/scanout, timer, owned memory, four boundary guests, three scheduler cases, capabilities, channels and SDK/runtime pass. Ordinary/device/surface host flavors each pass the same 126-test kernel suite; these are 126 unique tests, not 378 distinct tests. Host harness passes 19 tests with three explicitly ignored. Fresh ten-check Phase 0/tool/signing preflights and 22 individually compiled metadata flavors pass. Full M1/M2/M3 remain selected out for the reasons in the manifest. |

Unsafe review: fixed kernel ring metadata has one CPU/syscall owner; IRQ only
acknowledges and updates atomic counters. Supervisor MMIO/DMA accesses retain
checked ranges and ring ordering fences. App buffers are copied through the
existing checked boundary and never retained. CPL3 broker BSS and private queues
have one owning thread and no callback/signal reentry; threads/SMP require review.
No borrow or pin survives user execution. The [input runbook](../../../../docs/05-delivery/phase-4/input-runbook.md)
records bytes, ownership, cancellation and extraction handoff.

## Identities, failure history and retention

Native input build: `6181cdacb7e261216b36349baf2807a86393621e68152062215c4a5242ec4a87`.
ISO SHA-256: `2e7131c8e63ee1edcd534f54ead24321c107ca768d9c11b07bdd966ea22cae54`.
Final original PPM SHA-256: `e1defe2e6c76b2385fa320bc4b6bb386010588b8b42a84edc5f10807bc5ae4fd`.
Native execution takes 16741 ms host elapsed; this is a deadline observation,
not a benchmark. Ordinary ISO SHA-256:
`9690b0580fff8db71dd96f5546ec54ee955c1bda242f88a9a9fc37b628439f45`.
Tool identities/pins and context are in [tool-context.json](tool-context.json).
The literal normal runtime has no synthetic input policy or key/text diagnostic stream.

![Final captured input fixture](input-display.png)

The lossless PNG preserves original RGB pixels; [normalization](normalization.json)
records hashes. `*-selected.txt` retains named criterion groups as excerpts, with
intentional sequence gaps; it is not a complete serial transcript. Complete command
stdout/stderr (including empty streams), guest serial, QMP, original PPM/reference,
identities, commands, reviews and exact dirty source bytes are in the verified
local capsule. [Retention selection](retention-selection.json) explains every file.
The original full journal is about 4 MiB and stays local, rather than violating the
256 KiB permanent per-file limit. The compact boundary review retains the full
codegen review's digest; full observations are in the capsule.

[Development findings](development-findings.json) distinguish DMA admission, broker
stack, duplicate diagnostic field, two timeout investigations, pre-guest interruption
and the ordinary-client reviewer overconstraint. Each materially different failure
retains its original report/raw trace. No failed or interrupted run counts as acceptance.
No sole root cause is inferred from timeout alone. Early failed-source reconstruction
is not guaranteed; their exact binaries/logs remain local. Final tested source is
fully reconstructed. The old aggregate's exact three original reviewer blobs and
failed anchor remain available; this is a one-time correction, not relaxed reuse.

The runner measures 1847006919 bytes of active referenced evidence against the
predeclared 4 GiB budget; shared caches/tools and peak-space guarantees are outside
that count. [Capsule verification](capsule-verification.json) records the selected
compressed payload within 64 MiB. Exact original ISO/ELF/disassembly bytes remain
local for current review/resume; a capsule alone does not replace them. No deletion
occurred and owner defers deletion. Cleanup review remains required after PR merge.
[Closeout method](closeout-method.json) keeps this execution/capsule anchor immutable
and records final staged host-audit closure through a local locator, avoiding a
self-referential evidence rewrite.

The broker is co-located with the compositor; the separate input role still runs
baseline lifecycle workload. Independent replacement, protected focus/privacy,
IME/compose/lock LEDs, arbitrary devices, scaled guest painting, toolkit, editing,
reload and desktop efficiency remain open. Unconfirmed gestures expire after two
seconds. Upstream devices can lose whole reports before acquisition; this bounded
controlled fixture proves its counted stream, not unlimited transport retention.
No new public SDK family is frozen. P4-07 is next; M4 acceptance is P4-19.
