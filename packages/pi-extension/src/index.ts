import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import type { Component, TUI } from "@earendil-works/pi-tui";
import {
  deleteBook,
  ensureFishReadReady,
  importBook,
  listBooks,
  listReadingNavigation,
  readCurrent,
  readJump,
  readNext,
  readPrev,
  useBook,
} from "@fishread/sdk";
import type { ApiResponse, BookListItemDto, ReaderStateDto } from "@fishread/sdk";
import {
  BOSS_KEY,
  FR_SUBCOMMAND_DETAILS,
  FR_SUBCOMMANDS,
  NEXT_PAGE_KEY,
  NEXT_PAGE_KEY_LABEL,
  PREV_PAGE_KEY,
  PREV_PAGE_KEY_LABEL,
  STATUS_KEY,
  WIDGET_KEY,
} from "./constants.js";
import {
  PathInputOverlay,
  normalizePathInput,
  type PathInputOverlayResult,
  type PathInputOverlayState,
} from "./components/path-input-overlay.js";
import { isBossKeyOverlayResult } from "./components/overlay-result.js";
import {
  BookSwitchOverlay,
  createBookDeleteConfirmOverlay,
  type BookDeleteConfirmation,
  type BookLibraryResult,
  type BookSwitchOverlayState,
} from "./overlays/book-library.js";
import { TocOverlay, type TocOverlayResult, type TocOverlayState } from "./overlays/toc.js";
import { splitCommandArgs } from "./utils.js";
import { ChunkWidget } from "./widgets/chunk-widget.js";

type FishReadSurfaceId = "status" | "reader";
type OverlayRestore = () => void | Promise<void>;

let lastStatusText: string | undefined;
let lastReaderState: ReaderStateDto | undefined;

function buildStatusText(state: ReaderStateDto, theme: any): string {
  const { book, chapter, progress } = state;
  return (
    theme.fg("accent", "◆") +
    " " +
    theme.fg("text", book.title) +
    theme.fg("dim", ` · ${chapter.title}`) +
    theme.fg(
      "dim",
      ` · ${progress.chapter_percent.toFixed(0)}% · 全书 ${progress.book_percent.toFixed(0)}%`
    )
  );
}

async function reloadReaderState(ctx: ExtensionContext): Promise<void> {
  try {
    const result = await readCurrent();
    if (!result.ok) {
      lastReaderState = undefined;
      lastStatusText = ctx.ui.theme.fg("dim", "◆ FishRead · 使用 /fr books 选择图书开始阅读");
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

function showReaderWidget(ctx: ExtensionContext) {
  if (!ctx.hasUI) return;
  ctx.ui.setWidget(
    WIDGET_KEY,
    (_tui: TUI, theme: any): Component & { dispose?(): void } =>
      new ChunkWidget(theme, () => lastReaderState),
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
  private restoreOverlay: OverlayRestore | undefined;

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

  hideWithOverlayRestore(ctx: ExtensionContext, restoreOverlay: OverlayRestore): void {
    if (this.hidden) return;

    this.restoreOverlay = restoreOverlay;
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
    const restoreOverlay = this.restoreOverlay;
    this.restoreOverlay = undefined;

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
    if (restoreOverlay) {
      void restoreOverlay();
    }
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

export default function (pi: ExtensionAPI) {
  pi.on("session_start", async (_event, ctx) => {
    const ready = await ensureFishReadReady();
    if (!ready.ok) {
      lastReaderState = undefined;
      lastStatusText = ctx.ui.theme.fg("dim", "◆ FishRead · 数据库准备失败");
      ctx.ui.notify(`[fishread] ${ready.error.code}: ${ready.error.message}`, "error");
      bossKey.show(ctx, "status");
      return;
    }

    await reloadReaderState(ctx);
    bossKey.show(ctx, "status");
    bossKey.show(ctx, "reader");
  });

  pi.registerShortcut(BOSS_KEY, {
    description: "FishRead Boss Key — 切换阅读 UI",
    handler: async (ctx) => {
      await bossKey.toggle(ctx);
    },
  });

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

  async function handleToc(ctx: ExtensionContext, initialState?: TocOverlayState) {
    if (bossKey.isHidden()) return;
    if (ctx.mode !== "tui") {
      ctx.ui.notify("[fishread] 目录需要 TUI 模式", "error");
      return;
    }

    const navigation = await listReadingNavigation();
    if (!navigation.ok) {
      ctx.ui.notify(`[fishread] ${navigation.error.code}: ${navigation.error.message}`, "error");
      return;
    }

    const selection = await ctx.ui.custom<TocOverlayResult>(
      (tui, theme, _kb, done) => new TocOverlay(navigation.data, theme, tui, done, initialState),
      {
        overlay: true,
        overlayOptions: {
          width: "82%",
          minWidth: 54,
          maxHeight: 20,
          anchor: "center",
          margin: 2,
        },
      }
    );
    if (isBossKeyOverlayResult<TocOverlayState>(selection)) {
      bossKey.hideWithOverlayRestore(ctx, () => handleToc(ctx, selection.state));
      return;
    }
    if (!selection) return;

    const result = await readJump(selection.chapterIndex, selection.chunkIndex);
    if (!result.ok) {
      ctx.ui.notify(`[fishread] ${result.error.code}: ${result.error.message}`, "error");
      return;
    }
    applyReaderState(ctx, result);
  }

  async function handleBooks(ctx: ExtensionContext, initialState?: BookSwitchOverlayState) {
    if (bossKey.isHidden()) return;
    if (ctx.mode !== "tui") {
      ctx.ui.notify("[fishread] 图书管理需要 TUI 模式", "error");
      return;
    }

    let restoreState = initialState;
    while (true) {
      const books = await listBooks();
      if (!books.ok) {
        ctx.ui.notify(`[fishread] ${books.error.code}: ${books.error.message}`, "error");
        return;
      }
      if (books.data.books.length === 0) {
        ctx.ui.notify("[fishread] 书库为空", "info");
        return;
      }

      const confirmDelete = (book: BookListItemDto) =>
        ctx.ui.custom<BookDeleteConfirmation>(
          (tui, theme, _kb, done) => createBookDeleteConfirmOverlay(book, tui, theme, done),
          {
            overlay: true,
            overlayOptions: {
              width: 54,
              maxHeight: 10,
              anchor: "center",
              offsetX: 4,
              offsetY: 1,
              margin: 2,
            },
          }
        );

      const action = await ctx.ui.custom<BookLibraryResult>(
        (tui, theme, _kb, done) => new BookSwitchOverlay(books.data, theme, tui, done, confirmDelete, restoreState),
        {
          overlay: true,
          overlayOptions: {
            width: "72%",
            minWidth: 52,
            maxHeight: 20,
            anchor: "center",
            margin: 2,
          },
        }
      );
      restoreState = undefined;
      if (isBossKeyOverlayResult<BookSwitchOverlayState>(action)) {
        bossKey.hideWithOverlayRestore(ctx, () => handleBooks(ctx, action.state));
        return;
      }
      if (!action) return;

      if (action.type === "delete") {
        const deleted = await deleteBook(action.book.id);
        if (!deleted.ok) {
          ctx.ui.notify(`[fishread] ${deleted.error.code}: ${deleted.error.message}`, "error");
          return;
        }

        ctx.ui.notify(`[fishread] 已删除《${deleted.data.deleted.title}》`, "info");
        if (deleted.data.cleared_current) {
          await reloadReaderState(ctx);
          bossKey.show(ctx, "status");
          bossKey.show(ctx, "reader");
        }
        continue;
      }

      const used = await useBook(action.book.id);
      if (!used.ok) {
        ctx.ui.notify(`[fishread] ${used.error.code}: ${used.error.message}`, "error");
        return;
      }

      const current = await readCurrent();
      if (!current.ok) {
        ctx.ui.notify(`[fishread] ${current.error.code}: ${current.error.message}`, "error");
        return;
      }
      applyReaderState(ctx, current);
      return;
    }
  }

  async function handleImport(ctx: ExtensionContext, pathArg?: string, initialState?: PathInputOverlayState) {
    if (bossKey.isHidden()) return;

    let rawPath = pathArg && pathArg.trim() ? pathArg : undefined;
    if (!rawPath) {
      if (ctx.mode !== "tui") {
        rawPath = await ctx.ui.input("FishRead 导入", "输入 EPUB 文件路径");
      } else {
        const selection = await ctx.ui.custom<PathInputOverlayResult>(
          (tui, theme, _kb, done) =>
            new PathInputOverlay(
              theme,
              tui,
              done,
              {
                bossKey: BOSS_KEY,
                title: "FishRead 导入 EPUB",
                emptyMessage: "没有匹配的目录或 EPUB 文件",
                footer: "Tab 补全 · ↑↓ 选择 · Enter 导入 · Esc 取消",
                directoryLabel: "目录",
                fileLabel: "EPUB",
                fileExtensions: [".epub"],
                suggestionLimit: 6,
              },
              initialState
            ),
          {
            overlay: true,
            overlayOptions: {
              width: "68%",
              minWidth: 52,
              maxHeight: 14,
              anchor: "center",
              margin: 2,
            },
          }
        );
        if (isBossKeyOverlayResult<PathInputOverlayState>(selection)) {
          bossKey.hideWithOverlayRestore(ctx, () => handleImport(ctx, undefined, selection.state));
          return;
        }
        rawPath = selection;
      }
    }

    if (!rawPath) return;

    const importPath = normalizePathInput(rawPath);
    if (!importPath) return;

    const result = await importBook(importPath);
    if (!result.ok) {
      ctx.ui.notify(`[fishread] ${result.error.code}: ${result.error.message}`, "error");
      return;
    }

    const warningText =
      result.data.warnings.length > 0 ? `，${result.data.warnings.length} 个警告` : "";
    ctx.ui.notify(
      `[fishread] 已导入《${result.data.book.title}》，${result.data.chapters_count} 个可读项${warningText}`,
      "info"
    );

    await reloadReaderState(ctx);
    bossKey.show(ctx, "status");
    bossKey.show(ctx, "reader");
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
    description: `FishRead — /fr <next|prev|toc|books|import> (next: ${NEXT_PAGE_KEY_LABEL}, prev: ${PREV_PAGE_KEY_LABEL})`,
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

      const { subcommand, rest } = splitCommandArgs(args);
      switch (subcommand) {
        case "next": return handleNext(ctx);
        case "prev": return handlePrev(ctx);
        case "toc": return handleToc(ctx);
        case "books": return handleBooks(ctx);
        case "import": return handleImport(ctx, rest);
        default:
          ctx.ui.notify(
            `未知子命令: ${subcommand || "(空)"}。可用: ${FR_SUBCOMMANDS.join(", ")}`,
            "error"
          );
      }
    },
  });
}
