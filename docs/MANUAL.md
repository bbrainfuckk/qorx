# Qorx CLI Manual

Version: 0.0.1-ylem.

Qorx CLI is the command line for the local Qorx runtime. It starts the gateway,
connects supported AI agents, indexes local projects, and asks for proof without
dumping the whole workspace into chat.

Qorx Void is the runtime. Qorx CLI controls it.

## Pick What You Want To Do

| I want to | Run this | You should see |
| --- | --- | --- |
| Check Qorx | `qorx doctor` | A health report. |
| Start Qorx | `qorx daemon start` | A gateway on `127.0.0.1:47187`. |
| Connect Codex | `qorx install -p codex` | Codex connector files repaired. |
| See connectors | `qorx integrate status` | MCP, hook, and installed states. |
| Open the monitor | `http://127.0.0.1:47187/monitor` | Counters, map, and controls. |
| Read a project | `qorx index .` | The current folder added to the local index. |
| See the map | `qorx atlas` | Important files and local file links. |
| Ask with proof | `qorx strict-answer "question"` | An answer with local evidence, or a refusal. |
| Turn Qorx off | `qorx integrate settings --automcp false --autohook false` | Agent connectors stop being prepared. |

## First Run

```sh
qorx doctor
qorx daemon start
qorx install -p codex
qorx integrate status
```

Then open:

```text
http://127.0.0.1:47187/monitor
```

If the monitor does not open, run:

```sh
qorx daemon status
qorx daemon start
```

## Connect Agents

Connect everything Qorx can manage:

```sh
qorx install
qorx -i
```

Connect one agent:

```sh
qorx install -p codex
qorx -i -p codex
qorx integrate activate -p codex
qorx -in -p codex
```

Legacy example that still works:

```sh
qorx -in -p antigravity
```

Plain meaning:

- MCP gives an agent a local Qorx tool.
- Hooks prepare the start of a task where the client supports it.
- Fix in the monitor rewrites Qorx-owned connector files.
- Some clients need a restart after connector changes.

Qorx does not copy provider secrets. Each agent keeps its own login and auth.

## Work With A Project

Start inside the project folder:

```sh
qorx index .
qorx atlas
```

Use the map when you do not know where to start:

```sh
qorx atlas query "what should I read first?"
qorx map "change monitor wording"
qorx orcl "where is the CLI manual?"
```

Use proof mode when a claim matters:

```sh
qorx strict-answer "what version is this repo on?"
qorx ground "version proof" --answer "Qorx is on 0.0.1-ylem."
```

## Counters

```sh
qorx stats
qorx stats reset
```

- Kept here: context Qorx did not send upstream.
- Sent to AI: context that did go to the provider.
- Reduction: local estimate based on kept versus sent context.
- Avoided input cost: local estimate, not a provider invoice.

Provider billing is decided by the provider. Qorx reports local accounting
unless routed provider telemetry proves a billable outcome.

## Fix Common Problems

| Problem | Run |
| --- | --- |
| Monitor is offline | `qorx daemon start` |
| Codex does not see Qorx | `qorx install -p codex` |
| Status looks stale | `qorx integrate status` |
| You want a clean restart | `qorx daemon stop` then `qorx daemon start` |
| Counters look old | `qorx stats reset` |
| A claim is unsupported | `qorx strict-answer "the claim"` |

## Agent Handoff

Use these when an agent needs local proof without reading the whole workspace:

```sh
qorx context nano "objective" --block
qorx context inject "objective" --block
qorx session
```

Plain meaning:

- A handle is not a file dump.
- The local runtime must resolve the handle before evidence is visible.
- If Qorx cannot prove a claim, the correct answer is that the local index does
  not prove it.

## Reference

```sh
qorx man install
qorx man daemon
qorx man stats
qorx man atlas
qorx man crux
qorx man lexicon
qorx --help
```

Shortcuts:

```text
-i   install
-in  integrate activate
-p   platform
```

Advanced tools:

```sh
qorx crux run --hours 1
qorx crux report
qorx qorx <file.qorx>
qorx qorx-check <file.qorx>
qorx qorx-compile <file.qorx> --out <file.qorxb>
qorx lexicon
```

## Boundary

Qorx is local-first context and proof infrastructure. It does not hide provider
billing, does not make an outside model know files it was never given, and does
not make security claims that were not verified on this machine.
