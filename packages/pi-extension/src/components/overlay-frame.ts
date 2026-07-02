import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";

export interface OverlayTheme {
  fg(style: string, text: string): string;
}

export class OverlayFrame {
  constructor(private theme: OverlayTheme) {}

  top(width: number): string {
    return this.theme.fg("dim", `┌${"─".repeat(width)}┐`);
  }

  bottom(width: number): string {
    return this.theme.fg("dim", `└${"─".repeat(width)}┘`);
  }

  separator(width: number): string {
    return this.theme.fg("dim", `├${"─".repeat(width)}┤`);
  }

  content(text: string, width: number): string {
    const visible = visibleWidth(text);
    const padded = visible < width ? text + " ".repeat(width - visible) : truncateToWidth(text, width, "");
    return this.theme.fg("dim", "│") + padded + this.theme.fg("dim", "│");
  }
}
