# Qorx Live Community Metrics

Qorx Community Edition exposes a public metrics worker:

```text
https://qorx-community-metrics.omniscius.workers.dev/metrics.json
```

The worker reads public GitHub sources and returns a compact JSON proof surface:

- repository stars, forks, issue count, pushed time, and license.
- Cargo version and latest GitHub release tag.
- latest build, TestSprite, and daily proof workflow status.
- latest local benchmark summary from `docs/benchmarks/live.json`.
- Qorx notes that separate local accounting from provider billing.

The response is cached at the Cloudflare edge for one day:

```text
cache-control: public, max-age=300, s-maxage=86400, stale-while-revalidate=3600
```

Badge endpoints:

```text
/badge/version
/badge/build
/badge/testsprite
/badge/reduction
```

These badges are used in the README so the public page shows current proof
status without hand-editing numbers.

## Source Of The Numbers

`docs/benchmarks/live.json` is refreshed by the scheduled GitHub workflow:

```text
Daily Community Proof
```

That workflow builds Qorx from source, runs tests, runs the metrics worker unit
test, executes the local benchmark, and commits the refreshed benchmark proof
only when generated outputs changed.

The benchmark numbers are local Qorx estimates. They are not provider invoice
savings and they do not prove model answer quality.
