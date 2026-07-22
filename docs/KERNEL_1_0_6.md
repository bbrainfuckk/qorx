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

## Measured Development Artifacts

Native CI passed on Windows, Linux, and macOS for x64 and ARM64. Every artifact
passed the 1 MiB release gate and its downloaded bytes matched the emitted
SHA-256 sidecar.

| Target | Bytes | MiB | SHA-256 |
| --- | ---: | ---: | --- |
| Windows x64 | 229,888 | 0.22 | `85331b85b7ecabb724d4cba46ee41334dedf3ffec63bb91b85712e8fb507afd9` |
| Windows ARM64 | 215,552 | 0.21 | `1086347491b254658df83009a0119ac620020be72476b1f45ec7c5d3d786c6e6` |
| Linux x64 | 409,488 | 0.39 | `632db483aae490ebe1690b53444684a734320716924faf86892e7865cdea4f3d` |
| Linux ARM64 | 397,944 | 0.38 | `b10a3c69cc766696e44e9ce663387c8eb64f3b4a25e0a3058ae1bb4a01b97058` |
| macOS x64 | 378,704 | 0.36 | `73fba256dda675f42e9c2f6f8eb6c7e51e5a8895a26786f2738e8b7989c411f0` |
| macOS ARM64 | 386,016 | 0.37 | `62fbfc85af59c8fd2cda3965aceefe84e52d470d37818f99622fea0d700367f9` |

The Windows x64 CI artifact was also executed on a local Windows machine. It
reported Qorx 1.0.6, produced the expected `qorx.eco.v1` report, and completed
the scoped 13.2M descriptor lookup benchmark under one millisecond.

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
