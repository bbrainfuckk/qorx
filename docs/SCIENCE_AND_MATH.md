# Qorx science and math

Qorx is built around a plain measurement idea: keep large local context on the
machine, send a small visible carrier to the model, and resolve exact evidence
locally when the workflow needs it.

This page gives the math in public. There is no secret science here. Some parts
are shipped in Community Edition today. Some parts, such as account entitlements
and managed provider routing, belong to Qorx Ayie or Qorx Cloud.

## Token estimate

Qorx uses a deterministic local token estimate unless a command says otherwise:

```text
tokens_est(text) = ceil(utf8_character_count(text) / 4)
```

This is simple and repeatable. It is not a provider tokenizer and it is not an
invoice number. It is good enough for local planning, benchmark comparison, and
budgeted evidence selection.

## Baseline-to-Compact

Baseline-to-Compact accounting compares the local context mass with the
model-visible carrier:

```text
baseline_tokens = sum(tokens_est(indexed_local_quarks))
visible_tokens = tokens_est(carrier_or_pack_text)
b2c = baseline_tokens / max(1, visible_tokens)
omitted_tokens = max(0, baseline_tokens - visible_tokens)
```

In the current live proof, Qorx indexes the repo locally and sends a much smaller
carrier or evidence pack. That proves context omission and resolver availability.
It does not prove that every downstream answer is correct.

## Evidence packing

Qorx indexes files into bounded quarks. A quark carries path, line range, hash,
token estimate, symbols, and sparse lexical features.

At query time, the packer chooses quarks under a token budget:

```text
score = lexical_match
      + path_match
      + symbol_match
      + structure_match
      + cache_reuse_value
      - token_cost
      - duplication_penalty
      - omission_risk
```

The exact weights live in the Rust source. The important part is the shape:
Qorx picks useful local evidence before spending model-visible tokens.

## Squeeze

`qorx squeeze` is extractive. It keeps matching lines from indexed quarks and
drops unrelated lines:

```text
squeeze_reduction = source_quark_tokens / max(1, squeezed_excerpt_tokens)
```

It does not invent missing context. If the local index does not support an
answer, strict evidence commands return an unsupported result instead of filling
the gap with guesswork.

## Session and pack reduction

The live benchmark reports three useful ratios:

```text
session_reduction = indexed_tokens / max(1, session_carrier_tokens)
pack_reduction = indexed_tokens / max(1, evidence_pack_tokens)
squeeze_reduction = indexed_tokens / max(1, squeeze_extract_tokens)
```

These are local Qorx ratios. They are strong proof that the visible context is
smaller than the local evidence store. They are not a universal quality score.

## USD estimate

Qorx can estimate avoided input cost from omitted local tokens:

```text
estimated_input_usd = omitted_tokens * input_price_per_1m / 1_000_000
estimated_cached_usd = cached_tokens * cached_price_per_1m / 1_000_000
```

Provider invoices are the source of truth for billed savings. Qorx local numbers
are planning and proof numbers.

## Request allowance math

Qorx Ayie Starter uses a service-side request allowance:

```text
included_requests = 5000
used_requests = count(successful_metered_edge_cloud_requests)
remaining_requests = max(0, included_requests - used_requests)
usage_ratio = used_requests / included_requests
```

Only successful metered Ayie/Cloud service actions should count. Local CE
commands stay unmetered.

The service should record an idempotent request id so retries do not burn extra
allowance:

```text
usage_key = hash(account_id, install_id, request_id, feature, day)
```

The client can display remaining requests, but the server owns the ledger. That
is how Qorx can stay open source locally while protecting the official hosted
service from unlimited free usage.

## Why this helps hallucination risk

Qorx does not magically make models truthful. It reduces one common failure:
asking a model to answer from context it never received.

The strict path is:

```text
index local evidence
select bounded evidence
cite exact paths and line ranges
answer only from selected evidence
return unsupported when evidence is missing
```

That gives the downstream agent a smaller, cited working set. It is still the
agent's job to use the evidence correctly, and it is still the maintainer's job
to test the final code.

## Reference papers and sources

The local source map is [REFERENCE_PAPERS.md](REFERENCE_PAPERS.md). The current
paper set and source list includes:

| Area | References |
| --- | --- |
| Prompt compression | LLMLingua, LongLLMLingua, Active Context Compression, Gist Tokens, Experience Compression Spectrum |
| Repository and code context | ReACC, Codebase-Memory, BM25 and lexical retrieval, Aider repository map |
| Agent memory | AgeMem, AtomMem, Titans, TokMem, Memory Survey, Structural Memory |
| Hierarchical memory | H-MEM, HiMem, TierMem, GAM |
| Cache and reuse | Preble, Don't Break the Cache, Similarity Caching, GPT Semantic Cache, RAGCache, Cache-Craft, Approximate Caching for RAG, Domain-Specific Semantic Cache, vCache, ContextPilot, QVCache |
| Runtime and KV scope | TurboQuant, vCache, QVCache |
| Provider and tooling docs | OpenAI prompt caching, Anthropic prompt caching, Gemini context caching, Claude Code memory, Gemini CLI `GEMINI.md`, Cursor codebase indexing |
| Provenance and storage | Protocol Buffers, NIST FIPS 204, C2PA Specification, Microsoft kernel-mode signing requirements |

These references support the architecture class. Qorx-specific claims still need
Qorx-specific benchmarks, tests, routed usage logs, and task success evaluation.
