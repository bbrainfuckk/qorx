# Qorx Benchmark Report

Generated: `2026-05-03T10:05:28+00:00`

Suite: `qorx-live`

Target: `.`

Qorx version: `qorx 1.0.4`

Git commit: `0a596f2`

## Summary

| Metric | Value |
| --- | ---: |
| Indexed local tokens | 189042 |
| Session visible tokens | 69 |
| Session reduction | 2739.74x |
| Pack used tokens | 566 |
| Pack reduction | 334.00x |
| Squeeze used tokens | 292 |
| Squeeze reduction | 647.40x |
| Bench average reduction | 380.33x |
| Strict task pass rate | 100.0% |
| Expected refusal pass rate | 100.0% |
| Agent provider calls | 0 |

## Strict Tasks

| Question | Expected | Actual | Pass | Evidence | Used tokens |
| --- | --- | --- | ---: | ---: | ---: |
| Qorx Community Edition AGPL source line | supported | supported | yes | 3 | 232 |
| d8ffdb34e9eb4e4085fbb7ab6067785f | not_found | not_found | yes | 0 | 8 |

## Bench Rows

| Query | Used tokens | Omitted tokens | Reduction | Quarks |
| --- | ---: | ---: | ---: | ---: |
| Qorx Community Edition AGPL source line | 566 | 188476 | 334.00x | 3 |
| qorx carriers .qorx .qorxb qorx handle | 423 | 188619 | 446.91x | 2 |
| strict answer unsupported claims | 525 | 188517 | 360.08x | 4 |

## Claim Notes

This benchmark uses Qorx local accounting only. Token counts are deterministic
`ceil(chars / 4)` estimates unless the runtime reports another estimator. The
report does not claim provider invoice savings, production throughput, or
downstream model answer quality.

To reproduce:

```powershell
python scripts/run-benchmark.py --target . --suite qorx-live --budget-tokens 600 --squeeze-budget-tokens 450 --query "Qorx Community Edition AGPL source line" --query "qorx carriers .qorx .qorxb qorx handle" --query "strict answer unsupported claims" --supported-question "Qorx Community Edition AGPL source line" --unsupported-question d8ffdb34e9eb4e4085fbb7ab6067785f --agent-objective "prove Qorx Community Edition AGPL source line" --output-json docs/benchmarks/live.json --output-md docs/benchmarks/live.md
```
