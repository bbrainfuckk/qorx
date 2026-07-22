# Qorx 1.0.6

**An agnostic programming language for humans and AI agents.**

AI tools get expensive when they keep receiving the same repo, notes, logs,
policies, and project rules. Qorx keeps that repeated context local, sends a
small carrier, and pulls the exact local lines only when a task needs them.

Qorx gives that workflow a language: plain `.qorx` source, checked bytecode,
local evidence, explicit budgets, and refusal when the available evidence does
not support an answer. It is provider-agnostic and runs on Windows, Linux, and
macOS across x64 and ARM64.

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19875352.svg)](https://doi.org/10.5281/zenodo.19875352)
[![Preprint DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19953308.svg)](https://doi.org/10.5281/zenodo.19953308)
[![Software Heritage](https://img.shields.io/badge/Software%20Heritage-archived-ff6600)](https://archive.softwareheritage.org/browse/origin/directory/?origin_url=https://github.com/bbrainfuckk/qorx)
[![License: AGPL-3.0-only](https://img.shields.io/github/license/bbrainfuckk/qorx?color=blue)](LICENSE)
[![Rust stable](https://img.shields.io/badge/rust-stable-orange?logo=rust)](https://www.rust-lang.org/)

```text
repo + notes + rules + logs
            |
            v
      Qorx stays local
            |
            v
 small carrier + exact cited lines
            |
            v
     human or AI agent
```

## The product family

| Name | What it is |
| --- | --- |
| **Qorx** | The open language, compiler, portable bytecode runtime, and local evidence tools in this repository. |
| **Qorx Void** | The private local project-memory product. This repository publishes [Void documentation](docs/void/README.md), not Void source or proprietary internals. |
| **Qorx Zero** | Four clean-room hackathon applications that demonstrate bounded local evidence, memory, and routing without using private Qorx Void code. |

## The language

Qorx source is meant to be readable by a person and deterministic enough for an
agent to generate, check, compile, and inspect.

```qorx
QORX 1
use std.evidence
use std.branch as br

let question = "which files define the release boundary?"
let fallback = "local evidence does not support this answer"

pack evidence from question budget 700
cache evidence key question ttl 3600
strict answer from evidence limit 2
if supported(answer) then emit answer else emit fallback
```

```sh
qorx qorx-check goal.qorx
qorx qorx-compile goal.qorx --out goal.qorxb
qorx qorx goal.qorxb
qorx qorx-inspect goal.qorxb
```

The implementation path is explicit:

1. The parser and semantic checker validate `.qorx` source.
2. The compiler lowers the source through an AST and QIR.
3. The compiler emits integrity-checked `.qorxb` bytecode.
4. The local runtime executes supported instructions and resolves cited evidence.
5. Unsupported claims can be refused instead of filled with invented context.

The 1.0.6 compiler is bootstrapped in Rust. Qorx is not described as
self-hosted until the stage-1 and stage-2 reproducibility gate passes. See
[Self-hosting](docs/SELF_HOSTING.md).

## Quick start

Install from the 1.0.6 source tag:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --tag v1.0.6 --locked qorx
qorx --version
```

Then start a local workflow:

```sh
qorx doctor
qorx daemon start
qorx index .
qorx map "which files control authentication?"
qorx strict-answer "which files control authentication?"
qorx eco --local-tokens 13200000 --sent-tokens 8
```

The local monitor listens at `http://127.0.0.1:47187/monitor` by default.

Package recipes exist for Cargo, npm, PyPI, Arch/AUR, Homebrew, Debian, RPM,
Snap, Scoop, WinGet, Nix, and container workflows. Treat a registry channel as
live only when its public package page shows `1.0.6`.

## Qorx Void

Qorx Void keeps project memory beside the work. Repositories, notes,
instructions, logs, policies, and prior decisions remain available locally;
the current task receives a compact proof-shaped frame and can request narrower
source lines when needed.

The public handbook documents:

- the user-visible workflow and local security boundary;
- all public CLI and MCP tools used by Void operators;
- supported agent integrations;
- the measured AMD MI300X results and comparison board;
- exactly what is and is not published in this repository.

It does **not** publish Qorx Void source, private algorithms, prompts, signing
operations, account infrastructure, release procedures, or proprietary
implementation details.

Start with the [Qorx Void handbook](docs/void/README.md), then see the
[complete tool surface](docs/void/tools.md) and [benchmark evidence](docs/void/benchmarks.md).

## Measured evidence

The product benchmark published on [qorx.eu.cc](https://qorx.eu.cc/#benchmark)
used an AMD Radeon Instinct MI300X GPU with ROCm and GPT-OSS 120b-ROCm7 on a
machine with 192 GB VRAM, 20 vCPU, and 240 GB RAM.

| Measurement | Result |
| --- | ---: |
| Indexed context | 184,789,445 tokens |
| Average carrier | 14.0 tokens |
| Context reduction | 13,199,246.07x average |
| Local core latency | 0.8974 ms average, 3.512 ms maximum |
| Quality scorecard | 38 perfect target checks out of 52 |
| Grounding gates | 1.0 pass |
| Provider calls during the local run | 0 |

These measurements belong to that disclosed run. Core latency is not model
inference latency, context reduction is not a universal answer-quality score,
and the result does not guarantee that an outside model cannot hallucinate.

The separate [compact-kernel contract](docs/KERNEL_1_0_6.md) records six native
x64/ARM64 CI artifacts from 0.21 to 0.39 MiB. Those are compact offline-kernel
artifacts, not the full Windows x64 CLI; the local 1.0.6 release build of that
CLI measured 4.65 MiB.

The smaller public-repository benchmark is reproducible from committed data:

| Case | Local context | Model-visible | Reduction |
| --- | ---: | ---: | ---: |
| Session carrier | 388,573 tokens | 69 tokens | 5,631.49x |
| Evidence pack | 388,573 tokens | 410 tokens | 947.74x |
| Squeeze extract | 388,573 tokens | 448 tokens | 867.35x |

See [live benchmark notes](docs/benchmarks/live.md), [benchmark JSON](docs/benchmarks/live.json),
and the [scoped comparison with other context systems](docs/void/benchmarks.md#comparison-board).

## Qorx Zero at four hackathons

Each Qorx Zero edition is an independently runnable clean-room application. It
demonstrates some combination of local records, capped proof frames, expiry and
forgetting, inspectable source hashes, and deterministic local-first routing.

Qorx is a one-man team led by its creator, **Marvin Sarreal Villanueva**.

| Hackathon edition | What it demonstrated | Links |
| --- | --- | --- |
| **NamasteDev Hackathon** | Device-local IndexedDB memory, bounded recall, and an OpenAI Responses API adapter. | [Repository](https://github.com/bbrainfuckk/qorx-zero-namaste) · [Live demo](https://bbrainfuckk.github.io/qorx-zero-namaste/) · [Video](https://youtu.be/NjWIGFTAFok) |
| **Qwen Cloud Global AI Hackathon, Track 1: MemoryAgent** | Persistent local memory, TTL and forgetting, proof-supported learning, and a Qwen Cloud adapter through Alibaba Cloud Model Studio. | [Repository](https://github.com/bbrainfuckk/qorx-zero-qwen-memory) |
| **OpenAI Build Week** | Local proof frames, visible recall scores, Codex build evidence, and a GPT-5.6 Terra Responses API adapter. | [Repository](https://github.com/bbrainfuckk/qorx-zero-build-week) · [Live demo](https://qorx-zero-build-week.omniscius.workers.dev) · [Video](https://youtu.be/GBPWgpuye-Q) |
| **AMD Developer Hackathon: ACT II, T1 General-Purpose AI Agent** | Ranked **106** on the AMD automated scoring leaderboard with **78.9% accuracy** and **9,382 tokens**. Submitted July 13, 2026 at 04:57 GMT+8 and scored at 11:44 GMT+8. The event recorded 20,728 participants across 4,892 teams and was marked as judging in progress when this result was documented. | [Submission](https://lablab.ai/submissions/d043jqk70ns3skwd28sjpyt4) · [Leaderboard](https://lablab.ai/ai-hackathons/amd-developer-hackathon-act-ii/live) · [Event](https://lablab.ai/ai-hackathons/amd-developer-hackathon-act-ii) |

The Qorx Zero repositories do not contain or depend on private Qorx Void
source, compiler internals, binaries, or private datasets.

## Environmental accounting

`qorx eco` reports supplied token counts and reduction arithmetic locally. It
only calculates energy, CO2e, or water scenarios when you provide factors for
the hardware, workload, electricity source, cooling system, and reporting
boundary.

```sh
qorx eco --local-tokens 13200000 --sent-tokens 8
```

The command makes no network call and does not invent a universal
tokens-to-impact conversion. Its stable output contract is
[`qorx.eco.v1`](schemas/qorx.eco.v1.schema.json).

## Build and verify

```sh
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo build --release --locked
```

Useful proof commands:

```sh
qorx doctor --json
qorx index .
qorx session
qorx context verify
qorx security attest
qorx ground "release proof" --answer "Qorx is on 1.0.6."
```

## Documentation

| Area | Start here |
| --- | --- |
| Language and compiler | [Language handbook](docs/handbook/language.md) · [Self-hosting boundary](docs/SELF_HOSTING.md) |
| Qorx Void | [Void handbook](docs/void/README.md) · [Tools](docs/void/tools.md) · [Benchmarks](docs/void/benchmarks.md) |
| Installation and commands | [Install](docs/INSTALL.md) · [Command guide](docs/COMMANDS.md) · [Manual](docs/MANUAL.md) |
| Metrics and science | [Metrics](docs/METRICS.md) · [Science](docs/SCIENCE.md) · [SAFE-R](docs/SAFE-R.md) · [Technical credibility](docs/TECHNICAL_CREDIBILITY.md) |
| Security and production | [Security](SECURITY.md) · [Production status](docs/PRODUCTION.md) · [Void release boundary](docs/void/release-boundary.md) |
| Release | [Qorx 1.0.6 notes](docs/releases/v1.0.6.md) · [Distribution](docs/DISTRIBUTION.md) |

## Credits and acknowledgements

Qorx was created and is maintained by **Marvin Sarreal Villanueva** as a
one-man team. If you use or cite it, see [CITATION.cff](CITATION.cff) and the
[Qorx Local Context Resolution preprint](https://doi.org/10.5281/zenodo.19953308).

Qorx acknowledges the following 50 public projects, systems, standards, and
research references. They are background reading, influence, or comparison
points, not bundled dependencies. Inclusion does not imply copied code,
affiliation, or endorsement. Qorx has its own language, compiler/runtime
design, product architecture, and implementation.

| # | Project or reference | Area |
| ---: | --- | --- |
| 1 | [TempleOS](https://templeos.org/) | Language and small-system design |
| 2 | [HolyC](https://templeos.org/Wb/Doc/HolyC.html) | Language and compiler design |
| 3 | [ZealOS](https://github.com/Zeal-Operating-System/ZealOS) | Living historical systems reference |
| 4 | [C](https://www.open-std.org/jtc1/sc22/wg14/) | Portable systems language |
| 5 | [Zig](https://ziglang.org/) | Systems language and toolchain |
| 6 | [Rust](https://www.rust-lang.org/) | Memory-safe systems language |
| 7 | [Python](https://www.python.org/) | Scripting and interoperability |
| 8 | [TinyCC / TCC](https://bellard.org/tcc/) | Small compiler baseline |
| 9 | [QBE](https://c9x.me/compile/) | Compact compiler backend |
| 10 | [Cranelift](https://cranelift.dev/) | Code generation and runtime compilation |
| 11 | [LLVM / Clang](https://llvm.org/) | Industrial compiler architecture |
| 12 | [GNU Mes](https://www.gnu.org/software/mes/) | Auditable bootstrapping |
| 13 | [Bootstrappable Builds](https://bootstrappable.org/) | Reproducible compiler bootstrap work |
| 14 | [LLVM Kaleidoscope](https://llvm.org/docs/tutorial/) | Compiler construction reference |
| 15 | [WebAssembly](https://webassembly.org/specs/) | Portable validated execution |
| 16 | [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) | Parsing and editor tooling |
| 17 | [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) | Editor interoperability |
| 18 | [Protocol Buffers](https://protobuf.dev/) | Structured messages and schemas |
| 19 | [SHA-256 / FIPS 180-4](https://csrc.nist.gov/pubs/fips/180-4/upd1/final) | Content integrity |
| 20 | [C2PA](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html) | Content provenance |
| 21 | [NIST FIPS 204](https://csrc.nist.gov/pubs/fips/204/final) | Post-quantum signatures |
| 22 | [Kortex by Arjay](https://github.com/H4D3ZS/kortex) | Early local-context direction |
| 23 | [Graphify](https://github.com/safishamsi/graphify) | Codebase graph representation |
| 24 | [ReACC](https://aclanthology.org/2022.acl-long.431/) | Retrieval-augmented code completion |
| 25 | [CodePromptZip](https://arxiv.org/abs/2502.14925) | Code-context compression |
| 26 | [Stingy Context](https://arxiv.org/abs/2601.19929) | Context selection under tight budgets |
| 27 | [Codebase-Memory](https://arxiv.org/abs/2603.27277) | Repository memory for agents |
| 28 | [AtomMem](https://arxiv.org/abs/2601.08323) | Agent memory architecture |
| 29 | [AgeMem](https://arxiv.org/abs/2601.01885) | Long-horizon agent memory |
| 30 | [SURE-RAG](https://arxiv.org/abs/2605.03534) | Retrieval and answer reliability |
| 31 | [FaithfulRAG](https://arxiv.org/abs/2506.08938) | Evidence-grounded generation |
| 32 | [RAGChecker](https://arxiv.org/abs/2408.08067) | Retrieval-augmented generation evaluation |
| 33 | [ARES](https://arxiv.org/abs/2311.09476) | Automated RAG evaluation |
| 34 | [RT4CHART](https://arxiv.org/abs/2603.27752) | Retrieval and grounding evaluation |
| 35 | [MedRAGChecker](https://arxiv.org/abs/2601.06519) | Domain-specific RAG evaluation |
| 36 | [Submodular coverage](https://doi.org/10.1007/BF01588971) | Budgeted set selection |
| 37 | [Minimum Description Length](https://doi.org/10.1016/0005-1098(78)90005-5) | Compact model selection |
| 38 | [Conditional Value at Risk](https://doi.org/10.21314/JOR.2000.038) | Tail-risk-aware optimization |
| 39 | [QAOA](https://arxiv.org/abs/1411.4028) | Combinatorial optimization reference |
| 40 | [Ising / QUBO](https://www.frontiersin.org/journals/physics/articles/10.3389/fphy.2014.00005/full) | Discrete optimization formulation |
| 41 | [QVCache](https://arxiv.org/abs/2602.02057) | KV-cache compression |
| 42 | [TurboQuant](https://arxiv.org/abs/2504.19874) | Quantized inference state |
| 43 | [LLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Prompt compression comparison |
| 44 | [LongLLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Long-context compression comparison |
| 45 | [LLMLingua-2](https://arxiv.org/abs/2403.12968) | Task-agnostic prompt compression comparison |
| 46 | [Context Mode](https://context-mode.com/) | Local context management comparison |
| 47 | [LeanCTX](https://leanctx.com/) | Context compression comparison |
| 48 | [sqz CLI](https://github.com/ojuschugh1/sqz) | Repository-context compression comparison |
| 49 | [indxr](https://docs.rs/crate/indxr/0.2.0) | Local code indexing comparison |
| 50 | [Aider Repo Map](https://aider.chat/docs/repomap.html) | Repository mapping comparison |

The detailed academic and engineering map is in [Research](docs/research.md).
Thanks also to AMD and ROCm, Lablab.ai, Fireworks AI, OpenAI Build Week, Qwen
Cloud and Alibaba Cloud, Devpost, and NamasteDev for the hardware, tools, or
event settings used in the published hackathon work. No endorsement is implied.

Copyright (c) 2026 Marvin Sarreal Villanueva. Code and operational
documentation are licensed under [AGPL-3.0-only](LICENSE). See
[CONTRIBUTING](CONTRIBUTING.md), [GOVERNANCE](GOVERNANCE.md), and
[TRADEMARKS](TRADEMARKS.md).
