import type { ExtensionAPI, ExtensionCommandContext, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { Key } from "@earendil-works/pi-tui";
import type { Component, TUI } from "@earendil-works/pi-tui";
import { readCurrent, readNext, readPrev } from "@fishread/sdk";
import type { ApiResponse, ReaderStateDto } from "@fishread/sdk";
import { renderChunk, type ChunkMessageDetails } from "./renderers/chunk.js";

const FR_SUBCOMMANDS = ["next", "prev"] as const;
const STATUS_KEY = "fishread";
const WIDGET_KEY = "fishread-reader";

// ── Global visibility gate ────────────────────────────────────────────────────
let fishreadVisible = true;
let lastStatusText: string | undefined;
let lastReaderState: ReaderStateDto | undefined;

// ── Helpers ───────────────────────────────────────────────────────────────────

function buildStatusText(state: ReaderStateDto, theme: any): string {
  const { book, chapter, progress } = state;
  return (
    theme.fg("accent", "◆") +
    " " +
    theme.fg("text", book.title) +
    theme.fg("dim", ` · ${chapter.title}`) +
    theme.fg(
      "dim",
      ` · 第 ${chapter.index + 1} 章 · ${progress.chapter_percent.toFixed(0)}% · 全书 ${progress.book_percent.toFixed(0)}%`
    )
  );
}

function syncStatusLine(ctx: ExtensionContext) {
  ctx.ui.setStatus(STATUS_KEY, fishreadVisible ? lastStatusText : undefined);
}

// Widget Component: reads module-level state on every render call.
class ChunkWidget implements Component {
  constructor(private theme: any) {}

  render(width: number): string[] {
    if (!fishreadVisible || !lastReaderState) return [];
    const box = renderChunk(
      lastReaderState.chunk.text,
      { state: lastReaderState } satisfies ChunkMessageDetails,
      this.theme
    );
    return box.render(width);
  }

  invalidate() {}
}

function mountWidget(ctx: ExtensionContext) {
  if (!ctx.hasUI) return;
  ctx.ui.setWidget(
    WIDGET_KEY,
    (_tui: TUI, theme: any): Component & { dispose?(): void } => new ChunkWidget(theme),
    { placement: "aboveEditor" }
  );
}

// ── Extension entry ───────────────────────────────────────────────────────────

export default function (pi: ExtensionAPI) {
  // On session start: init status line + mount persistent reading widget.
  pi.on("session_start", async (_event, ctx) => {
    const result = await readCurrent();
    if (!result.ok) {
      lastStatusText = ctx.ui.theme.fg("dim", "◆ FishRead · 使用 /fr import <path> 导入一本书开始阅读");
    } else {
      lastReaderState = result.data;
      lastStatusText = buildStatusText(result.data, ctx.ui.theme);
    }
    syncStatusLine(ctx);
    mountWidget(ctx);
  });

  // Boss key: flip gate → sync all UI immediately.
  pi.registerShortcut(Key.ctrlAlt("f"), {
    description: "FishRead Boss Key — 切换阅读 UI",
    handler: (ctx) => {
      fishreadVisible = !fishreadVisible;
      syncStatusLine(ctx);
      mountWidget(ctx); // re-mount so widget re-renders with new gate value
    },
  });

  // ── Navigation ──────────────────────────────────────────────────────────────

  function applyReaderState(ctx: ExtensionCommandContext, result: ApiResponse<ReaderStateDto>) {
    const { data } = result;
    lastReaderState = data;
    lastStatusText = buildStatusText(data, ctx.ui.theme);
    if (!fishreadVisible) return;
    syncStatusLine(ctx);
    mountWidget(ctx); // re-mount to push fresh state into the widget
  }

  async function handleNext(ctx: ExtensionCommandContext) {
    const result = await readNext();
    if (!result.ok) {
      ctx.ui.notify(`[fishread] ${result.error.code}: ${result.error.message}`, "error");
      return;
    }
    applyReaderState(ctx, result);
  }

  async function handlePrev(ctx: ExtensionCommandContext) {
    const result = await readPrev();
    if (!result.ok) {
      ctx.ui.notify(`[fishread] ${result.error.code}: ${result.error.message}`, "error");
      return;
    }
    applyReaderState(ctx, result);
  }

  pi.registerCommand("fr", {
    description: "FishRead — /fr <next|prev>",
    getArgumentCompletions: (prefix) => {
      return FR_SUBCOMMANDS
        .filter((s) => s.startsWith(prefix))
        .map((s) => ({ value: s, label: s }));
    },
    handler: async (args, ctx) => {
      const [sub] = args.trim().split(/\s+/);
      switch (sub) {
        case "next": return handleNext(ctx);
        case "prev": return handlePrev(ctx);
        default:
          ctx.ui.notify(
            `未知子命令: ${sub || "(空)"}。可用: ${FR_SUBCOMMANDS.join(", ")}`,
            "error"
          );
      }
    },
  });
}
