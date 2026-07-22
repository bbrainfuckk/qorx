# Qorx package channels

This folder keeps Qorx package-channel files. These recipes install or build the
same Qorx 1.0.5 language, compiler, and runtime from the canonical repository.

## Channel status

| Channel | Files | Status |
| --- | --- | --- |
| PyPI | `packages/python/` | release-asset Python wrapper, uses trusted publishing |
| npm | `packages/npm/` | release-asset Node wrapper, needs npm token to publish |
| Arch/AUR | `packaging/arch/PKGBUILD` | source-build PKGBUILD, needs AUR maintainer push |
| Homebrew | `packaging/homebrew/qorx.rb` | source-build formula, needs tap push |
| Scoop | `packaging/scoop/qorx.json` | Windows manifest, needs release asset hash before bucket push |
| WinGet | `packaging/winget/` | Windows manifest, needs release asset hash before PR |
| Snap | `packaging/snap/snapcraft.yaml` | source-build Snapcraft file, needs Snapcraft login |
| Docker | `Dockerfile` | source-build container image, needs registry login to publish |
| Nix | `flake.nix` | local flake package |
| Deb/RPM | `packaging/nfpm/qorx.yaml` | nfpm config, needs built Linux binary |

## Local runtime

Package-manager installs run the same local compiler and runtime. Hosted
services are not required to check, compile, or execute a `.qorx` program.

## Maintainer validation

Run:

```powershell
.\scripts\check-package-channels.ps1
```

The GitHub workflow `Package Channel Manifests` runs the same check and validates
the npm, PyPI, and Docker packaging surfaces.
