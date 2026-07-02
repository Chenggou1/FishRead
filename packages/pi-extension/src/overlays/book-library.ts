import { Key, matchesKey, truncateToWidth } from "@earendil-works/pi-tui";
import type { Component, TUI } from "@earendil-works/pi-tui";
import type { BookListDto, BookListItemDto } from "@fishread/sdk";
import { BOSS_KEY } from "../constants.js";
import { ChoiceConfirmOverlay, type ConfirmationResult } from "../components/confirm-overlay.js";
import { OverlayFrame, type OverlayTheme } from "../components/overlay-frame.js";
import { isBossKeyOverlayResult, type BossKeyOverlayResult } from "../components/overlay-result.js";
import {
  TextInputOverlay,
  type TextInputOverlayResult,
} from "../components/text-input-overlay.js";
import { clampIndex } from "../utils.js";

const DELETE_CONFIRM_TITLE_MAX_WIDTH = 48;

export type BookLibraryAction =
  | { type: "use"; book: BookListItemDto }
  | { type: "rename"; book: BookListItemDto; title: string }
  | { type: "delete"; book: BookListItemDto };
export type BookLibraryResult =
  | BookLibraryAction
  | BossKeyOverlayResult<BookSwitchOverlayState>
  | undefined;
export type BookDeleteConfirmation = ConfirmationResult;
export type BookRenamePromptResult = TextInputOverlayResult;

export interface BookSwitchOverlayState {
  selectedIndex: number;
  topIndex: number;
}

export class BookSwitchOverlay implements Component {
  private readonly books: BookListItemDto[];
  private selectedIndex: number;
  private topIndex = 0;

  constructor(
    bookList: BookListDto,
    private theme: OverlayTheme,
    private tui: TUI,
    private done: (action: BookLibraryResult) => void,
    private confirmDelete: (book: BookListItemDto) => Promise<BookDeleteConfirmation>,
    private promptRename: (book: BookListItemDto) => Promise<BookRenamePromptResult>,
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
    if (data === "r") {
      const book = this.selectedBook();
      if (book) {
        void this.promptRename(book).then((result) => {
          if (typeof result === "string") {
            this.done({ type: "rename", book, title: result });
          } else if (isBossKeyOverlayResult(result)) {
            this.done({ type: "boss-key", state: this.snapshot() });
          } else {
            this.tui.requestRender();
          }
        });
      }
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
    const frame = new OverlayFrame(this.theme);
    const lines: string[] = [];

    lines.push(frame.top(contentWidth));
    lines.push(frame.content(this.theme.fg("accent", "FishRead 图书管理"), contentWidth));
    lines.push(frame.separator(contentWidth));

    for (let row = 0; row < maxRows; row++) {
      const book = visibleBooks[row];
      const bookIndex = book ? this.books.indexOf(book) : -1;
      const left = book
        ? this.renderBookItem(book, bookIndex === this.selectedIndex, listWidth)
        : "".padEnd(listWidth, " ");
      const right = this.renderBookDetailLine(selected, row, detailWidth);
      lines.push(frame.content(`${left}${" ".repeat(gap)}${right}`, contentWidth));
    }

    lines.push(frame.separator(contentWidth));
    lines.push(frame.content(this.footerText(contentWidth), contentWidth));
    lines.push(frame.bottom(contentWidth));
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
        truncateToWidth(`进度 位置 ${book.position.chapter_index + 1} · ${book.reading_anchor_label}`, width)
      ),
      this.theme.fg("dim", truncateToWidth(`导入 ${importedAt}`, width)),
      "",
    ];
    return truncateToWidth(rows[row] ?? "", width, "", true);
  }

  private footerText(width: number): string {
    return this.theme.fg("dim", truncateToWidth("↑↓ 选择 · Enter 切换 · r 重命名 · d 删除 · Esc 关闭", width));
  }
}

export function createBookRenameOverlay(
  book: BookListItemDto,
  tui: TUI,
  theme: OverlayTheme,
  done: (result: BookRenamePromptResult) => void
): Component {
  return new TextInputOverlay(theme, tui, done, {
    bossKey: BOSS_KEY,
    title: "重命名图书",
    label: "书名",
    initialValue: book.title,
    emptyMessage: "书名不能为空",
    footer: "Enter 保存 · Esc 取消",
    body: (overlayTheme, width) => [
      overlayTheme.fg("dim", truncateToWidth("当前书名", width, "...", true)),
      overlayTheme.fg("text", truncateToWidth(book.title, width, "...", true)),
    ],
  });
}

export function createBookDeleteConfirmOverlay(
  book: BookListItemDto,
  tui: TUI,
  theme: OverlayTheme,
  done: (confirmed: BookDeleteConfirmation) => void
): Component {
  return new ChoiceConfirmOverlay(theme, tui, done, {
    bossKey: BOSS_KEY,
    title: "确认删除这本书？",
    confirmLabel: "是，删除",
    body: (overlayTheme, width) => {
      const titleWidth = Math.min(width, DELETE_CONFIRM_TITLE_MAX_WIDTH);
      return [
        overlayTheme.fg("accent", truncateToWidth(book.title, titleWidth, "...", true)),
        overlayTheme.fg("dim", truncateToWidth(book.author ? `作者 ${book.author}` : "作者未知", width, "...", true)),
        overlayTheme.fg(
          "dim",
          truncateToWidth(`进度 位置 ${book.position.chapter_index + 1} · ${book.reading_anchor_label}`, width, "...", true)
        ),
        "",
        overlayTheme.fg("dim", truncateToWidth("将删除本地书籍、章节和阅读进度。", width, "...", true)),
      ];
    },
  });
}
