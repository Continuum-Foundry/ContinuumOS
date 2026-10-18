# Risks, open questions and bounded experiments

Status: research conclusions and unresolved implementation/product choices.
Likelihood/impact are qualitative judgments, not statistical estimates.

| ID | Risk / uncertainty | Likelihood / impact | Owner role | Resolution / trigger |
| --- | --- | --- | --- | --- |
| K01 | Host build/VM tooling unavailable or incompatible | Medium / high | Build | M0 inventory and minimal compile/launch; do not infer WSL distro from launcher |
| K02 | Modern language dependencies still require OS hooks | High / high | Runtime | Exact feature/symbol/allocator audit and guest execution spike |
| K03 | GPU acceleration requires much more guest infrastructure | High / high | Graphics | Software path first; B29 backend/driver experiment |
| K04 | Single-address-space prototype becomes permanent | Medium / high | Kernel | M3 hardware-isolation gate before native app updates |
| K05 | Broker becomes a monolith/single recovery bottleneck | Medium / high | Services | Narrow ownership map, failure/restart tests; keep stable core explicit |
| K06 | Quiescence never arrives for busy/blocked component | High / high | Updates | Bounded drain/cancel policy, deadline and safe abort; no forced unload |
| K07 | Stale endpoints bypass writer fencing | Medium / critical | IPC | Every authoritative mutation checks epoch or travels through stable proxy |
| K08 | Post-update rollback discards accepted user work | High / critical | State | Latest-state compatibility/replay tests; forward recovery after irreversible changes |
| K09 | Durable ACK precedes real storage durability | Medium / critical | Storage | Block/flush model, ordered journal tests and explicit failure reporting |
| K10 | Temporary new+old generations exceed RAM | High / high | Performance | Headroom check and peak accounting before staging |
| K11 | Hidden code references leak or use freed memory | High / critical | Updates | Callback/timer/thread inventory, 100-cycle test, safe deferred retirement |
| K12 | User-space driver DMA defeats memory isolation | High absent IOMMU / critical | Platform | State trust limit; later enforced DMA-domain/IOMMU tests |
| K13 | Unicode/accessibility underscoped | High / high | UI | Script/semantic corpus from M2, native assistive roadmap, honest supported subset |
| K14 | Development endpoint becomes release backdoor | Medium / critical | Security | Separate modes/roots, negative release tests, no universal dev key |
| K15 | Host UI performance does not transfer to guest | High / high | Graphics | Shared semantics plus real guest benchmarks; no timing inference |
| K16 | Expanded scope prevents first useful vertical slice | High / high | Project | M0–M6 gates; defer browser/full hardware/general kernel reload |
| K17 | Supply-chain/license problems discovered late | Medium / high | Build | Exact dependency/font inventory before distribution |
| K18 | Timers/FP/SMP bugs undermine deterministic behavior | Medium / critical | Kernel | Context-state and clock tests before enabling optimizations/SMP |
| K19 | Arbitrary hot kernel changes cannot be supported safely | High / high | Kernel | Keep L5 reboot class; separate research proposal if requested |
| K20 | UI aesthetics promise exceeds actual design evidence | Medium / medium | Product/UI | Reference scene variants, visual review and measured reduced-effects parity |
| K21 | Toolkit features/preview/license do not fit guest/platform | Medium / high | UI/Build | B36 exact-target comparison, text/effect/semantic corpus and distribution review |
| K22 | Optional AI displaces reload work or hides memory | Medium / high | Runtime/Project | B39 after M5; separate profiles; no core dependency |
| K23 | Boot rollback is mistaken for live-update correctness | Medium / high | Recovery | Independent B24/B40 tests; preserve current user data |

## Product choices remaining

Working folder: ContinuumOS. Final identity, first-party license, visual style, target physical hardware,
priority between maximum language ergonomics and shortest kernel path, accessibility
release scope beyond semantics, network exposure, installation policy and first
public release definition remain open. Working defaults are enough for M0–M5;
avoid blocking reversible prototypes on branding or speculative hardware support.

## Experiment register

| Experiment | Procedure | Pass criterion | Decision affected |
| --- | --- | --- | --- |
| E01 host platform | Inventory and run smallest locked build/VM/ISO toolchain | Repeatable compile/launch path, exact versions/limits recorded | ADR-0001/0002/0013 |
| E02 UI comparison | Slint/custom scenes with common surface/input/action contracts and exact guest features | M2 provisional choice; M4 guest/text/semantic/performance evidence | ADR-0008/0011 |
| E03 update safety | Replace editor process under typing with faults at each phase | Preserved accepted text; one writer; same boot identity | ADR-0004/0005 |
| E04 persistence | Faulting block model and VM crash around durable decisions | Latest acknowledged durable state or explicit recovery; no dangling generation | ADR-0009 |
| E05 Wasm | Bounded UI logic with adversarial loops/memory/imports | Termination/quota enforcement and acceptable responsiveness/footprint | ADR-0006 |
| E06 acceleration | Exact QEMU host backend plus guest command stack | Real guest acceleration and workload benefit with fallback | ADR-0008 |
| E07 network trust | Stack/TLS with time/entropy/provider and negative cases | Correct validation and bounded resources; no bypass flags | ADR-0012 |
| E08 alternative Swift | Optional pinned x86 freestanding compile/runtime spike | Documented target/platform hooks and comparative workflow evidence | ADR-0001 |
| E09 typed actions | UI/automation operation with stale rights/revisions and provider replacement | Same authorized results; forged/expired requests rejected | ADR-0015 |
| E10 optional inference | Small CPU model, scoped proposals, pressure, crash and unload | Desktop/reload usable; limits/accounting enforced; no privilege bypass | ADR-0016 |
| E11 boot recovery | Interrupt boot/attempt/health records, repeat failures, newer data schema | Bounded fallback/rescue; data retained; exact ISO evidence | ADR-0002/0009 |

E08 is optional, not a prerequisite once Rust meets requirements. Every experiment
has an owner role and evidence output. No experiment is completed by citing another
project's capabilities.

## Estimation policy

No credible delivery date can be promised before M0 and a measured M1/M2 slice.
The core prototype is a significant systems project; broad drivers, browser,
full security assurance and physical laptop support are separate long-running
programs. Estimate each gated task after feasibility rather than compressing them
into a misleading single “build an OS” estimate.
