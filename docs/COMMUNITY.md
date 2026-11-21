# Qorx public source

This repository is the canonical public source for Qorx.

The public source lets people inspect the implementation, build the CLI, test
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
- release-asset recipes for maintainer tags.
- package-channel files for PyPI, npm, Arch/AUR, Homebrew, Scoop, WinGet, Snap,
  Docker, Nix, and Deb/RPM packaging through nfpm.
- live public metrics through the Qorx Community Metrics worker.

## Qorx Void And Cloud Add

- Qorx Void for Windows in the [Microsoft Store](https://apps.microsoft.com/detail/9NDC66CH9VL8?mode=direct): a free 24-hour trial, then USD 15 a month plus tax.
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

Those surfaces live in Qorx Void or Qorx Cloud. The community still gets the
source, the language, the local evidence routes, and release recipes for direct
CLI use. Release assets are live only after a maintainer attaches them to a
GitHub release.

Package-channel files in this repo install or build Qorx. They do
not add Void account services, and they do not cap local CLI usage.

## Qorx Void in the Microsoft Store

Qorx Void is the Windows app in the [Microsoft Store](https://apps.microsoft.com/detail/9NDC66CH9VL8?mode=direct). It starts with a free 24-hour
trial with no automatic charge; after that the monthly plan is USD 15 plus tax.
It adds what this repository does not carry: shared notes across Codex, Claude
Code and OpenCode, automatic skill sharing, the tray and app window, and
automatic updates through the Store.

Local source-built commands such as `index`, `pack`, `squeeze`,
`strict-answer`, `qorx-check`, `qorx-compile` and `skills share` stay local and
free in this repository. Access to the app is checked by the Qorx account
service, not inside the AGPL source binary. That keeps the source honest:
people can study and fork the local code, but the official app still needs a
valid account.

## What the community build runs

The 1.0.7 source runs every command it contains, including `daemon`, `tray`,
`integrate` and `skills`. The official product layer (the signed Store app,
account access, shared notes and automatic updates) lives outside this
repository.

## Forks

The AGPL license allows forks under its terms. The Qorx name, logo, official
release identity, and product marks are not granted for confusing use. A fork
must not imply that it is the official Qorx distribution.

## Product Lines

Qorx Void is the supported local product:

- signed installer.
- a free 24-hour trial, then USD 15 a month plus tax, through the Microsoft Store.
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

Qorx can be built for source review, experimentation, and local use.
