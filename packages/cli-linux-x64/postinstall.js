// Ensure the binary is executable after install
import { chmodSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const bin = resolve(dirname(fileURLToPath(import.meta.url)), "bin", "fishread");
if (existsSync(bin)) {
  chmodSync(bin, 0o755);
}
