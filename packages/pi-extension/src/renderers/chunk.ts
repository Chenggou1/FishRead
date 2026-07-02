import { Text, Box } from "@earendil-works/pi-tui";
import type { ReaderStateDto } from "@fishread/sdk";

export interface ChunkMessageDetails {
  state: ReaderStateDto;
}

export function renderChunk(
  content: string,
  _details: ChunkMessageDetails,
  _theme: any
): any {
  const body = content;
  const box = new Box(0, 0);
  box.addChild(new Text(body, 0, 0));

  return box;
}
