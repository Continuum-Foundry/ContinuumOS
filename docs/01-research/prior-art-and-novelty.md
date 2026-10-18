# Prior art and honest novelty

## Relevant precedents

| Work | Verified contribution relevant here | What to learn | What not to infer |
| --- | --- | --- | --- |
| Theseus | Rust-based OS structure/state management supports live evolution | Limit state held on behalf of other modules; treat evolution as structural | Our different kernel inherits neither mechanism nor results |
| MINIX service updates | Message-based service quiescence and state transfer | Explicit safe points and lifecycle supervision | Every driver or kernel change becomes reloadable |
| Erlang code loading | Current/old module versions and controlled transitions | Old execution needs a retirement policy | BEAM semantics apply to native Rust pointers |
| Linux livepatch | Function replacement with consistency rules | Define which functions/states can coexist | Arbitrary kernel layouts or scheduler designs can update live |
| Singularity | Managed-language isolation and communication contracts | Language/runtime/OS co-design is possible | Modern hardware alone makes all runtime costs negligible |
| seL4 | Capability-based kernel with precisely scoped assurance | Minimize and state the trusted boundary | Our Rust kernel is formally verified |
| Fuchsia | Handles, channels and routed capabilities | Resource identity and authority should be explicit | We should adopt its complete platform or APIs |
| Genode | Component/driver decomposition | Narrow device-independent interfaces | User-space DMA drivers are automatically harmless |
| Qt / Wayland / Android | Retained rendering, surfaces, paced composition | Retain work; separate app content and display policy | Existing Linux display infrastructure is present in our guest |

Sources: [S14](sources.md#s14), [S15](sources.md#s15), [S16](sources.md#s16),
[S17](sources.md#s17), [S13](sources.md#s13), [S19](sources.md#s19),
[S21](sources.md#s21), [S24](sources.md#s24), [S39](sources.md#s39),
[S42](sources.md#s42), [S44](sources.md#s44).

## Candidate differentiation

Our possible contribution is a coherent developer experience: a from-scratch OS
with a deliberately reloadable desktop, an inspectable state/resource model,
content-addressed generations, efficient native graphics, and excellent recovery.
The combination could be distinctive, but research has not established patent
novelty or global firstness. Do not use “first-ever hot-reload ISO” in project claims.

The ISO is a reproducible snapshot. The running system receives new component
generations. A compelling demonstration proves continuity of actual work and
records which part changed; it does not disguise a background VM reboot.

## Architecture alternatives considered

**Monolithic kernel with dynamic modules:** easiest initial direct calls and shared
state, but native unload and state ownership become difficult. A bad driver can
corrupt everything. Suitable only as an explicitly temporary bootstrap stage.

**Small kernel and isolated services:** more loader/IPC/broker work at the outset,
but failures and replacement have concrete boundaries. Selected default.

**Intralingual single-address-space design:** Theseus demonstrates valuable ideas,
but adopting that structure would require careful language-level guarantees and
toolchain integration. Our proposal keeps hardware process isolation and borrows
state discipline; it does not blend the two models without proof.

**Managed/Wasm-centered foundation:** can improve replacement boundaries and app
language choices, but runtime porting and scheduling become critical infrastructure.
Use experimentally above a native core, not as a prerequisite for boot.

**Existing microkernel with a new desktop:** potentially faster product delivery,
but contradicts the chosen interpretation of from scratch. Keep as a documented
scope-changing alternative only.

## Reading strategy for future agents

Read primary papers for invariants and failure cases, not just diagrams. When
borrowing code rather than an idea, inspect the exact license and record provenance.
State which portion of a result transfers to our design and which requires new
evidence. Do not accumulate research indefinitely after a bounded prototype can
answer the actual integration question.
