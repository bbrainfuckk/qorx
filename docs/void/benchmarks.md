# Qorx Void benchmark evidence

The numbers on this page come from two different public proof surfaces. They
must remain attached to their hardware, data, and measurement boundary.

## AMD MI300X product run

The benchmark published on [qorx.eu.cc](https://qorx.eu.cc/#benchmark) used:

- AMD Radeon Instinct MI300X GPU with ROCm;
- GPT-OSS 120b-ROCm7;
- 192 GB VRAM, 20 vCPU, and 240 GB RAM;
- BEIR retrieval sets including SciFact, NFCorpus, FiQA, FEVER, and ArguAna;
- SQuAD 2.0, HotpotQA distractor, and Needle-in-haystack recall checks.

| Measurement | Published result |
| --- | ---: |
| Indexed context | 184,789,445 tokens |
| Average carrier | 14.0 tokens |
| Context reduction | 13,199,246.07x average |
| Local core latency | 0.8974 ms average |
| Maximum local core latency | 3.512 ms |
| Quality scorecard | 38 perfect target checks out of 52 |
| Test tracks completed | 10 out of 10 |
| Grounding gates | 1.0 pass |
| Provider calls during the local run | 0 |

The latency figure covers the measured local core path, not model inference or
network time. The 13.2M figure is context reduction for this run, not a promise
that every corpus, query, or agent will produce the same ratio. The quality
scorecard and grounding gates are reported separately because a smaller carrier
does not prove a correct answer by itself.

## Public repository run

The committed public benchmark indexed this repository and used the local
`ceil(chars / 4)` token estimate.

| Case | Indexed | Used or visible | Reduction |
| --- | ---: | ---: | ---: |
| Session carrier | 388,573 | 69 | 5,631.49x |
| Evidence pack | 388,573 | 410 | 947.74x |
| Squeeze extract | 388,573 | 448 | 867.35x |

Reproduce or inspect it from [live.md](../benchmarks/live.md) and
[live.json](../benchmarks/live.json). These are local deterministic estimates,
not provider invoice savings.

## Comparison board

This is not a universal ranking. Each project reports a different operation,
dataset, carrier shape, latency boundary, or evaluation method. The table keeps
the reference metric in its published scope and places the Qorx AMD result next
to it for orientation.

| System | Published reference metric | Qorx AMD MI300X reference | Category |
| --- | --- | --- | --- |
| [LLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | Up to 20x prompt compression with little performance loss. | 13,199,246.07x context reduction in the disclosed local run. | Compression |
| [LongLLMLingua](https://www.microsoft.com/en-us/research/project/llmlingua/) | 4x compression, 17.1% performance gain, and 1.4x to 3.8x latency speedup. | 13.2M x context reduction; 0.8974 ms average local core latency. | Compression and latency |
| [LLMLingua-2](https://arxiv.org/abs/2403.12968) | 2x to 5x compression and 1.6x to 2.9x end-to-end acceleration. | 184,789,445 indexed tokens to a 14.0-token average carrier. | Compression |
| [Context Mode](https://context-mode.com/) | 315 KB to 5.4 KB (98%); an example reports 30x fewer tokens over 50 turns. | 13.2M x context reduction; zero provider calls in the local run. | Context spend |
| [LeanCTX](https://leanctx.com/) | 145.2K raw tokens to a 7.5K-token map; 98.5% saved. | 184.8M indexed tokens to a 14.0-token average carrier. | Repository context |
| [sqz CLI](https://github.com/ojuschugh1/sqz) | 24.7% average reduction, 92% saved on repeated file reads, and 13-token cached references. | 14.0 average used tokens with a minimum of two quarks. | Repeated output |
| [indxr](https://docs.rs/crate/indxr/0.2.0) | About 5x fewer tokens than full file reads; sub-20 ms indexing for most projects. | 0.8974 ms average local core latency. The measured operations differ. | Local index |
| [Aider Repo Map](https://aider.chat/docs/repomap.html) | 1,024 default map tokens. | 14.0 average carrier tokens. Carrier and repo-map semantics differ. | Repository map |

## How to read the board

- Do not divide one row by another and call the result an end-to-end speedup.
- Do not compare local core latency with model inference latency.
- Do not treat a carrier token count as equivalent to a full prompt-compression
  benchmark unless the inputs and outputs are matched.
- Re-run systems on the same corpus, hardware, query set, and quality gates
  before making a direct winner claim.
- Keep provider cost, output tokens, cache discounts, and account terms outside
  the result unless they were actually measured.

The value of the published board is transparency: it shows what each number
means and where a fairer head-to-head test still needs to be run.
