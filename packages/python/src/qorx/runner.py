from __future__ import annotations

import hashlib
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

VERSION = "1.0.6"
TAG = f"v{VERSION}"
REPO = "https://github.com/bbrainfuckk/qorx"


def bin_name() -> str:
    return "qorx.exe" if os.name == "nt" else "qorx"


def cache_dir() -> Path:
    if os.name == "nt":
        root = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local"))
    elif sys.platform == "darwin":
        root = Path.home() / "Library" / "Caches"
    else:
        root = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
    return root / "qorx" / VERSION


def asset_name() -> str | None:
    arch = platform.machine().lower()
    if arch in {"amd64", "x86_64"}:
        arch = "x64"
    elif arch in {"aarch64", "arm64"}:
        arch = "arm64"
    else:
        return None

    if sys.platform.startswith("win"):
        return f"qorx-{TAG}-windows-{arch}.zip"
    if sys.platform.startswith("linux"):
        return f"qorx-{TAG}-linux-{arch}.tar.gz"
    if sys.platform == "darwin":
        return f"qorx-{TAG}-macos-{arch}.tar.gz"
    return None


def configured_binary() -> Path | None:
    override = os.environ.get("QORX_BIN")
    if override:
        path = Path(override)
        if path.exists():
            return path
    path = cache_dir() / "bin" / bin_name()
    return path if path.exists() else None


def find_binary(root: Path) -> Path | None:
    for path in root.rglob("*"):
        if path.name in {bin_name(), "qorx"} and path.is_file():
            return path
    return None


def verify_download(archive: Path, checksum: Path) -> None:
    expected = checksum.read_text(encoding="utf-8").split()[0].lower()
    actual = hashlib.sha256(archive.read_bytes()).hexdigest()
    if actual != expected:
        raise RuntimeError(f"Qorx release checksum mismatch: expected {expected}, got {actual}")


def extract_tar_safely(archive_path: Path, destination: Path) -> None:
    root = destination.resolve()
    with tarfile.open(archive_path, "r:gz") as archive:
        for member in archive.getmembers():
            target = (destination / member.name).resolve()
            if os.path.commonpath((root, target)) != str(root) or member.issym() or member.islnk():
                raise RuntimeError(f"unsafe path in Qorx release archive: {member.name}")
        archive.extractall(destination)


def extract_zip_safely(archive_path: Path, destination: Path) -> None:
    root = destination.resolve()
    with zipfile.ZipFile(archive_path) as archive:
        for member in archive.infolist():
            target = (destination / member.filename).resolve()
            is_symlink = (member.external_attr >> 16) & 0o170000 == 0o120000
            if os.path.commonpath((root, target)) != str(root) or is_symlink:
                raise RuntimeError(f"unsafe path in Qorx release archive: {member.filename}")
        archive.extractall(destination)


def download_asset() -> Path | None:
    asset = asset_name()
    if not asset:
        return None

    url = f"{REPO}/releases/download/{TAG}/{asset}"
    tmp = cache_dir() / "download" / asset
    checksum = tmp.with_name(f"{asset}.sha256")
    out = cache_dir() / "extract"
    tmp.parent.mkdir(parents=True, exist_ok=True)
    out.mkdir(parents=True, exist_ok=True)

    try:
        urllib.request.urlretrieve(url, tmp)
        urllib.request.urlretrieve(f"{url}.sha256", checksum)
        verify_download(tmp, checksum)
    except (OSError, RuntimeError, urllib.error.URLError):
        return None

    shutil.rmtree(out, ignore_errors=True)
    out.mkdir(parents=True, exist_ok=True)
    if asset.endswith(".zip"):
        extract_zip_safely(tmp, out)
    else:
        extract_tar_safely(tmp, out)

    binary = find_binary(out)
    if not binary:
        return None
    dest = cache_dir() / "bin" / bin_name()
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(binary, dest)
    if os.name != "nt":
        dest.chmod(0o755)
    return dest


def cargo_install() -> Path | None:
    if not shutil.which("cargo"):
        return None
    root = cache_dir() / "cargo"
    result = subprocess.run(
        [
            "cargo",
            "install",
            "--git",
            "https://github.com/bbrainfuckk/qorx",
            "--tag",
            TAG,
            "--locked",
            "--root",
            str(root),
            "qorx",
        ],
        check=False,
    )
    if result.returncode != 0:
        return None
    binary = root / "bin" / bin_name()
    return binary if binary.exists() else None


def ensure_binary() -> Path:
    binary = configured_binary() or download_asset() or cargo_install()
    if binary and binary.exists():
        return binary
    raise SystemExit(
        "qorx binary is unavailable. Set QORX_BIN, install Rust/Cargo, "
        "or use a platform with a Qorx release asset. If this version is not "
        "published yet, install from the public source tag with Cargo."
    )


def main() -> int:
    binary = ensure_binary()
    return subprocess.call([str(binary), *sys.argv[1:]])


if __name__ == "__main__":
    raise SystemExit(main())
