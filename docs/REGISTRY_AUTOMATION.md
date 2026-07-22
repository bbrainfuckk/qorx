# Registry Automation

Qorx registry publishing is automated by `.github/workflows/publish-registries.yml`.

The workflow is safe to rerun. It checks whether the current version already
exists before publishing.

## GitHub Secrets

Set these repository secrets:

```text
CARGO_REGISTRY_TOKEN
NPM_TOKEN
```

Do not store npm recovery codes in GitHub Secrets. Recovery codes are for
account recovery and interactive 2FA emergencies, not CI publishing.

PyPI uses trusted publishing through the GitHub environment named `pypi`, so no
long-lived PyPI token is stored in this repository.

## Current Version Mapping

Qorx uses the version form each registry expects. This is the intended
`1.0.6` mapping; a channel is not live until its package page or release
asset exists publicly.

```text
Cargo/crates.io: 1.0.6
npm:             1.0.6
PyPI:            1.0.6
Arch/AUR:        1.0.6
Source tag:      v1.0.6
```

For npm prereleases, the workflow publishes under the `next` dist-tag. Stable
versions publish under `latest`.

For PyPI prereleases, users install with:

```text
pip install --pre qorx
```

or:

```text
pip install qorx==1.0.6
```

## Why Rotate Pasted Tokens

Tokens pasted into chat should be treated as exposed. Rotation is cheap compared
with debugging a compromised registry account. The durable path is:

1. create a scoped registry token;
2. put it in GitHub Actions Secrets;
3. publish from the workflow;
4. rotate the token on a normal schedule or immediately after exposure.
