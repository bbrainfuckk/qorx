# TestSprite enterprise QA

TestSprite is an optional cloud QA gate for Qorx deployments. It is not needed
to build or use the open-source CLI, daemon, package recipes, or desktop tray.

Use it for the hosted Qorx SaaS/API surface, not for a private laptop daemon.
The TestSprite GitHub Action needs a reachable `base_url`. `127.0.0.1:47187`
inside your Windows machine is not reachable from TestSprite cloud.

## Secret

If a TestSprite key was pasted into chat, public logs, an issue, or a commit,
revoke or rotate it in the TestSprite Web Portal before using it again.

Create a new key in TestSprite, then store it as a GitHub Actions secret:

```text
TESTSPRITE_API_KEY
```

Do not put the key in `.env`, workflow YAML, docs, source files, release notes,
or screenshots.

## Workflow

The workflow is:

```text
TestSprite Enterprise QA
```

Run it from GitHub Actions with:

- `base_url`: public staging URL for the Qorx SaaS/API deployment.
- `blocking`: `true` to fail the workflow when TestSprite reports failure.

The workflow has two stages:

1. Build the release binary and smoke-test the local daemon health route in the
   GitHub runner.
2. Run `TestSprite/run-action@v1` against the supplied public staging URL.

The local daemon smoke is not a cloud test. It proves the checked-in daemon can
start and answer `/health` on the runner. The TestSprite step is the cloud test.

## Test suite

The GitHub Action only runs a TestSprite suite. It does not generate the suite
for this repo by itself.

This repo includes a small release-site smoke suite under `testsprite_tests/`.
It checks the public documentation surface: version text, the daemon page, and
the TestSprite QA page. That is useful for the open-source release site. It is
not a SaaS product test.

For the hosted product, generate or maintain a separate suite through
TestSprite MCP or the TestSprite Web Portal. Commit generated files under
`testsprite_tests/` only when the workflow expects repo-managed tests.
Portal-managed suites may not need committed test files, but the run still
needs `TESTSPRITE_API_KEY` and a reachable `base_url`.

## SaaS target

For enterprise SaaS use, expose Qorx behind your own gateway:

- TLS.
- Authentication.
- Rate limits.
- Audit logs.
- Separate test tenant.
- Test-only credentials.
- No private local corpora in cloud test fixtures.

Keep the raw Qorx daemon loopback-only unless it sits behind a controlled
private network or reverse proxy.

## Local command

Before relying on the cloud gate, verify the repo wiring locally:

```powershell
.\scripts\check-testsprite-enterprise.ps1
```

This check verifies that the workflow uses the GitHub secret, requires a public
base URL, keeps blocking mode explicit, runs a daemon health smoke, documents
key rotation, and does not contain a literal TestSprite-style secret. It also
expects `testsprite_tests/tmp/test_results.json` from a completed TestSprite
run. That generated result is intentionally gitignored; the local check does
not create or invent a cloud-run result.
