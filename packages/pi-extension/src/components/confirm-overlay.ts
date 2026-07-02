import { Key, matchesKey, truncateToWidth } from "@earendil-works/pi-tui";
import type { Component, KeyId, TUI } from "@earendil-works/pi-tui";
import { OverlayFrame, type OverlayTheme } from "./overlay-frame.js";

export type ConfirmationResult = "confirmed" | "cancelled" | "boss-key";

export interface ChoiceConfirmOverlayOptions {
  bossKey: KeyId;
  title: string;
  body: (theme: OverlayTheme, width: number) => string[];
  cancelLabel?: string;
  confirmLabel?: string;
  footer?: string;
}

export class ChoiceConfirmOverlay implements Component {
  private selectedIndex = 0;

  constructor(
    private theme: OverlayTheme,
    private tui: TUI,
    private done: (result: ConfirmationResult) => void,
    private options: ChoiceConfirmOverlayOptions
  ) {}

  handleInput(data: string): void {
    if (matchesKey(data, this.options.bossKey)) {
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
    const frame = new OverlayFrame(this.theme);
    const rows = [
      this.theme.fg("text", this.options.title),
      ...this.options.body(this.theme, contentWidth),
      "",
      this.renderChoice(this.options.cancelLabel ?? "否，取消", false, contentWidth),
      this.renderChoice(this.options.confirmLabel ?? "是，确认", true, contentWidth),
      this.theme.fg("dim", this.options.footer ?? "↑↓ 选择 · Enter 确认 · Esc 取消"),
    ];

    return [
      frame.top(contentWidth),
      ...rows.map((row) => frame.content(row, contentWidth)),
      frame.bottom(contentWidth),
    ];
  }

  invalidate() {}

  private renderChoice(label: string, destructive: boolean, width: number): string {
    const selected = this.selectedIndex === (destructive ? 1 : 0);
    const prefix = selected ? "› " : "  ";
    const text = truncateToWidth(`${prefix}${label}`, width, "", true);
    if (!selected) return this.theme.fg("dim", text);
    return destructive ? this.theme.fg("error", text) : this.theme.fg("accent", text);
  }
}
