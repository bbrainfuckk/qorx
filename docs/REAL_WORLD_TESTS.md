# Qorx Real-World Test Check

Use this check before a public release.

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
.\target\release\qorx.exe --version
.\target\release\qorx.exe index .
.\target\release\qorx.exe session
.\target\release\qorx.exe strict-answer "which files explain the resolver proof flow?"
.\target\release\qorx.exe context snapshot
.\target\release\qorx.exe context verify
.\target\release\qorx.exe security attest
```

The release passes when all commands succeed and the evidence routes cite local
state instead of unsupported claims.
