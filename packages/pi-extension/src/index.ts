import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { Key, matchesKey, truncateToWidth, visibleWidth, wrapTextWithAnsi } from "@earendil-works/pi-tui";
import type { Component, TUI } from "@earendil-works/pi-tui";
import {
  deleteBook,
  listBooks,
  listReadingNavigation,
  readCurrent,
  readJump,
  readNext,
  readPrev,
  useBook,
} from "@fishread/sdk";
import type {
  ApiResponse,
  BookListDto,
  BookListItemDto,
  ChapterListDto,
  ChapterListItemDto,
  ReaderStateDto,
  ReadingAnchorDto,
} from "@fishread/sdk";
import { renderChunk, type ChunkMessageDetails } from "./renderers/chunk.js";

const FR_SUBCOMMANDS = ["next", "prev", "toc", "books"] as const;
const BOSS_KEY = Key.ctrlShift("h");
const NEXT_PAGE_KEY = Key.ctrlShift("right");
const PREV_PAGE_KEY = Key.ctrlShift("left");
const NEXT_PAGE_KEY_LABEL = "ctrl+shift+right";
const PREV_PAGE_KEY_LABEL = "ctrl+shift+left";
const STATUS_KEY = "fishread";
const WIDGET_KEY = "fishread-reader";
const DELETE_CONFIRM_TITLE_MAX_WIDTH = 48;

const FR_SUBCOMMAND_DETAILS: Record<
  (typeof FR_SUBCOMMANDS)[number],
  { description: string; shortcut: string }
> = {
  next: { description: "下一页", shortcut: NEXT_PAGE_KEY_LABEL },
  prev: { description: "上一页", shortcut: PREV_PAGE_KEY_LABEL },
  toc: { description: "目录", shortcut: "/fr toc" },
  books: { description: "图书管理", shortcut: "/fr books" },
};

type FishReadSurfaceId = "status" | "reader";
type OverlayRestore = () => void | Promise<void>;
type BossKeyOverlayResult<TState> = { type: "boss-key"; state: TState };

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

function clampIndex(index: number, length: number): number {
  if (length <= 0) return 0;
  return Math.max(0, Math.min(length - 1, index));
}

function isBossKeyOverlayResult<TState>(result: unknown): result is BossKeyOverlayResult<TState> {
  return !!result && typeof result === "object" && (result as { type?: unknown }).type === "boss-key";
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

type BookLibraryAction =
  | { type: "use"; book: BookListItemDto }
  | { type: "delete"; book: BookListItemDto };
type BookLibraryResult = BookLibraryAction | BossKeyOverlayResult<BookSwitchOverlayState> | undefined;
type BookDeleteConfirmation = "confirmed" | "cancelled" | "boss-key";
interface BookSwitchOverlayState {
  selectedIndex: number;
  topIndex: number;
}

class BookSwitchOverlay implements Component {
  private readonly books: BookListItemDto[];
  private selectedIndex: number;
  private topIndex = 0;

  constructor(
    bookList: BookListDto,
    private theme: any,
    private tui: TUI,
    private done: (action: BookLibraryResult) => void,
    private confirmDelete: (book: BookListItemDto) => Promise<BookDeleteConfirmation>,
    initialState?: BookSwitchOverlayState
  ) {
    this.books = bookList.books;
    const defaultSelectedIndex = Math.max(
      0,
      this.books.findIndex((book) => book.current)
    );
    this.selectedIndex = clampIndex(initialState?.selectedIndex ?? defaultSelectedIndex, this.books.length);
    this.topIndex = clampIndex(initialState?.topIndex ?? 0, this.books.length);
  }

  handleInput(data: string): void {
    if (matchesKey(data, BOSS_KEY)) {
      this.done({ type: "boss-key", state: this.snapshot() });
      return;
    }
    if (matchesKey(data, Key.up)) {
      this.move(-1);
      return;
    }
    if (matchesKey(data, Key.down)) {
      this.move(1);
      return;
    }
    if (matchesKey(data, Key.enter)) {
      const book = this.selectedBook();
      this.done(book ? { type: "use", book } : undefined);
      return;
    }
    if (data === "d") {
      const book = this.selectedBook();
      if (book) {
        void this.confirmDelete(book).then((confirmation) => {
          if (confirmation === "confirmed") {
            this.done({ type: "delete", book });
          } else if (confirmation === "cancelled") {
            this.tui.requestRender();
          } else {
            this.done({ type: "boss-key", state: this.snapshot() });
          }
        });
      }
      return;
    }
    if (matchesKey(data, Key.escape) || matchesKey(data, Key.ctrl("c"))) {
      this.done(undefined);
    }
  }

  render(width: number): string[] {
    const contentWidth = Math.max(48, width - 4);
    const listWidth = Math.max(24, Math.min(42, Math.floor(contentWidth * 0.48)));
    const gap = 2;
    const detailWidth = Math.max(18, contentWidth - listWidth - gap);
    const maxRows = 12;
    const visibleBooks = this.visibleBooks(maxRows);
    const selected = this.selectedBook();
    const lines: string[] = [];

    lines.push(this.borderTop(contentWidth));
    lines.push(this.padContent(this.theme.fg("accent", "FishRead 图书管理"), contentWidth));
    lines.push(this.separator(contentWidth));

    for (let row = 0; row < maxRows; row++) {
      const book = visibleBooks[row];
      const bookIndex = book ? this.books.indexOf(book) : -1;
      const left = book
        ? this.renderBookItem(book, bookIndex === this.selectedIndex, listWidth)
        : "".padEnd(listWidth, " ");
      const right = this.renderBookDetailLine(selected, row, detailWidth);
      lines.push(this.padContent(`${left}${" ".repeat(gap)}${right}`, contentWidth));
    }

    lines.push(this.separator(contentWidth));
    lines.push(this.padContent(this.footerText(contentWidth), contentWidth));
    lines.push(this.borderBottom(contentWidth));
    return lines;
  }

  invalidate() {}

  private selectedBook(): BookListItemDto | undefined {
    return this.books[this.selectedIndex];
  }

  private move(delta: number): void {
    if (this.books.length === 0) return;
    this.selectedIndex = Math.max(0, Math.min(this.books.length - 1, this.selectedIndex + delta));
    this.tui.requestRender();
  }

  private snapshot(): BookSwitchOverlayState {
    return {
      selectedIndex: this.selectedIndex,
      topIndex: this.topIndex,
    };
  }

  private visibleBooks(maxRows: number): BookListItemDto[] {
    if (this.selectedIndex < this.topIndex) {
      this.topIndex = this.selectedIndex;
    } else if (this.selectedIndex >= this.topIndex + maxRows) {
      this.topIndex = this.selectedIndex - maxRows + 1;
    }
    return this.books.slice(this.topIndex, this.topIndex + maxRows);
  }

  private renderBookItem(book: BookListItemDto, selected: boolean, width: number): string {
    const marker = book.current ? "◆" : " ";
    const prefix = selected ? "›" : " ";
    const raw = `${prefix} ${marker} ${book.title}`;
    const text = truncateToWidth(raw, width, "", true);
    return selected ? this.theme.fg("accent", text) : this.theme.fg("dim", text);
  }

  private renderBookDetailLine(book: BookListItemDto | undefined, row: number, width: number): string {
    if (!book) {
      return truncateToWidth(this.theme.fg("dim", "书库为空"), width, "", true);
    }

    const importedAt = new Date(book.imported_at * 1000).toLocaleDateString();
    const rows = [
      this.theme.fg("text", truncateToWidth(book.title, width)),
      this.theme.fg("dim", truncateToWidth(book.author ? `作者 ${book.author}` : "作者未知", width)),
      this.theme.fg("dim", truncateToWidth(`格式 ${book.format}`, width)),
      this.theme.fg(
        "dim",
        truncateToWidth(`进度 第 ${book.position.chapter_index + 1} 章 · ${book.reading_anchor_label}`, width)
      ),
      this.theme.fg("dim", truncateToWidth(`导入 ${importedAt}`, width)),
      "",
    ];
    return truncateToWidth(rows[row] ?? "", width, "", true);
  }

  private footerText(width: number): string {
    return this.theme.fg("dim", truncateToWidth("↑↓ 选择 · Enter 切换 · d 删除 · Esc 关闭", width));
  }

  private borderTop(width: number): string {
    return this.theme.fg("dim", `┌${"─".repeat(width)}┐`);
  }

  private borderBottom(width: number): string {
    return this.theme.fg("dim", `└${"─".repeat(width)}┘`);
  }

  private separator(width: number): string {
    return this.theme.fg("dim", `├${"─".repeat(width)}┤`);
  }

  private padContent(text: string, width: number): string {
    const visible = visibleWidth(text);
    const padded = visible < width ? text + " ".repeat(width - visible) : truncateToWidth(text, width, "");
    return this.theme.fg("dim", "│") + padded + this.theme.fg("dim", "│");
  }
}

class BookDeleteConfirmOverlay implements Component {
  private selectedIndex = 0;

  constructor(
    private book: BookListItemDto,
    private tui: TUI,
    private theme: any,
    private done: (confirmed: BookDeleteConfirmation) => void
  ) {}

  handleInput(data: string): void {
    if (matchesKey(data, BOSS_KEY)) {
      this.done("boss-key");
      return;
    }
    if (matchesKey(data, Key.up) || matchesKey(data, Key.down)) {
      this.selectedIndex = this.selectedIndex === 0 ? 1 : 0;
      this.tui.requestRender();
      return;
    }
    if (matchesKey(data, Key.enter)) {
      this.done(this.selectedIndex === 1 ? "confirmed" : "cancelled");
      return;
    }
    if (matchesKey(data, Key.escape) || matchesKey(data, Key.ctrl("c"))) {
      this.done("cancelled");
    }
  }

  render(width: number): string[] {
    const contentWidth = Math.max(40, width - 4);
    const titleWidth = Math.min(contentWidth, DELETE_CONFIRM_TITLE_MAX_WIDTH);
    const rows = [
      this.theme.fg("text", "确认删除这本书？"),
      this.theme.fg("accent", truncateToWidth(this.book.title, titleWidth, "...", true)),
      this.theme.fg("dim", truncateToWidth(this.book.author ? `作者 ${this.book.author}` : "作者未知", contentWidth, "...", true)),
      this.theme.fg(
        "dim",
        truncateToWidth(`进度 第 ${this.book.position.chapter_index + 1} 章 · ${this.book.reading_anchor_label}`, contentWidth, "...", true)
      ),
      "",
      this.theme.fg("dim", truncateToWidth("将删除本地书籍、章节和阅读进度。", contentWidth, "...", true)),
      this.renderChoice("否，取消", false, contentWidth),
      this.renderChoice("是，删除", true, contentWidth),
      this.theme.fg("dim", "↑↓ 选择 · Enter 确认 · Esc 取消"),
    ];

    return [
      this.borderTop(contentWidth),
      ...rows.map((row) => this.padContent(row, contentWidth)),
      this.borderBottom(contentWidth),
    ];
  }

  invalidate() {}

  private borderTop(width: number): string {
    return this.theme.fg("dim", `┌${"─".repeat(width)}┐`);
  }

  private borderBottom(width: number): string {
    return this.theme.fg("dim", `└${"─".repeat(width)}┘`);
  }

  private renderChoice(label: string, destructive: boolean, width: number): string {
    const selected = this.selectedIndex === (destructive ? 1 : 0);
    const prefix = selected ? "› " : "  ";
    const text = truncateToWidth(`${prefix}${label}`, width, "", true);
    if (!selected) return this.theme.fg("dim", text);
    return destructive ? this.theme.fg("error", text) : this.theme.fg("accent", text);
  }

  private padContent(text: string, width: number): string {
    const visible = visibleWidth(text);
    const padded = visible < width ? text + " ".repeat(width - visible) : truncateToWidth(text, width, "");
    return this.theme.fg("dim", "│") + padded + this.theme.fg("dim", "│");
  }
}

interface NavigationSelection {
  chapterIndex: number;
  chunkIndex: number;
}
interface TocOverlayState {
  selectedChapterIndex: number;
  selectedAnchorIndex: number;
  activePane: "chapter" | "anchor";
  chapterTopIndex: number;
}
type TocOverlayResult = NavigationSelection | BossKeyOverlayResult<TocOverlayState> | undefined;

class TocOverlay implements Component {
  private readonly chapters: ChapterListItemDto[];
  private selectedChapterIndex: number;
  private selectedAnchorIndex: number;
  private activePane: "chapter" | "anchor";
  private chapterTopIndex = 0;

  constructor(
    navigation: ChapterListDto,
    private theme: any,
    private tui: TUI,
    private done: (selection: TocOverlayResult) => void,
    initialState?: TocOverlayState
  ) {
    this.chapters = navigation.chapters;
    const defaultSelectedChapterIndex = Math.max(
      0,
      navigation.chapters.findIndex((chapter) => chapter.current)
    );
    this.selectedChapterIndex = clampIndex(
      initialState?.selectedChapterIndex ?? defaultSelectedChapterIndex,
      this.chapters.length
    );
    this.activePane = initialState?.activePane ?? "chapter";
    this.selectedAnchorIndex = this.clampAnchorIndex(
      initialState?.selectedAnchorIndex ?? this.defaultAnchorIndex()
    );
    this.chapterTopIndex = clampIndex(initialState?.chapterTopIndex ?? 0, this.chapters.length);
  }

  handleInput(data: string): void {
    if (matchesKey(data, BOSS_KEY)) {
      this.done({ type: "boss-key", state: this.snapshot() });
      return;
    }
    if (matchesKey(data, Key.up)) {
      this.move(-1);
      return;
    }
    if (matchesKey(data, Key.down)) {
      this.move(1);
      return;
    }
    if (matchesKey(data, Key.right)) {
      this.activePane = "anchor";
      this.tui.requestRender();
      return;
    }
    if (matchesKey(data, Key.left)) {
      this.activePane = "chapter";
      this.tui.requestRender();
      return;
    }
    if (matchesKey(data, Key.enter)) {
      if (this.activePane === "chapter") {
        this.activePane = "anchor";
        this.tui.requestRender();
        return;
      }

      const anchor = this.selectedAnchor();
      this.done(anchor ? { chapterIndex: anchor.position.chapter_index, chunkIndex: anchor.position.chunk_index } : undefined);
      return;
    }
    if (matchesKey(data, Key.escape) || matchesKey(data, Key.ctrl("c"))) {
      this.done(undefined);
    }
  }

  render(width: number): string[] {
    const contentWidth = Math.max(48, width - 4);
    const chapterWidth = Math.max(20, Math.min(34, Math.floor(contentWidth * 0.34)));
    const anchorWidth = 10;
    const gap = 2;
    const previewWidth = Math.max(16, contentWidth - chapterWidth - anchorWidth - gap * 2);
    const maxRows = 14;
    const visibleChapters = this.visibleChapters(maxRows);
    const selectedChapter = this.selectedChapter();
    const selectedAnchor = this.selectedAnchor();
    const anchors = selectedChapter?.anchors ?? [];
    const lines: string[] = [];

    lines.push(this.borderTop(contentWidth));
    lines.push(this.padContent(this.theme.fg("accent", "FishRead 目录"), contentWidth));
    lines.push(this.separator(contentWidth));

    for (let row = 0; row < maxRows; row++) {
      const chapter = visibleChapters[row];
      const chapterIndex = chapter ? this.chapters.indexOf(chapter) : -1;
      const left = chapter
        ? this.renderChapterItem(chapter, chapterIndex === this.selectedChapterIndex, chapterWidth)
        : "".padEnd(chapterWidth, " ");
      const anchor = anchors[row];
      const middle = anchor
        ? this.renderAnchorItem(anchor, row === this.selectedAnchorIndex, anchorWidth)
        : "".padEnd(anchorWidth, " ");
      const right = this.renderPreviewLine(selectedChapter, selectedAnchor, row, previewWidth);
      lines.push(this.padContent(`${left}${" ".repeat(gap)}${middle}${" ".repeat(gap)}${right}`, contentWidth));
    }

    lines.push(this.borderBottom(contentWidth));
    return lines;
  }

  invalidate() {}

  private selectedChapter(): ChapterListItemDto | undefined {
    return this.chapters[this.selectedChapterIndex];
  }

  private selectedAnchor(): ReadingAnchorDto | undefined {
    return this.selectedChapter()?.anchors?.[this.selectedAnchorIndex];
  }

  private move(delta: number): void {
    if (this.activePane === "chapter") {
      if (this.chapters.length === 0) return;
      this.selectedChapterIndex = Math.max(0, Math.min(this.chapters.length - 1, this.selectedChapterIndex + delta));
      this.selectedAnchorIndex = this.defaultAnchorIndex();
    } else {
      const anchors = this.selectedChapter()?.anchors ?? [];
      if (anchors.length === 0) return;
      this.selectedAnchorIndex = Math.max(0, Math.min(anchors.length - 1, this.selectedAnchorIndex + delta));
    }
    this.tui.requestRender();
  }

  private snapshot(): TocOverlayState {
    return {
      selectedChapterIndex: this.selectedChapterIndex,
      selectedAnchorIndex: this.selectedAnchorIndex,
      activePane: this.activePane,
      chapterTopIndex: this.chapterTopIndex,
    };
  }

  private visibleChapters(maxRows: number): ChapterListItemDto[] {
    if (this.selectedChapterIndex < this.chapterTopIndex) {
      this.chapterTopIndex = this.selectedChapterIndex;
    } else if (this.selectedChapterIndex >= this.chapterTopIndex + maxRows) {
      this.chapterTopIndex = this.selectedChapterIndex - maxRows + 1;
    }
    return this.chapters.slice(this.chapterTopIndex, this.chapterTopIndex + maxRows);
  }

  private renderChapterItem(chapter: ChapterListItemDto, selected: boolean, width: number): string {
    const marker = chapter.current ? "◆" : " ";
    const prefix = selected && this.activePane === "chapter" ? "›" : " ";
    const raw = `${prefix} ${marker} 第 ${chapter.index + 1} 章 ${chapter.title}`;
    const text = truncateToWidth(raw, width - 2, "", true);
    return selected ? this.theme.fg("accent", text) : this.theme.fg("dim", text);
  }

  private renderAnchorItem(anchor: ReadingAnchorDto, selected: boolean, width: number): string {
    const marker = anchor.current ? "◆" : " ";
    const prefix = selected && this.activePane === "anchor" ? "›" : " ";
    const text = truncateToWidth(`${prefix} ${marker} ${anchor.label}`, width, "", true);
    return selected ? this.theme.fg("accent", text) : this.theme.fg("dim", text);
  }

  private renderPreviewLine(
    chapter: ChapterListItemDto | undefined,
    anchor: ReadingAnchorDto | undefined,
    row: number,
    width: number
  ): string {
    if (!chapter || !anchor) {
      return truncateToWidth(this.theme.fg("dim", "暂无目录"), width, "", true);
    }

    const title = `${chapter.title} · ${anchor.label}`;
    const meta = `chunk ${anchor.chunk.index}`;
    const previewLines = wrapTextWithAnsi(anchor.preview, width);
    const rows = [
      this.theme.fg("text", truncateToWidth(title, width)),
      this.theme.fg("dim", truncateToWidth(meta, width)),
      "",
      ...previewLines,
    ];
    return truncateToWidth(rows[row] ?? "", width, "", true);
  }

  private defaultAnchorIndex(): number {
    const anchors = this.selectedChapter()?.anchors ?? [];
    const currentIndex = anchors.findIndex((anchor) => anchor.current);
    return Math.max(currentIndex, 0);
  }

  private clampAnchorIndex(index: number): number {
    const anchors = this.selectedChapter()?.anchors ?? [];
    return clampIndex(index, anchors.length);
  }

  private borderTop(width: number): string {
    return this.theme.fg("dim", `┌${"─".repeat(width)}┐`);
  }

  private borderBottom(width: number): string {
    return this.theme.fg("dim", `└${"─".repeat(width)}┘`);
  }

  private separator(width: number): string {
    return this.theme.fg("dim", `├${"─".repeat(width)}┤`);
  }

  private padContent(text: string, width: number): string {
    const visible = visibleWidth(text);
    const padded = visible < width ? text + " ".repeat(width - visible) : truncateToWidth(text, width, "");
    return this.theme.fg("dim", "│") + padded + this.theme.fg("dim", "│");
  }
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
          (tui, theme, _kb, done) => new BookDeleteConfirmOverlay(book, tui, theme, done),
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

  pi.registerShortcut(NEXT_PAGE_KEY, {
    description: `FishRead — 下一页 (${NEXT_PAGE_KEY_LABEL})`,
    handler: handleNext,
  });

  pi.registerShortcut(PREV_PAGE_KEY, {
    description: `FishRead — 上一页 (${PREV_PAGE_KEY_LABEL})`,
    handler: handlePrev,
  });

  pi.registerCommand("fr", {
    description: `FishRead — /fr <next|prev|toc|books> (next: ${NEXT_PAGE_KEY_LABEL}, prev: ${PREV_PAGE_KEY_LABEL})`,
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
        case "toc": return handleToc(ctx);
        case "books": return handleBooks(ctx);
        default:
          ctx.ui.notify(
            `未知子命令: ${sub || "(空)"}。可用: ${FR_SUBCOMMANDS.join(", ")}`,
            "error"
          );
      }
    },
  });
}
