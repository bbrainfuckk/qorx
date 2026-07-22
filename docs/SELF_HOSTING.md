# Qorx compiler bootstrap

Qorx 1.0.5 has a real parser, semantic checker, compiler, bytecode format, and
interpreter. The shipping compiler is currently bootstrapped in Rust. That is a
normal starting point for a language implementation, but it is not the same as
being written and compiled in Qorx.

## Current proof

```text
.qorx source
  -> parser and semantic checker
  -> AST
  -> QIR
  -> canonical opcode and QSTK streams
  -> integrity-checked .qorxb bytecode
  -> local interpreter
```

The implementation and tests for that pipeline live in `src/qorx.rs`,
`src/qorx_language.rs`, and the language test suite.

## Self-hosting gate

Qorx will use the label “self-hosted” only after all of these are reproducible:

1. The Rust bootstrap compiler compiles `compiler/qorxc.qorx` into stage 1.
2. The stage-1 compiler compiles the same Qorx compiler source into stage 2.
3. Stage-1 and stage-2 canonical bytecode SHA-256 hashes are identical.
4. The language and tamper-rejection test suites pass under stage 2.

`scripts/check-bootstrap.ps1` enforces the manifest and refuses a
`self_hosted: true` declaration unless the compiler source and both matching
stage artifacts exist.

Qorx 1 does not yet expose enough general parsing and file primitives to write
that compiler without native bootstrap support, so the 1.0.5 manifest says
`self_hosted: false`. A wrapper that merely calls the Rust compiler would not be
self-hosting and is deliberately not counted as proof.

## Inspectability and protection

Source code published under an open-source license is inspectable and can be
reverse engineered. No compiler, obfuscator, bytecode envelope, or unusual
implementation language can guarantee otherwise. Qorx bytecode integrity hashes
detect tampering; they do not hide algorithms. Secrets, hosted orchestration,
private datasets, signing keys, and private service policy must remain outside
this repository when they are intended to stay private.
