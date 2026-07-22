# @brainfukk/qorx

![Qorx banner](https://raw.githubusercontent.com/bbrainfuckk/qorx/main/docs/assets/qorx-img.jpg)

npm wrapper for the Qorx Rust binary.

Use this package only after npm shows the current `1.0.6` version. Until
then, use the source install in the main repo docs.

```sh
npm install -g @brainfukk/qorx
qorx --version
```

The installer downloads a matching GitHub release asset when one exists. If no
asset is available for the current platform, it builds from the public source
tag with Cargo:

```sh
cargo install --git https://github.com/bbrainfuckk/qorx --tag v1.0.6 --locked qorx
```

Set `QORX_BIN` to use an existing local binary.
