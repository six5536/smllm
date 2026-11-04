#!/usr/bin/env node
"use strict";

// Thin launcher: resolve the prebuilt binary for this platform and run it,
// forwarding arguments, stdio, and the exit code.

const { spawnSync } = require("node:child_process");
const { resolveBinary, exitCode } = require("../lib/binary");

let binary;
// sokf-implements: LNCH-1_AC-5
// sokf-implements: LNCH-1_AC-6
// sokf-implements: LNCH-1_AC-8
try {
  binary = resolveBinary(process.platform, process.arch);
} catch (err) {
  process.stderr.write(`${err.message}\n`);
  process.exit(1);
}

// sokf-implements: LNCH-1_AC-1
const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });

// sokf-implements: LNCH-1_AC-7
// sokf-implements: LNCH-1_AC-8
if (result.error) {
  process.stderr.write(`failed to run smllm: ${result.error.message}\n`);
  process.exit(1);
}

// sokf-implements: LNCH-1_AC-2
process.exit(exitCode(result));
