# Research

Qorx is built from a language idea and a runtime idea. Marvin Sarreal
Villanueva describes its origin as years spent reading thousands of papers
across information theory, retrieval, compilers, formal methods, cryptography,
optimization, agent memory, quantum information, nonlinear dynamics, and
energy-aware computing.

The language idea: humans and AI agents should be able to write direct
instructions that preserve intent. Qorx uses an English-like, explicit grammar
of verbs, targets, budgets, receipts, and proof steps as its readable surface.

The runtime idea: a local machine can hold repeated project memory, resolve the current task against that memory, and send a small proof frame instead of replaying a large workspace.

## Academic Map

These public buckets are a high-level map, not proof that Qorx implements every
theory named here. The center is information under budget, carried by programs
and resolved by local evidence.

| Bucket | Qorx connection |
| --- | --- |
| Classical-Quantum Information Theory | Treats context as structured information that can be compressed, selected, and proven without treating every token as equally useful. |
| Certified Compilation and Proof-Carrying Semantics | Keeps execution tied to receipts and proof records so a turn can show why selected context belongs there. |
| Quantum-Inspired Combinatorial Optimization | Uses selection pressure over many local context candidates rather than a flat prompt dump. |
| Retrieval-Augmented Agent Memory Systems | Turns local workspace memory into task-specific quarks and carriers. |
| Cryptographic Provenance and Local Runtime Architecture | Keeps proof records and source ownership tied to local artifacts and checksums. |
| Nonlinear Dynamics and Quantum Chaos | Supplies a research lens for stability and sensitivity; Qorx does not implement a physical chaos model. |
| Computational Sustainability | Treats repeated model-bound input as avoidable work when deployment measurements support that conclusion. |

Qorx 1.0.6 is deterministic classical software. The quantum subjects are
conceptual and optimization references; the runtime neither executes a quantum
algorithm nor requires quantum hardware.

## Core Subjects

| # | Subject | Qorx connection |
| ---: | --- | --- |
| 1 | Classical Information Theory | Token budgets, compression pressure, entropy, and signal selection. |
| 2 | Quantum Information / Quantum Computing Theory | State, superposition-inspired selection, and measurement language for context. |
| 3 | Combinatorial Optimization | Choosing the best context subset under hard limits. |
| 4 | Operations Research / Budgeted Selection | Practical scheduling, capacity, and cost tradeoffs. |
| 5 | Formal Methods | Checks, invariants, and proof records around selected context. |
| 6 | Proof-Carrying Code / Proof-Carrying Semantics | Making selected context carry its reason for being present. |
| 7 | Programming Language Theory | Human-readable source mapped into structured execution. |
| 8 | Compiler Construction | Parsing, verification, lowering, and bounded runtime execution. |
| 9 | Runtime Systems / Operating-Systems-Style Memory | Local state, memory layers, and turn-time dispatch. |
| 10 | Information Retrieval | Finding relevant files, notes, and evidence. |
| 11 | Retrieval-Augmented Generation / Knowledge Systems | Giving agents grounded, task-specific memory. |
| 12 | Graph-Based Codebase and Knowledge Representation | Project maps, relationships, and provenance edges. |
| 13 | Cryptography / Authenticated Data Structures | Hashes, signed records, and tamper-evident artifacts. |
| 14 | Provenance, Audit, and Trust Infrastructure | Receipts, review paths, and customer-visible proof. |
| 15 | Nonlinear Dynamics / Quantum Chaos | Conceptual vocabulary for stability and sensitivity, not an implemented physics simulation. |
| 16 | Energy-Aware Computing / Computational Sustainability | Scenario-based accounting for repeated input work under explicit measurement boundaries. |

## Secondary Subjects

| # | Subject | Qorx connection |
| ---: | --- | --- |
| 17 | KV Cache / Inference Runtime Optimization | Avoiding repeated inference work when state can be reused or represented more compactly. |
| 18 | Agent Memory and Long-Horizon Autonomy | Keeping useful workspace memory across many turns without making every turn start over. |
| 19 | Protocol and Interoperability Design: MCP, A2A, HTTP, protobuf | Connecting local proof, tools, and agents through clear transport and schema boundaries. |
| 20 | Cost Accounting / Computational Economics | Tracking token, time, and compute budgets as product constraints. |
| 21 | Software Supply Chain and Release Engineering | Separating private implementation from public documentation, releases, and checksums. |
| 22 | Human-Computer Interaction / Developer Tooling | Making a complex local memory system understandable inside the developer's daily workflow. |

## Language And Compiler References

These projects are background reading and engineering references, not Qorx
contributors or bundled implementations. The Qorx language, compiler, runtime,
and tests in this repository are public under the repository license. Qorx Void
source and proprietary service internals are not published here.

- Zig: https://github.com/ziglang/zig
- Rust: https://github.com/rust-lang/rust
- LLVM: https://github.com/llvm/llvm-project
- TinyCC: https://github.com/TinyCC/tinycc
- Tree-sitter: https://github.com/tree-sitter/tree-sitter
- Protocol Buffers: https://github.com/protocolbuffers/protobuf
- Wasmtime / Cranelift: https://github.com/bytecodealliance/wasmtime
- TempleOS historical reference: https://github.com/cia-foundation/TempleOS
- ZealOS historical reference: https://github.com/Zeal-Operating-System/ZealOS

## Qorx Compiler Reference Spine

These links describe the public compiler and systems references around the Qorx compiler direction. They are references, not copied implementation.

| Reference | Link | Why it matters |
| --- | --- | --- |
| TempleOS / HolyC | https://github.com/cia-foundation/TempleOS | Small-system directness and tight language/runtime feel. |
| ZealOS | https://github.com/Zeal-Operating-System/ZealOS | Living historical reference for the TempleOS line. |
| TinyCC / TCC | https://github.com/TinyCC/tinycc | Small compiler baseline. |
| QBE | https://c9x.me/compile/ | Compact compiler backend reference. |
| LLVM / Clang | https://github.com/llvm/llvm-project | Industrial compiler architecture reference. |
| Cranelift | https://github.com/bytecodealliance/wasmtime/tree/main/cranelift | Modern code generation and runtime compiler reference. |
| GNU Mes | https://www.gnu.org/software/mes/ | Auditable bootstrap-chain reference. |
| Bootstrap seeds | https://github.com/oriansj/bootstrap-seeds | Minimal seed-program lineage for bootstrapping research. |
| WebAssembly core spec | https://webassembly.github.io/spec/core/ | Validation and bounded execution reference. |
| Tree-sitter | https://github.com/tree-sitter/tree-sitter | Incremental parsing and editor-aware syntax reference. |
| Language Server Protocol | https://github.com/microsoft/language-server-protocol | Editor and tool interoperability reference. |
| Protocol Buffers | https://github.com/protocolbuffers/protobuf | Structured message and schema reference. |
| C2PA | https://github.com/contentauth/c2pa-rs | Provenance and content-authentication reference. |

## Agent Context References

- Kortex by Arjay, credited for helping shape the local-context direction behind Qorx: https://github.com/H4D3ZS/kortex
- Gemini CLI: https://github.com/google-gemini/gemini-cli
- Aider: https://github.com/Aider-AI/aider

Kortex is credited for influence, not used as a dependency. Qorx is independently authored and uses a separate language, compiler/runtime design, quark and carrier model, benchmark record, product architecture, and implementation. Qorx does not copy, import, redistribute, or package Kortex source code in this public repository.
