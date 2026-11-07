# Historical UI pins and current shared build context

Review date: 2026-11-06. Scope: P4-01's fresh hosted M2 recheck, local Codex
self-review. The provisional custom selection, frozen Slint comparison, selected
dependency versions/features/checksums/notices and distribution status are unchanged.

The first fresh M2 attempt passed selected native/resource/performance, corpus,
action, text and lifecycle stages, then rejected two byte-changed shared build
files against P2-09's older input pins. That attempt remains failed at
`build/m2/P2-13/acceptance-1794019250258404600/`.

| Shared file | Historical pin | Reviewed current difference |
| --- | --- | --- |
| `.cargo/config.toml` | `edca149321ded860b2cb2ff4b0c05268fa25a8f3b5138d4915737c1f622c208b` | Comment says native Windows or Linux host; parsed settings, locked alias and offline policy are identical |
| `Cargo.toml` | `40c91f4a3829e0e5fff178aa466ebf544e833cdd508b2d503b6d3a6924e34521` | Adds only `[profile.kernel-stress]`, inheriting `kernel-dev` with integer `opt-level = 3`; every old workspace/package/profile field stays identical |

`tools/Run-P2Acceptance.py` retrieves the exact historical blobs locally from
`420cf61da7b2200ec426e88bbfb19bec5f491a78`, verifies their original SHA-256 pins,
and compares parsed TOML including types and array order. Git transport is denied,
lazy fetching is disabled and reads have a ten-second bound. Missing local history
fails. Only these two named shared paths admit this proof. Every frozen Slint
source/asset, dependency lock, fixture and notice still requires its original byte
hash. No legal selection record, old pin or historical evidence was rewritten.

Typed canonical comparison distinguishes Boolean/integer and integer/float values.
`tools/Test-M2SharedContext.py` rejects fourteen cases: changed alias/offline policy
or type, added global target/alias, changed UI profile, workspace membership/edition,
wrong kernel profile/value type, an extra profile, malformed TOML, unrelated source
and wrong historical pin. These checks run before expensive hosted acceptance.
The selection report records old/current hashes and the archive revision explicitly.
All 32 selected packages retain their versions/features/checksums/terms, and the
171 preserved comparison-evidence files still match their immutable manifests.

This proves current shared host context remains compatible with the historical
comparison; it does not select a new toolkit/library, implement a guest port or
clear distribution. Complete M2 acceptance still builds/tests the actual selected
artifact and final freestanding link, exercises negative hooks/std failures and
all sixteen rows, enforces source guards and retains target misses. The complete
P4-01 gate passes on final tool source after this repair; see the
[compact review](../../../evidence/phase-4/P4-01/final-1794022944926085700/README.md). Earlier M3/host-plan
successes and the failed M2 attempt remain separate evidence.
