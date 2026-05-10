# Qorx

Stop resending the same AI context.

AI tools get expensive when they keep receiving the same repo, notes, logs,
policies, and project rules. Qorx keeps that repeated context local, sends a
small carrier, and pulls cited proof only when a task needs it.

The first proof:

| Scenario | Number | Meaning |
| --- | ---: | --- |
| Current public repo benchmark | 388,573 local tokens -> 69 sent | 5,631.49x smaller session carrier |
| Website planning model, 2,000-person team | 42.5B repeated input tokens/year kept local | Bounded estimate, not a provider invoice |

The point is not mystery. Qorx does not make tokens vanish. It shows what
stayed local, what was sent, and what proof was selected.

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19875352.svg)](https://doi.org/10.5281/zenodo.19875352)
[![Preprint DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19953308.svg)](https://doi.org/10.5281/zenodo.19953308)
[![Software Heritage](https://img.shields.io/badge/Software%20Heritage-archived-ff6600)](https://archive.softwareheritage.org/browse/origin/directory/?origin_url=https://github.com/bbrainfuckk/qorx)
[![License: AGPL-3.0-only](https://img.shields.io/github/license/bbrainfuckk/qorx?color=blue)](LICENSE)
[![Rust stable](https://img.shields.io/badge/rust-stable-orange?logo=rust)](https://www.rust-lang.org/)

## What Qorx Does

Qorx is a local context runtime and small `.qorx` language.

In plain terms:

1. It indexes a workspace on your machine.
2. It sends a small carrier or evidence pack to the AI workflow.
3. It pulls cited proof only when a task needs it.
4. It marks unsupported claims instead of pretending the evidence exists.

The core boundary is simple:

```text
large local context -> small model-visible carrier -> cited proof on demand
```

That is the claim. The rest of the repo exists to make the claim buildable,
testable, and bounded.

![Qorx banner](docs/assets/qorx-img.jpg)

## What It Is Not

Qorx is not a prompt trick, a billing bypass, or magic compression.

It cannot make a remote model read hidden local files without a Qorx resolver.
It does not prove answer quality just because fewer tokens were sent. It does
not claim provider invoice savings unless routed provider billing evidence
exists.

## Current Line

Current public version: `0.0.1-ylem`.

This repo is source-first right now. The source tag exists. Binary installers
and release assets are not attached yet.

Install from source:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --tag v0.0.1-ylem --locked qorx
qorx --version
```

For local development:

```sh
git clone https://github.com/bbrainfuckk/qorx.git
cd qorx
cargo test
cargo build --release
```

Package recipes are included, but a package channel should be treated as live
only when its public package page shows `0.0.1-ylem`.

## Proof In Numbers

The current public benchmark indexed this repo and measured how much context
had to be visible to the model.

| Case | Local context | Sent to model | Local reduction |
| --- | ---: | ---: | ---: |
| Session carrier | 388,573 tokens | 69 tokens | 5,631.49x |
| Evidence pack | 388,573 tokens | 410 tokens | 947.74x |
| Squeeze extract | 388,573 tokens | 448 tokens | 867.35x |

These are deterministic Qorx estimates using `ceil(chars / 4)`. They show how
much local context stayed out of the model-visible request. They are not a
provider bill.

Source:

- [Live benchmark](docs/benchmarks/live.md)
- [Live benchmark JSON](docs/benchmarks/live.json)

## What That Can Mean For Cost

The cost math is intentionally plain:

```text
avoided input cost = omitted input tokens / 1,000,000 * input price
```

For the session carrier above, Qorx omitted about 388,504 estimated input
tokens. At $2 per 1M input tokens, that is about $0.78 of repeated input avoided
for one context send. At $5 per 1M input tokens, it is about $1.94.

For planning, the public website uses this team model:

```text
100,000 repeated input tokens per developer per workday
85% of that repeated input kept local
250 workdays per year
example input prices from $2 to $5 per 1M tokens
```

That gives this estimate:

| Team size | Avoided input tokens per day | Avoided input tokens per year | Estimated yearly range |
| ---: | ---: | ---: | ---: |
| 100 people | 8,500,000 | 2.125B | $4,250 to $10,625 |
| 500 people | 42,500,000 | 10.625B | $21,250 to $53,125 |
| 2,000 people | 170,000,000 | 42.5B | $85,000 to $212,500 |

Use your own provider rate. Output tokens, new input, provider cache behavior,
discounts, and account terms can change the real bill.

## Which Qorx Should I Try?

Qorx has two product paths around the same idea.

| If your repeated context lives in... | Start with | Trial |
| --- | --- | --- |
| Your computer, editor, repos, notes, logs, and long AI sessions | Qorx Void Desktop | Free 24-hour local demo |
| Your app, dashboard, support bot, n8n flow, or hosted agent call | Qorx Cloud API | Free 5,000 hosted calls |

This public repo is the buildable CLI/runtime source line. Qorx Void Desktop is
the finished desktop product around the runtime: account, license, support,
installer flow, tray controls, and managed local operation.

Read: [Trials](docs/TRIALS.md) and [Void boundary](docs/VOID_BOUNDARY.md).

## Start Here

Pick the path that matches your question.

| Question | Start with |
| --- | --- |
| I want to use Qorx with AI tools and local repos. | [Reader guide](docs/AUDIENCE_GUIDE.md), [Manual](docs/MANUAL.md), [Commands](docs/COMMANDS.md) |
| I want to understand token and cost impact. | [Reader guide](docs/AUDIENCE_GUIDE.md), [Metrics](docs/METRICS.md), [Live benchmark](docs/benchmarks/live.md) |
| I want to review the runtime as software. | [Qorx for Rust reviewers](docs/QORX_1_0_4_RUST.md), [Production status](docs/PRODUCTION.md), [Server and daemon](docs/SERVER.md) |
| I want the science and math boundary. | [Science](docs/SCIENCE.md), [Science and math](docs/SCIENCE_AND_MATH.md), [SAFE-R](docs/SAFE-R.md) |

## First Commands

After installing:

```sh
qorx doctor
qorx daemon start
qorx index .
qorx strict-answer "which files explain the resolver boundary?"
```

Open the local monitor:

```text
http://127.0.0.1:47187/monitor
```

Useful command groups:

```sh
qorx --help
qorx man
qorx stats
qorx atlas
qorx context snapshot
qorx context verify
qorx security attest
```

## Minimal `.qorx` File

```text
QORX 1
use std.evidence
use std.branch as br
let question = "which files explain how Qorx keeps local evidence outside the model prompt?"
let fallback = "local evidence does not support this answer"
pack evidence from question budget 700
cache evidence key question ttl 3600
strict answer from evidence limit 2
if supported(answer) then emit answer else emit fallback
```

Check it:

```sh
qorx qorx-check goal.qorx
qorx qorx-compile goal.qorx --out goal.qorxb
qorx goal.qorxb
```

## Core Terms

You do not need these terms to start. They help when reading the code and
benchmark output.

| Term | Meaning |
| --- | --- |
| `.qorx` | Human-readable Qorx source file. |
| `.qorxb` | Checked protobuf-envelope bytecode. |
| carrier | Small model-visible object: source, bytecode, handle, or evidence pack. |
| `qorx://s/...` | Session handle for indexed local state. |
| evidence pack | Selected cited local evidence under a token budget. |
| resolver boundary | Line between local state and model-visible text. |
| B2C | Baseline-to-Compact accounting. Local estimate, not a provider invoice. |

Qorx also has compact vocabulary for logs and UI. Those names are labels, not
physics claims. The full boundary is in [SAFE-R](docs/SAFE-R.md).

## Build And Test

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

Proof checks:

```sh
qorx --version
qorx doctor --json
qorx index .
qorx session
qorx b2c-plan "which files explain the resolver boundary?" --budget-tokens 900
qorx strict-answer "which files explain the resolver boundary?"
qorx context snapshot
qorx context verify
qorx security attest
```

## Repository Map

| Path | Purpose |
| --- | --- |
| `src/` | Rust parser, runtime, resolver, index, cache, daemon, protocol, and CLI. |
| `tests/` | Runtime, language, capsule, context, lattice, MCP, and strict evidence tests. |
| `docs/` | Manual, command guide, science notes, metrics, production boundary, and reviews. |
| `docs/benchmarks/` | Reproducible local benchmark reports. |
| `packages/` | npm and Python wrapper sources. |
| `packaging/` | Linux, Windows, macOS, systemd, and registry recipes. |
| `scripts/` | Release, proof, distribution, and safety helpers. |

## Main Docs

- [Reader guide](docs/AUDIENCE_GUIDE.md)
- [Manual](docs/MANUAL.md)
- [Command guide](docs/COMMANDS.md)
- [Install guide](docs/INSTALL.md)
- [Metrics](docs/METRICS.md)
- [Production status](docs/PRODUCTION.md)
- [Server and daemon](docs/SERVER.md)
- [Science](docs/SCIENCE.md)
- [Science and math](docs/SCIENCE_AND_MATH.md)
- [Reference papers](docs/REFERENCE_PAPERS.md)
- [Independent review brief](docs/INDEPENDENT_REVIEW.md)
- [Technical credibility](docs/TECHNICAL_CREDIBILITY.md)
- [Release notes](docs/releases/v0.0.1-ylem.md)

## License And Marks

Copyright (c) 2026 Marvin Sarreal Villanueva.

- Code and operational docs: [AGPL-3.0-only](LICENSE)
- Citation metadata: [CITATION.cff](CITATION.cff)
- Qorx Local Context Resolution preprint: [10.5281/zenodo.19953308](https://doi.org/10.5281/zenodo.19953308)
- Contribution terms: [CONTRIBUTING.md](CONTRIBUTING.md)
- Security policy: [SECURITY.md](SECURITY.md)
- Governance: [GOVERNANCE.md](GOVERNANCE.md)
- Marks and project identity: [TRADEMARKS.md](TRADEMARKS.md)
