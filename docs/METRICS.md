# Qorx metrics

Qorx has three public-facing metric surfaces. They use the same B2C accounting
language, but they do not share one counter store.

## Qorx Ayie v1.0 / Qorx Ayie Starter

Ayie is the supported local product runtime: daemon, tray, provider routing,
MCP/CLI activation, ORCL, account-backed service features, and the local
resolver experience.

Primary metric source:

```text
GET http://127.0.0.1:47187/stats
```

Main Ayie counters:

- `requests`
- `raw_prompt_tokens`
- `compressed_prompt_tokens`
- `saved_prompt_tokens`
- `context_pack_requests`
- `context_indexed_tokens`
- `context_sent_tokens`
- `context_omitted_tokens`
- `cache_lookups`
- `cache_hits`
- `cache_misses`
- `provider_cached_prompt_tokens`
- `provider_cache_write_tokens`
- `context_usd_saved`
- `proxy_usd_saved`
- `provider_cache_usd_saved`
- `total_estimated_usd_saved`

Boundary: Ayie metrics are local runtime evidence. Starter includes 5,000
Ayie/Cloud service requests, but local Community Edition commands stay
unmetered.

## Qorx API v1.0

Qorx API is the hosted account and usage surface for tenant keys, optimizer
calls, resolver handles, trial keys, pricing, checkout, and usage metering.

Primary metric source:

```text
GET /api/v1/usage
```

Main API counters:

- `events`
- `billableUnits`
- `resolverCalls`
- `tokenBlocks`
- `proofPages`
- `auditMonths`

Boundary: API metrics are hosted tenant usage and plan enforcement metrics.
They do not replace the local Ayie `/stats` ledger.

## Qorx CLI Community Edition

Qorx CLI Community Edition is this AGPL source line. It is the local
language/runtime/proof surface.

Main CLI metrics:

- indexed local tokens
- model-visible tokens
- local reduction ratio
- pack/squeeze/strict-answer proof counts
- local `stats`
- local benchmark output
- security/provenance attestation results

Primary public proof files:

```text
docs/benchmarks/live.json
docs/benchmarks/live.md
```

Boundary: CLI CE metrics are local, reproducible proof numbers. They are not
provider invoice savings and they are not capped by Qorx Ayie Starter.

## Product labels

Use these labels when documenting or building UI:

- Ayie Monitor: `Local saved`
- API Dashboard: `Cloud usage`
- Optimizer Preview: `Estimated per-call savings`
- Community Edition: `Local proof metrics`
