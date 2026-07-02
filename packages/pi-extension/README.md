# @fishread/pi-extension

FishRead 是一个藏在 Pi 里的小说阅读器，专为摸鱼间隙设计。老板键一按，阅读界面立刻隐藏。

[GitHub 仓库](https://github.com/Chenggou1/FishRead)

## 安装

```sh
pi install npm:@fishread/pi-extension
```

重启 Pi 后，使用 `/fr import` 导入小说：

```text
/fr import ~/Books/demo.epub
```

FishRead 目前支持导入 EPUB 小说，并会把书籍内容写入本地书库。阅读进度保存在本地 SQLite 中，老板键隐藏后也能恢复到之前的阅读状态。

## 快捷键

| 快捷键 | 功能 |
| --- | --- |
| `Ctrl+Shift+H` | 隐藏或恢复 FishRead 阅读界面 |
| `Ctrl+Shift+Right` | 阅读下一段 |
| `Ctrl+Shift+Left` | 回到上一段 |

## Slash 命令

| 命令 | 功能 |
| --- | --- |
| `/fr next` | 阅读下一段 |
| `/fr prev` | 回到上一段 |
| `/fr toc` | 打开目录和阅读锚点导航 |
| `/fr books` | 打开书库管理 |
| `/fr import [path]` | 导入本地小说文件 |

## 功能

- 在 Pi 对话线程里阅读小说。
- 支持本地 EPUB 导入，后续可扩展更多书籍格式。
- 自动保存当前书籍和阅读位置。
- 目录导航支持章节和章节内阅读锚点。
- 书库管理支持切换、重命名和删除图书。
- 老板键会隐藏 FishRead 阅读界面，并在恢复时保留之前的阅读状态。
