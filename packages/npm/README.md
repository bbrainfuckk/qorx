# @brainfukk/qorx

![Qorx banner](https://raw.githubusercontent.com/bbrainfuckk/qorx/main/docs/assets/qorx-img.jpg)

npm wrapper for the Qorx Rust binary.

```sh
npm install -g @brainfukk/qorx
qorx --version
```

The installer downloads a matching GitHub release asset when one exists. If no
asset is available for the current platform, it tries:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --tag v0.0.1-ylem --locked qorx
```

Set `QORX_BIN` to use an existing local binary.
