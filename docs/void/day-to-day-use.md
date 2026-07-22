# Day-to-day use

Qorx Void is designed to remove repeated setup from local AI work. The public
commands below describe the operator surface without exposing the private Void
implementation.

## 1. Check the local runtime

```sh
qorx doctor
qorx daemon start
qorx integrate status
```

The monitor is local by default:

```text
http://127.0.0.1:47187/monitor
```

## 2. Connect an agent

```sh
qorx install -p codex
qorx integrate activate -p codex
```

Qorx 1.0.6 recognizes these integration slugs:

```text
codex, claude, opencode, copilot, vscode, aider, claw, droid,
trae, trae-cn, gemini, hermes, kiro, pi, cursor, antigravity
```

`all` selects every supported integration and `windows` selects Windows-managed
surfaces. Support differs by client: some use MCP, some use hooks or generated
configuration, and some need a restart or manual enable step.

## 3. Index the work

```sh
qorx index .
qorx atlas
qorx session
```

Only index material you are authorized to use. The session handle identifies
local state; it is not the full file content.

## 4. Ask for the smallest useful context

```sh
qorx context inject "review the authentication boundary" --block
qorx map "which files control authentication?"
qorx squeeze "what changed in authentication?" --budget-tokens 900
qorx strict-answer "which files control authentication?"
```

When a carrier needs expansion:

```sh
qorx context fault "show the exact policy lines" --handle <qorx-handle>
qorx context expand <qorx-handle>
```

## 5. Check a proposed answer

```sh
qorx ground "authentication policy" --answer "the API accepts anonymous writes"
qorx judge "the API accepts anonymous writes" --query "authentication policy"
```

An unsupported result is useful. It means the local evidence did not justify
the claim under the current query and limit.

## 6. Inspect the accounting

```sh
qorx stats
qorx eco --local-tokens 13200000 --sent-tokens 8
```

`qorx eco` reports supplied token arithmetic. Energy, CO2e, and water outputs
remain empty unless the operator supplies scenario factors.

## 7. End or maintain the local session

```sh
qorx memory summarize
qorx context snapshot
qorx context verify
qorx security attest
qorx daemon stop
```

## Safe issue reports

Attach command versions, sanitized JSON reports, operating-system details, and
reproduction steps. Do not attach provider keys, private files, full prompts,
model response bodies, local usernames, hostnames, account data, or proprietary
Void artifacts.

The complete command and MCP inventory is in [Tools](tools.md).
