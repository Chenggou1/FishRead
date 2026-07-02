import { Key } from "@earendil-works/pi-tui";

export const FR_SUBCOMMANDS = ["next", "prev", "toc", "books", "import"] as const;
export const BOSS_KEY = Key.ctrlShift("h");
export const NEXT_PAGE_KEY = Key.ctrlShift("right");
export const PREV_PAGE_KEY = Key.ctrlShift("left");
export const NEXT_PAGE_KEY_LABEL = "ctrl+shift+right";
export const PREV_PAGE_KEY_LABEL = "ctrl+shift+left";
export const STATUS_KEY = "fishread";
export const WIDGET_KEY = "fishread-reader";

export const FR_SUBCOMMAND_DETAILS: Record<
  (typeof FR_SUBCOMMANDS)[number],
  { description: string; shortcut: string }
> = {
  next: { description: "下一页", shortcut: NEXT_PAGE_KEY_LABEL },
  prev: { description: "上一页", shortcut: PREV_PAGE_KEY_LABEL },
  toc: { description: "目录", shortcut: "/fr toc" },
  books: { description: "图书管理", shortcut: "/fr books" },
  import: { description: "导入 EPUB", shortcut: "/fr import [path]" },
};
