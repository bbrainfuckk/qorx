#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const exeName = process.platform === "win32" ? "qorx.exe" : "qorx";
const packageRoot = path.resolve(__dirname, "..");
const installedBin = path.join(packageRoot, "vendor", "bin", exeName);
const command = process.env.QORX_BIN || installedBin;

if (!fs.existsSync(command)) {
  console.error(
    [
      "qorx binary was not found.",
      "Run `npm rebuild qorx`, install Cargo, or set QORX_BIN to an existing qorx binary.",
    ].join(" "),
  );
  process.exit(1);
}

const result = spawnSync(command, process.argv.slice(2), {
  stdio: "inherit",
  shell: process.platform === "win32",
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
