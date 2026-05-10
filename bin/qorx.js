#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const binName = process.platform === "win32" ? "qorx.exe" : "qorx";
const root = path.resolve(__dirname, "..");
const candidates = [
  process.env.QORX_BIN,
  path.join(root, "vendor", binName),
  path.join(root, "vendor", "cargo", "bin", binName)
].filter(Boolean);

function firstExisting(paths) {
  for (const file of paths) {
    if (fs.existsSync(file)) return file;
  }
  return null;
}

const binary = firstExisting(candidates);
if (!binary) {
  console.error("qorx binary is not installed for this npm package.");
  console.error("Run `npm rebuild -g qorx`, install Rust/Cargo, or set QORX_BIN.");
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 0);
