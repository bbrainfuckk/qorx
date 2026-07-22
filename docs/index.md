# Qorx handbook

![Qorx banner](assets/qorx-img.jpg)

Qorx stops AI workflows from pasting the same files into every prompt. It is an
agnostic programming language, compiler, and local bytecode runtime for humans
and AI agents.

A `.qorx` file can be a compact directive file or a named resolver program.
Qorx compiles that source to protobuf-envelope `.qorxb` bytecode and resolves it
against local state.

Current public version: `1.0.6`.

## Start here

- [Reader guide](AUDIENCE_GUIDE.md)
- [Install](INSTALL.md)
- [Language and runtime](QORX.md)
- [Technology and physics-inspired vocabulary](technology.md)
- [Research map](research.md)
- [Reference papers and public sources](REFERENCE_PAPERS.md)
- [Handbook](handbook/README.md)
- [Science notes](handbook/science.md)
- [Science](SCIENCE.md)
- [Science and math](SCIENCE_AND_MATH.md)
- [Trial guide](TRIALS.md)
- [Qorx Void handbook](void/README.md)
- [Qorx Void tools](void/tools.md)
- [Qorx Void benchmarks](void/benchmarks.md)
- [Environmental accounting](benchmarks.md#environmental-accounting)
- [Void boundary](VOID_BOUNDARY.md)
- [Live metrics](LIVE_METRICS.md)
- [Qorx metrics](METRICS.md)
- [Compiler bootstrap](SELF_HOSTING.md)
- [Command reference](COMMANDS.md)
- [Production status](PRODUCTION.md)
- [Server and daemon](SERVER.md)
- [Distribution](DISTRIBUTION.md)
- [TestSprite enterprise QA](TESTSPRITE.md)
- [Media and reviewer notes](MEDIA.md)
- [Independent review brief](INDEPENDENT_REVIEW.md)
- [Technical credibility](TECHNICAL_CREDIBILITY.md)
- [Qorx language handbook](handbook/language.md)
- [Benchmarks](benchmarks/README.md)
- [Qorx papers](papers/README.md)
- [Release 1.0.6](releases/v1.0.6.md)

## Package surfaces

The current `1.0.6` line is source-first. Use [Install](INSTALL.md).
Registry and binary package files are kept in the repo, but a channel should be
treated as live only after the matching public package page or GitHub release
asset exists for this version.

- [Source tag](https://github.com/bbrainfuckk/qorx/tree/v1.0.6)
- [Crates.io package page](https://crates.io/crates/qorx)
- [npm package page](https://www.npmjs.com/package/@brainfukk/qorx)
- [PyPI package page](https://pypi.org/project/qorx/)
- [AUR package page](https://aur.archlinux.org/packages/qorx)
- [Homebrew tap](https://github.com/bbrainfuckk/homebrew-qorx)
- [Scoop bucket](https://github.com/bbrainfuckk/scoop-qorx)

## Boundary

Qorx handles work when the receiving workflow can route them to a Qorx resolver.
Without that resolver, they are identifiers.
