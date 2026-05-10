# Qorx handbook

![Qorx banner](assets/qorx-img.jpg)

Qorx stops AI workflows from pasting the same files into every prompt. Qorx is
a small domain-specific language and local runtime for context resolution.

A `.qorx` file can be a compact directive file or a named resolver program.
Qorx compiles that source to protobuf-envelope `.qorxb` bytecode and resolves it
against local state.

Current public version: `0.0.1-ylem`.

## Start here

- [Install](INSTALL.md)
- [Language and runtime](QORX.md)
- [Handbook](handbook/README.md)
- [Science notes](handbook/science.md)
- [Science](SCIENCE.md)
- [Science and math](SCIENCE_AND_MATH.md)
- [Trial guide](TRIALS.md)
- [Void boundary](VOID_BOUNDARY.md)
- [Live metrics](LIVE_METRICS.md)
- [Qorx metrics](METRICS.md)
- [Community guide](COMMUNITY.md)
- [Command reference](COMMANDS.md)
- [Production status](PRODUCTION.md)
- [Server and daemon](SERVER.md)
- [Distribution](DISTRIBUTION.md)
- [TestSprite enterprise QA](TESTSPRITE.md)
- [Media and reviewer notes](MEDIA.md)
- [Independent review brief](INDEPENDENT_REVIEW.md)
- [Technical credibility](TECHNICAL_CREDIBILITY.md)
- [Qorx 0.0.1-ylem for Rust reviewers](QORX_1_0_4_RUST.md)
- [Benchmarks](benchmarks/README.md)
- [Qorx papers](papers/README.md)
- [Release 0.0.1-ylem](releases/v0.0.1-ylem.md)

## Package surfaces

- [GitHub release](https://github.com/bbrainfuckk/qorx/releases/tag/v0.0.1-ylem)
- [Crates.io](https://crates.io/crates/qorx)
- [npm](https://www.npmjs.com/package/@brainfukk/qorx)
- [PyPI](https://pypi.org/project/qorx/)
- [AUR](https://aur.archlinux.org/packages/qorx)
- [Homebrew tap](https://github.com/bbrainfuckk/homebrew-qorx)
- [Scoop bucket](https://github.com/bbrainfuckk/scoop-qorx)

## Boundary

Qorx handles work when the receiving workflow can route them to a Qorx resolver.
Without that resolver, they are identifiers.
