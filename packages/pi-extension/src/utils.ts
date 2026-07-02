export function clampIndex(index: number, length: number): number {
  if (length <= 0) return 0;
  return Math.max(0, Math.min(length - 1, index));
}

export function splitCommandArgs(args: string): { subcommand: string; rest: string } {
  const trimmed = args.trim();
  if (!trimmed) return { subcommand: "", rest: "" };

  const match = /^(\S+)(?:\s+([\s\S]*))?$/.exec(trimmed);
  return {
    subcommand: match?.[1] ?? "",
    rest: match?.[2]?.trim() ?? "",
  };
}
