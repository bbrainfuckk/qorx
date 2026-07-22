# Technology

Qorx is a programming language and local runtime for humans and AI agents.

Qorx Void is the buyer-facing product: local project memory for Codex users. It keeps repeated workspace context on the customer's computer and gives Codex a compact proof-shaped frame when the next turn needs it.

## Core Terms

| Term | Public meaning |
| --- | --- |
| Qorx | The language and runtime family. |
| Qorx Void | The Codex product: local project memory with active Void Hours. |
| Qorx Void app | The installed app when referring specifically to the local app. |
| Quark | A local memory unit. Quarks hold workspace facts, proof traces, and task-relevant context without forcing the whole workspace back into the prompt. |
| Carrier | A small frame selected from quarks for the current turn. |
| Photon | A physics-inspired alias for a model-visible carrier. |
| Proof record | A compact record that explains why a carrier was selected and what local evidence supports it. |
| Hot / penumbra / abyss | Resident, semi-active, and cold local storage layers. These are storage metaphors. |
| Prism | A planning term for splitting one objective into agent-ready packs. |

These names come from physics and astronomy, but the objects are software
records. Hot and cold are not quark flavours, photon does not mean a physical
particle, and Qorx does not claim to simulate those systems.

## How A Turn Works

1. The user asks Codex to work on a task.
2. Qorx Void reads the local workspace state already known to the machine.
3. The runtime resolves the task against local quarks.
4. Qorx sends a compact carrier to Codex.
5. Codex continues the work with less repeated context in the prompt.

The important point is locality. Qorx is built so the repeated project memory lives on the user's computer, not as a public source bundle or hosted prompt dump.

## Qorx Language

Qorx uses an English-like, explicit programming surface. Nouns, verbs, targets,
conditions, budgets, receipts, and proof steps become checked structure for the
runtime.

The public repository includes `.qorx` examples, the Rust bootstrap compiler,
bytecode runtime, and tests. It does not include Qorx Void source, private
algorithms, hosted orchestration, signing keys, or proprietary service policy.

## Academic Map

The public buckets are a research map, not a claim that every named theory is
implemented. Qorx's center is information under budget, carried by programs and
resolved by local evidence.

| Bucket | What it names |
| --- | --- |
| Classical-Quantum Information Theory | Context as useful information under cost, uncertainty, and measurement. |
| Certified Compilation and Proof-Carrying Semantics | Programs that carry the evidence needed to trust their selected context. |
| Quantum-Inspired Combinatorial Optimization | Budgeted selection across many possible context states. |
| Retrieval-Augmented Agent Memory Systems | Local workspace memory that can be reused across agent turns. |
| Cryptographic Provenance and Local Runtime Architecture | Signed state, local ownership, receipts, and audit trails. |
| Nonlinear Dynamics and Quantum Chaos | A conceptual lens for stability and sensitivity, not an implemented physical model. |
| Energy-Aware Computing | Conditional accounting for work avoided when repeated model-bound input is reduced. |

Qorx 1.0.6 runs deterministic classical algorithms on ordinary hardware. It
does not execute a quantum algorithm or require quantum hardware.

## Core Subjects

| # | Subject |
| ---: | --- |
| 1 | Classical Information Theory |
| 2 | Quantum Information / Quantum Computing Theory |
| 3 | Combinatorial Optimization |
| 4 | Operations Research / Budgeted Selection |
| 5 | Formal Methods |
| 6 | Proof-Carrying Code / Proof-Carrying Semantics |
| 7 | Programming Language Theory |
| 8 | Compiler Construction |
| 9 | Runtime Systems / Operating-Systems-Style Memory |
| 10 | Information Retrieval |
| 11 | Retrieval-Augmented Generation / Knowledge Systems |
| 12 | Graph-Based Codebase and Knowledge Representation |
| 13 | Cryptography / Authenticated Data Structures |
| 14 | Provenance, Audit, and Trust Infrastructure |
| 15 | Nonlinear Dynamics / Quantum Chaos |
| 16 | Energy-Aware Computing / Computational Sustainability |

## Secondary Subjects

| # | Subject |
| ---: | --- |
| 17 | KV Cache / Inference Runtime Optimization |
| 18 | Agent Memory and Long-Horizon Autonomy |
| 19 | Protocol and Interoperability Design: MCP, A2A, HTTP, protobuf |
| 20 | Cost Accounting / Computational Economics |
| 21 | Software Supply Chain and Release Engineering |
| 22 | Human-Computer Interaction / Developer Tooling |

The product experience stays simple: keep the Codex plan you already own, add local project memory, and reduce repeated context.
