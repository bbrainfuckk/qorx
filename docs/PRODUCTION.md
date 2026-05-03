# Community status

This page defines the public Community Edition release scope.

## Verdict

Qorx Community Edition is suitable for source review, local experiments,
language/runtime testing, and research reproduction.

It is not the official production local product. It is not a hosted SaaS. It is
not a managed team runtime. It does not include signed installers, automatic
updates, tray UX, provider routing, account activation, or fleet controls.

## Ready in CE

| Surface | Status | Evidence |
| --- | --- | --- |
| `.qorx` source language | Ready | `qorx qorx <file>` and `qorx qorx-compile <file>` |
| `.qorxb` bytecode | Ready | `qorx qorx-inspect <file>` |
| Source build | Ready | `cargo test`, `cargo build --release` |
| Local indexing | Ready | `qorx index`, `qorx search` |
| Evidence commands | Ready | `qorx strict-answer`, `qorx pack`, `qorx squeeze`, `qorx judge` |
| Provenance checks | Ready | `qorx security attest`, `qorx security verify` |
| Operator check | Ready | `qorx doctor --json` |

## Qorx Edge And Cloud Add

| Surface | Product line |
| --- | --- |
| Official binaries | Qorx Edge or maintainer-controlled community channels |
| Windows tray | Qorx Edge |
| Auto-update | Qorx Edge |
| Daemon startup | Qorx Edge |
| Provider proxy routing | Qorx Edge |
| One-click CLI integrations | Qorx Edge |
| Hosted account features | Qorx API |
| Cloud capsule sync | Qorx Cloud or Qorx Edge |
| Team policy and fleet controls | Team/Enterprise product |
| Public SaaS runtime | Separate hosted product with auth, tenancy, logs, backups, and SLOs |

## CE Proof Check

Run this before publishing Community Edition claims:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
qorx --version
qorx doctor --json
qorx index .
qorx security attest
```

Do not use the CE repo to advertise official production distribution channels.
Use Qorx Edge for the supported local runtime and Qorx Cloud for hosted account
features.

## Allowed claim

Use this wording:

```text
Qorx Community Edition is the AGPL source line for the Qorx language, bytecode,
local indexing, and evidence-command model. The official local product is Qorx
Edge.
```

Do not present Community Edition as the complete official local product.
