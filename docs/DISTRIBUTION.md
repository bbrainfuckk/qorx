# Release Channels

Qorx Community Edition is source-first and now has maintainer-controlled
cross-platform CLI release assets.

## CE distribution

Source install:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --branch main --locked qorx
```

Or clone the repo and build:

```sh
git clone https://github.com/bbrainfuckk/qorx.git
cd qorx
cargo test
cargo build --release
```

## GitHub release assets

Release tags build and upload community CLI archives for:

- Windows x64.
- Linux x64.
- macOS.

These are unsigned AGPL Community Edition CLI assets. They are meant for users
who want a quick binary after the source build and tests pass in CI.

## Package channel files

The public repo now includes package-channel files for:

- PyPI.
- npm.
- Arch/AUR.
- Homebrew.
- Scoop.
- WinGet.
- Snap.
- Docker.
- Nix.
- Deb/RPM packaging through nfpm.

These files live under `packaging/`, plus the root `Dockerfile` and `flake.nix`.
The `Package Channel Manifests` workflow validates the package metadata and
Docker build shape.

Registry publishing still needs maintainer credentials or external submissions:
PyPI token, npm token, AUR push access, Homebrew tap access, Scoop bucket PR,
WinGet PR, Snapcraft login, Docker registry login, and any Debian/RPM/Nix
publishing path.

Qorx Ayie Starter includes 5,000 included Ayie/Cloud requests across Windows,
macOS, and Linux before subscription. Community Edition package files do not cap
local CLI usage. The request cap belongs to the Qorx account service because a
local AGPL build can be forked and changed.

Signed installers and auto-update remain Qorx Ayie surfaces.

## Maintainer note

Historical tags and archives may still exist. New public work should point to
Community Edition for AGPL source/CLI assets and to Qorx Ayie for the supported
always-on local product.
