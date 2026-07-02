# FishRead

> FishRead 是一个藏在 Pi 里的小说阅读器，专为摸鱼间隙设计。老板键一按，阅读界面立刻隐藏。

FishRead 目前支持导入 EPUB 小说，并会把书籍内容写入本地书库。你可以在 Pi 对话线程里阅读、翻页、通过目录跳转和管理书库；阅读进度保存在本地 SQLite 中，老板键隐藏后也能恢复到之前的阅读状态。

## Why FishRead?

- **为摸鱼看小说设计**：AI 回复、命令执行、测试运行的空档，都可以顺手看两段小说。
- **藏在 Pi 里**：不用切应用，阅读内容就在对话线程旁边。
- **老板键一键隐藏**：按下 `ctrl+shift+h` 立刻隐藏阅读界面，再按一次恢复。
- **本地书库和进度**：导入后的书籍、阅读位置和状态保存在本地 SQLite 中。

## Quick Start

### Pi Extension

```sh
pi install npm:@fishread/pi-extension
```

重启 Pi 后，使用 `/fr import` 导入小说：

```text
/fr import 
```

导入完成后，FishRead 会自动显示当前阅读内容，并在本地保存阅读进度。

## Usage

### Slash Commands

| Command | Description |
| --- | --- |
| `/fr next` | 阅读下一段 |
| `/fr prev` | 回到上一段 |
| `/fr toc` | 打开目录和阅读锚点导航 |
| `/fr books` | 打开书库管理 |
| `/fr import [path]` | 导入本地小说文件 |

### Keybindings

| Action | Key |
| --- | --- |
| 老板键隐藏 / 恢复 | `ctrl+shift+h` |
| 下一段 | `ctrl+shift+right` |
| 上一段 | `ctrl+shift+left` |

## Features

- 在 Pi 对话线程里阅读小说。
- 支持本地 EPUB 导入，后续可扩展更多书籍格式。
- 自动保存当前书籍和阅读位置。
- 目录导航支持章节和章节内阅读锚点。
- 书库管理支持切换、重命名和删除图书。
- 老板键会隐藏 FishRead 阅读界面，并在恢复时保留之前的阅读状态。
- Rust CLI 输出稳定 JSON，方便 SDK 和扩展集成。

## Development

```sh
pnpm run build
pnpm run check
pnpm run test:rust
pnpm run dev:pi
```

## Project Map

- `rust/fishread-core/`：书库、导入、阅读状态和核心业务逻辑。
- `rust/fishread-cli/`：命令解析、JSON 输出和退出码。
- `packages/cli/`：npm CLI wrapper，负责解析并运行平台二进制。
- `packages/sdk/`：TypeScript SDK，供 UI 包消费本地阅读运行时。
- `packages/pi-extension/`：Pi 扩展和 TUI 阅读界面。
