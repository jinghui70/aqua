# CLI 模式实现

## 背景

aqua v2 P1 第一个任务。CLI 模式是验证核心逻辑(generators)命令行可用的最快路径。

- 设计规范: `docs/architecture.md` §7 CLI Tauri 双模式
- 现状: aqua-core generators 已完成,CLI 入口待实现
- 位置: `src-tauri/src/main.rs` 参数解析

## 目标

实现 CLI 模式,支持 `aqua generate --type ddl/java` 命令,验证 generators 可用。

**包含**:
- 参数解析: clap
- CLI 命令: `aqua generate --type ddl --dialect mysql --output schema.sql`
- 调用 aqua-core generators
- 文件写入: stdout 或指定路径

## 范围(不含)

- import 命令(import-module 任务做)
- GUI 模式(Tauri commands 任务做)
- 配置文件(暂不支持,全部通过参数传递)

## 验收标准

**实现**:
- [ ] `src-tauri/src/cli.rs` 参数定义
- [ ] `src-tauri/src/commands/generate.rs` 生成器命令
- [ ] `src-tauri/src/main.rs` 入口判断(有 args 走 CLI)
- [ ] 支持 DDL 生成: `aqua generate --type ddl --input schema.json --dialect mysql`
- [ ] 支持 Java 生成: `aqua generate --type java --input schema.json --table SYS_USER`

**测试**:
- [ ] 手动测试: `cargo run -p aqua -- generate --type ddl --input tests/fixtures/valid-full.json`
- [ ] 输出验证: DDL 可直接执行,Java 可编译

**质量**:
- [ ] `cargo test` 通过
- [ ] `cargo clippy -- -D warnings` 无 warning

## 约束

- 无 args 开 GUI,有 args 走 CLI(不开窗)
- 错误输出到 stderr
- 成功返回 0,失败返回 1

## 参考

- clap 文档: https://docs.rs/clap/
- architecture.md §7 CLI
