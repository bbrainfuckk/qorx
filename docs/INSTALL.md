# Installing Qorx Community Edition

Community Edition is source-first. Build it with Rust and Cargo, or use the
maintainer-controlled GitHub release assets when a tag is published.

## Source build

```sh
git clone https://github.com/bbrainfuckk/qorx.git
cd qorx
cargo test
cargo build --release
./target/release/qorx --version
```

Windows:

```powershell
git clone https://github.com/bbrainfuckk/qorx.git
cd qorx
cargo test
cargo build --release
.\target\release\qorx.exe --version
```

## Cargo git install

Install from the current public source branch:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --branch main --locked qorx
qorx --version
```

## GitHub release assets

Release tags build community CLI archives for Windows, Linux, and macOS. They
are unsigned CLI assets, not Qorx Edge installers.

Use source builds when you need the most auditable path. Use release assets when
you want the quickest community CLI path.

## Package channels

Community Edition package-channel files are available for:

- PyPI and npm wrappers that install the CLI through Cargo.
- Arch/AUR through `packaging/arch/PKGBUILD`.
- Homebrew through `packaging/homebrew/qorx.rb`.
- Scoop and WinGet through `packaging/scoop/` and `packaging/winget/`.
- Snap through `packaging/snap/snapcraft.yaml`.
- Docker through the root `Dockerfile`.
- Nix through the root `flake.nix`.
- Deb/RPM packaging through `packaging/nfpm/qorx.yaml`.

Publishing those channels still needs the maintainer's registry accounts or
package-submission PRs. The files are in the repo so the package work is
auditable and repeatable.

Qorx Edge Starter is the cross-platform product trial path for service-backed
features. It includes 5,000 included Edge/Cloud requests. Community Edition
package installs do not cap local CLI usage.

## What to run

Use the CE command set after building:

```powershell
.\target\release\qorx.exe doctor --json
.\target\release\qorx.exe index .
.\target\release\qorx.exe strict-answer "what proves Qorx is a language runtime?"
.\target\release\qorx.exe b2c-plan "what proves Qorx is a language runtime?" --budget-tokens 900
.\target\release\qorx.exe security attest
```

Read [Community Edition](COMMUNITY.md) before treating a self-built binary as an
official Qorx product.
