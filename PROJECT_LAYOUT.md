# Qorx Public Project Layout

This repository is the public Qorx CLI and runtime source line.

| Area | Path | Purpose |
| --- | --- | --- |
| Runtime | `src/` | Rust implementation for the CLI, language runtime, local index, daemon, monitor, integrations, and proof tools. |
| Tests | `tests/` | Rust tests for language, context, MCP, strict evidence, benchmarks, and runtime behavior. |
| Docs | `docs/` | Handbook, science notes, benchmark proof, papers, install guides, and product boundaries. |
| Examples | `examples/` | Small local fixtures for benchmark and evidence workflows. |
| Packages | `packages/`, `packaging/`, `snap/`, `flake.nix` | npm, Python, Linux, Windows, macOS, Docker, Nix, and registry recipes. |
| CI | `.github/` | Build, proof, package, and release workflows. |
| Metrics | `cloudflare/`, `docs/benchmarks/` | Public benchmark proof and the community metrics worker. |

## What Is Not Here

The public repo does not contain Qorx customer accounts, production secrets,
payment ledgers, signing keys, hosted service credentials, or release binaries.
Those belong to the managed Qorx service and official release process.

The source is enough to build and inspect the CLI/runtime. It is not a clone of
the hosted account system.
