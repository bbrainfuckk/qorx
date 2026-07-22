# Distribution

This file is for maintainers cutting Qorx packages.

## Local Gate

Use the release prep script for normal version bumps. It updates Cargo,
`package.json`, npm, PyPI, AUR, RPM, Debian, Snap, Scoop, WinGet, Homebrew,
CITATION, Zenodo, and release docs.

```powershell
.\scripts\prepare-release.ps1 -Version 1.0.6
```

The script does not publish by default. For a new version, pass hashes when they
exist:

```powershell
.\scripts\prepare-release.ps1 -Version <next-version> `
  -CrateSha256 <crates-archive-sha256> `
  -WindowsZipSha256 <release-zip-sha256> `
  -HomebrewRevision <tag-commit-sha>
```

Manual gate:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo package
qorx doctor --json
```

On Windows, build the portable asset:

```powershell
.\scripts\build-release-assets.ps1 -Version 1.0.6
```

## Registries

The publish script refuses to upload when the matching token is missing.

```powershell
.\scripts\publish-registries.ps1 -Version 1.0.6 -DryRun
.\scripts\publish-registries.ps1 -Version 1.0.6
```

Or run the dry-run path through the prep script:

```powershell
.\scripts\prepare-release.ps1 -Version 1.0.6 -DryRunRegistries
```

Expected credentials:

```text
CARGO_REGISTRY_TOKEN
NPM_TOKEN or NODE_AUTH_TOKEN
TWINE_USERNAME=__token__
TWINE_PASSWORD=<PyPI token>
```

## Linux And Desktop Package Surfaces

The repo carries packaging recipes for:

```text
Homebrew/Linuxbrew: packaging/homebrew/qorx.rb
Arch/AUR:          packaging/aur/PKGBUILD
Debian/Ubuntu:    packaging/debian/
Fedora/RHEL:      packaging/rpm/qorx.spec
Nix:              flake.nix
Snap:             snap/snapcraft.yaml
Scoop:            packaging/scoop/qorx.json
WinGet:           packaging/winget/manifests/
Docker:           Dockerfile, docker-compose.yml
systemd:          packaging/systemd/qorx.service
```

Central distribution still needs submission to each upstream package index.
Do not mark a channel as live until the public package page exists.
