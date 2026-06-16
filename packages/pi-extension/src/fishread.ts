import { spawnSync } from "node:child_process";
import { resolveFishreadPath } from "@fishread/cli";
import type { ApiResult, ReaderStateDto } from "./types.js";

function run(args: string[]): ApiResult<unknown> {
  const bin = resolveFishreadPath();
  const result = spawnSync(bin, args, { encoding: "utf8" });
  if (result.error) {
    throw new Error(`fishread spawn failed: ${result.error.message}`);
  }
  return JSON.parse(result.stdout) as ApiResult<unknown>;
}

export function readCurrent(): ApiResult<ReaderStateDto> {
  return run(["read", "current"]) as ApiResult<ReaderStateDto>;
}

export function readNext(): ApiResult<ReaderStateDto> {
  return run(["read", "next"]) as ApiResult<ReaderStateDto>;
}

export function readPrev(): ApiResult<ReaderStateDto> {
  return run(["read", "prev"]) as ApiResult<ReaderStateDto>;
}
