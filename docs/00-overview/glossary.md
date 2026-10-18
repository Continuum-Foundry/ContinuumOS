# Glossary

| Term | Meaning in this project |
| --- | --- |
| ABI | Binary calling/data convention, distinct from a source API |
| ADR | Architecture decision record: context, alternatives, choice, consequences, verification gate |
| Activation | Switching a logical service route to a prepared generation |
| Boot ID | Identifier initialized once per kernel boot; used with uptime to demonstrate no reboot |
| Broker | Service that owns a resource or route and mediates access through controlled handles |
| Capability | Unforgeable authority to act on an object; a process-local integer alone is not a complete capability design |
| Checkpoint | Versioned, validated logical state that can seed another instance |
| Compositor | Combines app surfaces and desktop elements into display output |
| Damage | Region whose displayed result may have changed |
| Development core | Initially stable supervisor, update endpoint, essential brokers, and kernel |
| Epoch | Monotonic logical generation used to reject stale writers and replies |
| Frame budget | Time between display deadlines; not the same as total input latency |
| Generation | Immutable set of package identities plus compatible configuration/protocol versions |
| Hot reload | Supported code/resource changes appear while the machine remains running |
| Hot restart | A component restarts while the OS remains up; may or may not restore its state |
| ISO | Bootable optical image distribution artifact; normally read-only in the guest |
| Live update | Coordinated replacement of a component and its state/resources |
| Migration | Explicit conversion from one version of logical state to another |
| Quiescence | Component has reached a declared safe point with controlled in-flight work |
| RCU | Read-copy-update style deferred reclamation; not a substitute for migration semantics |
| Retained scene | Persistent representation of visual objects, changed incrementally |
| Safe point | Controlled execution boundary where replacement can proceed without running old callbacks |
| Service restart | New process instance; continuity depends on checkpoints and brokers |
| Software renderer | Rasterizes on CPU; can use SIMD only after CPU-state management is correct |
| State owner | The single authoritative component allowed to mutate particular data |
| TCB | Trusted computing base relative to a specific property, not just the kernel |
| VirtIO | Standardized virtual-device interfaces requiring real guest drivers |
| VM | Here normally virtual machine; virtual memory is spelled out where ambiguous |
| W^X | Writable mappings should not simultaneously be executable |
| Wasm | WebAssembly execution format; needs an embedded runtime and explicit host imports |
| WIT | Interface description language associated with the Wasm Component Model |

“Zero-copy,” “atomic,” “real-time,” “secure,” “from scratch,” and “rollback” must
always state their scope. A pointer swap is not an atomic external transaction;
shared memory may still involve copies elsewhere; a measured fast path is not a
worst-case scheduling guarantee.
