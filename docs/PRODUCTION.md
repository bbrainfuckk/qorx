# Community status

This page defines the public Community Edition release scope.

## Verdict

Qorx Community Edition is suitable for source review, local experiments,
language/runtime testing, and research reproduction.

The supported production path is Qorx Edge for local installs and Qorx Cloud for
hosted account features. Qorx Edge Starter gives new accounts 5,000 included
Edge/Cloud requests across Windows, macOS, and Linux before subscription.

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
| 5,000 included Edge/Cloud requests | Qorx Edge Starter |
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

Use the CE repo for public source, tests, and local proof. Use Qorx Edge for the
supported local runtime and Qorx Cloud for hosted account features.

## Allowed claim

Use this wording:

```text
Qorx Community Edition is the AGPL source line for the Qorx language, bytecode,
local indexing, and evidence-command model. Qorx Edge is the supported local
product, and Qorx Edge Starter includes 5,000 Edge/Cloud requests before
subscription.
```
