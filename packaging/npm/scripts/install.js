#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const pkg = require("../package.json");

if (process.env.QORX_SKIP_INSTALL === "1" || process.env.QORX_SKIP_INSTALL === "true") {
  console.log("Skipping Qorx install because QORX_SKIP_INSTALL is set.");
  process.exit(0);
}

const repo = process.env.QORX_INSTALL_REPO || "https://github.com/bbrainfuckk/qorx";
const ref = process.env.QORX_INSTALL_REF || pkg.qorxTag || `v${pkg.version}`;
const root = path.resolve(__dirname, "..", "vendor");

fs.mkdirSync(root, { recursive: true });

const args = ["install", "--git", repo, "--tag", ref, "--locked", "--root", root, "qorx"];
console.log(`Installing Qorx ${ref} with Cargo...`);
const result = spawnSync("cargo", args, {
  stdio: "inherit",
  shell: process.platform === "win32",
});

if (result.error) {
  console.error("Cargo is required to install this source-build npm wrapper.");
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
