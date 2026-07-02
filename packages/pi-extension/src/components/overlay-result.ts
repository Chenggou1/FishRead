export type BossKeyOverlayResult<TState> = { type: "boss-key"; state: TState };

export function isBossKeyOverlayResult<TState>(result: unknown): result is BossKeyOverlayResult<TState> {
  return !!result && typeof result === "object" && (result as { type?: unknown }).type === "boss-key";
}
