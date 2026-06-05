# Qorx Void Architecture

This page describes the public architecture. It is not an implementation spec.

## System Model

Qorx Void sits beside Codex as a local context layer.

1. The user works in a local repository.
2. Qorx Void observes task-relevant workspace state.
3. Local memory units, called quarks, keep reusable facts and proof traces.
4. The runtime selects a compact carrier for the current task.
5. Codex receives the carrier and continues with less repeated context.

The key boundary is locality. The useful project memory starts on the user's machine and should not become a public source dump, prompt dump, or benchmark artifact.

## Public Components

| Component | Public description |
| --- | --- |
| Local memory | Stores task-relevant facts, proof traces, and prior work as quarks. |
| Context selection | Chooses a compact local frame for the current task. |
| Carrier | The compact frame sent to the agent when it needs context. |
| Proof record | A small record explaining why the carrier was selected. |
| Benchmarker | `qorx-free`, the public Linux AMD MI300X reproducibility build. |

## Private Boundary

The public repository does not publish Qorx Void source, unpublished implementation material, sensitive operational details, private data, or build and release procedures.

## Safe Depth

The public docs can explain the shape of a turn, the operator experience, the benchmark boundary, and expected outputs. They should not explain how to reproduce the private product.

Good public detail:

- what the user runs;
- what files a public benchmark writes;
- what data must not be written;
- what security checks are expected;
- what claims are blocked.

Boundary rule: public docs stay at product, benchmark, support, and release-boundary level. More specific private material stays outside GitHub.
