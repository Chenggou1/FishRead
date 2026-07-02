# @fishread/cli

JavaScript wrapper for the FishRead Rust CLI.

FishRead 是一个藏在 Pi 里的小说阅读器，专为摸鱼间隙设计。CLI 负责本地阅读运行时：初始化数据库、导入书籍、管理书库、读取当前段落，并输出稳定 JSON 供 SDK 和扩展消费。

## 安装

```sh
npm install @fishread/cli
```

安装后会暴露 `fishread` 命令：

```sh
fishread init
fishread import ./demo.epub
fishread read current
fishread read next
fishread chapter list --navigation
fishread book list
```

## JavaScript API

```js
import { resolveFishreadPath } from "@fishread/cli";

const fishread = resolveFishreadPath();
```

The wrapper resolves binaries in this order:

```text
FISHREAD_CLI_PATH
rust/target/debug/fishread
@fishread/cli-<platform>/bin/fishread
```

CLI output follows the FishRead JSON protocol:

- Success: `{"ok":true,"data":...}`
- Error: `{"ok":false,"error":...}`
