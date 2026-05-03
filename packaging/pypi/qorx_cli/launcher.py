from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

VERSION = "1.0.4"
REPO = "https://github.com/bbrainfuckk/qorx"


def _exe_name() -> str:
    return "qorx.exe" if os.name == "nt" else "qorx"


def _install_root() -> Path:
    override = os.environ.get("QORX_PYTHON_INSTALL_ROOT")
    if override:
        return Path(override).expanduser()
    return Path.home() / ".qorx" / "pypi" / VERSION


def _binary_path() -> Path:
    return _install_root() / "bin" / _exe_name()


def _install(binary: Path) -> None:
    root = _install_root()
    root.mkdir(parents=True, exist_ok=True)
    ref = os.environ.get("QORX_INSTALL_REF", f"v{VERSION}")
    repo = os.environ.get("QORX_INSTALL_REPO", REPO)
    command = [
        "cargo",
        "install",
        "--git",
        repo,
        "--tag",
        ref,
        "--locked",
        "--root",
        str(root),
        "qorx",
    ]
    try:
        subprocess.check_call(command)
    except FileNotFoundError as exc:
        raise RuntimeError("Cargo is required to install the Qorx PyPI wrapper.") from exc
    if not binary.exists():
        raise RuntimeError(f"Cargo install completed but {binary} was not created.")


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    env_binary = os.environ.get("QORX_BIN")
    binary = Path(env_binary) if env_binary else _binary_path()
    if not binary.exists():
        _install(binary)
    return subprocess.call([str(binary), *args])
