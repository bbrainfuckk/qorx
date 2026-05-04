# Qorx Ayie Starter

Qorx Ayie Starter is the try-it-for-real path for the paid product.

The idea is simple: people should experience the full Ayie/Cloud feature set on
Windows, macOS, and Linux before they pay. The starter allowance is 5,000
included Ayie/Cloud requests. After that, the account asks the user to subscribe
before continuing with Ayie/Cloud services.

This is generous if we count the right thing. A local `index` or `pack` command
should not burn allowance. A managed provider route, ORCL lookup, cloud capsule
sync, MCP/CLI activation, or account-backed daemon/API action should.

## What counts

These actions count toward the 5,000 included Ayie/Cloud requests:

- a routed provider call through Qorx Ayie or Qorx Cloud.
- an ORCL lookup served by the managed account service.
- a managed MCP or CLI activation call.
- a cloud capsule sync action.
- an account-backed daemon/API action that reaches Qorx service infrastructure.

These local actions stay unmetered:

- `qorx index`
- `qorx search`
- `qorx pack`
- `qorx squeeze`
- `qorx strict-answer`
- `qorx judge`
- `qorx qorx-check`
- `qorx qorx-compile`
- `qorx science`
- local tests and local benchmarks

Health checks, failed authentication attempts, local source builds, and local CE
commands should not reduce the allowance.

## Why the cap lives on the service

Putting the cap only in an AGPL local binary would be fake protection. Anyone
can remove a local counter and rebuild. That is normal open-source reality.

The enforceable part is the server-side service entitlement:

```text
included_requests = 5000
used_requests = count(successful_metered_service_requests)
remaining_requests = max(0, included_requests - used_requests)
```

Every metered Ayie/Cloud request should carry an account entitlement and an
idempotent request id. The Qorx account service checks the entitlement, records
the request, and returns the remaining count. Replaying the same request id
should not double-charge the user.

A fork can run local AGPL code. It cannot mint valid Qorx account entitlements,
write to the official usage ledger, or use the official hosted service without a
real account.

## What users see

The product should show the count plainly:

```text
Qorx Ayie Starter: 4,812 of 5,000 Ayie/Cloud requests remaining
```

When the allowance is used:

```text
Your 5,000 included Ayie/Cloud requests are used. Subscribe to keep using
Qorx Ayie and Qorx Cloud services. Local Community Edition commands still work.
```

That message is direct. It does not hide the source, the math, or the local
tools.

## Platform plan

Starter should apply to the same product line on every supported platform:

- Windows signed installer.
- macOS Intel and Apple Silicon builds.
- Linux x64 build.
- package channels such as Homebrew, WinGet, Scoop, npm, PyPI, Snap, Docker, or
  distro packages as those channels open.

The public repository does not ship the Qorx Ayie Starter installer binary yet.
Until signed assets are attached to releases, the website download buttons point
here and make that boundary explicit.

Community Edition package-channel files now live in the repo for PyPI, npm,
Arch/AUR, Homebrew, Scoop, WinGet, Snap, Docker, Nix, and Deb/RPM packaging.
Those package files do not enforce the 5,000 request allowance. The community
source release remains AGPL. The service cap protects the hosted capacity and
official Ayie/Cloud account features, not the public science.

## Metrics

Qorx Ayie Starter uses local Ayie runtime metrics plus account-backed service
request counts.

- local runtime metrics come from `GET http://127.0.0.1:47187/stats`.
- Starter allowance metrics come from the Qorx account service.
- Community Edition local commands stay unmetered.

See [Qorx metrics](METRICS.md) for the full Ayie/API/CLI split.
