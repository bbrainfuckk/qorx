# Qorx Community Edition

![Qorx banner](assets/qorx-img.jpg)

This is the public AGPL source line for Qorx.

Qorx is a small domain-specific language, compiler, and local context-resolution
CLI. Community Edition covers the language, bytecode, local indexing, evidence
commands, reproducible benchmarks, live public metrics, and community CLI
release assets.

Current public version: `1.0.4`.

## Start here

- [Community guide](COMMUNITY.md)
- [Install from source](INSTALL.md)
- [Live metrics](LIVE_METRICS.md)
- [Language and runtime](QORX.md)
- [Handbook](handbook/README.md)
- [Science notes](handbook/science.md)
- [Command reference](COMMANDS.md)
- [Runtime options](SERVER.md)
- [TestSprite QA](TESTSPRITE.md)
- [SAFE-R claim check](SAFE-R.md)
- [Technical credibility](TECHNICAL_CREDIBILITY.md)
- [Independent review brief](INDEPENDENT_REVIEW.md)
- [Qorx 1.0.4 for Rust reviewers](QORX_1_0_4_RUST.md)
- [Benchmarks](benchmarks/README.md)
- [Qorx papers](papers/README.md)

## Public surface

- source build with Cargo.
- cross-platform release assets from maintainer tags.
- `.qorx` and `.qorxb` language/runtime checks.
- local evidence commands.
- claim-check docs.
- tests and benchmark fixtures.
- daily metrics worker backed by public GitHub proof.

## Product Lines

Qorx Edge is the supported local product. Signed installers, auto-update, tray,
daemon management, provider routing, one-click CLI integrations, ORCL, cloud
capsule sync, and team policy are not shipped from this public repository.

Commands that need the always-on product layer return a clear Community Edition
edition message.
