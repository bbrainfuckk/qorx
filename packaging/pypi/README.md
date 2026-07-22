# Qorx PyPI wrapper

This PyPI package exposes the `qorx` command and installs Qorx from the matching
public Git tag using Cargo.

It runs local Qorx language and compiler commands without a hosted dependency.

```sh
python -m pip install qorx
qorx --version
```

Set `QORX_BIN` to use an existing binary or `QORX_INSTALL_REF` to install a
specific Git tag.
