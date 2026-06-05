# qorx-free

`qorx-free` is the public benchmark and reproducibility build for Linux AMD MI300X testers.

It is not Qorx Void. It is a small public surface that verifies the public bundle, checks the AMD MI300X hardware boundary, and writes sanitized benchmark artifacts.

## Public Commands

```sh
./qorx-free --version
./qorx-free hardware
./qorx-free doctor
./qorx-free demo
./qorx-free verify-demo
./qorx-free boundary
./qorx-free amd-run --suite big10 --sample 30 --distractors 12 --out ./qorx-free-run
```

## What It Is For

`qorx-free` helps testers:

- verify the public bundle has not been tampered with;
- check Linux AMD MI300X readiness;
- reproduce the public benchmark shape;
- produce issue-ready sanitized artifacts;
- compare behavior across ROCm, kernel, and machine changes.

## What It Does Not Include

`qorx-free` does not include Qorx Void, source, unpublished implementation material, sensitive operational details, private data, or build and release procedures.

## Release Boundary

The public release asset may contain:

- `qorx-free`;
- public bundle files;
- release manifest;
- release manifest signature;
- public verification key;
- checksums;
- README, boundary, license, and notice text.

The public release asset must not contain source-like files, archives inside the package, private paths, credentials, private logs, or private benchmark artifacts.
