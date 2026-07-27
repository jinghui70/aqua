# aqua CLI 与 SKILL — 实现计划

> 需求见 `prd.md`,技术设计见 `design.md`。验证用真实 fixture:
> `~/work/frs/backend/common-test/src/main/resources/frs.aqua`

## 有序清单

1. **建 crate**:`crates/aqua-cli`(bin),`Cargo.toml` 加依赖 `aqua-core`/`clap`(derive)/`serde_json`/`anyhow`;workspace `members` 增列。
   - 验证:`cargo build -p aqua-cli` 通过(空 main)。
2. **load.rs**:文件 → `serde_json::Value` → `parse_project`;错误分类(IO/JSON/schema/校验)→ stderr + 非零退出。
3. **query.rs — groups / tables / show**:按 design §4 映射输出文本。
   - 验证:对 `frs.aqua` 跑 `groups`、`tables --group <某组>`、`show <某表>`,人工核对与 GUI 打开一致。
4. **gen.rs — gen entity**:调 `generate_java_entity`,stdout 输出。
   - 验证:对 `frs.aqua` 某表生成 entity,与 aqua GUI 导出/现有生成器测试产物**逐字符对比一致**。
5. **gen.rs — gen datamodel**:调 `generate_frontend_json`(`FrontendJsonOptions{table:Some(..)}`),stdout。
   - 验证:同上,对比 DataModel JSON 一致。
6. **clap 顶层**:`main.rs` 装配子命令 + `--help`;统一退出码。
   - 验证:`aqua --help` / 各子命令 `--help` 可读;错误路径(表不存在、文件不存在)给清晰 stderr。
7. **分发**:写 `skills/aqua/SKILL.md`(含按平台选二进制的说明);`release.yml` matrix 加 `cargo build --release -p aqua-cli` + 上传二进制。
   - 验证:本地 `./target/debug/aqua-cli groups frs.aqua` 走通;release.yml 语法 `act` 或 push tag 前 review。
8. **SKILL.md**:按 design §6 写;不含文件路径;命令示例用占位 `<file.aqua>`。
   - 验证:通读,确认与 frs spec 无重复、无硬编码路径。

## 验证命令

```bash
FIXTURE=~/work/frs/backend/common-test/src/main/resources/frs.aqua
cargo build -p aqua-cli
./target/debug/aqua-cli "$FIXTURE" groups
./target/debug/aqua-cli "$FIXTURE" tables --group <g>
./target/debug/aqua-cli "$FIXTURE" show <table>
./target/debug/aqua-cli "$FIXTURE" gen entity <table>
./target/debug/aqua-cli "$FIXTURE" gen datamodel <table>
cargo test -p aqua-core   # 确认未回归(CLI 不改 core)
```

**产物一致性是核心验收**:gen entity/datamodel 的输出必须与现有生成器(GUI 路径)完全一致——因为 CLI 只是薄包装,不一致即接线错误。

## 风险 / 回滚点

- **唯一风险点**:`generate_frontend_json` 单表过滤语义(`table: None` 取首表 vs `Some`)——步骤 5 先验证 `Some(table)` 行为符合预期。
- CLI 是纯新增 crate,不改 `aqua-core`/`src-tauri`,回滚=删 crate + 撤 workspace/release.yml 改动,零副作用。
- H2 "参考实现" 与本任务无关(D6 已移出),不涉及。

## 完成定义

prd Acceptance Criteria 全绿;`frs.aqua` 上五条命令跑通且 gen 产物与现有生成器一致;`cargo test -p aqua-core` 无回归;SKILL.md 无硬编码路径、无与 frs spec 重复。
