# Qorx PyPI wrapper

This PyPI package exposes the `qorx` command and installs the Qorx Community
Edition CLI from the public Git tag using Cargo.

It does not meter local Community Edition commands. Qorx Ayie Starter account
features still use the 5,000 included Ayie/Cloud request allowance.

```sh
python -m pip install qorx
qorx --version
```

Set `QORX_BIN` to use an existing binary or `QORX_INSTALL_REF` to install a
specific Git tag.
