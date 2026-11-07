# AI model, runtime and deployment survey

Review date: 2026-11-07. Source-backed upstream findings and proposed engineering
choices, not native guest measurements. Catalogued sources S98–S114 supplement
the directly linked primary model/runtime pages below. No weights or packages
were downloaded, provider accounts configured or project context uploaded.

## Selected product split

Use Gemma as the supported local default, qualified by exact artifact and workload.
Offer GPT/Claude/Grok/Gemini as independently qualified public flagship routes.
Retain a third private organization/consortium destination and a provider-neutral
native task/tool interface. This gives a clear default without blocking enterprise
BYO models, alternative accelerators or federated services. Model names identify
families, not permanent API aliases or a promised version/license entitlement.

Google documents Gemma 4 configurations and runtime-dependent memory estimates;
these omit additional context/software costs and do not establish OS footprint.
Gemma 4's published Apache terms must not be generalized to older Gemma artifacts.
Choose the smallest configuration that passes the declared task corpus, rather
than qualifying a tiny model merely to fit a memory slogan. [Gemma overview](https://ai.google.dev/gemma/docs/core),
[Gemma 4 model card](https://huggingface.co/google/gemma-4-E2B-it),
[published Apache terms](https://ai.google.dev/gemma/apache_2).

The 256 MiB model-unloaded target remains. Fully counted local-resident/active and
loading/replacement profiles require separate ceilings after measurement. A
capable machine may offer a verified local model download with user/device
storage/network policy and desktop reserve; it does not automatically preload.
Thin clients can use eligible private/public infrastructure. If no route is
authorized/available, preserve task progress and conventional controls. There is
no automatic disclosure or billing fallback.

The 2 GiB reference VM is not assumed to admit a useful Gemma configuration. A
local-AI profile may require greater machine capacity while retaining the same
model-unloaded desktop target. Qualify CPU features and scheduler-owned state too:
the current eager x87/SSE subset does not establish AVX/AVX-512/XSAVE handling.
Reject unsupported instructions/backends or separately implement and accept that
state support before use. Audit threading, TLS, allocator, file mapping, platform
calls and foreign-runtime dependencies; a model file alone is not an executable
inference stack. Do not silently add SMP or an accelerator as an early prerequisite.

## Alternatives retained for enterprise qualification

These candidates preserve choice; they are not a promise to port every family or
evidence that a benchmark winner will perform well on this OS's workflows.

| Family and primary source | Why retain it | Qualification boundary |
| --- | --- | --- |
| [Qwen3.5-4B](https://huggingface.co/Qwen/Qwen3.5-4B) | A comparable small multimodal candidate with published Apache-2.0 artifact metadata | Pin exact artifact; prove operators, tools, context and licensing. Do not generalize one release's license to all Qwen. |
| [Ministral 3](https://mistral.ai/news/mistral-3/) | Published 3/8/14B variants and Apache release provide another local/private family | Quantization, language/task accuracy and native runtime closure need tests; advertised size is not resident memory. |
| [Granite 4.0](https://www.ibm.com/granite/docs/models/granite4-0) | Hybrid model designs offer another enterprise-oriented candidate | Active parameters do not equal stored weights; evaluate state/context footprint and tool behavior. |
| [Phi-4-mini](https://huggingface.co/microsoft/Phi-4-mini-instruct) | Small instruction model with published MIT artifact license | Training focus and function-call format do not prove native actions or repair reliability. |
| [gpt-oss-20b](https://huggingface.co/openai/gpt-oss-20b) | Open-weight option for larger workstations/private serving | Separate from public flagship GPT; exact quantization/runtime/hardware/format must qualify. |
| [Olmo](https://docs.allenai.org/models/olmo/index.html) | Research transparency and model lineage offer a distinct auditability option | Transparency supports inspection, not an assurance or memory result for this deployment. |
| [Nemotron](https://huggingface.co/nvidia), [Llama](https://www.llama.com/), [DeepSeek](https://huggingface.co/deepseek-ai), [GLM](https://huggingface.co/zai-org), [Kimi](https://huggingface.co/moonshotai), [Falcon](https://huggingface.co/tiiuae), [Cohere](https://docs.cohere.com/docs/private-deployments) | Broader private/data-center procurement options; preserve adapters for these families and commercial private deployments | License, use conditions, redistribution, regional handling, hardware, tool/runtime formats and operational costs vary by artifact/contract. This survey is not blanket license clearance or a supported-model list. |

Pin weight/tokenizer/converter/runtime hashes, exact license/notices, tool and prompt
formats, context limits, modalities, supported structured outputs, refusal/error
semantics, cancel/streaming behavior and evaluation set at qualification. Embedding,
speech, vision or safety-specific models can be separately budgeted components;
they do not replace the default capable assistant or grant permission. Fine-tuning
and federated training are distinct authorized workloads.

## Runtime and infrastructure layers

| Layer | Candidate/source | Decision |
| --- | --- | --- |
| Local inference | [LiteRT-LM](https://github.com/google-ai-edge/LiteRT-LM), [llama.cpp](https://github.com/ggml-org/llama.cpp) | Compare exact Gemma backend/format/CPU integration at B39. Upstream desktop/mobile support is not a no-OS port; C/C++ dependencies need inventory and a bounded boundary. First-party OS integration remains Rust. |
| Earlier CPU spike | [Burn embedded inference](https://burn.dev/books/burn/advanced/no-std.html) | Retain S80 historical feasibility; not a tiny-model product requirement or an assumed Gemma backend. |
| Private model engine | [vLLM](https://docs.vllm.ai/), [SGLang](https://docs.sglang.ai/) | Remote endpoint candidates initially. Linux/PyTorch/CUDA/driver dependencies are not supplied by this OS. |
| Cluster serving | [llm-d](https://llm-d.ai/docs/architecture), [NVIDIA Dynamo](https://docs.nvidia.com/dynamo/) | Scheduling, cache/disaggregation and distributed execution are cluster concerns, separate from OS task coordination and trust federation. |
| Gateway/routing | [LiteLLM](https://docs.litellm.ai/docs/proxy/architecture) | Useful protocol/cost comparison, not proof of our reservations, issuer policies or privacy semantics. Native policy remains authoritative. |
| Interoperability | [MCP](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization), [A2A](https://a2a-protocol.org/latest/specification/), [SPIFFE federation](https://spiffe.io/docs/latest/spiffe-specs/spiffe_federation/) | Consider versioned adapters for tools/tasks/workload identities; discovery/authentication never grants resource authority. |

vLLM explicitly describes internal distributed interfaces and insecure dependency
services requiring private-network protection. Do not expose GPU collectives,
cache transport or management endpoints as a consortium trust interface. Put an
authenticated, policy-constrained service boundary in front of an appropriately
secured cluster. [vLLM security guidance](https://docs.vllm.ai/en/latest/usage/security/).

Connecting this desktop to a private engine does not establish a native server OS.
B80/M26 studies CPU/SMP, accelerator drivers and runtime, server storage/network,
fleet identity/update/recovery, scheduling, high availability, operations and costs
before proposing a separate program. Federation can work through independently
administered existing infrastructure without that program.

## Public providers, tools and terms

Public adapters must pin model/API capabilities, credential scopes, price bounds,
feature-specific retention/residency/training terms, hosted tool/search charges,
rate limits, cancellation and uncertain-effect/billing reconciliation. A website
subscription is not assumed to include an API entitlement. Browser sign-in or an
OpenAI-compatible endpoint label is not a conformance test.

Review exact provider documentation at B73, then qualify each named configuration.
Provider data controls can differ by feature/endpoint and third party; do not
treat public inference, hosted tools and private deployments as identical.
[OpenAI data controls](https://developers.openai.com/api/docs/guides/your-data),
[Anthropic documentation](https://platform.claude.com/docs/),
[Gemini API documentation](https://ai.google.dev/gemini-api/docs),
[xAI documentation](https://docs.x.ai/).

Apply source-use and onward-disclosure policy to prompts, web searches, tool
arguments, fetched URLs, embeddings, summaries and traces. Local inference can
still disclose data through tools. Publisher preference mechanisms are fragmented;
machine policy needs supported signals and reviewed rules, not a promise to
interpret every website's legal prose. [IAB workshop report](https://www.rfc-editor.org/rfc/rfc9969.html).

## Evidence needed before selection becomes support

Follow [AI roadmap](../05-delivery/ai-roadmap.md): native M21 experiment after M5;
durable personal/private/public M23 after storage/account/package/network gates;
M24 cooperation; M25 two-domain federation; M26 separate server feasibility.
Test whole-system outcomes, resource/cost limits and human interaction; advertised
model stability, capability scores, model voting and published phone results cannot
establish reliability or native feasibility. All critical authority/effect checks
must pass independently of the proposed ordinary-task success threshold.

Native integration must also pass the [large-artifact/resource contract gate](../06-contracts/package-manifest.md#ai-artifact-and-memory-profile-gate). Current 64 MiB object and 2 GiB package-memory bounds do not establish a multi-GiB model loader. B67 defines bounded shards/streaming and any versioned extension, B39 proves the guest subset, and B70 qualifies ordinary delivery without silently relaxing old limits.
