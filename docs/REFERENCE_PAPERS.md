# Qorx reference papers and external sources

This is the public source map behind the Qorx research narrative. Every entry
links to a paper, standard, official project page, or public report. A citation
supports a design direction; it does not prove a Qorx-specific benchmark or
mean that Qorx contains the cited implementation.

## Foundational research

| Reference | Qorx connection and boundary |
| --- | --- |
| [Shannon, *A Mathematical Theory of Communication* (1948)](https://doi.org/10.1002/j.1538-7305.1948.tb00917.x) | Classical information theory motivates useful information under a budget. Qorx's carrier ratio is an engineering measurement, not a new information-theoretic law. |
| [Necula, *Proof-Carrying Code* (1997)](https://doi.org/10.1145/263699.263712) | Proof-carrying systems motivate checked artifacts and explicit evidence. Qorx does not claim formal verification of arbitrary AI answers. |
| [Lewis et al., *Retrieval-Augmented Generation* (2020)](https://papers.neurips.cc/paper/2020/file/6b493230205f780e1bc26945df7481e5-Paper.pdf) | Retrieval-backed generation motivates separating stored knowledge from task-time evidence. Qorx uses a deterministic local retrieval boundary. |
| [Yao et al., *ReAct* (2022)](https://arxiv.org/abs/2210.03629) | Interleaved reasoning and tool use motivates making the local resolver an explicit agent action. Qorx does not claim to implement or replace the ReAct agent architecture. |
| [Packer et al., *MemGPT* (2023)](https://arxiv.org/abs/2310.08560) | Operating-system-style memory management is background for local long-horizon context. Qorx uses its own deterministic local state and carrier model. |
| [Liu et al., *Lost in the Middle* (2023)](https://arxiv.org/abs/2307.03172) | Long-context position effects motivate selecting task-relevant evidence instead of assuming more visible tokens are always more useful. |
| [Farhi, Goldstone, and Gutmann, *A Quantum Approximate Optimization Algorithm*](https://arxiv.org/abs/1411.4028) | A reference for quantum-inspired combinatorial selection. Qorx 1.0.6 executes classical deterministic selection and does not implement QAOA on quantum hardware. |
| [Bohigas, Giannoni, and Schmit, *Characterization of Chaotic Quantum Spectra*](https://doi.org/10.1103/PhysRevLett.52.1) | Quantum-chaos research is a conceptual influence on questions of stability and sensitivity. It is not an implemented Qorx physics model. |
| [Lorenz, *Deterministic Nonperiodic Flow* (1963)](https://doi.org/10.1175/1520-0469%281963%29020%3C0130%3ADNF%3E2.0.CO%3B2) | Nonlinear dynamics is a research lens for state and sensitivity, not a claim that Qorx simulates the Lorenz system. |
| [NIST FIPS 180-4, Secure Hash Standard](https://csrc.nist.gov/pubs/fips/180-4/upd1/final) | Cryptographic hashing is used for integrity and provenance records. Hashes detect changes; they do not hide public source. |
| [NIST FIPS 204, Module-Lattice-Based Digital Signature Standard](https://csrc.nist.gov/pubs/fips/204/final) | Post-quantum signature practice is a reference for Qorx attestation work. It does not make the runtime quantum. |

## Prompt and context reduction

| Reference | Qorx connection and boundary |
| --- | --- |
| [LLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Learned prompt compression supports shortening task input. Qorx does not bundle LLMLingua and reports a different local carrier operation. |
| [LongLLMLingua](https://arxiv.org/abs/2310.06839) | Long-context budget control is relevant to context selection. Its published ratios are not directly comparable without a matched run. |
| [LLMLingua-2](https://arxiv.org/abs/2403.12968) | Task-agnostic prompt compression is a close public metric reference. Qorx uses deterministic local evidence resolution. |
| [Active Context Compression](https://arxiv.org/abs/2601.07190) | Agent-side context pruning supports keeping only task-relevant material visible. Its SWE-bench experiment is not a Qorx head-to-head result. |
| [Gist Tokens](https://arxiv.org/abs/2304.08467) | Learned soft-token memory shows a model-side compression path. Qorx carriers remain inspectable and do not require learned gist-token support. |
| [CodePromptZip](https://arxiv.org/abs/2502.14925) | Code-context compression is relevant to repository agents. Qorx's public proof is attached to its own corpus and measurement boundary. |
| [MInference](https://arxiv.org/abs/2407.02490) | Long-context inference optimization is adjacent model-runtime work, not a local index-to-carrier comparison. |

## Repository retrieval and agent memory

| Reference | Qorx connection and boundary |
| --- | --- |
| [ReACC](https://aclanthology.org/2022.acl-long.431/) | Retrieval-augmented code completion supports bringing related code into task context. |
| [GraphCoder](https://arxiv.org/abs/2406.07003) | Graph-based code context motivates explicit repository relations. Qorx keeps its public core deterministic and local. |
| [GraphRAG survey](https://arxiv.org/abs/2501.00309) | Separates query processing, retrieval, organization, generation, and data sources. |
| [GraphRAG-R1](https://arxiv.org/abs/2507.23581) | Process-constrained graph retrieval motivates inspectable evidence paths. |
| [MMGraphRAG](https://arxiv.org/abs/2507.20804) | Multimodal graph retrieval motivates preserving modality-specific evidence paths. |
| [MegaRAG](https://arxiv.org/abs/2512.20626) | Multimodal knowledge-graph RAG is relevant to local document and visual evidence. |
| [M³KG-RAG](https://arxiv.org/abs/2512.20136) | Grounded retrieval and selective pruning are relevant to evidence budgets. |
| [Codebase-Memory](https://arxiv.org/abs/2603.27277) | Persistent repository knowledge graphs are a close code-agent memory reference. |
| [AtomMem](https://arxiv.org/abs/2601.08323) | Learned atomic memory operations are adjacent to Qorx's explicit local memory objects; the implementations differ. |
| [AgeMem](https://arxiv.org/abs/2601.01885) | Unified long- and short-term agent memory informs memory-operation design. |
| [Titans](https://arxiv.org/abs/2501.00663) | Neural long-term memory is model-side research. Qorx does not claim a Titans-like learned runtime. |
| [TokMem](https://arxiv.org/abs/2510.00444) | One-token procedural memory is a learned model technique, not the current Qorx carrier format. |
| [Structural Memory](https://arxiv.org/abs/2412.15266) | Compares memory structures and retrieval methods for agents. |
| [H-MEM](https://arxiv.org/abs/2507.22925) | Hierarchical memory supports layered retrieval. |
| [HiMem](https://arxiv.org/abs/2601.06377) | Hierarchical long-horizon memory is adjacent to Qorx's local memory layers. |
| [TierMem](https://arxiv.org/abs/2602.17913) | Provenance-aware tiered memory is close to Qorx's summary-to-raw-evidence boundary. |
| [GAM](https://arxiv.org/abs/2604.12285) | Graph-based agent memory informs relation-aware retrieval. |

## Cache and inference boundaries

| Reference | Qorx connection and boundary |
| --- | --- |
| [Preble](https://arxiv.org/abs/2407.00023) | Prefix-aware request scheduling is relevant to stable-prefix planning. Provider cache hits remain provider measurements. |
| [Similarity caching for language models](https://arxiv.org/abs/1912.03888) | Approximate reuse can reduce work but introduces correctness tradeoffs; Qorx defaults to exact replay where replay is used. |
| [RAGCache](https://arxiv.org/abs/2404.12457) | Caches intermediate states of retrieved knowledge. This is an inference-system optimization, not Qorx context omission. |
| [Cache-Craft](https://arxiv.org/abs/2502.15734) | Reuses RAG chunk KV caches while managing recomputation and quality. |
| [QVCache](https://arxiv.org/abs/2602.02057) | Query-vector caching is adjacent cache research, not prompt compression. |
| [TurboQuant](https://arxiv.org/abs/2504.19874) | Quantized KV-cache work is a model-runtime measurement problem. Qorx claims no realized runtime gain without a runtime proof. |

## Environmental accounting

| Reference | Qorx connection and boundary |
| --- | --- |
| [IEA, *Energy and AI* executive summary](https://www.iea.org/reports/energy-and-ai/executive-summary) | Establishes the broader data-centre electricity context. It does not provide a universal token-to-energy factor. |
| [Lawrence Berkeley National Laboratory, *2024 United States Data Center Energy Usage Report*](https://eta-publications.lbl.gov/sites/default/files/2024-12/us_data_center_energy_usage_report_lbnl-2001637_0.pdf) | Documents energy and direct-water boundaries for US data centres. Qorx water scenarios still require a workload-specific factor. |
| [SweetSpot: energy efficiency of LLM inference](https://arxiv.org/abs/2602.05695) | Shows that inference efficiency depends nonlinearly on input and output lengths, model, and hardware. |
| [Towards Green AI: energy of LLM inference in software development](https://arxiv.org/abs/2602.05712) | Separates input prefill from output decoding and measures model-dependent energy behaviour. |

These sources support a mechanism, not a guaranteed impact number: reducing
repeated model-bound input can reduce prefill and data-movement work in some
deployments. Actual energy, CO2e, and water depend on hardware, model, batching,
electricity, cooling, and the reporting boundary. `qorx eco` therefore performs
scenario arithmetic only from user-supplied factors.

## Official provider and tooling boundaries

| Source | Qorx boundary |
| --- | --- |
| [OpenAI prompt caching](https://platform.openai.com/docs/guides/prompt-caching) | Provider-side caching is separate from Qorx local context omission. |
| [Anthropic prompt caching](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching) | Qorx can help structure stable prefixes, but upstream cache hits must be measured by the provider. |
| [Gemini context caching](https://ai.google.dev/gemini-api/docs/caching/) | Same provider-cache boundary. |
| [Claude Code memory](https://docs.anthropic.com/en/docs/claude-code/memory) | Instruction files are useful but are not the same as budgeted local evidence retrieval. |
| [Gemini CLI `GEMINI.md`](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/gemini-md.md) | Hierarchical context files differ from a local evidence resolver. |
| [Cursor codebase indexing](https://docs.cursor.com/context/codebase-indexing) | Cursor's indexing deployment model differs from Qorx's local core. |
| [Protocol Buffers](https://protobuf.dev/) | Qorx uses typed persisted envelopes; protobuf does not prove application-level correctness. |
| [C2PA specification](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html) | Qorx provenance is local metadata, not a complete C2PA media manifest. |

## Reading the map

- Research citations belong here, not in the contributor credits.
- The closest numeric comparisons belong in the
  [Qorx Void benchmark board](void/benchmarks.md#comparison-board).
- Qorx-specific accuracy, latency, cost, and impact claims require Qorx-specific
  evidence.
- The next fair comparison is a matched-corpus, matched-task, matched-quality
  rerun with provider traffic and energy measured at the same boundary.
