# Qorx Community Edition

This repository is Qorx Community Edition.

Community Edition exists so people can inspect the source, build the CLI, test
the language/runtime model, verify the research claims, and use Qorx in local
evidence-first workflows. It is the public source line, not a dead sample.

## Included

- AGPL source code.
- `.qorx` source parsing.
- `.qorxb` bytecode.
- local indexing and search.
- strict evidence answers.
- B2C accounting estimates.
- SAFE-R claim checks.
- source build instructions.
- cross-platform GitHub release assets from maintainer tags.
- package-channel files for PyPI, npm, Arch/AUR, Homebrew, Scoop, WinGet, Snap,
  Docker, Nix, and Deb/RPM packaging through nfpm.
- live public metrics through the Qorx Community Metrics worker.

## Qorx Ayie And Cloud Add

- Qorx Ayie Starter with 5,000 included Ayie/Cloud requests.
- signed official installers.
- Windows tray UX.
- auto-update.
- daemon autostart.
- provider proxy routing.
- one-click MCP and hook activation for supported agent clients.
- ORCL endpoint and MCP tools.
- hosted Qorx API account features.
- cloud capsule sync.
- team policy and fleet controls.
- commercial support.

Those surfaces live in Qorx Ayie or Qorx Cloud. The community still gets the
source, the language, the local evidence routes, and release assets for direct
CLI use.

Package-channel files in this repo install or build Community Edition. They do
not add Ayie account services, and they do not cap local CLI usage.

## Qorx Ayie Starter

Starter is the generous try-it-for-real plan. It gives new accounts the full
Ayie/Cloud feature set on Windows, macOS, and Linux with 5,000 included
Ayie/Cloud requests.

Metered requests are service calls that Qorx has to run or pay for: provider
routing, ORCL lookup, MCP/CLI activation, cloud capsule sync, and account-backed
daemon/API actions. Local source-built commands such as `index`, `pack`,
`squeeze`, `strict-answer`, `qorx-check`, and `qorx-compile` stay local and
unmetered.

When the 5,000 included Ayie/Cloud requests are used, Ayie/Cloud asks the user
to subscribe before continuing with those services. This counter lives on the
Qorx account service, not inside the AGPL source binary. That keeps the source
honest: people can study and fork the local code, but official hosted capacity
still needs a valid account entitlement.

## Current CE Behavior

Commands that require the official always-on product layer return a Community
Edition edition message:

```text
bootstrap
daemon
tray
startup
drive
hot
integrate
run
patch
```

The source still carries protocol and research code so the public record remains
auditable. The official product layer lives outside this repository.

## Forks

The AGPL license allows forks under its terms. The Qorx name, logo, official
release identity, and product marks are not granted for confusing use. A fork
must not imply that it is the official Qorx distribution.

## Product Lines

Qorx Ayie is the supported local product:

- signed installer.
- 5,000 included Ayie/Cloud requests through Qorx Ayie Starter.
- local tray and updater.
- account activation.
- managed local vault UX.
- provider and CLI integration.
- MCP plus managed hooks for Codex, Gemini CLI, and Claude Code where the
  installed client exposes that surface.
- MCP-only activation for clients such as Antigravity, OpenCode, Copilot CLI,
  VS Code Copilot Chat, Aider, and Cursor until they expose a supported hook
  surface.
- ORCL ranked contracts, bounded links, and MCP tool surface.
- cloud capsule sync when enabled by the user.
- team controls and support.

Community Edition is for source review and experimentation. Qorx Ayie is the
local product customers should install.
