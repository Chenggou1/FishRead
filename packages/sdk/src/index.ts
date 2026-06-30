import { spawn } from "node:child_process";
import { resolveFishreadPath } from "@fishread/cli";
import { PROTOCOL_VERSION, type ApiResult, type ReaderStateDto } from "./types.js";

export * from "./types.js";

export class FishReadSdkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "FishReadSdkError";
  }
}

async function run(args: string[]): Promise<ApiResult<unknown>> {
  const bin = resolveFishreadPath();
  const child = spawn(bin, args, { stdio: ["ignore", "pipe", "pipe"] });

  const stdoutChunks: Buffer[] = [];
  const stderrChunks: Buffer[] = [];

  child.stdout.on("data", (chunk: Buffer) => stdoutChunks.push(chunk));
  child.stderr.on("data", (chunk: Buffer) => stderrChunks.push(chunk));

  const exitCode = await new Promise<number>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code) => resolve(typeof code === "number" ? code : 1));
  });

  const stdout = Buffer.concat(stdoutChunks).toString("utf8");
  const stderr = Buffer.concat(stderrChunks).toString("utf8").trim();

  let parsed: unknown;
  try {
    parsed = JSON.parse(stdout);
  } catch (err) {
    const details = stderr ? ` stderr: ${stderr}` : "";
    throw new FishReadSdkError(
      `fishread returned invalid JSON with exit code ${exitCode}.${details}`
    );
  }

  if (!isProtocolResult(parsed)) {
    throw new FishReadSdkError("fishread returned a response outside the CLI JSON Protocol");
  }

  return parsed;
}

function isProtocolResult(value: unknown): value is ApiResult<unknown> {
  if (!value || typeof value !== "object") {
    return false;
  }

  const response = value as {
    protocol_version?: unknown;
    ok?: unknown;
    data?: unknown;
    error?: { code?: unknown; message?: unknown };
  };
  if (response.protocol_version !== PROTOCOL_VERSION) {
    throw new FishReadSdkError(
      `Unsupported FishRead protocol version: ${String(response.protocol_version)}`
    );
  }

  if (response.ok === true) {
    return "data" in response;
  }

  if (response.ok === false) {
    return (
      !!response.error &&
      typeof response.error.code === "string" &&
      typeof response.error.message === "string"
    );
  }

  return false;
}

export function readCurrent(): Promise<ApiResult<ReaderStateDto>> {
  return run(["read", "current"]) as Promise<ApiResult<ReaderStateDto>>;
}

export function readNext(): Promise<ApiResult<ReaderStateDto>> {
  return run(["read", "next"]) as Promise<ApiResult<ReaderStateDto>>;
}

export function readPrev(): Promise<ApiResult<ReaderStateDto>> {
  return run(["read", "prev"]) as Promise<ApiResult<ReaderStateDto>>;
}
