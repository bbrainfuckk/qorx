# Security model

Qorx Void is local-first, but "local" is a boundary to verify, not a slogan.

## What stays local by default

- workspace files and the local index;
- session handles and local memory records;
- cache, provenance, and accounting state;
- resolver work performed by the local runtime;
- benchmark inputs and outputs unless the operator exports them.

## What can cross the boundary

- a carrier or evidence pack returned to an AI agent;
- exact cited lines requested for the current task;
- text the operator deliberately sends through a provider client;
- sanitized reports the operator chooses to publish.

A remote model cannot resolve a `qorx://` handle by itself. It only sees text or
artifacts that the local runtime or operator gives it.

## Network boundary

The public gateway binds to loopback by default. Do not expose it directly to an
untrusted network. A shared or non-loopback deployment needs authentication,
authorization, TLS, tenant isolation, rate limits, network policy, monitoring,
backups, and an upgrade plan supplied by the operator.

Qorx does not copy provider credentials. Provider clients keep their own login
and authentication state.

## Claim boundary

- Reduced context does not guarantee a correct answer.
- Evidence gating can refuse unsupported claims; it cannot guarantee that an
  outside model never hallucinates.
- A content hash supports integrity and reproducibility; it is not a signature,
  security audit, or blockchain proof by itself.
- Environmental output is a scenario unless its factors and measurement
  boundary are supplied.
- Any executable distributed to a user can be inspected. The public protection
  boundary is to publish only approved artifacts and keep proprietary Void
  source and operations out of this repository.

## Source boundary

This repository contains the open Qorx language, compiler, runtime, schemas,
and public CLI. It does not contain the proprietary Qorx Void desktop/service
source, private kernel internals, unpublished algorithms, prompts, customer
systems, signing keys, or production release procedures.

The public docs describe observable behavior and supported commands. They are
not a blueprint for rebuilding Qorx Void.

## Before publishing an artifact

Confirm that:

- every included file is approved for public release;
- secrets, private paths, usernames, hostnames, prompts, and customer data are absent;
- generated reports are sanitized;
- hashes and signatures are verified where the release promises them;
- claims match the measured hardware and workload;
- the package remains safe even if every byte is inspected.
