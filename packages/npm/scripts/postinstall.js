"use strict";

const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const pkg = require("../package.json");
const repo = "https://github.com/bbrainfuckk/qorx";
const version = pkg.version;
const tag = `v${version}`;
const binName = process.platform === "win32" ? "qorx.exe" : "qorx";
const root = path.resolve(__dirname, "..");
const vendorDir = path.join(root, "vendor");
const vendorBin = path.join(vendorDir, binName);

function assetName() {
  const arch = process.arch === "x64" ? "x64" : process.arch === "arm64" ? "arm64" : null;
  if (!arch) return null;
  if (process.platform === "win32" && arch === "x64") return `qorx-${tag}-windows-x64.zip`;
  if (process.platform === "linux") return `qorx-${tag}-linux-${arch}.tar.gz`;
  if (process.platform === "darwin") return `qorx-${tag}-macos-${arch}.tar.gz`;
  return null;
}

function run(command, args, options = {}) {
  return spawnSync(command, args, { stdio: "inherit", ...options });
}

function commandWorks(command) {
  const probe = process.platform === "win32"
    ? spawnSync("where.exe", [command], { stdio: "ignore" })
    : spawnSync("sh", ["-c", `command -v ${command}`], { stdio: "ignore" });
  return probe.status === 0;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const request = https.get(url, response => {
      if ([301, 302, 303, 307, 308].includes(response.statusCode)) {
        return download(response.headers.location, dest).then(resolve, reject);
      }
      if (response.statusCode !== 200) {
        response.resume();
        reject(new Error(`download failed with HTTP ${response.statusCode}`));
        return;
      }
      const file = fs.createWriteStream(dest);
      response.pipe(file);
      file.on("finish", () => file.close(resolve));
      file.on("error", reject);
    });
    request.on("error", reject);
  });
}

function findBinary(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      const found = findBinary(full);
      if (found) return found;
    } else if (entry.name === binName || entry.name === "qorx") {
      return full;
    }
  }
  return null;
}

function extract(archive, outDir) {
  fs.rmSync(outDir, { recursive: true, force: true });
  fs.mkdirSync(outDir, { recursive: true });
  if (archive.endsWith(".zip")) {
    const tarResult = run("tar", ["-xf", archive, "-C", outDir]);
    if (tarResult.status === 0) return;
    if (process.platform === "win32") {
      const result = run("powershell.exe", [
        "-NoProfile",
        "-Command",
        `Import-Module Microsoft.PowerShell.Archive; Expand-Archive -LiteralPath '${archive.replace(/'/g, "''")}' -DestinationPath '${outDir.replace(/'/g, "''")}' -Force`
      ]);
      if (result.status !== 0) throw new Error("Expand-Archive failed");
    } else {
      const result = run("unzip", ["-o", archive, "-d", outDir]);
      if (result.status !== 0) throw new Error("unzip failed");
    }
  } else {
    const result = run("tar", ["-xzf", archive, "-C", outDir]);
    if (result.status !== 0) throw new Error("tar extraction failed");
  }
}

function cargoInstall() {
  if (!commandWorks("cargo")) return false;
  const cargoRoot = path.join(vendorDir, "cargo");
  fs.mkdirSync(cargoRoot, { recursive: true });
  const result = run("cargo", [
    "install",
    "--git",
    "https://github.com/bbrainfuckk/qorx",
    "--tag",
    tag,
    "--locked",
    "--root",
    cargoRoot,
    "qorx"
  ]);
  return result.status === 0;
}

async function main() {
  if (fs.existsSync(vendorBin) || process.env.QORX_SKIP_DOWNLOAD === "1") return;
  fs.mkdirSync(vendorDir, { recursive: true });

  const asset = assetName();
  if (asset) {
    const url = `${repo}/releases/download/${tag}/${asset}`;
    const tmp = path.join(os.tmpdir(), asset);
    const extracted = path.join(os.tmpdir(), `qorx-${process.pid}`);
    try {
      await download(url, tmp);
      extract(tmp, extracted);
      const binary = findBinary(extracted);
      if (!binary) throw new Error("archive did not contain qorx binary");
      fs.copyFileSync(binary, vendorBin);
      if (process.platform !== "win32") fs.chmodSync(vendorBin, 0o755);
      return;
    } catch (error) {
      console.warn(`qorx asset install skipped: ${error.message}`);
    } finally {
      fs.rmSync(extracted, { recursive: true, force: true });
    }
  }

  if (!cargoInstall()) {
    console.warn("qorx was installed without a bundled binary. Install Rust/Cargo and run `npm rebuild qorx`, or set QORX_BIN.");
  }
}

main().catch(error => {
  console.warn(`qorx postinstall warning: ${error.message}`);
});
