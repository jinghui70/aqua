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

## 技术栈
Tauri 2.x 桌面 + Rust(`crates/aqua-core` 纯逻辑核心 + `src-tauri` 壳) + Vue3/element-plus(`app/`) + Java connector(`connector/`)。
连接层: MySQL/PG 走 Rust native 免 Java;Oracle/信创/H2 走 Java JDBC(spawn `connector.jar`,用户自备 JDK 17+)。
包管理用 **pnpm**。改 connector Java 后须 `pnpm build:connector`(或 `pnpm dev:full`)重建 jar,否则 `pnpm dev` 跑的是旧 jar。

## 代码结构
`crates/aqua-core/src/`(纯逻辑核心,全链路已接通 Tauri 命令):
- `schema/` — 数据模型(Project/Table/Field/Index/枚举/业务类型/校验/自增策略)
- `driver/` — 连接层:`trait Driver` 统一,native(`mysql`/`postgres`,用连接池)+ `jdbc`(spawn connector);批量反解走 `import_tables`
- `import/` — 从数据库反解生成 schema(`import_from_db`)
- `generators/` — `ddl` / `java`(entity) / `frontend_json` / `strconst`
- `dataset/` — 数据集 load/save/import/export
- `diff/` + `alter.rs` — schema 差异比对 + ALTER 生成
- `datasource/` — 数据源持久化(密码 AES 加密)

Tauri 命令层见 `src-tauri/src/commands/`;前端组件见 `app/src/`。
