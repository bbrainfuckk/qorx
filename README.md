# Qorx CLI

Buildable local context runtime for Qorx.

This is the public source line for the Qorx CLI, language runtime, local
evidence index, proof commands, benchmark fixtures, and package recipes.
Qorx Void Desktop is the commercial desktop experience around the same runtime:
account, license, support, installer flow, tray controls, and managed local
operation.

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19875352.svg)](https://doi.org/10.5281/zenodo.19875352)
[![Preprint DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19953308.svg)](https://doi.org/10.5281/zenodo.19953308)
[![Software Heritage](https://img.shields.io/badge/Software%20Heritage-archived-ff6600)](https://archive.softwareheritage.org/browse/origin/directory/?origin_url=https://github.com/bbrainfuckk/qorx)
[![License: AGPL-3.0-only](https://img.shields.io/github/license/bbrainfuckk/qorx?color=blue)](LICENSE)
[![Rust stable](https://img.shields.io/badge/rust-stable-orange?logo=rust)](https://www.rust-lang.org/)

![Qorx banner](docs/assets/qorx-img.jpg)

Qorx stops AI workflows from pasting the same files into every prompt.

Qorx is a small domain-specific language, compiler, and local runtime for
context resolution. A `.qorx` file can be a compact directive file or a named
resolver program with variables, steps, semantic checks, assertions, and an
`emit` target. Programs can branch on supported evidence. Qorx compiles that
source to protobuf-envelope `.qorxb` bytecode with AST, QIR, canonical opcodes,
and integrity hashes, then runs it against local state. The 0.0.1-ylem line also
emits `qstk`, a Forth-inspired stack tape inside the protobuf envelope for tiny
local dispatch.
The model sees a small carrier or evidence pack. The resolver keeps the index,
receipts, cache, provenance, and proof pages local.

The break is architectural: context is addressed and faulted in, not pasted over
and over. That is the claim. It is testable through the command surface and the
checked proof commands below.

## Status

Current local runtime line: `0.0.1-ylem`.

Reviewer docs now track the `0.0.1-ylem` Qorx CLI and Void line. This runtime
reports its binary/package version with `qorx --version`.

Try the product path when you want the finished desktop experience:

- Qorx Void Desktop: free 24-hour local demo.
- Qorx Cloud API: free 5,000 hosted calls.

Run `qorx` with no arguments for the Qorx CLI splash, `qorx --help` for
the command tree, and `qorx man` for the field manual.

Qorx is free software under `AGPL-3.0-only`. The project identity, Qorx name,
`.qorx` source format, `.qorxb` bytecode format, and `qorx://` resolver scheme
are reserved for attribution and ecosystem clarity. Forks are allowed under the
license, but official branding is covered by [TRADEMARKS.md](TRADEMARKS.md).

## Live Proof

The current public proof files are
[`docs/benchmarks/live.md`](docs/benchmarks/live.md) and
[`docs/benchmarks/live.json`](docs/benchmarks/live.json). The latest local run
reports:

| Case | Indexed local tokens | Model-visible tokens | Local reduction |
| --- | ---: | ---: | ---: |
| Session carrier | 388,573 | 69 | 5,631.49x |
| Evidence pack | 388,573 | 410 | 947.74x |
| Squeeze extract | 388,573 | 448 | 867.35x |

These are Qorx local `ceil(chars / 4)` estimates. They are not provider invoice
savings, and they do not prove answer quality. They show the boundary Qorx is
built to measure: large local state, small visible carrier, resolver available.

## Read First

Start with the path that matches your question:

| If you want to... | Start with |
| --- | --- |
| Build with AI tools and local repos | [Reader guide](docs/AUDIENCE_GUIDE.md) and [Manual](docs/MANUAL.md) |
| Understand token and cost impact | [Reader guide](docs/AUDIENCE_GUIDE.md), [Metrics](docs/METRICS.md), and [Live proof](docs/benchmarks/live.md) |
| Review the runtime as software | [Qorx for Rust reviewers](docs/QORX_1_0_4_RUST.md) and [Command reference](docs/COMMANDS.md) |
| Check the science and claim boundary | [Science](docs/SCIENCE.md), [Science and math](docs/SCIENCE_AND_MATH.md), and [SAFE-R](docs/SAFE-R.md) |

Then use the handbook as the source of truth:

- [Handbook](docs/handbook/README.md)
- [Language](docs/handbook/language.md)
- [Runtime](docs/handbook/runtime.md)
- [Science notes](docs/handbook/science.md)
- [Science](docs/SCIENCE.md)
- [Science and math](docs/SCIENCE_AND_MATH.md)
- [Trial guide](docs/TRIALS.md)
- [Void boundary](docs/VOID_BOUNDARY.md)
- [Live metrics](docs/LIVE_METRICS.md)
- [Qorx metrics](docs/METRICS.md)
- [Community guide](docs/COMMUNITY.md)
- [Protocol](docs/handbook/protocol.md)
- [Command reference](docs/COMMANDS.md)
- [Manual](docs/MANUAL.md)
- [Audience guide](docs/AUDIENCE_GUIDE.md)
- [Production status](docs/PRODUCTION.md)
- [Server and daemon](docs/SERVER.md)
- [SAFE-R anti-hype gate](docs/SAFE-R.md)
- [TestSprite enterprise QA](docs/TESTSPRITE.md)
- [Media and reviewer notes](docs/MEDIA.md)
- [Independent review brief](docs/INDEPENDENT_REVIEW.md)
- [Technical credibility](docs/TECHNICAL_CREDIBILITY.md)
- [Qorx 0.0.1-ylem for Rust reviewers](docs/QORX_1_0_4_RUST.md)
- [Benchmarks](docs/benchmarks/README.md)
- [Qorx papers and preprint](docs/papers/README.md)
- [Release notes](docs/releases/v0.0.1-ylem.md)

Qorx is not a prompt trick, a billing bypass, a general-purpose language, or
universal compression of unknown data. It works when a workflow carries `.qorx`
source, `.qorxb` bytecode, Qorx handles, or Qorx evidence packs and has a
resolver available.

## Minimal Program

```text
QORX 1
use std.evidence
use std.branch as br
let question = "which files explain how Qorx keeps local evidence outside the model prompt?"
let fallback = "qv0d: local evidence does not support this answer"
pack evidence from question budget 700
cache evidence key question ttl 3600
strict answer from evidence limit 2
if supported(answer) then emit answer else emit fallback
```

Check it:

```powershell
cargo run -- qorx-check .\goal.qorx
```

Run it:

```powershell
cargo run -- qorx .\goal.qorx
```

Compile it:

```powershell
cargo run -- qorx-compile .\goal.qorx --out .\goal.qorxb
cargo run -- qorx .\goal.qorxb
```

## Core Model

| Term | Short name | Meaning |
| --- | --- | --- |
| `.qorx` | qwav | Human-readable source. |
| `.qorxb` | qfal | Protobuf-envelope bytecode after semantic checks and compile. |
| QIR | qir | Lowered Qorx intermediate representation used for compiler-visible resolver calls. |
| stack tape | qstk | Forth-inspired bytecode word stream for tiny local dispatch. |
| cache policy | qcas | Source-level cache binding for stable resolver outputs near the runtime. |
| resolver step | qop | Named opcode such as `pack`, `strict`, `squeeze`, `map`, or `session`. |
| carrier | phot | Small model-visible object: source, bytecode, handle, or evidence pack. |
| `qorx://s/...` | qses | Session handle for indexed local state. |
| `qorx://c/...` | qcap | Capsule handle for a local context bundle. |
| `qorx://u/...` | qevt | Event handle for a local receipt. |
| quark | qrk | Bounded, hashed, token-estimated evidence chunk. |
| local state | qosm | Local Qorx state: index, cache, receipts, provenance, lattice, traces. |
| resolver boundary | hzon | Line between local state and model-visible carrier. |
| qshf factor | qshf | Baseline-to-Compact ratio between local context mass and visible carrier mass. |
| B2C | b2c | Baseline-to-Compact accounting. Local estimate, not a provider invoice. |
| B2C allocator | qalc | Local budgeted quark selector used by `b2c-plan` and `pack`. |

These are Qorx vocabulary labels, not physics claims. The short names are for
logs, UI, and compact docs after the reader has seen the concrete workflow. The
full boundary is in [SAFE-R](docs/SAFE-R.md).

The current public glossary also exposes one hundred 3-character Qorx terms via
`qorx lexicon`. Older labels such as `qosm`, `qshf`, and `qv0d` remain
compatibility vocabulary for saved handles and old proof strings.

## Build

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

## Install

The current public `0.0.1-ylem` line is source-first. The source tag is the
install target; binary assets are not attached yet:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --tag v0.0.1-ylem --locked qorx
```

Shortcut install surface:

```sh
qorx -i
qorx -i -p codex
qorx -in -p antigravity
```

Package-manager wrappers and Linux packaging recipes are in the repo as
maintainer packaging surfaces. Use them after the matching public package page
or release asset exists for `0.0.1-ylem`:

- [Install guide](docs/INSTALL.md)
- [Distribution notes](docs/DISTRIBUTION.md)
- `packages/npm/`
- `packages/python/`
- `packaging/`
- `flake.nix`
- `snap/snapcraft.yaml`

## Daemon

The official background runtime is the daemon:

```sh
qorx daemon start
qorx daemon status
qorx daemon stop
```

`qorx daemon` and `qorx daemon run` run the same gateway in the foreground for
systemd, Docker, or a terminal. Windows also has an optional tray command. The
tray is a control surface for the daemon; Linux and macOS use the daemon and
their normal supervisors.

## Proof Commands

```powershell
.\target\release\qorx.exe --version
.\target\release\qorx.exe doctor --json
.\target\release\qorx.exe index .
.\target\release\qorx.exe session
.\target\release\qorx.exe b2c-plan "which files explain the resolver boundary?" --budget-tokens 900
.\target\release\qorx.exe strict-answer "which files explain the resolver boundary?"
.\target\release\qorx.exe context snapshot
.\target\release\qorx.exe context verify
.\target\release\qorx.exe security attest
.\scripts\safer-check.ps1 -Exe .\target\release\qorx.exe -SkipCargo
```

Proof numbers use Qorx's deterministic `ceil(chars / 4)` token estimate unless
another tokenizer is explicitly named. Do not present those numbers as provider
invoice savings.

## Repository Map

| Path | Purpose |
| --- | --- |
| `src/` | Rust implementation of the parser, runtime, resolver, index, cache, protocol, and CLI. |
| `tests/` | Runtime, language, capsule, context, lattice, and strict evidence tests. |
| `docs/handbook/` | Manual-style operating documentation. |
| `docs/COMMANDS.md` | Command catalog. |
| `docs/PRODUCTION.md` | Production boundary and readiness gate. |
| `docs/SERVER.md` | Daemon, HTTP gateway, Docker, and systemd notes. |
| `docs/releases/` | Release notes. |
| `Dockerfile` | Container build for the daemon. |
| `packaging/systemd/` | systemd service template. |
| `examples/` | Small fixtures for impact and evidence routes. |
| `scripts/` | Publishing and proof helpers. |

## Boundaries

Qorx can resolve Qorx-known local handles, bytecode, indexed evidence, and
receipts. It cannot reconstruct arbitrary unknown files from a tiny message. It
cannot make a remote model know hidden local data without a resolver path. It
does not certify task quality by token savings alone.

## License

Copyright (c) 2026 Marvin Sarreal Villanueva.

- Code and operational docs: [AGPL-3.0-only](LICENSE)
- Citation metadata: [CITATION.cff](CITATION.cff)
- Qorx Local Context Resolution preprint: [10.5281/zenodo.19953308](https://doi.org/10.5281/zenodo.19953308)
- Contribution terms: [CONTRIBUTING.md](CONTRIBUTING.md)
- Security policy: [SECURITY.md](SECURITY.md)
- Governance: [GOVERNANCE.md](GOVERNANCE.md)
- Marks and project identity: [TRADEMARKS.md](TRADEMARKS.md)
