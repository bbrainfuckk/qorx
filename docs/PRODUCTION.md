# Production Status

This page is the public production boundary for Qorx.

## Verdict

Qorx 1.0.6 is suitable for local evaluation and controlled internal use as a
runtime, CLI, compiler, and service component. Production suitability still
depends on the operator's workload, threat model, recovery plan, and validation.

Qorx is not an exposed multi-user cloud service. The daemon has no built-in user
accounts, tenant isolation, public auth layer, or published load-test SLO.

That distinction matters. Run it on a workstation, build runner, internal
server, or controlled automation host. Do not put the daemon on the public
internet without a reverse proxy, authentication, TLS, rate limits, logs, and
backups.

## Implemented and tested

| Surface | Status | Evidence |
| --- | --- | --- |
| `.qorx` source language | Implemented | `qorx qorx-check <file>`, `qorx qorx <file>`, and `qorx qorx-compile <file>` |
| `.qorxb` bytecode | Implemented | AST, QIR, opcodes, `qstk`, and `qorx qorx-inspect <file>` |
| Local runtime | Implemented | `qorx index`, `qorx strict-answer`, `qorx context verify` |
| Local HTTP gateway | Implemented; loopback by default | `qorx daemon start`, `qorx daemon status`, `/health`, `/stats`, `/strict-answer` |
| Release binaries | Automated | `release-assets.yml` builds six x64/ARM64 targets from `v1.0.6` |
| Package wrappers | Built locally | Registry channels are live only when the public package page shows `1.0.6` |
| Provenance checks | Implemented | `qorx security attest`, `qorx security verify` |
| Operator check | Implemented | `qorx doctor --json` |

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

## Scoped claim

Use this wording:

```text
Qorx 1.0.6 implements a local-first context-resolution runtime, language,
compiler, bytecode format, local daemon, HTTP gateway, package surfaces, and
operator checks. It is suitable for controlled local and internal evaluation;
deployment owners must validate production suitability for their environment.
```

Do not present Qorx as a public SaaS platform until the external SaaS layer is
real: auth, tenancy, backups, monitoring, rate limits, incident handling, and
published load data.
