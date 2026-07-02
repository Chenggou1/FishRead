# @fishread/sdk

TypeScript SDK for FishRead's local reading runtime.

FishRead 是一个藏在 Pi 里的小说阅读器，专为摸鱼间隙设计。SDK 封装 `fishread` CLI 的稳定 JSON 协议，供 Pi 扩展和其他 UI 包读取书库、阅读状态和目录导航。

## 安装

```sh
npm install @fishread/sdk
```

## Usage

```ts
import {
  ensureFishReadReady,
  importBook,
  readCurrent,
  readNext,
  listReadingNavigation,
} from "@fishread/sdk";

await ensureFishReadReady();
await importBook("~/Books/demo.epub");

const current = await readCurrent();
const next = await readNext();
const navigation = await listReadingNavigation();
```

Every runtime call returns a FishRead API result:

```ts
type ApiResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: { code: string; message: string } };
```

The SDK runs database migrations automatically before runtime commands.
