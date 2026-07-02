import { Input, truncateToWidth } from "@earendil-works/pi-tui";
import type { Component, KeyId, TUI } from "@earendil-works/pi-tui";
import { OverlayFrame, type OverlayTheme } from "./overlay-frame.js";
import type { BossKeyOverlayResult } from "./overlay-result.js";
import { matchesKey } from "@earendil-works/pi-tui";

export interface TextInputOverlayState {
  value: string;
}

export interface TextInputOverlayOptions {
  bossKey: KeyId;
  title: string;
  label?: string;
  initialValue?: string;
  emptyMessage?: string;
  footer?: string;
  body?: (theme: OverlayTheme, width: number) => string[];
}

export type TextInputOverlayResult = string | BossKeyOverlayResult<TextInputOverlayState> | undefined;

export class TextInputOverlay implements Component {
  private readonly input = new Input();
  private error: string | undefined;

  constructor(
    private theme: OverlayTheme,
    private tui: TUI,
    private done: (result: TextInputOverlayResult) => void,
    private options: TextInputOverlayOptions,
    initialState?: TextInputOverlayState
  ) {
    this.input.setValue(initialState?.value ?? options.initialValue ?? "");
    this.input.onSubmit = (value) => this.submit(value);
    this.input.onEscape = () => this.done(undefined);
  }

  handleInput(data: string): void {
    if (matchesKey(data, this.options.bossKey)) {
      this.done({ type: "boss-key", state: this.snapshot() });
      return;
    }

    this.error = undefined;
    this.input.handleInput(data);
    this.tui.requestRender();
  }

  render(width: number): string[] {
    const contentWidth = Math.max(40, width - 4);
    const label = `${this.options.label ?? "输入"} `;
    const inputWidth = Math.max(12, contentWidth - label.length);
    const inputLine = this.input.render(inputWidth)[0] ?? "";
    const body = this.options.body?.(this.theme, contentWidth) ?? [];
    const frame = new OverlayFrame(this.theme);
    const rows = [
      this.theme.fg("accent", this.options.title),
      ...body,
      "",
      `${this.theme.fg("dim", label)}${inputLine}`,
      this.theme.fg(
        this.error ? "error" : "dim",
        truncateToWidth(this.error ?? this.options.footer ?? "Enter 确认 · Esc 取消", contentWidth)
      ),
    ];

    return [
      frame.top(contentWidth),
      ...rows.map((row) => frame.content(row, contentWidth)),
      frame.bottom(contentWidth),
    ];
  }

  invalidate() {}

  private submit(value: string): void {
    const trimmed = value.trim();
    if (!trimmed) {
      this.error = this.options.emptyMessage ?? "内容不能为空";
      this.tui.requestRender();
      return;
    }

    this.done(trimmed);
  }

  private snapshot(): TextInputOverlayState {
    return {
      value: this.input.getValue(),
    };
  }
}
