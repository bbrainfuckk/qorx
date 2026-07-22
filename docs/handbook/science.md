# Qorx Science Notes

This page names the math and engineering used in Qorx. It is not a claim that
Qorx is peer reviewed. It is a map from terms to code paths and commands.

## Measurement Rule

Qorx uses local estimates unless a command says otherwise.

```text
mass(text) = ceil(len_utf8_chars(text) / 4)
qshf = indexed_local_mass / visible_carrier_mass
omitted_mass = indexed_local_mass - visible_carrier_mass
```

These numbers are for local planning and accounting. They are not provider
invoice numbers.

## B2C Quant Allocator

Command:

```powershell
qorx b2c-plan "login route session audit" --budget-tokens 900
```

Used by:

```powershell
qorx pack "login route session audit" --budget-tokens 900
```

Code path:

```text
src/b2c_quant.rs
src/index.rs
src/proxy.rs
```

The allocator works on indexed quarks. It scores local candidates, then chooses
a portfolio under a token budget.

```text
net = expected_value + cache_value
      - token_cost
      - redundancy_penalty
      - omission_risk_penalty
```

The planner runs five local lanes:

| Lane | Role |
| --- | --- |
| retrieval | Find matching quarks with local sparse search. |
| portfolio | Prefer high value per token and avoid duplicate evidence. |
| risk | Reject high omission risk before the first pick. |
| cache | Give stable quarks small reuse value. |
| carrier | Choose `handle`, `pack`, `squeeze`, or `fault`. |

What this proves: Qorx can select local evidence with deterministic math before
spending provider-visible tokens.

What it does not prove: revenue, invoice savings, conversion lift, or task
quality. Those require routed provider logs, user outcomes, and evaluation data.

## Sparse Retrieval

Code path:

```text
src/index.rs
```

Qorx hashes terms into sparse vectors and stores them in `repo_index.pb`. Search
uses lexical matches, path matches, symbol matches, structural signals, and
vector overlap. There are no bundled embedding weights in the core binary.

## Structural Signals

Code path:

```text
src/index.rs
```

Qorx marks imports, routes, tests, errors, branches, assignments, calls, and
config-like lines. These signals boost retrieval. They do not replace a full AST
parser.

## Squeeze

Command:

```powershell
qorx squeeze "production gate routed provider evidence" --budget-tokens 700
```

Code path:

```text
src/squeeze.rs
```

Squeeze extracts query-matching lines from local quarks. It is deterministic and
local. It is not neural compression.

## Cache Plan

Command:

```powershell
qorx cache-plan "stable prefix`n--- QORX_DYNAMIC ---`nuser turn"
```

Code path:

```text
src/cache_plan.rs
src/proxy.rs
```

The plan separates a stable prefix from a dynamic tail. The proxy exposes the
plan in headers such as `x-qorx-cache-plan` and
`x-qorx-cacheable-prefix-tokens`. Provider cache savings are counted only when
the provider returns cache metadata.

## Exact Replay Cache

Code path:

```text
src/response_cache.rs
src/proxy.rs
```

Non-streaming routed calls can be replayed locally when the normalized request
matches a cached response. This saves an upstream call for that request. It does
not generalize semantically similar prompts.

## KV Hints

Code path:

```text
src/kv.rs
src/lattice.rs
scripts/adapter-proofs/
```

Qorx can emit safetensors-style KV hints for adapters. The core release does
not compress a live model KV cache. That requires a runtime that owns the actual
KV tensors.

## USD Accounting

Command:

```powershell
qorx stats
qorx money --claim-usd 0.01
```

Code path:

```text
src/stats.rs
src/money.rs
src/proxy.rs
```

Qorx reports omitted local context, compressed routed input, exact replay cache
hits, provider cache reads, provider cache writes, and estimated USD saved. The
estimate uses configured input prices and local token estimates. Provider bills
remain the authority for invoice claims.

## Release Gate

Run the full release prep before publishing a new version:

```powershell
.\scripts\prepare-release.ps1 -Version 0.0.1-ylem
```

For registry dry runs:

```powershell
.\scripts\prepare-release.ps1 -Version 0.0.1-ylem -DryRunRegistries
```
