# Qorx Void

**Local project memory for Codex and other supported AI agents.**

Qorx Void keeps repeated workspace context on the user's machine. When a task
needs local knowledge, Void returns a small carrier, exact cited lines, or a
bounded evidence pack instead of sending the whole repository again.

This directory is the public Qorx Void handbook. It is documentation only.
There is no Qorx Void source code, private kernel implementation, proprietary
selection logic, prompt library, signing operation, account service, or release
procedure here.

## What Void changes

Without a local memory layer, an AI tool often has to reread the same repository,
notes, logs, policies, and project rules. Void keeps that stable material beside
the work and resolves only what the current task can justify.

```text
current task
    |
    v
local Qorx session -> bounded carrier -> AI agent
    ^                                      |
    |------ exact local proof on demand ---|
```

The model can only use context it actually receives. Void reduces repeated
context; it does not make a remote model know unseen files, hide provider
billing, or guarantee that another model will never hallucinate.

## What is documented

- [Architecture](architecture.md): the public data flow and trust boundary.
- [Day-to-day use](day-to-day-use.md): a normal local workflow.
- [Tools](tools.md): every public 1.0.6 CLI command, MCP tool, action group, and supported integration.
- [Benchmarks](benchmarks.md): the AMD MI300X result, public repo proof, and scoped competitor comparison.
- [Security model](security-model.md): what stays local and what can cross the boundary.
- [Release boundary](release-boundary.md): what this repository may publish.

## Relationship to Qorx

| Surface | Public status |
| --- | --- |
| Qorx language, compiler, bytecode runtime, schemas, and public CLI | Open in the main repository under AGPL-3.0-only. |
| Qorx Void product behavior and operator documentation | Documented in this directory. |
| Qorx Void proprietary source and production internals | Not published here. |
| Qorx Zero hackathon applications | Separate clean-room repositories with no private Void code. |

## Start here

For the public CLI workflow:

```sh
qorx doctor
qorx daemon start
qorx install -p codex
qorx index .
qorx context inject "continue the current release work" --block
```

Then read [day-to-day use](day-to-day-use.md). Qorx Void distribution and
account access are handled outside this source repository.
