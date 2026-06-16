import { Text, Box, Spacer } from "@earendil-works/pi-tui";
import type { ReaderStateDto } from "../types.js";

export interface ChunkMessageDetails {
  state: ReaderStateDto;
}

export function renderChunk(
  content: string,
  details: ChunkMessageDetails,
  theme: any
): any {
  const { state } = details;
  const { chapter, progress } = state;

  const header =
    theme.fg("accent", `◆ ${chapter.title}`) +
    "  " +
    theme.fg(
      "dim",
      `第 ${progress.chapter_index + 1} 章 · 段落 ${progress.chunk_index + 1} · ${progress.chapter_percent.toFixed(0)}%`
    );

  const body = theme.fg("text", content);

  const navHints = [
    "/next 下一段",
    "/prev 上一段",
    "/toc 目录",
  ].join("  ");
  const footer = theme.fg("dim", `── ${navHints} ──`);

  const box = new Box(2, 1);
  box.addChild(new Text(header, 0, 1));
  box.addChild(new Text(body, 0, 1));
  box.addChild(new Spacer(1));
  box.addChild(new Text(footer, 0, 0));

  return box;
}
