import { Key, matchesKey, truncateToWidth, wrapTextWithAnsi } from "@earendil-works/pi-tui";
import type { Component, TUI } from "@earendil-works/pi-tui";
import type { ChapterListDto, ChapterListItemDto, ReadingAnchorDto } from "@fishread/sdk";
import { BOSS_KEY } from "../constants.js";
import { OverlayFrame, type OverlayTheme } from "../components/overlay-frame.js";
import type { BossKeyOverlayResult } from "../components/overlay-result.js";
import { clampIndex } from "../utils.js";

export interface NavigationSelection {
  chapterIndex: number;
  chunkIndex: number;
}

export interface TocOverlayState {
  selectedChapterIndex: number;
  selectedAnchorIndex: number;
  activePane: "chapter" | "anchor";
  chapterTopIndex: number;
}

export type TocOverlayResult = NavigationSelection | BossKeyOverlayResult<TocOverlayState> | undefined;

export class TocOverlay implements Component {
  private readonly chapters: ChapterListItemDto[];
  private selectedChapterIndex: number;
  private selectedAnchorIndex: number;
  private activePane: "chapter" | "anchor";
  private chapterTopIndex = 0;

  constructor(
    navigation: ChapterListDto,
    private theme: OverlayTheme,
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
    const frame = new OverlayFrame(this.theme);
    const lines: string[] = [];

    lines.push(frame.top(contentWidth));
    lines.push(frame.content(this.theme.fg("accent", "FishRead 目录"), contentWidth));
    lines.push(frame.separator(contentWidth));

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
      lines.push(frame.content(`${left}${" ".repeat(gap)}${middle}${" ".repeat(gap)}${right}`, contentWidth));
    }

    lines.push(frame.bottom(contentWidth));
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
    const raw = `${prefix} ${marker} ${chapter.title}`;
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
}
