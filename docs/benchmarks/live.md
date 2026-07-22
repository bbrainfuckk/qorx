# Qorx Benchmark Report

Generated: `2026-07-22T09:50:55+00:00`

Suite: `live`

Target: `.`

Qorx version: `qorx 1.0.6`

Git commit: `6a30742`

## Summary

| Metric | Value |
| --- | ---: |
| Indexed local tokens | 407902 |
| Session visible tokens | 69 |
| Session reduction | 5911.62x |
| Pack used tokens | 410 |
| Pack reduction | 994.88x |
| Squeeze used tokens | 448 |
| Squeeze reduction | 910.50x |
| Bench average reduction | 920.77x |
| Strict task pass rate | 100.0% |
| Expected refusal pass rate | 100.0% |
| Agent provider calls | 0 |

## Strict Tasks

| Question | Expected | Actual | Pass | Evidence | Used tokens |
| --- | --- | --- | ---: | ---: | ---: |
| Context injection is a compact agent contract | supported | supported | yes | 3 | 325 |
| galactic banana escrow treaty | not_found | not_found | yes | 0 | 8 |

## Bench Rows

| Query | Used tokens | Omitted tokens | Reduction | Quarks |
| --- | ---: | ---: | ---: | ---: |
| Context injection is a compact agent contract | 410 | 407492 | 994.88x | 1 |
| context fault proof pages resolver boundary | 448 | 407454 | 910.50x | 2 |
| strict answer refusal unsupported claims | 476 | 407426 | 856.94x | 3 |

## Boundary

This benchmark uses Qorx local accounting only. Token counts are deterministic
`ceil(chars / 4)` estimates unless the runtime reports another estimator. The
report does not claim provider invoice savings, production throughput, or
downstream model answer quality.

To reproduce:

```powershell
python scripts/run-benchmark.py --target . --suite live --budget-tokens 600 --squeeze-budget-tokens 450 --query "Context injection is a compact agent contract" --query "context fault proof pages resolver boundary" --query "strict answer refusal unsupported claims" --supported-question "Context injection is a compact agent contract" --unsupported-question "galactic banana escrow treaty" --agent-objective "prove Context injection is a compact agent contract" --output-json docs/benchmarks/live.json --output-md docs/benchmarks/live.md
```
