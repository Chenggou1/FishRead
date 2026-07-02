import type { Component } from "@earendil-works/pi-tui";
import type { ReaderStateDto } from "@fishread/sdk";
import { renderChunk, type ChunkMessageDetails } from "../renderers/chunk.js";

export class ChunkWidget implements Component {
  constructor(
    private theme: any,
    private getReaderState: () => ReaderStateDto | undefined
  ) {}

  render(width: number): string[] {
    const state = this.getReaderState();
    if (!state) return [];
    const box = renderChunk(
      state.chunk.text,
      { state } satisfies ChunkMessageDetails,
      this.theme
    );
    return box.render(width);
  }

  invalidate() {}
}
