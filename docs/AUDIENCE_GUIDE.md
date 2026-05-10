# Reader Guide

Qorx has a few doors in. Pick the one that matches your question. This is not a
ranking of readers.

## I Build With AI Tools

Qorx is for the moment when an AI tool needs your repo, notes, logs, or project
rules again and again.

Without Qorx, the usual move is to paste more context. With Qorx, the local
runtime indexes the workspace, sends a small carrier, and pulls cited proof only
when a task needs it.

Start here:

```sh
qorx doctor
qorx daemon start
qorx index .
qorx strict-answer "which files explain the resolver boundary?"
```

Read next:

- [CLI manual](MANUAL.md)
- [Command guide](COMMANDS.md)
- [Install guide](INSTALL.md)

## I Care About Cost

Qorx measures repeated input that stayed local. That is useful for cost
planning, but it is not the same as a provider invoice.

Current public benchmark:

| Case | Local context | Sent to model | Local reduction |
| --- | ---: | ---: | ---: |
| Session carrier | 388,573 tokens | 69 tokens | 5,631.49x |
| Evidence pack | 388,573 tokens | 410 tokens | 947.74x |
| Squeeze extract | 388,573 tokens | 448 tokens | 867.35x |

Simple estimate:

```text
avoided input cost = omitted input tokens / 1,000,000 * input price
```

For the session carrier above, Qorx omitted about 388,504 estimated input
tokens. At an example input price of $2 per 1M tokens, that is about $0.78 of
repeated input avoided for that one context send. At $5 per 1M tokens, it is
about $1.94.

The website planning example uses a company model:

```text
100,000 repeated input tokens per person using AI tools per workday
85% of that repeated input kept local
250 workdays per year
example input prices from $2 to $5 per 1M tokens
```

That gives this planning range:

| People using AI tools | Avoided input tokens per day | Estimated yearly range |
| ---: | ---: | ---: |
| 100 people | 8,500,000 | $4,250 to $10,625 |
| 500 people | 42,500,000 | $21,250 to $53,125 |
| 2,000 people | 170,000,000 | $85,000 to $212,500 |

Use your own provider rate. Output tokens, new input, discounts, provider cache
rules, and account contracts can change the real bill.

Read next:

- [Metrics](METRICS.md)
- [Live benchmark](benchmarks/live.md)
- [Trials](TRIALS.md)
- [Enterprise AI spend calculator](https://qorx.orin.work/#enterprise-calculator)

## I Review Code Or Operations

Treat Qorx like a local runtime with explicit boundaries.

Check the basics:

```sh
qorx --version
qorx doctor --json
qorx context snapshot
qorx context verify
qorx security attest
```

Then inspect the implementation:

- `src/` for the Rust runtime, CLI, index, daemon, context VM, and proof tools.
- `tests/` for parser, runtime, capsule, context, lattice, and MCP checks.
- `docs/COMMANDS.md` for the public command surface.
- `docs/PRODUCTION.md` for what is ready and what is not.

Read next:

- [Qorx for Rust reviewers](QORX_1_0_4_RUST.md)
- [Production status](PRODUCTION.md)
- [Server and daemon](SERVER.md)

## I Check The Science Or Math

The narrow claim is measurable:

```text
large local context -> small carrier -> cited proof on demand
```

Qorx does not claim universal compression. It does not claim a remote model can
read hidden files. It does not claim provider invoice savings without provider
billing evidence.

What should hold:

- Same indexed input should produce stable hashes and handles.
- A context pack should stay within its budget.
- Unsupported claims should be refused or marked unsupported.
- Local reduction ratios should state the token estimator.
- Provider-bill claims should require routed provider evidence.

Read next:

- [Science](SCIENCE.md)
- [Science and math](SCIENCE_AND_MATH.md)
- [SAFE-R anti-hype gate](SAFE-R.md)
- [Reference papers](REFERENCE_PAPERS.md)

## Shared Rule

Do not sell token reduction as truth. Qorx should show what stayed local, what
was sent, what evidence was selected, and where the claim stops.
