# qorx npm wrapper

This npm package installs the Qorx Community Edition CLI from the public Git tag
using Cargo.

It does not meter local Community Edition commands. Qorx Edge Starter account
features still use the 5,000 included Edge/Cloud request allowance.

```sh
npm install -g qorx
qorx --version
```

Set `QORX_SKIP_INSTALL=1` to skip the Cargo install step, or `QORX_BIN` to point
the wrapper at an existing `qorx` binary.
