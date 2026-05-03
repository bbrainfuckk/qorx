# Registry Automation

Qorx registry publishing is automated by `.github/workflows/publish-registries.yml`.

The workflow is safe to rerun. It checks whether the current version already
exists before publishing.

## GitHub Secrets

Set these repository secrets:

```text
CARGO_REGISTRY_TOKEN
NPM_TOKEN
PYPI_API_TOKEN
AUR_SSH_PRIVATE_KEY
```

Do not store npm recovery codes in GitHub Secrets. Recovery codes are for
account recovery and interactive 2FA emergencies, not CI publishing.

For AUR, use a dedicated SSH key whose public key is added to the AUR account
that maintains `qorx`. Store only the private key as `AUR_SSH_PRIVATE_KEY`.

## Current Version Mapping

Qorx uses the version form each registry expects:

```text
Cargo/crates.io: 1.0.4-a.0
npm:             1.0.4-a.0
PyPI:            1.0.4a0
Arch/AUR:        1.0.4_a.0
Release tag:     v1.0.4a
```

For npm prereleases, the workflow publishes under the `next` dist-tag. Stable
versions publish under `latest`.

For PyPI prereleases, users install with:

```text
pip install --pre qorx
```

or:

```text
pip install qorx==1.0.4a0
```

## Why Rotate Pasted Tokens

Tokens pasted into chat should be treated as exposed. Rotation is cheap compared
with debugging a compromised registry account. The durable path is:

1. create a scoped registry token;
2. put it in GitHub Actions Secrets;
3. publish from the workflow;
4. rotate the token on a normal schedule or immediately after exposure.
