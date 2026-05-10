# Qorx 0.0.1-ylem: local context resolution runtime in Rust

Qorx 0.0.1-ylem is a Rust CLI/runtime and small domain-specific language for local
context resolution. It defines `.qorx` source, checked `.qorxb` protobuf
bytecode, named resolver steps, branches, runtime assertions, a Forth-inspired
`qstk` stack tape, and `qorx://` handles that resolve against local state.

The project is early. This page is written for Rust reviewers who want more
than a repository link.

## What it is

A `.qorx` file describes a mode, an objective, an evidence budget, and
optionally a handle:

```text
QORX 1
let question = "which files explain how Qorx keeps local evidence outside the model prompt?"
pack evidence from question budget 700
cache evidence key question ttl 3600
strict answer from evidence limit 2
assert supported(answer)
emit answer
```

The Rust implementation parses that source, checks symbol references, lowers it
to QIR, compiles it to a bytecode envelope, and resolves it through local
index/cache/provenance state.

```sh
qorx qorx-check goal.qorx
qorx qorx-compile goal.qorx --out goal.qorxb
qorx goal.qorxb
```

## Why Rust

Qorx uses Rust because the runtime boundary matters. The parser, bytecode
envelope, cache, and receipt paths all handle local state that should fail
closed rather than guess.

The useful Rust pieces are ordinary ones:

- explicit error paths through `anyhow` and typed domain errors
- deterministic hashing for evidence chunks and receipts
- structured CLI commands with stable output modes
- tests around parser/runtime boundaries instead of only end-to-end examples
- release packaging through Cargo first, with npm/Python wrappers around the
  Rust binary

## What changed in 0.0.1-ylem

Version 0.0.1-ylem carries the current Qorx CLI and Void line while preserving the clarity and operator-facing readiness work:

- problem-first README wording
- a practical first `.qorx` example
- `use std...` imports for the first standard-library boundary
- `qorx-check`, AST output, and QIR output for language files
- source-level cache policies for stable local resolver outputs
- `assert supported(...)` for fail-closed evidence checks
- `if supported(...) then emit ... else emit ...` branches
- `qstk`, a Forth-inspired bytecode stack tape inside `.qorxb`
- README-visible benchmark numbers
- shorter terminology for compact UI/log surfaces
- `qorx doctor --json`
- daemon and gateway documentation
- Docker, compose, and systemd surfaces
- Windows release build verification
- package recipes for crates.io, npm, PyPI, AUR, Homebrew tap, and Scoop

The current `0.0.1-ylem` line is source-first until the matching package pages
and release assets are live:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --tag v0.0.1-ylem --locked qorx
```

## What needs review

The main question is not whether a CLI can parse a small file. It can. The
question is whether the boundary is worth keeping.

Useful review questions:

- Is `.qorx` a small language, or should it be described as a configuration
  format?
- Is `.qorxb` useful outside the CLI?
- Does local context resolution reduce repeated prompt payloads in real
  workflows?
- Are cache, receipt, and provenance boundaries clear enough?
- What should change before people treat Qorx as production tooling?

## Boundaries

Qorx is not a hosted AI service, a general-purpose language, a Forth
implementation, or a general compression system. It cannot reconstruct arbitrary
unknown files from a tiny message. It only works when the workflow has a Qorx
source file, bytecode file, evidence pack, or handle that a resolver can use.

That is the narrow claim. It should be tested as a Rust runtime and CLI, not as
a magic context shortcut.
