<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

This project is managed by Trellis. The working knowledge you need lives under `.trellis/`:

- `.trellis/workflow.md` — development phases, when to create tasks, skill routing
- `.trellis/spec/` — package- and layer-scoped coding guidelines (read before writing code in a given layer)
- `.trellis/workspace/` — per-developer journals and session traces
- `.trellis/tasks/` — active and archived tasks (PRDs, research, jsonl context)

If a Trellis command is available on your platform (e.g. `/trellis:finish-work`, `/trellis:continue`), prefer it over manual steps. Not every platform exposes every command.

<!-- TRELLIS:END -->

## Project Design

- **技术架构(authoritative)**: [`docs/architecture.md`](./docs/architecture.md) — aqua v2 Rust+Tauri 架构(grill-me Q1-Q11 决策)。实现前必读。
- **业务设计(authoritative)**: [`docs/design.md`](./docs/design.md) — 数据模型/逻辑类型/DDL 规则/功能边界/UI 需求。技术栈章节(§2/§8)已过时,以 architecture.md 为准。
- **访谈记录**: [`docs/grill-me-2026-07-11.md`](./docs/grill-me-2026-07-11.md)。
- **旧 TS 版参考**: `~/work/aqua-legacy`(作逻辑蓝本,不直接迁代码)。

## 技术栈
Tauri 2.x 桌面 + Rust(`crates/aqua-core` 纯逻辑核心 + `src-tauri` 壳) + Vue3/element-plus(`app/`) + Java connector(复用,`connector/`)。
连接层: MySQL/PG 走 Rust native 免 Java;Oracle/信创/H2 走 Java JDBC(用户自备 JDK 17+)。

## 移植路线
`schema` → `generators` → `dataset` → `import`(依赖 Driver trait) → `diff/ALTER`(新功能)。
当前在 schema 起点(见 `crates/aqua-core/src/schema/mod.rs`)。
