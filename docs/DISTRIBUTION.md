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

## Release Channels Still Opening

- npm package.
- PyPI package.
- WinGet manifest.
- Scoop bucket.
- Homebrew tap.
- Snap package.
- AUR, Debian, RPM, or Nix recipes.
- Docker image.
- signed installers.
- auto-update channel.

Those channels need registry credentials, signing, or support policy. They stay
out of the public CE repo until the maintainer intentionally opens that channel.
Signed installers and auto-update are Qorx Edge surfaces.

## Maintainer note

Historical tags and archives may still exist. New public work should point to
Community Edition for AGPL source/CLI assets and to Qorx Edge for the supported
always-on local product.
