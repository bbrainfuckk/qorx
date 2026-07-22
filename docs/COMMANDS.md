# Qorx CLI Command Guide

Version: 0.0.1-ylem.

This guide explains the public CLI surface in the order most people need it.
Use `qorx --help` for the live command tree and `qorx man` for the short field
manual.

## Start Here

| Command | Use it when | Example |
| --- | --- | --- |
| `qorx doctor` | You want to know whether the install is healthy. | `qorx doctor` |
| `qorx daemon start` | You need the local gateway running. | `qorx daemon start` |
| `qorx install -p codex` | You want Codex connected to Qorx. | `qorx install -p codex` |
| `qorx integrate status` | You want to see which agents are wired. | `qorx integrate status` |
| `qorx stats` | You want the local counters. | `qorx stats` |

Monitor:

```text
http://127.0.0.1:47187/monitor
```

## Connect Agents

| Command | What it does |
| --- | --- |
| `qorx install` | Installs the local runtime pieces Qorx can safely manage. |
| `qorx -i` | Shortcut for `qorx install`. |
| `qorx install -p codex` | Installs Qorx support for Codex only. |
| `qorx -i -p codex` | Shortcut for Codex install. |
| `qorx integrate activate -p codex` | Rewrites Qorx-owned Codex connector files. |
| `qorx -in -p codex` | Shortcut for `integrate activate -p codex`. |
| `qorx integrate settings --automcp false --autohook false` | Turns connector automation off. |

Plain meaning:

- MCP gives an agent a local Qorx tool.
- Hooks prepare the start of a task where the client supports it.
- Fix means Qorx rewrites its own connector files.
- Some clients need a restart after connector changes.

## Work With A Project

| Command | What it does |
| --- | --- |
| `qorx index <folder>` | Adds a folder to the local evidence index. |
| `qorx atlas` | Prints the readable workspace map. |
| `qorx atlas query "question"` | Asks Atlas what local areas matter. |
| `qorx atlas path <start> <end>` | Finds a route between two files. |
| `qorx atlas export --out qorx-atlas` | Writes a shareable local Atlas pack. |
| `qorx map "question"` | Maps a task to local files and symbols. |
| `qorx orcl "question"` | Returns ranked local evidence contracts. |
| `qorx impact "question"` | Shows what local areas a change may affect. |

## Prove A Claim

| Command | What it does |
| --- | --- |
| `qorx strict-answer "question"` | Answers only from local evidence; refuses unsupported claims. |
| `qorx squeeze "question"` | Returns a compact evidence pack. |
| `qorx pack "question"` | Builds a larger context pack under a budget. |
| `qorx ground "question" --answer "claim"` | Checks evidence, judges the claim, and shows local savings math. |
| `qorx judge "answer"` | Checks answer text against local evidence. |
| `qorx b2c-plan "question"` | Plans the smallest useful context pack. |

Use this before publishing claims:

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
qorx doctor --json
qorx index .
qorx ground "version proof" --answer "Qorx is on 0.0.1-ylem."
```

## Handles And Agent Handoff

| Command | What it does |
| --- | --- |
| `qorx session` | Prints the current local session handle. |
| `qorx context nano "objective" --block` | Creates the smallest local handoff. |
| `qorx context inject "objective" --block` | Creates a readable local handoff. |
| `qorx context vm "objective"` | Shows the full resolver contract. |
| `qorx context expand <handle>` | Expands a supported local handle. |
| `qorx context fault "query" --handle <handle>` | Pulls proof for a specific query. |
| `qorx capsule create <folder> --block` | Creates a portable project capsule. |

Boundary:

- A handle is not file content.
- The local runtime must resolve it.
- If the local index cannot prove the claim, Qorx should refuse or mark it
  unsupported.

## Runtime And State

| Command | What it does |
| --- | --- |
| `qorx daemon` | Runs the gateway in the foreground. |
| `qorx daemon run` | Same foreground service mode. |
| `qorx daemon start` | Starts the workstation background service. |
| `qorx daemon status` | Shows service state. |
| `qorx daemon stop` | Stops the service. |
| `qorx tray` | Opens the Windows tray controller. |
| `qorx stats` | Shows local counters. |
| `qorx stats reset` | Clears persisted counters. |
| `qorx startup status` | Checks startup registration. |

Default gateway:

```text
127.0.0.1:47187
```

Keep non-loopback binds behind your own auth, TLS, and network rules.

## Advanced Commands

| Command | What it does |
| --- | --- |
| `qorx qorx <file.qorx>` | Runs a Qorx task file. |
| `qorx qorx-check <file.qorx>` | Validates a task file. |
| `qorx qorx-compile <file.qorx> --out <file.qorxb>` | Compiles a task file. |
| `qorx qorx-inspect <file.qorxb>` | Inspects a compiled task artifact. |
| `qorx lexicon` | Prints the public glossary. |
| `qorx crux run --hours 1` | Runs a local integration stress pass. |
| `qorx crux report` | Shows the latest Crux state. |
| `qorx security attest` | Prints a local security attestation. |
| `qorx security verify` | Verifies supported security evidence. |
| `qorx memory summarize` | Summarizes local memory notes. |
| `qorx share capsule --to qorx-share.pb` | Exports a portable capsule. |
| `qorx adapters` | Lists optional adapters. |
| `qorx science` | Prints the scientific boundary. |
| `qorx bench` | Runs a local benchmark pack. |

## Shortcuts

```text
-i   install
-in  integrate activate
-p   platform
```

Examples:

```sh
qorx -i
qorx -i -p codex
qorx -in -p codex
```

## What Not To Claim

Qorx does not hide provider billing, does not make an outside model know files
it was never given, and does not prove a security claim that was not verified on
the local machine.
