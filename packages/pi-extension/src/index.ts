import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { Key } from "@earendil-works/pi-tui";
import type { Component, TUI } from "@earendil-works/pi-tui";
import { readCurrent, readNext, readPrev } from "@fishread/sdk";
import type { ApiResponse, ReaderStateDto } from "@fishread/sdk";
import { renderChunk, type ChunkMessageDetails } from "./renderers/chunk.js";

const FR_SUBCOMMANDS = ["next", "prev"] as const;
const BOSS_KEY = Key.ctrl("q");
const NEXT_PAGE_KEY = Key.ctrlShift("right");
const PREV_PAGE_KEY = Key.ctrlShift("left");
const NEXT_PAGE_KEY_LABEL = "ctrl+shift+right";
const PREV_PAGE_KEY_LABEL = "ctrl+shift+left";
const STATUS_KEY = "fishread";
const WIDGET_KEY = "fishread-reader";

const FR_SUBCOMMAND_DETAILS: Record<
  (typeof FR_SUBCOMMANDS)[number],
  { description: string; shortcut: string }
> = {
  next: { description: "下一页", shortcut: NEXT_PAGE_KEY_LABEL },
  prev: { description: "上一页", shortcut: PREV_PAGE_KEY_LABEL },
};

type FishReadSurfaceId = "status" | "reader";

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

async function reloadReaderState(ctx: ExtensionContext): Promise<void> {
  try {
    const result = await readCurrent();
    if (!result.ok) {
      lastReaderState = undefined;
      lastStatusText = ctx.ui.theme.fg("dim", "◆ FishRead · 使用 /fr import <path> 导入一本书开始阅读");
      return;
    }

    lastReaderState = result.data;
    lastStatusText = buildStatusText(result.data, ctx.ui.theme);
  } catch {
    lastReaderState = undefined;
    lastStatusText = undefined;
  }
}

function showStatusLine(ctx: ExtensionContext) {
  ctx.ui.setStatus(STATUS_KEY, lastStatusText);
}

function hideStatusLine(ctx: ExtensionContext) {
  ctx.ui.setStatus(STATUS_KEY, undefined);
}

// Widget Component: reads module-level state on every render call.
class ChunkWidget implements Component {
  constructor(private theme: any) {}

  render(width: number): string[] {
    if (!lastReaderState) return [];
    const box = renderChunk(
      lastReaderState.chunk.text,
      { state: lastReaderState } satisfies ChunkMessageDetails,
      this.theme
    );
    return box.render(width);
  }

  invalidate() {}
}

function showReaderWidget(ctx: ExtensionContext) {
  if (!ctx.hasUI) return;
  ctx.ui.setWidget(
    WIDGET_KEY,
    (_tui: TUI, theme: any): Component & { dispose?(): void } => new ChunkWidget(theme),
    { placement: "aboveEditor" }
  );
}

function hideReaderWidget(ctx: ExtensionContext) {
  if (!ctx.hasUI) return;
  ctx.ui.setWidget(WIDGET_KEY, undefined);
}

interface FishReadSurface {
  id: FishReadSurfaceId;
  show(ctx: ExtensionContext): void;
  hide(ctx: ExtensionContext): void;
}

class BossKeyController {
  private hidden = false;
  private readonly surfaces = new Map<FishReadSurfaceId, FishReadSurface>();
  private readonly activeSurfaces = new Set<FishReadSurfaceId>();
  private restoreSurfaces = new Set<FishReadSurfaceId>();

  register(surface: FishReadSurface): void {
    this.surfaces.set(surface.id, surface);
  }

  isHidden(): boolean {
    return this.hidden;
  }

  show(ctx: ExtensionContext, surfaceId: FishReadSurfaceId): void {
    const surface = this.surfaces.get(surfaceId);
    if (!surface) return;
    if (this.hidden) return;

    this.activeSurfaces.add(surfaceId);
    surface.show(ctx);
  }

  hide(ctx: ExtensionContext, surfaceId: FishReadSurfaceId): void {
    const surface = this.surfaces.get(surfaceId);
    if (!surface) return;

    this.activeSurfaces.delete(surfaceId);
    surface.hide(ctx);
  }

  async toggle(ctx: ExtensionContext): Promise<void> {
    if (this.hidden) {
      await this.restore(ctx);
      return;
    }

    this.enterHiddenState(ctx);
  }

  private enterHiddenState(ctx: ExtensionContext): void {
    this.restoreSurfaces = new Set(this.activeSurfaces);
    for (const surfaceId of this.restoreSurfaces) {
      this.surfaces.get(surfaceId)?.hide(ctx);
    }
    this.activeSurfaces.clear();
    this.hidden = true;
  }

  private async restore(ctx: ExtensionContext): Promise<void> {
    this.hidden = false;
    await reloadReaderState(ctx);

    for (const surfaceId of this.restoreSurfaces) {
      const surface = this.surfaces.get(surfaceId);
      if (!surface) continue;

      try {
        surface.show(ctx);
        this.activeSurfaces.add(surfaceId);
      } catch {
        surface.hide(ctx);
      }
    }

    this.restoreSurfaces.clear();
  }
}

const bossKey = new BossKeyController();

bossKey.register({
  id: "status",
  show: showStatusLine,
  hide: hideStatusLine,
});

bossKey.register({
  id: "reader",
  show: showReaderWidget,
  hide: hideReaderWidget,
});

// ── Extension entry ───────────────────────────────────────────────────────────

export default function (pi: ExtensionAPI) {
  // On session start: init status line + mount persistent reading widget.
  pi.on("session_start", async (_event, ctx) => {
    await reloadReaderState(ctx);
    bossKey.show(ctx, "status");
    bossKey.show(ctx, "reader");
  });

  // Boss key is the only FishRead interaction that remains active while hidden.
  pi.registerShortcut(BOSS_KEY, {
    description: "FishRead Boss Key — 切换阅读 UI",
    handler: async (ctx) => {
      await bossKey.toggle(ctx);
    },
  });

  // ── Navigation ──────────────────────────────────────────────────────────────

  function applyReaderState(ctx: ExtensionContext, result: ApiResponse<ReaderStateDto>) {
    const { data } = result;
    lastReaderState = data;
    lastStatusText = buildStatusText(data, ctx.ui.theme);
    bossKey.show(ctx, "status");
    bossKey.show(ctx, "reader");
  }

  async function handleNext(ctx: ExtensionContext) {
    if (bossKey.isHidden()) return;

    const result = await readNext();
    if (!result.ok) {
      ctx.ui.notify(`[fishread] ${result.error.code}: ${result.error.message}`, "error");
      return;
    }
    applyReaderState(ctx, result);
  }

  async function handlePrev(ctx: ExtensionContext) {
    if (bossKey.isHidden()) return;

    const result = await readPrev();
    if (!result.ok) {
      ctx.ui.notify(`[fishread] ${result.error.code}: ${result.error.message}`, "error");
      return;
    }
    applyReaderState(ctx, result);
  }

  pi.registerShortcut(NEXT_PAGE_KEY, {
    description: `FishRead — 下一页 (${NEXT_PAGE_KEY_LABEL})`,
    handler: handleNext,
  });

  pi.registerShortcut(PREV_PAGE_KEY, {
    description: `FishRead — 上一页 (${PREV_PAGE_KEY_LABEL})`,
    handler: handlePrev,
  });

  pi.registerCommand("fr", {
    description: `FishRead — /fr <next|prev> (next: ${NEXT_PAGE_KEY_LABEL}, prev: ${PREV_PAGE_KEY_LABEL})`,
    getArgumentCompletions: (prefix) => {
      return FR_SUBCOMMANDS
        .filter((s) => s.startsWith(prefix))
        .map((s) => ({
          value: s,
          label: `${s} — ${FR_SUBCOMMAND_DETAILS[s].description} (${FR_SUBCOMMAND_DETAILS[s].shortcut})`,
        }));
    },
    handler: async (args, ctx) => {
      if (bossKey.isHidden()) return;

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
