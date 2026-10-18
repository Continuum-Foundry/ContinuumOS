# Typed platform actions and optional AI

Status: proposed platform design. R19 is core; inference is an optional extension.
Decisions: [ADR-0015](../04-decisions/0015-typed-actions.md),
[ADR-0016](../04-decisions/0016-optional-inference.md).

## Core action architecture

Apps/services publish versioned typed operations. Buttons, shortcuts, accessibility
actions and authorized automation invoke the same operation contract. There is no
special bypass for an assistant. Providers remain authoritative about validation,
resource access and side effects. An action registry aids discovery but cannot grant
authority by naming an operation.

The executor resolves registered action IDs, validates typed inputs, checks caller
capabilities, expected state versions, quotas and cancellation policy, then invokes
the provider with correlation/idempotency IDs. It records the result and any explicitly
supported compensation/undo description. A preview is informative; revalidate permissions
and state when executing so a stale preview cannot authorize changed work.

The registry/executor can begin as a small SDK/service layer over existing IPC.
Do not block the first editor on a general workflow language or distributed transaction
engine. Start with document edit/save and one harmless settings action.

## Context objects

A context descriptor has logical identity, source/provenance, revision, owning service,
permitted consumer, purpose, expiry and allowed operations. The actual authority is a
validated handle/lease. Model-generated text cannot mint that authority. Avoid copying
whole documents where a narrow reference or selected excerpt suffices.

The producer and consumer check expiration/revocation, and live replacement rechecks
epochs. Checkpoints preserve only approved logical references; no secrets or raw handle
slots by default. Context data remains untrusted content, including text that looks like
instructions. Do not allow it to alter executor policy.

## Plans and effects

A plan describes ordered proposed actions, dependencies, expected revisions, requested
authority, estimated cost, preview and explicit effect/undo metadata. Authorization is
policy-based and scoped to the actual operation. Ordinary already-authorized UI actions
do not need redundant confirmation. Sensitive new authority or destructive/external
effects follow the product's explicit policy.

Individual operations can be transactional under their owner. A sequence spanning files,
network peers and devices is not automatically atomic. Keep completed step/result records;
stop on failed preconditions; invoke compensation only when the provider advertises it
and current versions permit it. Cancellation does not erase an already committed effect.
The [action contract](../06-contracts/actions-and-context.md) specifies MVP fields/errors.

## Optional inference service

Inference runs in a separately protected, quota-limited process. It receives scoped
context and produces proposals; it cannot directly gain kernel, filesystem or network
authority. The ordinary desktop must boot, render, save and live-update without it.
A crashed inference worker must leave the editor and update/recovery paths usable.

After M5, a Burn CPU/no-OS spike may evaluate a small supported classification or
structured prediction task. Official embedded inference examples establish a candidate
path, not compatibility with a large language model or a particular quantized format.
Validate the exact backend, operators, allocator, CPU features, model converter and
license. [S80](../01-research/sources.md#s80)

Correct FP/SIMD enablement/save/restore, scheduling isolation and memory quotas precede
inference. No assumed CUDA/Vulkan/NPU stack. No mandatory guest network connection or
hosted model service. A model cannot modify its own permission grants or approve repairs.

## Memory and performance

Report cold idle with model absent, warm idle with the selected model resident, and
active inference alongside interactive desktop work. Include weights/quantization
metadata, attention/KV caches where relevant, activations/scratch, tokenizer/runtime,
queues, shared/device memory and model-loading peaks. Unload actually releases the
accounted resources. Guest/host allocations and compiler work remain distinct.

The baseline cold-idle target remains 256 MiB on the declared profile. It is not replaced
by a 2 GiB allowance. VM configured RAM does not equal resident OS usage. Large-model
targets are set only after a specific measured model/runtime workload is selected.

## Acceptance

R19 tests invoke the same operation via UI and automation; reject forged/stale authority,
wrong revisions and oversized inputs; preserve idempotency through supported reloads.
Optional AI tests add adversarial context, unauthorized proposals, model process crash,
unload/reload, time/memory pressure and desktop responsiveness. They do not block the
AI-free core release unless the user explicitly changes scope later.
