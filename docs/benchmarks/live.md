# Qorx Benchmark Report

Generated: `2026-05-03T12:34:00+00:00`

Suite: `live`

Target: `.`

Qorx version: `qorx 1.0.4-a.0`

Git commit: `b838c23`

## Summary

| Metric | Value |
| --- | ---: |
| Indexed local tokens | 202986 |
| Session visible tokens | 69 |
| Session reduction | 2941.83x |
| Pack used tokens | 484 |
| Pack reduction | 419.39x |
| Squeeze used tokens | 419 |
| Squeeze reduction | 484.45x |
| Bench average reduction | 400.60x |
| Strict task pass rate | 100.0% |
| Expected refusal pass rate | 100.0% |
| Agent provider calls | 0 |

## Strict Tasks

| Question | Expected | Actual | Pass | Evidence | Used tokens |
| --- | --- | --- | ---: | ---: | ---: |
| context fault proof pages resolver boundary | supported | supported | yes | 3 | 380 |
| galactic banana escrow treaty | not_found | not_found | yes | 0 | 8 |

## Bench Rows

| Query | Used tokens | Omitted tokens | Reduction | Quarks |
| --- | ---: | ---: | ---: | ---: |
| context fault proof pages resolver boundary | 484 | 202502 | 419.39x | 3 |
| qorx carriers .qorx .qorxb qorx handle | 511 | 202475 | 397.23x | 3 |
| strict answer refusal unsupported claims | 527 | 202459 | 385.17x | 4 |

## Claim Notes

This benchmark uses Qorx local accounting only. Token counts are deterministic
`ceil(chars / 4)` estimates unless the runtime reports another estimator. The
report does not claim provider invoice savings, production throughput, or
downstream model answer quality.

To reproduce:

```powershell
python scripts/run-benchmark.py --target . --suite live --budget-tokens 600 --squeeze-budget-tokens 450 --query "context fault proof pages resolver boundary" --query "qorx carriers .qorx .qorxb qorx handle" --query "strict answer refusal unsupported claims" --supported-question "context fault proof pages resolver boundary" --unsupported-question "galactic banana escrow treaty" --agent-objective "prove context fault proof pages resolver boundary" --output-json docs/benchmarks/live.json --output-md docs/benchmarks/live.md
```
