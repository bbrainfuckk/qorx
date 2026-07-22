# Qorx Void tools

This page inventories the public Qorx 1.0.6 tool surface available to Void
operators and supported agents. It documents names and observable behavior, not
the proprietary Qorx Void implementation.

Use `qorx --help`, `qorx <command> --help`, and `qorx man` as the live command
reference.

## MCP tools

`qorx mcp` exposes nine stdio tools:

| Tool | Public purpose |
| --- | --- |
| `qorx.health` | Check whether the local gateway is reachable. |
| `qorx.stats` | Return local reduction, cache, context, and ledger counters. |
| `qorx.session` | Return the current `qorx://` session pointer. |
| `qorx.squeeze` | Extract a compact evidence pack for a query. |
| `qorx.map` | Map a query or diff to related local files, symbols, and edges. |
| `qorx.orcl` | Return ranked evidence contracts and bounded local links. |
| `qorx.strict_answer` | Answer from local indexed evidence only. |
| `qorx.ground` | Run evidence selection, answer checking, and optional savings arithmetic. |
| `qorx.context_inject` | Return a first-call context pointer for a task that needs local workspace evidence. |

## CLI: start, runtime, and accounting

| Command | Public purpose |
| --- | --- |
| `qorx bootstrap` | Set up the local runtime and approved integrations. |
| `qorx daemon` | Run, start, stop, or inspect the local gateway. |
| `qorx doctor` | Check the CLI, daemon, paths, configuration, and local state. |
| `qorx demo` | Show the time-limited demo status. |
| `qorx tray` | Open the Windows tray controller. |
| `qorx stats` | Show or reset local counters. |
| `qorx money` | Compare a claimed saving with local accounting. |
| `qorx eco` | Calculate token reduction and opt-in environmental scenarios. |
| `qorx startup` | Enable, disable, or inspect startup registration. |
| `qorx portable` | Initialize or inspect portable runtime state. |
| `qorx drive` | Manage supported local or RAM-backed drive helpers. |
| `qorx hot` | Inspect or install supported hot-path local state. |
| `qorx crux` | Run, stop, report, or roll back a local stress pass. |

## CLI: index, retrieve, and prove

| Command | Public purpose |
| --- | --- |
| `qorx index` | Index a folder for local evidence. |
| `qorx search` | Search the local index. |
| `qorx graph` | Print the local file and reference map as JSON. |
| `qorx atlas` | Read, query, route, merge, export, or register workspace maps. |
| `qorx graph-path` | Find a route between two local files. |
| `qorx map` | Map a question or diff to local files and symbols. |
| `qorx impact` | Show which local areas a proposed change may affect. |
| `qorx orcl` | Return ranked local evidence contracts. |
| `qorx squeeze` | Return a compact evidence pack under a budget. |
| `qorx pack` | Build a larger local context pack under a budget. |
| `qorx b2c-plan` | Plan the smallest useful context pack for a task. |
| `qorx strict-answer` | Answer only from cited local evidence. |
| `qorx judge` | Check answer text against local evidence. |
| `qorx ground` | Select proof, judge a claim, and show disclosed savings arithmetic. |
| `qorx cache-plan` | Explain how stable input could be cached. |
| `qorx bench` | Run the public local benchmark pack. |

## CLI: context, memory, and handoff

| Command | Public purpose |
| --- | --- |
| `qorx context` | Create, expand, fault, snapshot, or verify local context handles. |
| `qorx session` | Print the current local session handle. |
| `qorx memory` | Create, read, update, delete, summarize, prune, collect, or evolve local notes. |
| `qorx lattice` | Inspect formal local-memory rules and attestations. |
| `qorx capsule` | Detect, create, inspect, or query portable project capsules. |
| `qorx share` | Export or import portable sessions and capsules. |
| `qorx kv` | Emit supported local key-value cache material. |
| `qorx agent` | Build compact task context for an agent. |
| `qorx marvin` | Backward-compatible owner shortcut for compact agent context. |
| `qorx a2a` | Produce agent-to-agent cards and task helpers. |
| `qorx cosmos` | Report compatibility state for older local data. |
| `qorx aim` | Inspect AIM-compatible local context. |

## CLI: language and compiler

| Command | Public purpose |
| --- | --- |
| `qorx qorx` | Run a `.qorx` source or supported `.qorxb` task. |
| `qorx qorx-check` | Parse and validate `.qorx` source. |
| `qorx qorx-compile` | Compile `.qorx` source to `.qorxb` bytecode. |
| `qorx qorx-inspect` | Inspect a compiled `.qorxb` artifact. |
| `qorx qorx-prompt` | Render the agent-facing text from a `.qorx` file. |
| `qorx lexicon` | Print the public language and runtime glossary. |
| `qorx man` | Print the plain-language field manual. |
| `qorx science` | Print scientific claims, evidence, and limitations. |

## CLI: security, integration, and providers

| Command | Public purpose |
| --- | --- |
| `qorx attest` | Print a local attestation report. |
| `qorx security` | Attest or verify supported local security evidence. |
| `qorx adapters` | List optional local adapters. |
| `qorx install` | Install Qorx and connect selected supported agents. |
| `qorx integrate` | Activate, deactivate, inspect, or configure agent connectors. |
| `qorx mcp` | Run the local MCP stdio server. |
| `qorx run` | Run a provider client through the public Qorx shim contract. |
| `qorx patch` | Ask a configured provider client for a patch, with explicit apply control. |

## Nested actions

| Group | Actions |
| --- | --- |
| `daemon` | `run`, `start`, `stop`, `status` |
| `stats` | `reset` |
| `atlas` | `export`, `query`, `path`, `merge`, `global add`, `global list`, `global path`, `hook` |
| `a2a` | `card`, `task` |
| `memory` | `create`, `read`, `update`, `delete`, `summarize`, `prune`, `gc`, `evolve` |
| `lattice` | `build`, `status`, `attest`, `kv-hints`, `evolve-rules`, `rules` |
| `share` | `export`, `capsule`, `import`, `session` |
| `kv` | `emit` |
| `security` | `attest`, `verify` |
| `hot` | `status`, `install` |
| `capsule` | `auto`, `detect`, `create`, `session`, `strict-answer` |
| `context` | `snapshot`, `verify`, `vm`, `fault`, `inject`, `nano`, `quetta`, `expand` |
| `startup` | `enable`, `disable`, `status` |
| `portable` | `init`, `status` |
| `drive` | `init`, `mount`, `unmount`, `status`, `install-startup`, `remove-startup`, `install-imdisk` |
| `integrate` | `activate`, `deactivate`, `status`, `settings` |
| `crux` | `run`, `stop`, `report`, `rollback` |

## Supported integration names

```text
codex, claude, opencode, copilot, vscode, aider, claw, droid,
trae, trae-cn, gemini, hermes, kiro, pi, cursor, antigravity
```

`all` and `windows` are selection groups. Availability and activation behavior
depend on the installed client. Provider credentials remain in the provider's
own client; Qorx does not copy them.

## Boundary

This inventory is intentionally complete for the public 1.0.6 command and MCP
surface. It does not enumerate private Void services, methods, algorithms, or
production operations.
