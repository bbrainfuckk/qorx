# Qorx Void benchmark evidence

The numbers on this page come from two different public proof surfaces. They
must remain attached to their hardware, data, and measurement boundary.

## AMD MI300X product run

The benchmark published on [qorx.eu.cc](https://qorx.eu.cc/#benchmark) used:

- AMD Radeon Instinct MI300X GPU with ROCm;
- GPT-OSS 120b-ROCm7;
- 192 GB VRAM, 20 vCPU, and 240 GB RAM;
- BEIR retrieval sets including SciFact, NFCorpus, FiQA, FEVER, and ArguAna;
- SQuAD 2.0, HotpotQA distractor, and Needle-in-haystack recall checks.

| Measurement | Published result |
| --- | ---: |
| Indexed context | 184,789,445 tokens |
| Average carrier | 14.0 tokens |
| Context reduction | 13,199,246.07x average |
| Local core latency | 0.8974 ms average |
| Maximum local core latency | 3.512 ms |
| Quality scorecard | 38 perfect target checks out of 52 |
| Test tracks completed | 10 out of 10 |
| Grounding gates | 1.0 pass |
| Provider calls during the local run | 0 |

The latency figure covers the measured local core path, not model inference or
network time. The 13.2M figure is context reduction for this run, not a promise
that every corpus, query, or agent will produce the same ratio. The quality
scorecard and grounding gates are reported separately because a smaller carrier
does not prove a correct answer by itself.

## Public repository run

The committed public benchmark indexed this repository and used the local
`ceil(chars / 4)` token estimate.

| Case | Indexed | Used or visible | Reduction |
| --- | ---: | ---: | ---: |
| Session carrier | 388,573 | 69 | 5,631.49x |
| Evidence pack | 388,573 | 410 | 947.74x |
| Squeeze extract | 388,573 | 448 | 867.35x |

Reproduce or inspect it from [live.md](../benchmarks/live.md) and
[live.json](../benchmarks/live.json). These are local deterministic estimates,
not provider invoice savings.

## Comparison board

Only systems and papers that publish a token-, prompt-, or context-reduction
measurement belong in the closest-reference table. Coding agents, vector
databases, protocols, and evaluation suites are listed later as adjacent work,
not competitors.

| Closest reference | Published reference metric | Why the comparison is limited |
| --- | --- | --- |
| [LLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Up to 20x prompt compression with minimal performance loss. | Learned prompt compression is not local carrier resolution. |
| [LongLLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | 4x compression, a reported 17.1% performance gain, and 1.4x–3.8x latency speedup. | The paper evaluates long-context prompting, not a persistent local index. |
| [LLMLingua-2](https://arxiv.org/abs/2403.12968) | Evaluated at 2x–5x compression and reported 3x–6x faster compression than earlier methods. | Its compression and latency boundaries differ from Qorx's local core path. |
| [Active Context Compression](https://arxiv.org/abs/2601.07190) | 22.7% token reduction on a five-task SWE-bench Lite experiment while retaining the same 3/5 resolution count. | Small-sample agent pruning, not a carrier-to-index ratio. |
| [Gist Tokens](https://arxiv.org/abs/2304.08467) | Up to 26x prompt compression, up to 40% FLOPs reduction, and 4.2% wall-clock reduction. | Learned soft tokens require model support; Qorx emits inspectable local evidence. |
| [Context Mode](https://context-mode.com/) | Up to 98% context reduction; its 50-turn example reports 30x fewer tokens. | Product examples are not a matched Qorx benchmark corpus. |
| [LeanCTX](https://leanctx.com/how-it-works/) | Illustrates about 100K tokens becoming a roughly 5K map, reports 60%–90% less context noise, and roughly 13-token re-reads. | Repository maps and cached re-reads are different output objects from a Qorx carrier. |
| [sqz](https://github.com/ojuschugh1/sqz) | Reports 24.7% average savings over 3,003 compressions, up to 92% on repeated file reads, and 13-token references. | Tool-output caching and compression are not the same operation as local evidence resolution. |
| [indxr](https://docs.rs/crate/indxr/latest) | Reports about 5x fewer tokens than full-file reads and sub-20 ms indexing for most projects. | Indexing time is not comparable to Qorx's measured query-side local core latency. |
| [Codebase-Memory](https://arxiv.org/abs/2603.27277) | Reports ten times fewer tokens and 83% answer quality versus 92% for its file-exploration agent across 31 repositories. | It measures graph-assisted code exploration; Qorx reports a different carrier and scorecard. |

Qorx's **13,199,246.07x** factor is numerically larger than the published
factors in this table, within the disclosed AMD carrier measurement. That is
the narrow result Qorx can honestly claim. The projects were not rerun on the
same inputs, hardware, task, or quality gate, so this is not a head-to-head win
and does not establish superior answer quality.

## How to read the board

- Do not divide one row by another and call the result an end-to-end speedup.
- Do not compare local core latency with model inference latency.
- Do not treat a carrier token count as equivalent to a full prompt-compression
  benchmark unless the inputs and outputs are matched.
- Re-run systems on the same corpus, hardware, query set, and quality gates
  before making a direct winner claim.
- Keep provider cost, output tokens, cache discounts, and account terms outside
  the result unless they were actually measured.

The value of the published board is transparency: it shows what each number
means and where a fairer head-to-head test still needs to be run.

## Reviewed reference landscape: 60 items

This registry records the wider field reviewed for Qorx. It is not a
leaderboard. The category labels prevent a benchmark, protocol, model-side
cache, or vector database from being presented as though it performed the same
operation as Qorx.

### Context reduction, compression, and local indexing (1–15)

| # | Reference | Scope |
| ---: | --- | --- |
| 1 | [LLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Learned prompt compression |
| 2 | [LongLLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Long-context prompt compression |
| 3 | [LLMLingua-2](https://arxiv.org/abs/2403.12968) | Task-agnostic prompt compression |
| 4 | [Active Context Compression](https://arxiv.org/abs/2601.07190) | Agent context pruning |
| 5 | [CodePromptZip](https://arxiv.org/abs/2502.14925) | Code-context compression |
| 6 | [Gist Tokens](https://arxiv.org/abs/2304.08467) | Model-side soft-token compression |
| 7 | [Context Mode](https://context-mode.com/) | Tool-output context management |
| 8 | [LeanCTX](https://leanctx.com/how-it-works/) | Repository maps and cached re-reads |
| 9 | [sqz](https://github.com/ojuschugh1/sqz) | Tool-output compression and references |
| 10 | [indxr](https://docs.rs/crate/indxr/latest) | Local code index |
| 11 | [Aider repository map](https://aider.chat/docs/repomap.html) | Budgeted repository map |
| 12 | [Repomix](https://repomix.com/guide/configuration) | Repository packing and compression |
| 13 | [gitingest](https://github.com/cyclotruc/gitingest) | Repository-to-text packaging |
| 14 | [MInference](https://arxiv.org/abs/2407.02490) | Long-context model inference |
| 15 | [QVCache](https://arxiv.org/abs/2602.02057) | Query-vector cache, not prompt compression |

### Memory and retrieval research (16–35)

| # | Reference | Scope |
| ---: | --- | --- |
| 16 | [Kortex by Arjay](https://github.com/H4D3ZS/kortex) | Early local-context inspiration; not a dependency |
| 17 | [ReACC](https://aclanthology.org/2022.acl-long.431/) | Retrieval-augmented code completion |
| 18 | [GraphCoder](https://arxiv.org/abs/2406.07003) | Graph-based repository context |
| 19 | [GraphRAG survey](https://arxiv.org/abs/2501.00309) | Graph retrieval taxonomy |
| 20 | [GraphRAG-R1](https://arxiv.org/abs/2507.23581) | Process-constrained graph retrieval |
| 21 | [MMGraphRAG](https://arxiv.org/abs/2507.20804) | Multimodal graph retrieval |
| 22 | [MegaRAG](https://arxiv.org/abs/2512.20626) | Multimodal knowledge-graph RAG |
| 23 | [M³KG-RAG](https://arxiv.org/abs/2512.20136) | Multi-hop multimodal graph RAG |
| 24 | [Codebase-Memory](https://arxiv.org/abs/2603.27277) | Persistent repository knowledge graph |
| 25 | [AtomMem](https://arxiv.org/abs/2601.08323) | Learned atomic memory operations |
| 26 | [AgeMem](https://arxiv.org/abs/2601.01885) | Unified long- and short-term agent memory |
| 27 | [Titans](https://arxiv.org/abs/2501.00663) | Model-side neural long-term memory |
| 28 | [TokMem](https://arxiv.org/abs/2510.00444) | One-token learned procedural memory |
| 29 | [Structural Memory](https://arxiv.org/abs/2412.15266) | Agent memory structures and retrieval |
| 30 | [H-MEM](https://arxiv.org/abs/2507.22925) | Hierarchical agent memory |
| 31 | [HiMem](https://arxiv.org/abs/2601.06377) | Hierarchical long-horizon memory |
| 32 | [TierMem](https://arxiv.org/abs/2602.17913) | Provenance-aware tiered memory |
| 33 | [GAM](https://arxiv.org/abs/2604.12285) | Hierarchical graph-based agent memory |
| 34 | [RAGCache](https://arxiv.org/abs/2404.12457) | RAG intermediate-state caching |
| 35 | [Cache-Craft](https://arxiv.org/abs/2502.15734) | Reusable RAG chunk KV caches |

### Evaluation sets and benchmarks (36–50)

| # | Reference | Scope |
| ---: | --- | --- |
| 36 | [BEIR](https://github.com/beir-cellar/beir) | Heterogeneous information retrieval |
| 37 | [MTEB](https://github.com/embeddings-benchmark/mteb) | Text embedding evaluation |
| 38 | [RULER](https://github.com/NVIDIA/RULER) | Long-context model evaluation |
| 39 | [LongBench](https://github.com/THUDM/LongBench) | Long-context understanding |
| 40 | [SWE-bench](https://github.com/SWE-bench/SWE-bench) | Repository issue resolution |
| 41 | [SWE-agent](https://github.com/SWE-agent/SWE-agent) | Agent system and evaluation harness |
| 42 | [CodeSearchNet](https://github.com/github/CodeSearchNet) | Semantic code retrieval |
| 43 | [Terminal-Bench](https://github.com/laude-institute/terminal-bench) | Terminal-agent task evaluation |
| 44 | [OSWorld](https://github.com/xlang-ai/OSWorld) | Computer-use agent evaluation |
| 45 | [GDPval](https://openai.com/index/gdpval/) | Economically valuable task evaluation |
| 46 | [FEVER](https://fever.ai/) | Fact extraction and verification |
| 47 | [SQuAD 2.0](https://rajpurkar.github.io/SQuAD-explorer/) | Answerable and unanswerable QA |
| 48 | [HotpotQA](https://hotpotqa.github.io/) | Multi-hop question answering |
| 49 | [Needle In A Haystack](https://github.com/gkamradt/LLMTest_NeedleInAHaystack) | Long-context recall probe |
| 50 | [RAGChecker](https://arxiv.org/abs/2408.08067) | RAG evaluation framework |

### Adjacent protocols and infrastructure (51–60)

| # | Reference | Scope |
| ---: | --- | --- |
| 51 | [Model Context Protocol](https://modelcontextprotocol.io/) | Tool and context interoperability protocol |
| 52 | [Claude Code memory](https://docs.anthropic.com/en/docs/claude-code/memory) | Agent instruction and memory files |
| 53 | [Gemini CLI `GEMINI.md`](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/gemini-md.md) | Hierarchical agent context files |
| 54 | [Cursor codebase indexing](https://docs.cursor.com/context/codebase-indexing) | Hosted codebase index |
| 55 | [LlamaIndex](https://docs.llamaindex.ai/) | Data and RAG framework |
| 56 | [Haystack](https://docs.haystack.deepset.ai/) | RAG and agent framework |
| 57 | [LangGraph](https://langchain-ai.github.io/langgraph/) | Stateful agent orchestration |
| 58 | [Pinecone](https://docs.pinecone.io/) | Managed vector database |
| 59 | [Qdrant](https://qdrant.tech/documentation/) | Vector database |
| 60 | [Weaviate](https://docs.weaviate.io/) | Vector database and retrieval platform |
