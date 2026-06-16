import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { renderChunk, type ChunkMessageDetails } from "./renderers/chunk.js";
import { readCurrent, readNext, readPrev } from "./fishread.js";
import type { ApiResponse, ReaderStateDto } from "./types.js";

export default function (pi: ExtensionAPI) {
  // 每次向 LLM 发送前过滤掉小说内容，小说正文只进入 TUI 渲染层
  pi.on("context", async (event, _ctx) => {
    const filtered = event.messages.filter(
      (m: any) => m.customType !== "fishread-chunk"
    );
    return { messages: filtered };
  });

  pi.registerMessageRenderer("fishread-chunk", (message, _options, theme) => {
    return renderChunk(
      message.content as string,
      message.details as ChunkMessageDetails,
      theme
    );
  });

  function sendReaderState(result: ApiResponse<ReaderStateDto>) {
    const { data } = result;
    pi.sendMessage({
      customType: "fishread-chunk",
      content: data.chunk.text,
      display: true,
      details: { state: data } satisfies ChunkMessageDetails,
    });
  }

  pi.registerCommand("read", {
    description: "从当前位置继续阅读",
    handler: async (_args, _ctx) => {
      const result = readCurrent();
      if (!result.ok) throw new Error(`[fishread] ${result.error.code}: ${result.error.message}`);
      sendReaderState(result);
    },
  });

  pi.registerCommand("next", {
    description: "阅读下一段",
    handler: async (_args, _ctx) => {
      const result = readNext();
      if (!result.ok) throw new Error(`[fishread] ${result.error.code}: ${result.error.message}`);
      sendReaderState(result);
    },
  });

  pi.registerCommand("prev", {
    description: "阅读上一段",
    handler: async (_args, _ctx) => {
      const result = readPrev();
      if (!result.ok) throw new Error(`[fishread] ${result.error.code}: ${result.error.message}`);
      sendReaderState(result);
    },
  });
}
