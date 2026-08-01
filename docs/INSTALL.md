# Installing Qorx

Qorx ships as one Rust binary. Package managers are thin wrappers around that
binary; the runtime is still the same `qorx` command.

Run `qorx` with no arguments to see the ASCII splash. Use `qorx --help` for the
command tree and `qorx man` for the manual.

## Source

This is the `1.0.6` source install. Platform release assets use the same tag.

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --tag v1.0.6 --locked qorx
qorx --version
```

For local development:

```sh
git clone https://github.com/bbrainfuckk/qorx.git
cd qorx
cargo test
cargo build --release
```

## Binary Release

Release automation builds these platform assets:

```text
qorx-v1.0.6-windows-x64.zip
qorx-v1.0.6-windows-arm64.zip
qorx-v1.0.6-linux-x64.tar.gz
qorx-v1.0.6-linux-x64-static.tar.gz
qorx-v1.0.6-linux-arm64.tar.gz
qorx-v1.0.6-macos-x64.tar.gz
qorx-v1.0.6-macos-arm64.tar.gz
```

When those assets exist, extract the archive and put the directory containing
`qorx` or `qorx.exe` on `PATH`. The `linux-x64-static` musl build is the
portable choice for older Linux environments such as DataCamp's glibc 2.27
sandbox; the `linux-x64` build uses the normal GNU libc target.

## Package Managers

Each registry is current only when its public package page shows
`1.0.6`. Until then, use the source install above. Older registry packages
may still point at a previous public line.

```sh
npm install -g @brainfukk/qorx
pnpm add -g @brainfukk/qorx
pipx install qorx
yay -S qorx
```

pnpm installs the npm package; there is no separate pnpm registry release.

Release assets can also be installed directly before a central registry accepts
the package, but only after the GitHub release asset exists:

```sh
npm install -g https://github.com/bbrainfuckk/qorx/releases/download/<tag>/<npm-tarball>
pipx install https://github.com/bbrainfuckk/qorx/releases/download/<tag>/<python-wheel>
```

Linux distribution recipes live under `packaging/`:

```text
packaging/aur/PKGBUILD
packaging/debian/
packaging/rpm/qorx.spec
packaging/systemd/qorx.service
snap/snapcraft.yaml
```

Windows package-manager manifests live under:

```text
packaging/scoop/qorx.json
packaging/winget/manifests/
```

## Boundary

If a registry package is not live yet, use the source install command above.
Publishing to npm, PyPI, crates.io, AUR, Homebrew taps, Snapcraft, WinGet, and
Scoop requires maintainer credentials for those services.

## Server

The same binary runs the local daemon:

```sh
qorx daemon start
qorx daemon status
qorx daemon stop
qorx daemon
curl -fsS http://127.0.0.1:47187/health
qorx doctor --json
```

`qorx daemon` runs in the foreground for supervisors. `qorx daemon start` runs a
background daemon for workstation use. On Windows, `qorx startup enable` installs
a login startup script for the daemon and tray. The tray is Windows-only.

Docker and service templates are included:

```text
Dockerfile
docker-compose.yml
packaging/systemd/qorx.service
```

Read [Server And Daemon](SERVER.md) and [Production Status](PRODUCTION.md)
before exposing the daemon outside loopback.

## AutoMCP And AutoHook

Qorx can wire its local MCP server and prompt hook connectors into supported AI
clients:

```sh
qorx install
qorx -i
qorx install --platform codex
qorx install -p codex
qorx -i -p codex
qorx install --platform claude
qorx install --platform gemini
qorx install --platform antigravity
qorx install --platform opencode
qorx install --platform copilot
qorx install --platform vscode
qorx install --platform aider
qorx install --platform claw
qorx install --platform droid
qorx install --platform trae
qorx install --platform trae-cn
qorx install --platform hermes
qorx install --platform kiro
qorx install --platform pi
qorx install --platform cursor
qorx integrate activate -p antigravity
qorx -in -p antigravity
qorx integrate settings --automcp true --autohook false
```

`qorx install` and first-run bootstrap are explicit local setup actions. They
install the local daemon, PATH shims, Qorx AutoMCP configs, and AutoHook
connector files where the target client supports hook loading.
`qorx integrate status` reports the active state.

The local monitor and Windows tray expose the same switches. AutoMCP and
AutoHook start on by default after setup and stay on until you turn them off:

```text
http://127.0.0.1:47187/monitor
AutoMCP
AutoHook
Turn on MCP + hooks
Turn off MCP + hooks
```

For automation:

```sh
curl -fsS http://127.0.0.1:47187/integrations
curl -fsS -X POST http://127.0.0.1:47187/integrations/settings \
  -H "content-type: application/json" \
  -d '{"automcp_enabled":true,"autohook_enabled":false}'
curl -fsS -X POST http://127.0.0.1:47187/integrations/activate \
  -H "content-type: application/json" \
  -d '{"platform":"all"}'
```

Platform behavior is explicit:

- Codex, Gemini CLI, and Claude Code get managed MCP plus hook wiring.
- Antigravity is quarantined from AutoMCP for now. This Antigravity build can
  stall when a Qorx MCP child is spawned, so Qorx removes its Antigravity MCP and
  prompt-rule entries unless `QORX_ANTIGRAVITY_MCP=1` is set for an explicit
  compatibility test. Qorx does not write global AGENTS.md/GEMINI.md injection
  rules for Antigravity by default.
- OpenCode, Copilot CLI, VS Code Copilot Chat, Aider, and Cursor are MCP-only
  unless those clients expose a supported hook surface.
- OpenClaw, Factory Droid, Trae, Trae CN, Hermes, Kiro, and Pi get MCP plus a
  local hook kit that the client may still need to load or enable.

Most clients need a restart or reload after MCP config changes. Qorx reports a
target as hook-active only when it can see the installed hook files.

Shortcut lexicon:

```text
-i   install
-in  integrate activate
-p   platform
```
