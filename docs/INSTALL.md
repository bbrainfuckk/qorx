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

## Release Channels

Qorx Edge and future package channels add:

- npm and PyPI wrappers.
- WinGet, Scoop, Homebrew, Snap, AUR, Debian, RPM, Nix, or Docker distribution.
- signed Windows installers.
- tray or auto-update packaging.

Those channels need registry credentials, signing, or product support. Signed
installers, tray, daemon management, auto-update, provider routing, and ORCL are
Qorx Edge surfaces.

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
