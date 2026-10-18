# P1-03 acceptance review — 2026-10-18

**Done for P1-03 on the declared Q35/TCG/OVMF profile; M1 remains incomplete.**
Codex performed a local self-review against all five story criteria. This is not
an external review, security certification, performance result or process ABI.

Source: working tree on `codex/p1-03-boot-metadata-validation`, based on `be75100`.
Exact source commit, dirty state and input hashes are in each build manifest.
The [source snapshot](source/) preserves every hashed build input byte-for-byte;
the retained source hashes were checked against the final normal build inventory.
Unchanged helper scripts are also recoverable from the recorded base commit.

## Accepted run and identities

Final acceptance: `acceptance-p1-03-1792346763755`.
All 11 commands in [the acceptance record](acceptance/acceptance-results.json)
returned zero. The [artifact review](review.json) checks every launch's identity,
terminal records, cleanup and complete kernel image bounds. Its exact
[review procedure](acceptance/review-procedure.ps1) and
[retention procedure](acceptance/retention-procedure.ps1) are retained.

| Check | Result |
| --- | --- |
| Host tests | 11 xtask + 7 kernel-library tests pass; formatting passes |
| Phase 0 | Clean link/provenance, path-with-spaces and host failure/cleanup checks pass |
| Dependency probes | All five libraries compile; allocation-dependent links remain expected blockers |
| M1 prerequisites | Pinned inputs/doctor and P1-01 rejection tests pass outside xorriso's MSYS2 sandbox restriction |
| Ordinary metadata boots | Seven: normal observation, two smoke, two metadata identity boots, real module and serial-only fixture |
| Metadata rejections | Eight exact diagnostic/exit-35 cases; no validation-success or completion marker |
| Negative ISO boots | Two expected failures: pre-entry halt and missing configured kernel; neither is positive boot evidence |
| VM lifecycle | All 17 launches use distinct host sessions, unchanged ISO hashes during launch and confirmed process reaping |

Each launch has its own configuration/ISO/session digest. Two fresh metadata boots
use the identical compiled ELF/build ID, and all their checkpoints carry the same
immutable session within a launch. The host maps those identities to the final ISO
hash outside the image. No guest uptime or secure random ID is claimed.

Exact build identities (full ELF digests are in [review.json](review.json)):

| Flavor | Build ID |
| --- | --- |
| Normal | `ac975ba981532862fd96a4b059f0b3981eb02fe09b5febc1cf9fdbee266928a5` |
| Smoke / ordinary metadata | `3c3a5e7d3a3a705c8aca14c9dd464eb8337e39fe9b0e8444c3f37d9b38e451ac` |
| Owned-input injection | `73d7b067a53271b98b7c5a8d42f9d438374207ac7b97873e1be0cf6cdc7267ff` |
| Pre-entry halt | `c05162e4fd4bb0d29c985e90ac56a68de8835e247e67947486b003067e89ebb3` |

## Story criteria review

| Criterion | Evidence and finding |
| --- | --- |
| Normal ISO validates owned metadata and resources | [Normal run](run/result.json) and [metadata matrix](boot-metadata/result.json); complete version-2 validation, region/framebuffer/module/ACPI-locator/reservation summaries |
| Fresh sessions and immutable checkpoints | Matrix identity comparison and per-launch `guest-records.json`; exact build/session/sequence checks pass |
| Required/revision/overflow/count/string rejection | [Rejection matrix](boot-reject/result.json), host tests and retained owned-input fixtures; exact field/reason and exit 35, never a success marker |
| Optional/invalid framebuffer handling | Identified serial-only fixture completes; bad pitch/model/range cases reject before publication |
| Host manifest maps final ISO without circular hashing | Each package/launch manifest includes build, session, fixture, configuration and ISO digests; payload extraction and before/after ISO checks pass |

## Failed and superseded experiments

- Initial raw-ELF backing validation rejected a real loader-reclaimable buffer.
  The raw file and loaded kernel segments now have distinct backing rules. See
  [the first rejected boot](failed-experiments/boot-metadata-1792345390047482600-26852/cold-boot-a-run/serial.log).
- An assumed 16-byte ACPI locator alignment rejected OVMF's extended locator.
  The locator is now a checked physical byte address with no dereference. See
  [the alignment rejection](failed-experiments/boot-metadata-1792345493499238800-34296/cold-boot-a-run/serial.log).
- The first full automated suite passed, but pre-completion review found a
  160-byte GOT beyond `__image_end`. No memory was reclaimed; the non-usable map
  remained reserved. That suite is **superseded**, not accepted completion evidence.
  [The failed boundary review](failed-experiments/image-boundary-review/review.json)
  and LLVM output are retained. GOT placement is now explicit, a host regression
  covers missing BSS/orphan memory, and every corrected build independently
  cross-checks ELF load extents against LLVM boundary symbols. The entire suite
  was rerun after this correction.

## Limits and retention

Bootstrap response/map readability still trusts the pinned firmware/loader.
Numeric pointer checks and owned-input injection do not prove safe dereference of
arbitrary pointers or resistance to malicious firmware. The optional-absence
fixture operates after safe capture; it is not different physical display hardware.

Every loader-reclaimable region remains reserved, including the live stack, GDT,
page tables and response storage. Zero bytes are reclaimed. ACPI table contents,
owned paging, exceptions, interrupts/timekeeping, rendering and live updates are
unimplemented. Virtual addresses describe loader mappings only. The 256 MiB
desktop cold-idle target is unchanged and unmeasured.

The profile remains one vCPU, 2 GiB guest capacity, `pc-q35-11.1`,
`qemu64-v1,+apic`, TCG, pinned OVMF, offline and without shares/data disks/snapshot
restore/direct-kernel boot. Normal observation uses host cleanup, not guest
shutdown. Emulator exits are not physical-machine shutdown.

[The artifact index](evidence-index.json) records raw and retained digests for 742
captured artifacts, including exact source copies. Personal workspace/profile
prefixes were sanitized in diagnostic artifacts; source copies and the tiny module
fixture are verbatim. The index excludes itself and these review files. ISO/ELF
binaries, firmware-variable copies and build caches stay under ignored `build/m1/`;
their exact digests, commands, catalog/payload checks and raw serial results are
retained here. Historical P1-02 evidence is unchanged. Everything remains local.

Next: P1-04 exception diagnostics, following the
[P1-03 runbook](../../../../docs/05-delivery/phase-1/boot-metadata-runbook.md).
