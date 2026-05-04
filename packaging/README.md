# Qorx package channels

This folder keeps the package-channel files for Qorx Community Edition.

These packages install or build the public source CLI. They do not turn
Community Edition into Qorx Ayie, and they do not make the open-source CLI stop
after 5,000 requests. The 5,000 included Ayie/Cloud requests belong to Qorx Ayie
Starter and are enforced by the Qorx account service.

## Channel status

| Channel | Files | Status |
| --- | --- | --- |
| PyPI | `packaging/pypi/` | source-build Python wrapper, needs PyPI token to publish |
| npm | `packaging/npm/` | source-build Node wrapper, needs npm token to publish |
| Arch/AUR | `packaging/arch/PKGBUILD` | source-build PKGBUILD, needs AUR maintainer push |
| Homebrew | `packaging/homebrew/qorx.rb` | source-build formula, needs tap push |
| Scoop | `packaging/scoop/qorx.json` | Windows manifest, needs release asset hash before bucket push |
| WinGet | `packaging/winget/` | Windows manifest, needs release asset hash before PR |
| Snap | `packaging/snap/snapcraft.yaml` | source-build Snapcraft file, needs Snapcraft login |
| Docker | `Dockerfile` | source-build container image, needs registry login to publish |
| Nix | `flake.nix` | local flake package |
| Deb/RPM | `packaging/nfpm/qorx.yaml` | nfpm config, needs built Linux binary |

## Request limit truth

Community Edition is AGPL source code. A local source build cannot honestly be
capped at 5,000 requests because a fork can remove a local counter and rebuild.

Qorx Ayie Starter is different. Ayie/Cloud requests go through a Qorx account
entitlement:

```text
included_requests = 5000
remaining_requests = max(0, included_requests - used_edge_cloud_requests)
```

After the 5,000 included Ayie/Cloud requests are used, Ayie/Cloud asks the user
to subscribe before continuing with service-backed features. Local CE commands
such as `index`, `pack`, `squeeze`, `strict-answer`, and `qorx-check` stay
unmetered.

## Maintainer validation

Run:

```powershell
.\scripts\check-package-channels.ps1
```

The GitHub workflow `Package Channel Manifests` runs the same check and validates
the npm, PyPI, and Docker packaging surfaces.
