# Qorx

Qorx is a programming language and local runtime for humans and AI agents.

Qorx Void gives Codex users local project memory. It keeps repeated workspace context on the user's machine, resolves the current task against local quarks, and sends a compact proof frame when Codex needs it.

This public repository is documentation only. It publishes the product story, benchmark records, research references, credits, citation metadata, and IP boundaries. It does not publish Qorx source, runnable product packages, private technical material, or distribution packages.

## What Qorx Does

Codex is powerful, but large projects make it reread the same context over and over. Qorx Void turns repeated local context into a small carrier for the next turn, so the user can keep working from the AI plan they already have before moving to a higher plan.

The public product is Qorx Void. The installed product is the Qorx Void app.

## Qorx Void Documentation

Qorx Void is documented directly in this main repository so reviewers, testers, and public technical readers can understand the product without receiving private implementation material.

- [Qorx Void Handbook](docs/void/README.md): the deeper public guide for Qorx Void.
- [Architecture](docs/void/architecture.md): the public system model and host boundary.
- [Day-To-Day Use](docs/void/day-to-day-use.md): how operators use Void and how testers use `qorx-free`.
- [qorx-free](docs/void/qorx-free.md): the Linux AMD MI300X public benchmarker.
- [Security Model](docs/void/security-model.md): what is visible, what stays private, and why.
- [Release Boundary](docs/void/release-boundary.md): what GitHub can publish and what must not be shipped.

These docs explain product behavior, operator workflows, benchmark methodology, release boundaries, and security expectations. They do not publish source, private implementation material, sensitive operational details, private data, or build and release procedures.

## AMD Benchmark

Measured machine:

| Hardware | Spec |
| --- | --- |
| Accelerator | AMD Radeon Instinct™ MI300X GPU ROCm™ enabled GPT-OSS 120b-ROCm7 |
| Short name | AMD MI300X |
| VRAM | 192 GB |
| CPU | 20 vCPU |
| RAM | 240 GB |

Measured Qorx Context Reduction result:

| Metric | Value |
| --- | ---: |
| Predeclared target | >= 12,500,000x |
| Measured average reduction | 13,199,246.07x |
| Indexed tokens | 184,789,445 |
| Average carrier | 14.0 tokens |
| Minimum quarks used | 2 |
| Average core latency | 0.8974 ms |
| Max core latency | 3.512 ms |
| Provider calls | 0 |

The companion quality scorecard covers BEIR retrieval, FEVER, SQuAD 2.0, HotpotQA, Needle-in-haystack recall, grounding, and refusal behavior: 38 perfect checks across 52 rows.

Read the benchmark notes in [docs/benchmarks.md](docs/benchmarks.md).

## 50 Major Context, Agent, And Benchmark Comparisons

This is the public comparison map for Qorx: 50 major systems, papers, benchmarks, and tools from big tech, universities, research labs, and serious engineering products. The table keeps each system in its own lane. A coding-agent benchmark score, a vector database limit, and a prompt-compression ratio are not the same metric.

Qorx's measured lane is local context transit: 13,199,246.07x measured average context reduction, 184,789,445 indexed tokens, 14.0 average carrier tokens, 0.8974 ms average core latency, 3.512 ms max core latency, and 0 provider calls in the AMD MI300X local run.

| # | System / work | Institution / org | Public metric or scale | Qorx comparison lane |
| ---: | --- | --- | --- | --- |
| 1 | [LLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Microsoft Research | Up to 20x prompt compression with little performance loss. | Qorx: 13,199,246.07x context reduction. |
| 2 | [LongLLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Microsoft Research | 4x compression; 17.1% performance gain; 1.4x-3.8x latency speedup. | Qorx: 13.2M x reduction and 0.8974 ms average core latency. |
| 3 | [LLMLingua-2](https://arxiv.org/abs/2403.12968) | Microsoft Research and collaborators | 2x-5x compression; 1.6x-2.9x end-to-end latency acceleration; 3x-6x faster than earlier LLMLingua methods. | Qorx: 184,789,445 indexed tokens to 14.0 average carrier tokens. |
| 4 | [MInference](https://www.microsoft.com/en-us/research/publication/minference-1-0-accelerating-pre-filling-for-long-context-llms-via-dynamic-sparse-attention/) | Microsoft Research and University of Surrey | Up to 10x prefill speedup on A100 for long-context inference while maintaining accuracy. | Qorx compares before inference: smaller model-bound context, not an attention kernel. |
| 5 | [ReadAgent](https://deepmind.google/research/publications/74917/) | Google DeepMind | Effective context length extended by 3.5x-20x across QuALITY, NarrativeQA, and QMSum. | Qorx compares as local evidence-carrier state, not document gist memory. |
| 6 | [RULER](https://github.com/NVIDIA/RULER) | NVIDIA | 17 open-source models, 4 task categories, 13 long-context tasks. | Qorx reports local scorecard rows separately; no RULER submission claim. |
| 7 | [NeMo Retriever](https://docs.nvidia.com/nemo/retriever/) | NVIDIA | Indexing and querying microservices for extraction, embedding, and reranking pipelines. | Qorx's measured position is before retrieval serving: compact repeated local context transit. |
| 8 | [BEIR](https://github.com/beir-cellar/beir) | TU Darmstadt, University of Waterloo, and collaborators | 18 retrieval datasets across diverse domains and tasks. | Qorx reports BEIR-style local scorecard rows separately. |
| 9 | [MTEB](https://huggingface.co/spaces/mteb/leaderboard) | Hugging Face and embedding benchmark community | Public embedding leaderboard across many tasks, datasets, and languages. | Qorx is not an embedding leaderboard entry; it reduces local context transit. |
| 10 | [DSPy](https://github.com/stanfordnlp/dspy) | Stanford NLP | Example RAG optimization improves a StackExchange subset from 53% to 61% in DSPy docs. | Qorx can wrap a DSPy program with smaller local context. |
| 11 | [SWE-bench](https://www.swebench.com/) | Stanford and Princeton | Real GitHub issue benchmark; SWE-bench Verified has 500 engineer-confirmed solvable problems. | Qorx should be tested as a context layer around an agent. |
| 12 | [SWE-agent](https://swe-agent.com/0.7/usage/benchmarking/) | Stanford and Princeton ecosystem | Public harness for SWE-bench-style software-engineering evaluation; scores depend on model and scaffold. | Qorx can be measured in Qorx-on/off SWE-agent runs. |
| 13 | [LoCoBench-Agent](https://www.salesforce.com/blog/locobench-agent/) | Salesforce AI Research | 8,000 long-context coding scenarios, 10 languages, 10K-1M token bands, and up to 50 turns. | Qorx targets repeated repo-context transit in long coding sessions. |
| 14 | [OpenAI Codex](https://openai.com/index/introducing-gpt-5-3-codex/) | OpenAI | GPT-5.3-Codex xhigh: 56.8% SWE-Bench Pro Public, 77.3% Terminal-Bench 2.0, 64.7% OSWorld-Verified, 70.9% GDPval favorable/tied evaluations, 77.6% CTF, 81.4% SWE-Lancer IC Diamond. | Qorx reduces and grounds context around Codex; it does not replace Codex. |
| 15 | [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) | Anthropic | Claude 3.5 Sonnet scaffold reached 49% SWE-bench Verified in Anthropic's report. | Qorx can provide smaller local proof context around Claude workflows. |
| 16 | [Model Context Protocol](https://www.anthropic.com/news/model-context-protocol) | Anthropic, Linux Foundation Agentic AI Foundation | Public update cites 10,000+ active public MCP servers and 97M+ monthly Python/TypeScript SDK downloads. | Qorx can provide compact local state through or beside MCP-style integrations. |
| 17 | [Amazon Q Developer](https://aws.amazon.com/about-aws/whats-new/2025/04/amazon-q-developer-releases-state-art-agent-feature-development/) | Amazon Web Services | Public AWS update cites 66% SWE-bench Verified and 49% SWTBench Verified. | Qorx compares as a context layer, not as an AWS coding agent. |
| 18 | [Augment SWE-bench Agent](https://github.com/augmentcode/augment-swebench-agent) | Augment Code | Public agent reports 65.4% on first SWE-bench Verified submission. | Qorx should be tested around the same agent loop as local context support. |
| 19 | [Cursor Composer 2](https://cursor.com/blog/composer-2) | Cursor | Public blog cites CursorBench 61.3, Terminal-Bench 2.0 61.7, and SWE-bench Multilingual 73.7. | Qorx compares on repeated context transit around coding work. |
| 20 | [Devin](https://cognition.ai/blog/swe-bench-technical-report) | Cognition | 13.86% on original SWE-bench in Cognition's technical report. | Qorx does not claim autonomous-agent replacement; it compares as local context support. |
| 21 | [OpenHands CodeAct 2.1](https://openhands.dev/blog/openhands-codeact-21-an-open-state-of-the-art-software-development-agent) | All Hands AI / OpenHands | 53% SWE-bench Verified and 41.7% SWE-bench Lite. | Qorx should be measured as memory/context support around the agent. |
| 22 | [Refact.ai Agent](https://refact.ai/blog/2025/open-source-sota-on-swe-bench-verified-refact-ai/) | Refact.ai | 70.4% SWE-bench Verified; 352 / 500 tasks solved. | Qorx needs Qorx-on/off task artifacts before making agent-score claims. |
| 23 | [Context Mode](https://context-mode.com/) | Context Mode | 315 KB to 5.4 KB, 98%; example reports 30x fewer tokens over 50 turns. | Qorx: 13.2M x context reduction with 0 provider calls in the local run. |
| 24 | [LeanCTX](https://leanctx.com/features) | LeanCTX | Sample project 145.2K raw to 7.5K map tokens, 98.5% saved; file reads up to 99%. | Qorx: 184.8M indexed tokens to 14.0 average carrier tokens. |
| 25 | [sqz CLI](https://docs.rs/crate/sqz-cli/1.0.4) | sqz | 24.7% average reduction; 92% saved on repeated file reads; 13-token cached refs. | Qorx: 14.0 average carrier tokens with minimum 2 quarks. |
| 26 | [indxr](https://docs.rs/crate/indxr/0.2.0) | indxr | About 5x reduction vs full file reads; sub-20 ms indexing for most projects. | Qorx: 0.8974 ms average core latency in the AMD run. |
| 27 | [Aider Repo Map](https://aider.chat/docs/repomap.html) | Aider | 1,024 default map tokens. | Qorx: 14.0 average carrier tokens. |
| 28 | [Repomix](https://repomix.com/guide/configuration) | Repomix | Token counting with `o200k_base`; optional Tree-sitter code compression and file summary output. | Qorx's measured comparison is carrier size for repeated local context. |
| 29 | [gitingest](https://github.com/coderamp-labs/gitingest) | gitingest | Reports file structure, extract size, and token count for prompt-friendly repository extraction. | Qorx's measured comparison is tiny repeated workspace state instead of prompt-ready repository extraction. |
| 30 | [Cline Bench](https://cline.bot/blog/cline-bench-initiative) | Cline | Public benchmark initiative; public page reports large community signal and benchmark-building work. | Qorx needs Cline-on/off tests for agent-score claims; its current measured claim is local carrier size. |
| 31 | [Terminal-Bench](https://www.tbench.ai/) | Terminal-Bench team | Terminal-task benchmark used in public coding-agent reports. | Qorx should be measured as local context support during terminal-agent work. |
| 32 | [OSWorld](https://os-world.github.io/) | OSWorld benchmark team | Desktop computer-use benchmark; OpenAI reports 64.7% OSWorld-Verified for GPT-5.3-Codex xhigh. | Qorx does not claim OSWorld completion; it reduces context around agent work. |
| 33 | [GDPval](https://openai.com/index/gdpval/) | OpenAI | Knowledge-work benchmark covering 44 occupations. | Qorx is adjacent: local project memory for knowledge-work agents. |
| 34 | [SWE-Lancer](https://openai.com/index/introducing-gpt-5-3-codex/) | SWE-Lancer / OpenAI-reported evaluation | OpenAI reports 81.4% SWE-Lancer IC Diamond for GPT-5.3-Codex xhigh. | Qorx should be tested as a context layer around the same class of tasks. |
| 35 | [FEVER](https://github.com/sheffieldnlp/fever-scorer) | Sheffield NLP and collaborators | Fact verification benchmark and scorer used in Qorx quality scorecard lanes. | Qorx reports local FEVER-style checks, not an official leaderboard submission. |
| 36 | [SQuAD 2.0](https://github.com/rajpurkar/SQuAD-explorer) | Stanford | Reading-comprehension and unanswerable-question benchmark used in Qorx scorecard lanes. | Qorx reports local SQuAD-style checks, not an official leaderboard submission. |
| 37 | [HotpotQA](https://github.com/hotpotqa/hotpot) | CMU, Stanford, and Mila | Multi-hop question-answering benchmark used in Qorx support-coverage lanes. | Qorx reports local HotpotQA-style checks, not an official leaderboard submission. |
| 38 | [Needle In A Haystack](https://github.com/gkamradt/LLMTest_NeedleInAHaystack) | Greg Kamradt / community benchmark | Long-context recall stress test used broadly for retrieval and context checks. | Qorx reports local needle recall lanes in the AMD scorecard. |
| 39 | [Pinecone](https://docs.pinecone.io/docs/limits) | Pinecone | Public limits include max `top_k` 10,000, max query result size 4 MB, and fetch/delete max 1,000 IDs. | Qorx is not a vector DB; it reduces local input before retrieval or vector serving. |
| 40 | [Portkey](https://portkey.ai/blog/the-most-reliable-ai-gateway-for-production-systems/) | Portkey | Public vendor post cites 10B+ LLM requests/month, 99.9999% uptime, and sub-10 ms latency. | Qorx sits before gateway routing as context reduction. |
| 41 | [LiteLLM proxy](https://docs.litellm.ai/) | LiteLLM / BerriAI | Public docs cover 100+ LLMs; model database lists many providers and models. | Qorx can reduce repeated local context before the proxy call. |
| 42 | [Langfuse](https://langfuse.com/docs/metrics/overview/) | Langfuse | Tracks cost, latency, quality, volume, prompt metrics, and scores. | Qorx reduces and grounds input before observability. |
| 43 | [LangSmith](https://docs.smith.langchain.com/) | LangChain | Tracing, evaluation, and observability workflow for LLM apps. | Qorx reduces local context before traces and evals observe the call. |
| 44 | [LangGraph](https://docs.langchain.com/oss/python/langgraph/durable-execution) | LangChain | Durable execution persists workflow state and supports resume after interruptions. | Qorx compares as compact local context continuity. |
| 45 | [LlamaIndex](https://docs.llamaindex.ai/en/stable/api_reference/callbacks/token_counter/) | LlamaIndex | TokenCountingHandler and cost-analysis APIs count LLM token usage. | Qorx reduces pre-model local context transit. |
| 46 | [Haystack](https://docs.haystack.deepset.ai/docs/pipelines) | deepset | Pipeline framework with typed components and DAG-style workflows. | Qorx is adjacent context memory, not a RAG pipeline replacement. |
| 47 | [Weaviate](https://docs.weaviate.io/weaviate/benchmarks) | Weaviate | Public benchmark pages and vector-index documentation. | Qorx is not a vector DB; it reduces local context before retrieval. |
| 48 | [Qdrant](https://qdrant.tech/documentation/) | Qdrant | Vector search engine and database documentation. | Qorx's measured position is local carrier size, not vector serving. |
| 49 | [Milvus / Zilliz](https://milvus.io/) | Milvus, Zilliz, and LF AI ecosystem | Large-scale vector similarity search system and ecosystem. | Qorx compares before vector storage, as a local context carrier. |
| 50 | [Elasticsearch vector search](https://www.elastic.co/elasticsearch/vector-database) | Elastic | Search and vector database product surface. | Qorx compares on repeated local context transit, not search infrastructure. |

Footer: important adjacent systems that do not have a clean same-lane public context-reduction metric in this README. Qorx's position here is narrow and specific: measured local workspace context transit, not ownership of their product category.

| Adjacent system | Institution / org | Where Qorx is positioned today |
| --- | --- | --- |
| [SWE-agent](https://swe-agent.com/0.7/usage/benchmarking/) | Stanford and Princeton ecosystem | Qorx's measured lane is carrier size: 184,789,445 indexed local tokens carried as a 14.0-token average proof frame in the AMD run. |
| [NeMo Retriever](https://docs.nvidia.com/nemo/retriever/) | NVIDIA | Qorx is positioned before retrieval serving: repeated local workspace context is reduced before a retriever or model sees it. |
| [Retrieval-Augmented Generation](https://ai.meta.com/research/publications/retrieval-augmented-generation-for-knowledge-intensive-nlp-tasks/) | Meta AI, University College London, and New York University | Qorx is positioned in local code/workspace context transit with zero provider calls and sub-4 ms core latency in the AMD run. |
| [Repomix](https://repomix.com/guide/configuration) | Repomix | Qorx is positioned around repeated workspace state when that state would otherwise be resent as a large prompt. |
| [gitingest](https://github.com/coderamp-labs/gitingest) | gitingest | Qorx is positioned around compact repeat carriers rather than full prompt-ready repository extraction. |
| [LangGraph](https://docs.langchain.com/oss/python/langgraph/durable-execution) | LangChain | Qorx's measured lane is local proof-frame size; LangGraph is the workflow-resume layer. |
| [LlamaIndex](https://docs.llamaindex.ai/en/stable/api_reference/callbacks/token_counter/) | LlamaIndex | Qorx is positioned before token counting: it reduces the repeated input being counted. |
| [Haystack](https://docs.haystack.deepset.ai/docs/pipelines) | deepset | Qorx is positioned on repeated local context transit; Haystack remains the RAG pipeline layer. |
| [Pinecone](https://docs.pinecone.io/docs/limits) | Pinecone | Qorx is positioned before vector serving: smaller local context reaches the retrieval stack. |
| [Weaviate](https://docs.weaviate.io/weaviate/benchmarks) | Weaviate | Qorx's measured lane is local carrier size, not vector-index serving. |
| [Qdrant](https://qdrant.tech/documentation/) | Qdrant | Qorx is positioned around compact workspace context before vector lookup. |
| [Milvus / Zilliz](https://milvus.io/) | Milvus, Zilliz, and LF AI ecosystem | Qorx is positioned before large-scale vector search when the recurring input is local workspace context. |
| [Chroma](https://www.trychroma.com/) | Chroma | Qorx is positioned around repeated local context transit, not vector collection management. |
| [Vespa](https://vespa.ai/) | Vespa | Qorx is positioned before search serving: tiny local proof carriers instead of repeated workspace payloads. |
| [Elasticsearch vector search](https://www.elastic.co/elasticsearch/vector-database) | Elastic | Qorx is positioned on pre-search local context reduction, not search infrastructure. |
| [Langfuse](https://langfuse.com/docs/metrics/overview/) | Langfuse | Qorx is positioned before observability: it reduces the input that would be traced. |
| [LangSmith](https://docs.smith.langchain.com/) | LangChain | Qorx is positioned on local context transit; LangSmith traces and evaluates after the call. |
| [Helicone](https://www.helicone.ai/) | Helicone | Qorx is positioned before request logging: less repeated context enters the gateway. |
| [Portkey](https://portkey.ai/blog/the-most-reliable-ai-gateway-for-production-systems/) | Portkey | Qorx is positioned before routing: repeated local input is reduced before provider traffic. |
| [LiteLLM proxy](https://docs.litellm.ai/) | LiteLLM / BerriAI | Qorx is positioned before proxy normalization: smaller repeated local context is sent into the proxy. |

Reference names are clickable public references only. They are not source, bundled code, dependency, ownership, endorsement, or implementation claims.

## Public Docs

- [Qorx Void Handbook](docs/void/README.md): deeper public docs for Void architecture, usage, `qorx-free`, security, and release boundaries.
- [Technology](docs/technology.md): Qorx, Qorx Void, quarks, carriers, and proof records.
- [Benchmarks](docs/benchmarks.md): AMD MI300X measurements, scorecard rows, and test sources.
- [Research](docs/research.md): public research areas and external references.
- [Security And Boundaries](docs/security-and-boundaries.md): what is public, what stays private, and how source protection is handled.
- [Review Brief](docs/review-brief.md): short reviewer-facing summary.
- [Media](docs/media.md): public naming, hardware label, and citation copy.

## Citation

If you cite Qorx, use [CITATION.cff](CITATION.cff).

Primary author: Marvin Sarreal Villanueva. ORCID: https://orcid.org/0009-0001-2017-5508.

Project DOI: https://doi.org/10.5281/zenodo.19875352

Preferred technical-report DOI: https://doi.org/10.5281/zenodo.19953308

## License And Source Boundary

Copyright (c) 2026 Marvin Sarreal Villanueva. All rights reserved.

This branch is a documentation-only public surface. No license is granted to copy, modify, distribute, compile, decompile, package, mirror, or create derivative works from Qorx source, unpublished implementation material, private artifacts, brand assets, or product packaging.

See [LICENSE](LICENSE), [NOTICE](NOTICE), and [TRADEMARKS.md](TRADEMARKS.md).

## Credits And GitHub Links

Qorx creator and repository owner:

- Marvin Sarreal Villanueva: https://github.com/bbrainfuckk
- ORCID author record: https://orcid.org/0009-0001-2017-5508
- Public site: https://qorx.eu.cc

Special thanks:

- Arjay, whose Kortex work helped shape the local-context direction behind Qorx: https://github.com/H4D3ZS/kortex

This is an attribution, not a dependency. Qorx is independently authored. It uses its own language, compiler/runtime design, quark and carrier model, benchmark record, product architecture, and implementation. This repository does not copy, import, redistribute, or package Kortex source code.

Benchmark and evaluation references:

- BEIR: https://github.com/beir-cellar/beir
- SQuAD Explorer / SQuAD 2.0: https://github.com/rajpurkar/SQuAD-explorer
- FEVER scorer: https://github.com/sheffieldnlp/fever-scorer
- HotpotQA: https://github.com/hotpotqa/hotpot
- Needle In A Haystack: https://github.com/gkamradt/LLMTest_NeedleInAHaystack

Language and systems references credited as inspiration or background reading:

- Zig: https://github.com/ziglang/zig
- Rust: https://github.com/rust-lang/rust
- LLVM: https://github.com/llvm/llvm-project
- TinyCC: https://github.com/TinyCC/tinycc
- Tree-sitter: https://github.com/tree-sitter/tree-sitter
- Protocol Buffers: https://github.com/protocolbuffers/protobuf
- Wasmtime / Cranelift: https://github.com/bytecodealliance/wasmtime
- TempleOS historical reference: https://github.com/cia-foundation/TempleOS
- ZealOS historical reference: https://github.com/Zeal-Operating-System/ZealOS

Agent and local-context ecosystem references:

- Gemini CLI context-file reference: https://github.com/google-gemini/gemini-cli
- Aider: https://github.com/Aider-AI/aider

Public documentation credits the projects above for benchmark sources, language references, compiler/runtime background, and agent-context comparisons. Qorx implementation code remains private.
