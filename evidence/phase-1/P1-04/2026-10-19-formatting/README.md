# Final source formatting check — 2026-10-19

This supplements the [full P1-04 acceptance](../2026-10-19/README.md), which remains
unchanged. Staging found one extra EOF blank line in the newly added assembly file.
The final source removes exactly that one newline; no assembler tokens changed.

The [comparison](p1-04-formatting-review.json) verifies that it is the sole changed
build input and that all 40 assembly entry/helper blocks have identical disassembly.
The final source compiled, passed ELF/bounds/no-FP-SIMD audits, packaged an exact ISO
and passed a fresh actual [invalid-opcode guest case](result.json), with expected
RIP/context/exit 37 and confirmed process reaping. This adds one launch to the
preceding full matrix's 21 launches; it does not represent a second full suite.

Final fault build ID:
`69849596921f4adf12d12352e93b97d1fa5a2d19de7b7bac2fd1cf5a611a6e71`.
Exact ELF/ISO/session and input hashes are in the retained manifests. Recover all
source inputs from the full bundle's snapshot, replacing only
[exception-entry.S](source/kernel/src/exception-entry.S) with this final copy.
Every final input hash was checked against the working tree during retention.

The initial comparison assumed all executable bytes would remain identical after
normalizing the embedded build-ID string. That assumption failed: changing the
source-derived identity changes Rust constant placement and corresponding address
operands. The [initial procedure](Compare-P1-04Formatting-initial.cjs) is retained;
its failure was `Error: Executable bytes changed`. The
[corrected comparison](Compare-P1-04Formatting.cjs) records actual segment hashes
and limits its equality claim to the unchanged assembly entry/helper blocks.
No guest failure occurred, and no whole-ELF/text identity is claimed.

The full host/regression/fault matrix is the preceding bundle, over the exact
pre-formatting source captured there. This narrowly scoped follow-up checks the
final whitespace-only edit without repeating unrelated cases. Stack guards,
actual double-fault entry, owned paging and the remaining M1 gates are still open.

Diagnostic path prefixes are sanitized; serial and final assembly source are
verbatim. Binaries remain in ignored `build/m1/`. Everything stays local.
