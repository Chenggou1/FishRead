#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { resolveFishreadPath } from "../index.js";

let bin;
try {
  bin = resolveFishreadPath();
} catch (err) {
  console.error(`[fishread] ${err.message}`);
  process.exit(1);
}

const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(`[fishread] ${result.error.message}`);
  process.exit(1);
}

process.exit(typeof result.status === "number" ? result.status : 1);
