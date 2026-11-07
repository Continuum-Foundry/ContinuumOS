# ADR-0030: Native AI with Gemma local and qualified remote routes

Status: Proposed. Date: 2026-11-07. Owner-requested direction; integration gates pending.

## Context

ADR-0016's small optional CPU experiment does not cover the requested assistant,
provider independence or enterprise deployments. The original OS, native actions,
live evolution and 256 MiB cold-idle target remain central.

## Decision

Offer a native assistant by default in AI-enabled profiles, with persistent user
off controls and conventional operation when inference is absent. Separate the
coordinator, policy/context/credential brokers, executor, model workers, adapters
and outcome verification. Models propose; resource owners authorize effects.
Keep them outside the kernel, compositor/input critical path and recovery core.

Gemma is the default local family, exact version/configuration qualified at M21.
Do not impose a tiny-model product tier. Internally use versioned capability-based
routes for on-device, private organization/consortium and public flagship
GPT/Claude/Grok/Gemini destinations. Enterprise BYO models qualify through the same
contract. No silent cloud disclosure, payer change or assumption of API equivalence.
Tools and web access are separately mediated even when inference is local.

Retain the 256 MiB model-unloaded target with dormant AI costs included; publish
resident/active/actual-default-boot and loading/replacement peak profiles separately.
Offer local download only with capacity, accepted package and user/device policy.
Preloading remains optional and measured. No assumed GPU, NPU, Linux ABI or runtime port.

## Alternatives and consequences

Tiny-only local models sacrifice the requested useful task capability. Gemma-only
internal APIs foreclose enterprise model choice. Cloud-only operation removes
offline/private options. A mandatory resident model breaks the baseline goal.
The selected split keeps a clear supported default while adding adapter,
qualification, licensing and resource-admission work.

## Gate and reconsideration

B67/B68 prepare contracts; B39/M21 follows M5; B69–B74/M23 adds durable personal
work and remote adapters after storage/accounts/packages/network prerequisites.
Measure supported journeys and user benefit. If Gemma's exact target cannot meet
the qualified profile, retain the capability and report the unsupported local
profile; revise the ADR with evidence rather than silently replacing the OS or
raising its baseline. M8/M14/M22/M15 exits and current P4-02 are unchanged.

Details: [architecture](../02-architecture/ai-platform.md),
[roadmap](../05-delivery/ai-roadmap.md), [sources S98–S101](../01-research/sources.md#s98).
ADR-0016 is amended for AI scope by this decision; its isolation/release protections remain.

## Audit clarification

Route-independent B69/B70/B71 do not wait for B39. B74 profile acceptance requires
every enabled route gate, not every possible deployment route. B72 private and
B73 public independently require B81 durable dispatch and B82 credential handling
after network/time/trust. Full M23 coverage still needs all planned route classes.
M21 is an explicitly invoked ephemeral fixed-fixture development test: verify its
authorized digest and parsing before load; no dynamic download/code. Persistent
off and ordinary package/update lifecycle arrive through B71/B70 at M23.

B67 also owns the large-artifact/resource contract: current 64 MiB object and
2 GiB package-memory limits remain. B39 must prove bounded model loading/mapping
and any separately reviewed versioned extension before its fixture runs. B70
qualifies ordinary distribution. No assumed large-file API or frozen-SDK bypass.
