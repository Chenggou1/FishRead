import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const PLATFORM_PACKAGES = {
  "darwin-arm64": "@fishread/cli-darwin-arm64",
  "darwin-x64": "@fishread/cli-darwin-x64",
  "linux-arm64": "@fishread/cli-linux-arm64",
  "linux-x64": "@fishread/cli-linux-x64",
  "win32-x64": "@fishread/cli-win32-x64",
};

// Lazy: 不在 import 时 resolve，避免二进制不存在就抛错
export function resolveFishreadPath() {
  // 1. 环境变量覆盖
  if (process.env.FISHREAD_CLI_PATH) {
    return process.env.FISHREAD_CLI_PATH;
  }

  // 2. 本地开发 binary（仓库内 rust/target/debug/）
  const devBinary = resolve(__dirname, "../../rust/target/debug/fishread");
  if (existsSync(devBinary)) {
    return devBinary;
  }

  // 3. npm 平台包
  const key = `${process.platform}-${process.arch}`;
  const pkgName = PLATFORM_PACKAGES[key];
  if (!pkgName) {
    throw new Error(
      `Unsupported platform: ${key}. Supported: ${Object.keys(PLATFORM_PACKAGES).join(", ")}`
    );
  }

  try {
    const pkgJsonPath = require.resolve(`${pkgName}/package.json`);
    const ext = process.platform === "win32" ? ".exe" : "";
    const binary = resolve(dirname(pkgJsonPath), "bin", `fishread${ext}`);
    if (existsSync(binary)) {
      return binary;
    }
  } catch {
    // optional dependency not installed
  }

  throw new Error(
    `fishread binary not found. Install ${pkgName} or set FISHREAD_CLI_PATH.`
  );
}
