# Qorx Community Edition

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19875352.svg)](https://doi.org/10.5281/zenodo.19875352)
[![Preprint DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19953308.svg)](https://doi.org/10.5281/zenodo.19953308)
[![Software Heritage](https://img.shields.io/badge/Software%20Heritage-archived-ff6600)](https://archive.softwareheritage.org/browse/origin/directory/?origin_url=https://github.com/bbrainfuckk/qorx)
[![License: AGPL-3.0-only](https://img.shields.io/github/license/bbrainfuckk/qorx?color=blue)](LICENSE)
[![Rust stable](https://img.shields.io/badge/rust-stable-orange?logo=rust)](https://www.rust-lang.org/)

![Qorx banner](docs/assets/qorx-img.jpg)

This repository is the AGPL Community Edition of Qorx.

Qorx is a small domain-specific language, compiler, and local context-resolution
CLI. It lets a workflow carry a checked `.qorx` program, `.qorxb` bytecode,
Qorx handle, or evidence pack instead of repeatedly pasting the same local files
into prompts.

The native `.qorx` language is the core path, but production teams do not have
to abandon their existing stack. Qorx can read a local adapter manifest for
external language servers, parser tools, compressors, provider shims, or runtime
services. Swapping a TypeScript, Python, Rust, Go, or custom runtime adapter is a
manifest update, not a Qorx rebuild.

Community Edition is the public source line, not a stripped demo. It gives the
community the parser, bytecode path, local index, strict evidence commands,
pack/squeeze routes, local accounting, reproducible tests, and benchmark
fixtures. Build it, inspect it, fork it under AGPL, and use it for local
evidence-first workflows.

Qorx Edge is the supported local product line for signed installers, tray UX,
auto-update, daemon management, provider routing, MCP/CLI activation, ORCL,
cloud capsule sync, team policy, support, and managed local-vault UX. Qorx Cloud
is the hosted API and account surface. Community Edition stays focused on the
open source language, runtime, CLI proof surface, and reproducible local
benchmarks.

## Qorx Edge Starter

Qorx Edge Starter is the full product trial path for people who want the real
Windows, macOS, and Linux experience without starting with a payment wall.

Starter starts with 5,000 included Edge/Cloud requests. A request means a managed
service action: provider routing, ORCL lookup, MCP/CLI activation, cloud capsule
sync, or an account-backed daemon/API call. Local Community Edition commands
such as `index`, `pack`, `squeeze`, `strict-answer`, `qorx-check`, and
`qorx-compile` stay local and unmetered.

After the 5,000 included Edge/Cloud requests are used, the account asks the user
to subscribe before continuing with Edge/Cloud services. The cap is enforced
server-side by the Qorx account service. That is the only honest place to put
it: a local AGPL binary can always be changed and rebuilt, but a fork cannot
mint valid Qorx service entitlements or use the official hosted service for
free forever.

## Status

Current public version: `1.0.4a`.

[![Qorx CE](https://qorx-community-metrics.omniscius.workers.dev/badge/version)](https://qorx-community-metrics.omniscius.workers.dev/metrics.json)
[![Build](https://qorx-community-metrics.omniscius.workers.dev/badge/build)](https://qorx-community-metrics.omniscius.workers.dev/metrics.json)
[![TestSprite](https://qorx-community-metrics.omniscius.workers.dev/badge/testsprite)](https://qorx-community-metrics.omniscius.workers.dev/metrics.json)
[![Local reduction](https://qorx-community-metrics.omniscius.workers.dev/badge/reduction)](https://qorx-community-metrics.omniscius.workers.dev/metrics.json)

Qorx is free software under `AGPL-3.0-only`. The Qorx name, logo, product marks,
and official distribution identity are separate from the code license. Forks are
allowed under the license, but they may not imply that they are official Qorx.
See [TRADEMARKS.md](TRADEMARKS.md).

## Live proof

The README badges are served by a Cloudflare Worker:

```text
https://qorx-community-metrics.omniscius.workers.dev/metrics.json
```

The worker refreshes from public GitHub APIs and committed Qorx benchmark proof
files with a daily edge cache. It reports the current repo version, latest
release, CI/TestSprite state, and latest local benchmark numbers. The scheduled
`Daily Community Proof` workflow refreshes `docs/benchmarks/live.json` and
`docs/benchmarks/live.md` from a real source build and local benchmark run.

Current checked-in live proof:
[`docs/benchmarks/live.md`](docs/benchmarks/live.md)

| Case | Indexed local tokens | Model-visible tokens | Local reduction |
| --- | ---: | ---: | ---: |
| Session carrier | 202,986 | 69 | 2,941.83x |
| Evidence pack | 202,986 | 484 | 419.39x |
| Squeeze extract | 202,986 | 419 | 484.45x |

These are Qorx local `ceil(chars / 4)` estimates. They are not provider invoice
savings, and they do not prove answer quality. They show the measurement Qorx is
built around: large local state, small visible carrier, resolver available.

## Global community release

Community Edition now has a maintainer-controlled cross-platform release
workflow. Tags build source-verified CLI assets for:

- Windows x64 zip.
- Linux x64 tarball.
- macOS tarball.

The assets are unsigned community CLI builds. Qorx Edge Starter gives users the
full Edge/Cloud feature set across Windows, macOS, and Linux with 5,000 included
Edge/Cloud requests before subscription. Qorx Edge adds signed installers, tray,
auto-update, daemon, provider routing, MCP activation, and ORCL.

The `1.0.4a` release also enforces the small-binary target in CI. Release builds
fail if the CLI binary is over 5 MiB. On this Windows workspace, the optimized
release binary is about 2.02 MiB before archive compression.

## Package channels

The repo now carries package-channel files for PyPI, npm, Arch/AUR, Homebrew,
Scoop, WinGet, Snap, Docker, Nix, and Deb/RPM packaging through nfpm. See
[packaging/README.md](packaging/README.md).

These package files install or build Qorx Community Edition. Community Edition
local commands are not capped at 5,000 requests. The 5,000 included Edge/Cloud
requests apply only to Qorx Edge Starter service features, where the Qorx account
service can enforce the allowance server-side.

## Read first

- [Community guide](docs/COMMUNITY.md)
- [Install from source](docs/INSTALL.md)
- [Qorx Edge Starter](docs/EDGE_STARTER.md)
- [Live metrics](docs/LIVE_METRICS.md)
- [Science and math](docs/SCIENCE_AND_MATH.md)
- [Package channels](packaging/README.md)
- [Registry automation](docs/REGISTRY_AUTOMATION.md)
- [Language](docs/handbook/language.md)
- [Command reference](docs/COMMANDS.md)
- [Runtime notes](docs/handbook/runtime.md)
- [Runtime options](docs/SERVER.md)
- [SAFE-R claim check](docs/SAFE-R.md)
- [Technical credibility](docs/TECHNICAL_CREDIBILITY.md)
- [Independent review brief](docs/INDEPENDENT_REVIEW.md)
- [Qorx 1.0.4a for Rust reviewers](docs/QORX_1_0_4_RUST.md)
- [Benchmarks](docs/benchmarks/README.md)
- [Papers and preprint](docs/papers/README.md)

Qorx is not a prompt trick, a billing bypass, a general-purpose language, or
universal compression of unknown data. It works when a workflow carries Qorx
source, bytecode, handles, or evidence packs and has a resolver available.

## Minimal program

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

Run it from source:

```powershell
cargo run -- qorx .\goal.qorx
```

Compile it:

```powershell
cargo run -- qorx-compile .\goal.qorx --out .\goal.qorxb
cargo run -- qorx .\goal.qorxb
```

## Core model

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
| resolver surface | hzon | Relationship between local state and the model-visible carrier. |
| qshf factor | qshf | Baseline-to-Compact ratio between local context mass and visible carrier mass. |
| B2C | b2c | Baseline-to-Compact accounting. Local estimate, not a provider invoice. |
| B2C allocator | qalc | Local budgeted quark selector used by `b2c-plan` and `pack`. |

These are Qorx vocabulary labels, not physics claims. The claim discipline is in
[SAFE-R](docs/SAFE-R.md).

## Build

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

## Install from public source

Use Cargo against the current public source branch:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --branch main --locked qorx
```

Or clone the repository and build:

```sh
git clone https://github.com/bbrainfuckk/qorx.git
cd qorx
cargo test
cargo build --release
```

## Community commands

```powershell
.\target\release\qorx.exe --version
.\target\release\qorx.exe doctor --json
.\target\release\qorx.exe index .
.\target\release\qorx.exe strict-answer "what proves Qorx is a language runtime?"
.\target\release\qorx.exe b2c-plan "what proves Qorx is a language runtime?" --budget-tokens 900
.\target\release\qorx.exe pack "what proves Qorx is a language runtime?" --budget-tokens 1200
.\target\release\qorx.exe security attest
```

Community Edition focuses on source-built CLI workflows. Commands that require
the always-on product layer, such as `bootstrap`, `daemon`, `tray`, `startup`,
`drive`, `hot`, `integrate`, `run`, and `patch`, explain that those surfaces are
available in Qorx Edge. New Edge accounts start with 5,000 included Edge/Cloud
requests through Qorx Edge Starter.

## Repository map

| Path | Purpose |
| --- | --- |
| `src/` | Rust implementation of the parser, runtime components, index, protocol, and CLI. |
| `tests/` | Runtime, language, capsule, context, lattice, and strict evidence tests. |
| `docs/handbook/` | Manual-style operating documentation. |
| `docs/COMMANDS.md` | Community command catalog. |
| `docs/COMMUNITY.md` | Edition guide for Community Edition, Qorx Edge, and Qorx Cloud. |
| `docs/SERVER.md` | Runtime options for source-built Community Edition users. |
| `examples/` | Small fixtures for impact and evidence routes. |
| `scripts/` | Proof, benchmark, and maintainer checks. |

## What Qorx Does

Qorx can resolve Qorx-known local handles, bytecode, indexed evidence, and
receipts. It cannot reconstruct arbitrary unknown files from a tiny message. It
cannot make a remote model know hidden local data without a resolver path. It
does not certify task quality by token savings alone.

Qorx Edge is the supported local product experience. Do not describe forks,
community builds, or self-built binaries as official Qorx products.

## License

Copyright (c) 2026 Marvin Sarreal Villanueva.

- Code and operational docs: [AGPL-3.0-only](LICENSE)
- Citation metadata: [CITATION.cff](CITATION.cff)
- Qorx Local Context Resolution preprint: [10.5281/zenodo.19953308](https://doi.org/10.5281/zenodo.19953308)
- Contribution terms: [CONTRIBUTING.md](CONTRIBUTING.md)
- Security policy: [SECURITY.md](SECURITY.md)
- Governance: [GOVERNANCE.md](GOVERNANCE.md)
- Marks and official identity: [TRADEMARKS.md](TRADEMARKS.md)
