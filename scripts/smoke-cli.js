#!/usr/bin/env node
/**
 * Smoke test for the npm CLI wrapper (@fishread/cli).
 * Verifies that the wrapper can locate the Rust binary and invoke it correctly.
 *
 * Usage: node scripts/smoke-cli.js
 */
import { spawnSync } from "node:child_process";
import { existsSync, copyFileSync, chmodSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");

let passed = 0;
let failed = 0;

function check(label, ok, detail = "") {
  if (ok) {
    console.log(`  ✓  ${label}`);
    passed++;
  } else {
    console.error(`  ✗  ${label}${detail ? ` — ${detail}` : ""}`);
    failed++;
  }
}

function runWrapper(extraEnv = {}) {
  const env = { ...process.env, ...extraEnv };
  return spawnSync("node", ["packages/cli/bin/fishread.js", "--help"], {
    cwd: ROOT,
    encoding: "utf8",
    env,
  });
}

console.log("── FishRead CLI wrapper smoke test ──\n");

// 1. Dev binary exists (prerequisite)
const devBinary = resolve(ROOT, "rust/target/debug/fishread");
const devBinaryExists = existsSync(devBinary);
check("dev binary exists (run `cargo build -p fishread-cli` first)", devBinaryExists, devBinary);

// 2. FISHREAD_CLI_PATH env override
if (devBinaryExists) {
  const r = runWrapper({ FISHREAD_CLI_PATH: devBinary });
  check("FISHREAD_CLI_PATH override → fishread --help exits 0", r.status === 0, r.stderr?.trim());
}

// 3. Dev binary auto-detection (no FISHREAD_CLI_PATH)
if (devBinaryExists) {
  const r = runWrapper({ FISHREAD_CLI_PATH: "" });
  check("dev binary auto-detection → fishread --help exits 0", r.status === 0, r.stderr?.trim());
}

// 4. Platform package binary
const platform = process.platform;
const arch = process.arch;
const pkgName = `cli-${platform}-${arch}`;
const pkgDir = resolve(ROOT, "packages", pkgName);

if (existsSync(pkgDir) && devBinaryExists) {
  const binName = platform === "win32" ? "fishread.exe" : "fishread";
  const platformBin = resolve(pkgDir, "bin", binName);

  try {
    copyFileSync(devBinary, platformBin);
    if (platform !== "win32") chmodSync(platformBin, 0o755);
    check(`platform package binary staged (${pkgName}/bin/${binName})`, existsSync(platformBin));
  } catch (e) {
    check(`platform package binary staged (${pkgName})`, false, String(e));
  }
} else {
  console.log(`  ~  platform package ${pkgName} not found — skipping platform resolution test`);
}

// 5. Missing binary gives a clear error (not a crash with no message)
{
  const r = runWrapper({ FISHREAD_CLI_PATH: "/nonexistent/fishread" });
  const hasError = r.status !== 0;
  check("missing binary → non-zero exit with message", hasError, r.stderr?.trim() || "(no stderr)");
}

console.log(`\n── ${passed} passed, ${failed} failed ──`);
if (failed > 0) process.exit(1);
