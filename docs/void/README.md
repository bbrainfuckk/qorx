# Qorx Void

Qorx Void is local project memory for Codex users.

It keeps repeated workspace context on the user's machine, resolves the current task against local memory units, and gives Codex a compact proof-shaped frame when a turn needs it. The goal is simple: keep the plan moving without sending the same project context again and again.

This directory is the public technical handbook for Qorx Void. It is deeper than the product website, but it is still a documentation-only boundary. It explains what Void does, how operators should think about it, how testers can reproduce benchmark behavior, and what stays private.

## Read First

- [Architecture](architecture.md): the public system model.
- [Day-To-Day Use](day-to-day-use.md): how users and testers use Qorx Void and `qorx-free`.
- [qorx-free](qorx-free.md): the public Linux AMD MI300X benchmarker.
- [Security Model](security-model.md): what the public docs reveal and what they do not reveal.
- [Release Boundary](release-boundary.md): what can be shipped on GitHub and what cannot.

## Public Promise

Qorx Void can be documented publicly without publishing the private implementation.

Public docs may describe:

- product behavior;
- operator workflows;
- high-level architecture;
- benchmark methodology;
- release package boundaries;
- security and support expectations.

Public docs must stay out of the private product. They must not include source, unpublished implementation material, sensitive operational details, private data, or build and release procedures.

## Product Split

| Name | Public role |
| --- | --- |
| Qorx | The language and runtime family. |
| Qorx Void | The full product: local project memory for Codex users. |
| `qorx-free` | The public benchmark and reproducibility build for Linux AMD MI300X testers. |

`qorx-free` is not Qorx Void. It is the public tester surface that verifies compiled Qorx bytecode, checks AMD MI300X readiness, and emits sanitized benchmark artifacts.
