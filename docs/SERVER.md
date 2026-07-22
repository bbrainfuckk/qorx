# Server And Daemon

Qorx is not only a CLI. The official background runtime is the daemon. The same
binary can run the local HTTP gateway in the foreground for supervisors:

```sh
qorx daemon
```

Equivalent explicit form:

```sh
qorx daemon run
```

For workstation use, control the background daemon directly:

```sh
qorx daemon start
qorx daemon status
qorx daemon stop
```

The Windows tray is optional. It is a notification-area control surface for the
daemon, not the required backend runtime. Linux and macOS use the daemon,
systemd, Docker, or another supervisor.

Default bind:

```text
127.0.0.1:47187
```

Override it with:

```sh
QORX_BIND=127.0.0.1:47187 qorx daemon
```

Use `0.0.0.0:47187` only inside a trusted container network, private subnet, or
behind a reverse proxy with authentication.

## State

Qorx stores local state under the platform app-data directory by default. For a
server, set a stable data directory:

```sh
QORX_HOME=/var/lib/qorx qorx daemon
```

Back up `QORX_HOME`. It contains the local index, quarks, cache, receipts,
provenance, and integration reports.

## Health

```sh
curl -fsS http://127.0.0.1:47187/health
curl -fsS http://127.0.0.1:47187/stats
qorx daemon status
qorx doctor --json
```

`qorx doctor` reports the active version, configured bind, data directory,
gateway health, local state files, package surfaces, and the current production
boundary.

## Routes

Core routes:

| Route | Method | Purpose |
| --- | --- | --- |
| `/health` | `GET` | Process health and runtime identity |
| `/stats` | `GET` | Request, cache, context, and local accounting stats |
| `/stats/reset` | `POST` | Reset local stats |
| `/money` | `GET` | Local cost accounting proof |
| `/session` | `GET` | Session pointer |
| `/capsule/session` | `GET` | Capsule prompt block |
| `/strict-answer` | `GET`, `POST` | Answer from indexed local evidence |
| `/squeeze` | `GET`, `POST` | Compact local evidence into a budget |
| `/judge` | `GET`, `POST` | Check an answer against local evidence |
| `/ground` | `GET`, `POST` | Run the proof-per-token grounding gate |
| `/cache-plan` | `GET`, `POST` | Explain cache behavior for a prompt |
| `/agent` | `GET`, `POST` | Build an agent-ready evidence packet |
| `/map` | `GET`, `POST` | Map likely impact from local evidence |
| `/memory` | `GET`, `POST` | Local memory operations |

Context routes:

```text
/context/vm
/context/fault
/context/inject
/context/nano
/context/quetta
/context/expand
```

Provider proxy routes:

```text
/anthropic/*
/gemini/*
/*
```

The catch-all route proxies OpenAI-compatible traffic after Qorx handling.
Keep that route behind the same network and auth controls as the daemon.

## systemd

A hardened unit template lives at:

```text
packaging/systemd/qorx.service
```

Install example:

```sh
sudo install -Dm0644 packaging/systemd/qorx.service /etc/systemd/system/qorx.service
sudo systemctl daemon-reload
sudo systemctl enable --now qorx
systemctl status qorx
curl -fsS http://127.0.0.1:47187/health
```

## Docker

Build:

```sh
docker build -t qorx:local .
```

Run loopback-only:

```sh
docker run --rm \
  -p 127.0.0.1:47187:47187 \
  -v qorx-data:/data \
  qorx:local
```

Compose:

```sh
docker compose up --build
```

The container sets:

```text
QORX_HOME=/data
QORX_BIND=0.0.0.0:47187
```

The published compose port is still loopback-only on the host.

## Boundary

Qorx does not ship a multi-user auth layer. For shared use, put it behind your
own gateway. Minimum controls are auth, TLS, rate limits, logs, backups, and a
private network path.
