# Qorx 1.0.6 Compact Kernel Contract

Status: development contract. Qorx 1.0.5 remains the published stable release.

The compact kernel is the offline-only Qorx runtime intended for developers,
engineers, researchers, legal teams, scientists, students, local AI agents, and
general local knowledge work. It does not include an HTTP client or server,
telemetry, an updater, accounts, cloud adapters, or a model downloader.

## Public Command Surface

| Command | Contract |
| --- | --- |
| `qorx check <file.qorx>` | Parse and validate local Qorx source. |
| `qorx compile <file.qorx> -o <file.qorxb>` | Compile deterministic local bytecode. |
| `qorx run <file.qorxb>` | Execute supported bytecode without a provider call. |
| `qorx evidence <file> <query>` | Emit exact matching source lines or refuse. |
| `qorx receipt <file>` | Emit a SHA-256 content receipt. |
| `qorx context pack <file> --tokens N` | Create a compact local context receipt with a declared token count. |
| `qorx context verify <file.qctx>` | Stream and verify the referenced local file. |
| `qorx context carrier <file.qctx>` | Emit the compact content-addressed carrier. |
| `qorx eco ...` | Calculate supplied token counts and opt-in environmental scenarios. |
| `qorx bench ...` | Measure in-memory handle resolution under a disclosed scope. |

The command surface is domain-neutral. Local files can be source code, technical
documentation, research notes, contracts, case material, operating procedures,
or any other text the operator is authorized to use.

## Measured Development Artifact

The current Windows x64 development binary is 229,376 bytes (0.219 MiB). Its
SHA-256 is
`8bb9def221a2eedd5067aea2af45e126781258b787dcd2f19726b625b24f9f65`.
The release gate is 1 MiB on every native target.

Native CI is defined for Windows, Linux, and macOS on x64 and ARM64. An artifact
is not considered available until that native job passes and publishes its own
checksum.

## Claims Boundary

- The 13.2M benchmark configures a local context descriptor and measures
  in-memory handle resolution. It does not load or semantically retrieve a
  13.2M-token corpus and does not measure model inference.
- Evidence mode emits exact local source lines or refuses. This sharply limits
  unsupported synthesis inside Qorx, but it cannot guarantee that another local
  model will never hallucinate.
- `qorx eco` treats token counts and conversion factors as user supplied. Energy,
  CO2e, and water values are scenarios because impact varies by hardware,
  workload, electricity source, cooling, and reporting boundary.
- SHA-256 receipts support content addressing and reproducibility. They are not
  digital signatures, a blockchain, a crypto token, or a security audit.
- The compact binary is stripped and its implementation source is not part of
  this public contract. A shipped executable can still be analyzed; no binary
  can be made impossible to reverse engineer.
- Qorx makes no network calls in the compact kernel. A program running on a
  user-controlled computer cannot prevent the user from piping its output into
  another program or network client.
- The current compact host is bootstrapped in Rust. Self-hosting in Qorx is not
  claimed until stage-1 and stage-2 compiler outputs reproduce and verify.

## Public And Protected Layers

The public layer contains the language and command contracts, schemas, package
launchers, examples, and non-sensitive accounting such as `qorx eco`. Protected
kernel implementation details stay outside the public repository. Release
binaries carry checksums and must pass the same public command contract.

