# Antigravity Quarantine Removed

Historically, the Qorx CLI explicitly quarantined Antigravity (Google's agentic assistant) from AutoMCP and context hook injection due to a bug where spawning a Qorx MCP child could stall the Antigravity agent process. Users previously had to opt-in using the `QORX_ANTIGRAVITY_MCP=1` environment variable to test compatibility.

## The Fix

The stalling issue has been resolved in the current Antigravity runtime integration. As a result, the quarantine has been **removed from the Qorx CLI default behavior**.

- **AutoMCP Enabled by Default:** Running `qorx install` or using the tray monitor will now automatically write the required MCP configuration to Antigravity's settings, enabling full native context polling.
- **Context Injection Enabled by Default:** `GEMINI.md` context injection rules are now installed automatically when Qorx hooks are active.
- **Crux Stress Tests:** Crux no longer reports Antigravity's status as "quarantined" and expects the Antigravity connectors to succeed when running `qorx crux run`.

## Verification

If you are using Antigravity, you can verify your connection to Qorx by running:

```sh
qorx integrate status
```

The output should confirm that Antigravity is active and the MCP config has been installed.

## Opting Out

If you experience any regressive stalling or performance issues, you can explicitly disable Qorx integration for Antigravity by setting the following environment variables to `0` or `false` before starting your daemon or agent session:

```env
QORX_ANTIGRAVITY_MCP=0
QORX_ANTIGRAVITY_CONTEXT_RULE=0
```
