# qorx

<!-- mcp-name: io.github.bbrainfuckk/qorx -->

![Qorx banner](https://raw.githubusercontent.com/bbrainfuckk/qorx/main/docs/assets/qorx-img.jpg)

Python wrapper for the Qorx Rust binary.

Install the matching version once PyPI shows that release. For versions not
yet on PyPI, use the source install in the main repository.

```sh
pipx install qorx==1.0.7
qorx --version
```

Set `QORX_BIN` to point at an existing Qorx binary. If no binary is configured,
the wrapper downloads a matching GitHub release asset or builds from the GitHub
tag with Cargo.

## MCP server

Once the matching release is on PyPI, configure an MCP client with command
`uvx` and arguments `qorx@1.0.7` and `mcp`. If installing the wrapper
manually, run `qorx mcp` after installation. The server uses stdio.

Before querying a workspace, run `qorx index <workspace-path>` with the
same Qorx data directory the MCP server will use. See the [stdio setup in
the main README](https://github.com/bbrainfuckk/qorx#connect-an-mcp-client).
