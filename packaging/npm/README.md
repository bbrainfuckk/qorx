# qorx npm wrapper

This npm package installs Qorx from the matching public Git tag
using Cargo. Publish it only after the source tag exists and npm is ready for
the current version.

It runs local Qorx language and compiler commands without a hosted dependency.

```sh
npm install -g qorx
qorx --version
```

Set `QORX_SKIP_INSTALL=1` to skip the Cargo install step, or `QORX_BIN` to point
the wrapper at an existing `qorx` binary.
