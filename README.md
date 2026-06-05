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
