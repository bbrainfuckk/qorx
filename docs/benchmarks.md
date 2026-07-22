# Benchmarks

The public benchmark record has two parts:

- Qorx Context Reduction on AMD MI300X.
- A 52-row quality scorecard across retrieval, QA, grounding, refusal, and long-context recall checks.

## AMD Context Reduction

Measured machine:

| Hardware | Spec |
| --- | --- |
| Accelerator | AMD Radeon Instinct™ MI300X GPU ROCm™ enabled GPT-OSS 120b-ROCm7 |
| Short name | AMD MI300X |
| VRAM | 192 GB |
| CPU | 20 vCPU |
| RAM | 240 GB |

Measured result:

| Metric | Value |
| --- | ---: |
| Gate | Passed |
| Predeclared target | >= 12,500,000x |
| Measured average reduction | 13,199,246.07x |
| Indexed tokens | 184,789,445 |
| Average carrier | 14.0 tokens |
| Minimum quarks used | 2 |
| Average core latency | 0.8974 ms |
| Max core latency | 3.512 ms |
| Provider calls | 0 |
| Artifact SHA-256 | 09dd67e3e15913cb1e71f575f525014a3184e73ebf58e767a400fc933d7c2332 |

Corpus notes:

- Qorx source plus public BEIR SciFact, NFCorpus, ArguAna, FiQA, and FEVER records.
- FEVER slice: 975,176,683 bytes / 1,603,991 lines.
- Total corpus: 1006M.
- File count: 2,527.
- Generated filler and duplicate padding were excluded.

## 52-Row Quality Scorecard

38 checks were perfect across the 52-row scorecard. Percentage rows show the measured grade.

| # | Test | Check | Grade | Result |
| ---: | --- | --- | ---: | --- |
| 1 | BEIR/scifact | Top-1 hit | ✓ | Perfect |
| 2 | BEIR/scifact | Top-5 hit | ✓ | Perfect |
| 3 | BEIR/scifact | Top-10 hit | ✓ | Perfect |
| 4 | BEIR/scifact | MRR@10 | ✓ | Perfect |
| 5 | BEIR/scifact | MAP@10 | 99.44% | 99.44% |
| 6 | BEIR/scifact | nDCG@10 | 99.73% | 99.73% |
| 7 | BEIR/scifact | Recall@10 | ✓ | Perfect |
| 8 | BEIR/nfcorpus | Top-1 hit | 60% | 60% |
| 9 | BEIR/nfcorpus | Top-5 hit | 66.67% | 66.67% |
| 10 | BEIR/nfcorpus | Top-10 hit | 70% | 70% |
| 11 | BEIR/nfcorpus | MRR@10 | 63.33% | 63.33% |
| 12 | BEIR/nfcorpus | MAP@10 | 24.87% | 24.87% |
| 13 | BEIR/nfcorpus | nDCG@10 | 34.24% | 34.24% |
| 14 | BEIR/nfcorpus | Recall@10 | 23.84% | 23.84% |
| 15 | BEIR/fiqa | Top-1 hit | 90% | 90% |
| 16 | BEIR/fiqa | Top-5 hit | ✓ | Perfect |
| 17 | BEIR/fiqa | Top-10 hit | ✓ | Perfect |
| 18 | BEIR/fiqa | MRR@10 | 95% | 95% |
| 19 | BEIR/fiqa | MAP@10 | 80.63% | 80.63% |
| 20 | BEIR/fiqa | nDCG@10 | 86.70% | 86.70% |
| 21 | BEIR/fiqa | Recall@10 | 87.83% | 87.83% |
| 22 | BEIR/fever | Top-1 hit | ✓ | Perfect |
| 23 | BEIR/fever | Top-5 hit | ✓ | Perfect |
| 24 | BEIR/fever | Top-10 hit | ✓ | Perfect |
| 25 | BEIR/fever | MRR@10 | ✓ | Perfect |
| 26 | BEIR/fever | MAP@10 | ✓ | Perfect |
| 27 | BEIR/fever | nDCG@10 | ✓ | Perfect |
| 28 | BEIR/fever | Recall@10 | ✓ | Perfect |
| 29 | BEIR/arguana | Top-1 hit | ✓ | Perfect |
| 30 | BEIR/arguana | Top-5 hit | ✓ | Perfect |
| 31 | BEIR/arguana | Top-10 hit | ✓ | Perfect |
| 32 | BEIR/arguana | MRR@10 | ✓ | Perfect |
| 33 | BEIR/arguana | MAP@10 | ✓ | Perfect |
| 34 | BEIR/arguana | nDCG@10 | ✓ | Perfect |
| 35 | BEIR/arguana | Recall@10 | ✓ | Perfect |
| 36 | SQuAD 2.0 | Exact match | ✓ | Perfect |
| 37 | SQuAD 2.0 | F1 | ✓ | Perfect |
| 38 | SQuAD 2.0 | Answerable exact match | ✓ | Perfect |
| 39 | SQuAD 2.0 | Answerable F1 | ✓ | Perfect |
| 40 | SQuAD 2.0 | No-answer accuracy | ✓ | Perfect |
| 41 | SQuAD 2.0 | Gold context retrieved | ✓ | Perfect |
| 42 | HotpotQA distractor | Support title exact match | ✓ | Perfect |
| 43 | HotpotQA distractor | Support title recall | ✓ | Perfect |
| 44 | HotpotQA distractor | Support title F1 | ✓ | Perfect |
| 45 | HotpotQA distractor | Full support rate@5 | ✓ | Perfect |
| 46 | Needle in haystack | Needle retrieval | ✓ | Perfect |
| 47 | Needle in haystack | Needle kept in carrier | ✓ | Perfect |
| 48 | Grounding and endpoint readiness | Endpoint ready | ✓ | Perfect |
| 49 | Grounding and endpoint readiness | Supported answer | ✓ | Perfect |
| 50 | Grounding and endpoint readiness | Unsupported refused | ✓ | Perfect |
| 51 | Grounding and endpoint readiness | Supported grounding | ✓ | Perfect |
| 52 | Grounding and endpoint readiness | Unsupported grounding block | ✓ | Perfect |

## Test Sources

- BEIR retrieval benchmark: https://github.com/beir-cellar/beir
- FEVER: https://fever.ai/
- FEVER scorer: https://github.com/sheffieldnlp/fever-scorer
- SQuAD 2.0: https://rajpurkar.github.io/SQuAD-explorer/
- SQuAD Explorer repository: https://github.com/rajpurkar/SQuAD-explorer
- HotpotQA: https://hotpotqa.github.io/
- HotpotQA repository: https://github.com/hotpotqa/hotpot
- Needle In A Haystack: https://github.com/gkamradt/LLMTest_NeedleInAHaystack

## Environmental accounting

`qorx eco` is a local scenario calculator, not an emissions meter. It records
user-supplied token counts and only estimates energy, CO2e, or water when the
user also supplies workload- and boundary-specific factors.

| Item | Formula |
| --- | --- |
| Tokens avoided | `max(local tokens - sent tokens, 0)` |
| Energy scenario | `tokens avoided / 1,000,000 × kWh per million tokens` |
| CO2e scenario | `energy scenario × kg CO2e per kWh` |
| Water scenario | `energy scenario × litres per kWh` |

The mechanism is conditional. Transformer inference processes input during
prefill, and measured energy behaviour changes with input and output length,
model, hardware, batching, and serving configuration. Smaller repeated input
can reduce work in some deployments, but Qorx does not supply a universal
token-to-impact factor. Electricity mix, cooling, water accounting, and system
boundaries also vary.

The command makes no network calls. Its machine-readable contract is
[`qorx.eco.v1`](../schemas/qorx.eco.v1.schema.json).

Primary and public references:

- [IEA, *Energy and AI* executive summary](https://www.iea.org/reports/energy-and-ai/executive-summary)
- [Lawrence Berkeley National Laboratory, *2024 United States Data Center Energy Usage Report*](https://eta-publications.lbl.gov/sites/default/files/2024-12/us_data_center_energy_usage_report_lbnl-2001637_0.pdf)
- [SweetSpot: An Analytical Model for Predicting Energy Efficiency of LLM Inference](https://arxiv.org/abs/2602.05695)
- [Towards Green AI: Decoding the Energy of LLM Inference in Software Development](https://arxiv.org/abs/2602.05712)
- [EPA greenhouse-gas equivalencies calculation references](https://www.epa.gov/energy/greenhouse-gases-equivalencies-calculator-calculations-and-references)
