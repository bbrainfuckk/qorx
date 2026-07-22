# Qorx Void Boundary

This repo makes Qorx understandable and buildable.

It does not hand out the production blueprint for the commercial desktop
service.

## Public In This Repo

- Qorx CLI and local runtime source.
- `.qorx` language and `.qorxb` bytecode behavior.
- Local evidence indexing, packing, strict-answer, squeeze, context VM,
  cache-plan, Atlas, ORCL, and benchmark commands.
- Hosted Qorx API source and tests.
- Public docs for pricing, metrics, install, and proof.

## Not Published Here

- Production secrets.
- Signing keys.
- Customer ledgers.
- Provider account credentials.
- Paid license material.
- Release binaries and installer attachments.
- Private deployment operations.

The source is enough to build and audit the CLI/runtime. It is not enough to
clone Qorx Void as a paid service with the same accounts, licenses, distribution,
brand, and production infrastructure.

## Version Line

The active repo line is:

```text
1.0.5
```

All public claims should use that line unless a later release changes it.

## Brand And License

Qorx source is published under `AGPL-3.0-only` where applicable. The Qorx name,
logos, resolver scheme, and product marks are separate brand assets. Forks must
respect the license and must not imply they are the official Qorx service.
