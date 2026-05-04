# Runtime Options

The public Community Edition is the source-built CLI line for Qorx.

In this repo, the daemon/proxy implementation is part of the Ayie product path.
The CE command line explains that the public daemon commands live in Qorx Ayie:

```text
qorx daemon
qorx daemon run
qorx daemon start
qorx daemon status
qorx daemon stop
```

Those commands are part of Qorx Ayie.

Qorx Ayie Starter gives new accounts the same Ayie/Cloud runtime path across
Windows, macOS, and Linux with 5,000 included Ayie/Cloud requests.

## Why

The daemon is the product layer that makes Qorx feel alive on a machine. It
controls the local HTTP gateway, provider routing, workstation startup, tray UX,
and tool integration path. Qorx Ayie packages that experience as the supported
local product.

## Community path

Use source-build CLI commands instead:

```sh
cargo build --release
./target/release/qorx doctor --json
./target/release/qorx index .
./target/release/qorx strict-answer "what proves Qorx is a language runtime?"
```

## Qorx Ayie Path

Use Qorx Ayie for:

- Qorx Ayie Starter account activation.
- local daemon management.
- Windows tray.
- provider proxy routing.
- auto-start.
- signed installer.
- auto-update.
- account activation.
- cloud capsule sync.
- team policy.

## Network Guidance

Do not expose self-built Qorx gateway experiments to untrusted networks. Any
shared deployment needs authentication, TLS, rate limits, logs, backups, and a
clear data-retention policy. The hosted Qorx API is the product path for that
surface.

The Starter request counter lives on the Qorx account service. Local health
checks and source-built CE commands are not counted.
