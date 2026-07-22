# Production Status

This page is the public production boundary for Qorx.

## Verdict

Qorx is production-ready as a local runtime, CLI, compiler, and internal service
component.

Qorx is not yet production-ready as an exposed multi-user cloud service. The
daemon has no built-in user accounts, tenant isolation, public auth layer, or
published load-test SLO.

That distinction matters. Run it on a workstation, build runner, internal
server, or controlled automation host. Do not put the daemon on the public
internet without a reverse proxy, authentication, TLS, rate limits, logs, and
backups.

## Ready

| Surface | Status | Evidence |
| --- | --- | --- |
| `.qorx` source language | Ready | `qorx qorx-check <file>`, `qorx qorx <file>`, and `qorx qorx-compile <file>` |
| `.qorxb` bytecode | Ready | AST, QIR, opcodes, `qstk`, and `qorx qorx-inspect <file>` |
| Local runtime | Ready | `qorx index`, `qorx strict-answer`, `qorx context verify` |
| Local HTTP gateway | Ready | `qorx daemon start`, `qorx daemon status`, `/health`, `/stats`, `/strict-answer` |
| Release binaries | Automated | `release-assets.yml` builds six x64/ARM64 targets from `v1.0.5` |
| Package wrappers | Built locally | Registry channels are live only when the public package page shows `1.0.5` |
| Provenance checks | Ready | `qorx security attest`, `qorx security verify` |
| Operator check | Ready | `qorx doctor --json` |

## Not Ready

| Surface | Status | Reason |
| --- | --- | --- |
| Public multi-user API | Not ready | No built-in authentication or authorization layer |
| Tenant-hosted SaaS | Not ready | No tenant isolation model |
| Public SLO claim | Not ready | No published external load-test data |
| Managed fleet upgrades | Not ready | No migration controller or rolling update system |
| Regulated production use without controls | Not ready | Operators must add audit, retention, access, and backup policy |

## Required Controls

For an internal server, add these controls before calling it production:

```text
supervisor: systemd, Docker, Kubernetes, or another process manager
network: loopback bind by default, private subnet if remote access is needed
auth: reverse proxy auth, VPN, SSH tunnel, or mTLS
tls: terminate at the reverse proxy
data: set QORX_HOME and back it up
monitoring: watch /health, /stats, disk, memory, and process restarts
versioning: pin the binary, package revision, or release tag
restore: test restore from the QORX_HOME backup
```

## Production Gate

Run this before publishing operational claims:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
qorx --version
qorx doctor --json
qorx index .
qorx qorx-check examples/goal.qorx
qorx context verify
qorx security attest
qorx daemon status
```

Then verify the gateway:

```sh
qorx daemon start
curl -fsS http://127.0.0.1:47187/health
curl -fsS http://127.0.0.1:47187/stats
qorx daemon stop
```

Windows users can run:

```powershell
.\scripts\smoke-gateway.ps1 -Exe .\target\release\qorx.exe
```

Linux and macOS users can run:

```sh
./scripts/smoke-gateway.sh ./target/release/qorx
```

## Allowed Claim

Use this wording:

```text
Qorx is a production-ready local-first resolution runtime and internal service
component for context resolution. It includes a small DSL, compiler, bytecode
format, local daemon, HTTP gateway, daemon control commands, package surfaces,
and operator checks.
```

Do not present Qorx as a public SaaS platform until the external SaaS layer is
real: auth, tenancy, backups, monitoring, rate limits, incident handling, and
published load data.
