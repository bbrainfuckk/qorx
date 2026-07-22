# Independent review brief

Qorx needs independent technical review. This page is for editors, reviewers,
and developers who want to evaluate the project without using maintainer copy.

## What to test

Qorx is an AGPL-licensed Rust project for local context resolution. It defines:

- `.qorx` source files
- `.qorxb` compiled bytecode
- AST, QIR, canonical opcodes, and `qstk` stack tape
- `qorx://` handles resolved by a local runtime
- local evidence, cache, receipt, and provenance records

The core claim is narrow: Qorx lets a workflow carry a small program or handle
and resolve context locally, instead of repeatedly pasting large prompt payloads.

## Install

Use the source path for the current `1.0.5` line:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --tag v1.0.5 --locked qorx
```

Registry and AUR packages may exist for older public lines. Do not treat a
package channel as current until its package page shows `1.0.5`.

## Quick check

```sh
qorx --version
qorx doctor --json
qorx strict-answer "which files explain the resolver boundary?"
```

Minimal source file:

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

Compile and run:

```sh
qorx qorx-compile goal.qorx --out goal.qorxb
qorx qorx goal.qorxb
```

## Review questions

- Is `.qorx` better described as a small language or as a configuration format?
- Is `.qorxb` bytecode useful outside the CLI?
- Does `qstk` add useful stack-machine dispatch, or is QIR enough?
- Does local context resolution reduce repeated prompt payloads in real use?
- Are the resolver, cache, receipt, and provenance boundaries clear?
- What should be changed before the project is treated as production tooling?

## Boundaries

Qorx is not a hosted AI service, a general-purpose language, a Forth
implementation, or a general compression system. It cannot reconstruct arbitrary
unknown files from a tiny message. It cannot make a remote model know hidden
local data without a resolver path.

Token counts in Qorx docs are deterministic local estimates unless another
tokenizer is explicitly named.

## Maintainer disclosure

Qorx is maintained by Marvin Sarreal Villanueva. Reviews, articles, and posts
written by the maintainer are not independent coverage. Independent reviewers
should write their own conclusions, including negative ones.
