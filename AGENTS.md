# FishRead Agent Guide

FishRead is a local EPUB reading runtime. The Rust CLI owns the reading engine and emits stable JSON for the Pi extension.

## Project Map

- `rust/fishread-core/` — business logic: books, chapters, EPUB parsing, import, storage, reader state, protocol DTOs.
- `rust/fishread-cli/` — CLI argument parsing, command dispatch, JSON output, and exit codes.
- `packages/cli/` — npm wrapper that resolves and runs the platform `fishread` binary.
- `packages/cli-*/` — platform package skeletons for Rust binaries.
- `packages/pi-extension/` — Pi extension bridge and TUI rendering.
- `migrations/` — SQLite schema migrations.
- `fixtures/epub/` and `samples/` — EPUB fixtures and sample source notes.
- `docs/agents/` — configuration consumed by engineering skills.
- `internal/docs/` — ignored original design notes and milestone references.

## Commands

- `pnpm run build` — build all modules.
- `pnpm run build:rust` — build the Rust workspace.
- `pnpm run build:js` — build JS/TS packages that define a build script.
- `pnpm run check` — run all checks.
- `pnpm run check:rust` — Rust format check, clippy, and tests.
- `pnpm run check:js` — JS/TS package checks.
- `pnpm run test:rust` — Rust test suite.
- `pnpm run format:rust` — format Rust code.
- `pnpm run install:rust` — install the local Rust CLI as `fishread`.

## Core Rules

- CLI output must be stable JSON: success as `{"ok":true,"data":...}`, error as `{"ok":false,"error":...}`.
- CLI commands should only parse arguments, call `fishread-core`, print JSON, and map exit codes.
- Keep business logic in `fishread-core`.
- EPUB crate types may appear only in `importer/epub.rs` and the `epub/` module.
- Database writes must use transactions and must not leave a partially imported book.
- Default root commands should operate on all modules; use `:rust`, `:js`, or another suffix only for scoped commands.
- When working with Python, use uv workflows: `uv run`, `uv run --with <pkg>`, `uvx`, `uv add`, and `uv sync` as needed.
- For library, framework, SDK, API, CLI tool, or cloud-service docs, fetch current docs with `ctx7` before answering or changing code.
- Git commit messages must use the `type: message` format, such as `docs: add project agent guidance`.
- Git commits must not mention Codex, AI assistance, generated-by trailers, or co-author attribution.

## Exit Codes

| Code | Meaning |
| ---- | ------- |
| 0 | Success |
| 1 | Business error |
| 2 | Argument error |
| 3 | Internal error |

## Documentation Index

- [Issue tracker setup](docs/agents/issue-tracker.md)
- [Triage label mapping](docs/agents/triage-labels.md)
- [Domain docs rules](docs/agents/domain.md)
- [Milestones](internal/docs/milestones/)
- [Architecture reference](internal/docs/references/架构设计.md)
- [Data model reference](internal/docs/references/数据模型.md)
- [SQLite schema reference](internal/docs/references/SQLite-Schema.md)
- [JSON protocol reference](internal/docs/references/JSON-Protocol.md)
- [EPUB conversion reference](internal/docs/references/EPUB数据转换.md)
- [Reading chunk design](internal/docs/references/阅读chunk设计.md)

## Agent Skills

Issues and PRDs are tracked in GitHub Issues for `Chenggou1/FishRead`; see `docs/agents/issue-tracker.md`.

Use the default triage labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`; see `docs/agents/triage-labels.md`.

This repo uses a single-context domain docs layout: root `CONTEXT.md` and `docs/adr/`; see `docs/agents/domain.md`.
