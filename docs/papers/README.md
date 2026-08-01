# Qorx Papers

This directory contains public technical writing for Qorx.

Use the handbook as the implementation authority. Use papers for the argument,
not for unstated runtime behavior.

## Current paper

The current author-issued preprint is:

```text
Villanueva, Marvin Sarreal. Measuring Compact Local Evidence Carriers:
A Reproducibility Audit of Qorx. Zenodo, 2026.
DOI: 10.5281/zenodo.21739113
```

Record: https://doi.org/10.5281/zenodo.21739113

- [Main paper](dist/qorx-compact-local-evidence-carriers.pdf)
- [Supplementary artifact audit](dist/qorx-compact-local-evidence-carriers-supplement.pdf)
- [Published reproducibility bundle](https://zenodo.org/records/21739113/files/qorx-reproducibility-materials.zip?download=1)

The main paper and supplement are licensed under CC BY 4.0. The repository
code remains AGPL-3.0-only.

## Reproduce and audit

There are two separate proof paths:

1. **Paper artifact audit.** Download the reproducibility bundle from the
   canonical Zenodo record. It contains the archived May receipts, checksums,
   reconstruction scripts, manuscript source, and public testing guide. This
   path verifies the paper's fixed 56-byte carrier and addressing arithmetic.
2. **Current repository benchmark.** From the repository root, run:

   ```powershell
   python scripts/run-benchmark.py
   ```

   This regenerates the current Qorx 1.0.6 repository report in
   [`docs/benchmarks`](../benchmarks/README.md). It is a separate evaluation
   layer and does not reconstruct the historical May executable.

Independent source-to-binary reconstruction of the May executable remains a
documented follow-on step because the public artifacts do not identify its
exact historical source revision. Do not describe receipt verification as a
reproducible build.

## Earlier preprint

The earlier *Qorx Local Context Resolution* preprint remains available for
historical reference at https://doi.org/10.5281/zenodo.19953308.

## Current Files

| File | Purpose |
| --- | --- |
| `dist/qorx-compact-local-evidence-carriers.pdf` | Current published paper. |
| `dist/qorx-compact-local-evidence-carriers-supplement.pdf` | Current supplementary artifact audit. |
| `qorx-ai-language-paper.md` | Technical paper for Qorx as a language/runtime. |
| `qorx-local-context-resolution-preprint.md` | Earlier preprint source. |
| `dist/qorx-local-context-resolution-preprint.pdf` | Earlier preprint PDF. |
| `ZENODO-PREPRINT-UPLOAD.md` | Zenodo preprint record and upload fields. |
| `zenodo-preprint-metadata.json` | Zenodo metadata for the preprint record. |
| `qorx-terminology.md` | Coined Qorx terminology and boundary notes. |
| `qorx-preprint-plan.md` | DOI/preprint/journal route and submission checklist. |
| `qorx-evidence-map.md` | Claim-to-proof map. |
| `qorx-scientific-formulas.md` | Local accounting formulas. |
| `qorx-impact-context-paper.md` | Impact-context notes. |
| `ARTICLE-LICENSE.md` | Article license notice. |

## Rule

Do not imply peer review unless the paper has actually passed peer review. Do
not cite local Redshift accounting as provider invoice savings.

## Build the earlier preprint PDF

```powershell
python scripts/build-preprint-pdf.py
```
