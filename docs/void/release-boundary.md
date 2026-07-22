# Release boundary

Qorx 1.0.6 and Qorx Void have different publication boundaries.

## Public Qorx repository

This repository may publish:

- Qorx language, compiler, bytecode runtime, schemas, examples, and public CLI source;
- operator documentation for Qorx Void;
- public command and MCP contracts;
- benchmark summaries and sanitized reproducibility data;
- research references and citation metadata;
- package recipes and approved Qorx release assets;
- licenses, security policy, governance, and trademark terms.

## Qorx Void material excluded here

This repository must not publish:

- proprietary Qorx Void source code or private kernel implementation;
- unpublished selection, routing, memory, or grounding algorithms;
- private prompts, datasets, evaluation material, or model transcripts;
- signing keys, provider credentials, customer ledgers, or account data;
- private deployment topology, build systems, or release procedures;
- internal binaries or archives that have not passed a separate public-release audit.

## Distribution

The source tag and package channels in this repository distribute Qorx 1.0.6.
They are not a source distribution of Qorx Void. Void product distribution,
licensing, accounts, and updates are handled outside this repository.

Documentation may link to an approved Void download or account page. It must not
commit a private Void package into git or attach one to a public Qorx release
without a separate release review.

## Clean-room demos

The Qorx Zero hackathon repositories are allowed public examples because they
were built as standalone clean-room applications. They do not contain or depend
on private Qorx Void source, compiler internals, binaries, or datasets.

## Review rule

If a file would disclose more than an operator needs to install, use, inspect,
or verify the public behavior, keep it out of this repository until it receives
an explicit release review.
