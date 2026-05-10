# qorx

![Qorx banner](https://raw.githubusercontent.com/bbrainfuckk/qorx/main/docs/assets/qorx-img.jpg)

Python wrapper for the Qorx Rust binary.

Use this package only after PyPI shows the current `0.0.1+ylem` version. Until
then, use the source install in the main repo docs.

```sh
pipx install qorx
qorx --version
```

Set `QORX_BIN` to point at an existing Qorx binary. If no binary is configured,
the wrapper downloads a matching GitHub release asset or builds from the GitHub
tag with Cargo.
