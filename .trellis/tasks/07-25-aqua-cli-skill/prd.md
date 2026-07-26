# aqua CLI 与 SKILL

## Goal

在 aqua 项目里交付**一个 Rust CLI(复用 aqua-core)+ 一个 `aqua` SKILL**,让消费项目(如 frs)里的 AI 能省 token 地读表结构、生成符合 dba/json-ui 规范的 entity 与 DataModel。填上 frs spec 中多处引用的 "`aqua` skill(待就绪)" 的坑。

## Background

- **消费方 frs**(`~/work/frs`)已用 Trellis 管理,其 spec 把 aqua 的角色定死:
  - aqua 文件按模块分文件、git 管理(具体位置是 frs 约定,由 frs spec 管,不入 SKILL)。
  - "AI 用 aqua 命令行直接读文件,获取表列表/表结构,生成 entity(dba 规范)/ DataModel(json-ui 规范)。"
  - 三大工具闭环:aqua 定义表 → entity(dba skill) / DataModel(json-ui skill) / DatabaseConstants。
  - 引用位置:`~/work/frs/.trellis/spec/guides/feature-development.md`、`backend/database-guidelines.md`。
- **aqua 现状**(`~/work/aqua`):
  - `.aqua` 文件即 `Project` 结构体的 `serde_json::to_string_pretty` 输出(`src-tauri/src/commands/project.rs:26`)。
  - `aqua-core` 是纯逻辑核心,依赖无 tauri(`crates/aqua-core/Cargo.toml`)→ 可干净抽出 CLI。
  - 生成器**精确对应** frs 产物:`java`=dba 规范 entity(`generators/java/mod.rs`)、`frontend_json`=json-ui DataModel。`strconst`=DatabaseConstants(本期不暴露)、`ddl`(含 h2/insert,不进 CLI)。
  - `release.yml` 已有跨平台发布 CI:`macos-latest`(arm64)+ `windows-latest`(x64)→ 覆盖团队平台,CLI 搭便车。

## Decisions

- **D1 分发**:CLI 二进制放进消费项目 `.claude/skills/aqua/` 随 SKILL 一起 git 管理。零安装、clone 即用、版本天然对齐。代价:二进制入 git(可接受)。
- **D2 平台**:团队 Windows + Mac(可能含 Intel Mac),无 Linux → `mac-arm64` + `win-x64`(视情况加 `mac-x64`)。**无统一 wrapper**,AI 按自身 OS 直接调 `bin/` 下对应二进制(SKILL.md 说明),消除 windows 的 bash 依赖。
- **D3 语言**:Rust,复用 aqua-core,挂 `release.yml` matrix。已确认(Python 重写因双源漂移 + Rust 跨平台 CI 已就绪而否决)。
- **D4 不写入**:CLI 只读 + 生成,全部无副作用。AI 设计表(直接编辑 `.aqua` JSON)+ `validate` 归后续。
- **D5 测试架构(方案 A)**:frs 测试建 H2 内存表由 **frs 的 Java `DbTest`/`MemoryDba` 自行实现**(读 `.aqua` → H2 DDL + `.data` → insert)。跨语言复用三条路(spawn 进程/平台耦合、预生成数据同步隐患、Java 重写)均有代价;H2 转换稳定 + H2 是 Java 生态原生(Hibernate H2Dialect)→ Java 重写代价最低。**属 frs 后续任务。**
- **D6 CLI 不做 DDL/insert/H2/constants**:DDL 手工建表用 GUI 导出;测试 H2 走 D5;constants 本期不做。CLI 在这些上无本期消费者。
- **D7 产物 I/O = stdout**:`gen` 打到 stdout,AI 读后用 Write 落到 frs 规定位置(产物落位由 frs 目录规范决定、只 AI 知)。包名 CLI 从 `.aqua` 自算,不用传。
- **D8 SKILL 内容**:aqua 简介 + schema 速览(精简,够读懂 `show`,不写完整 DSL)+ 命令清单 + 典型触发。**不写 `.aqua` 文件位置**(项目特有)。分工:SKILL=工具手册,frs spec=流程,互相引用不重复。SKILL 在 aqua 项目维护、随 release 分发,消费项目放进 `.claude/skills/aqua/`。

## CLI Commands（本期)

```
aqua groups <file.aqua>                    # 列所有表组(code + name)
aqua tables [--group <code>] <file.aqua>   # 列表(可按组过滤);code + name(+ 所属组)
aqua show   <table> <file.aqua>            # 单表结构:字段(code/name/逻辑类型/长度/主键/notNull/bizType)+ 索引
aqua gen entity    <table> <file.aqua>     # → stdout: dba 规范 entity Java
aqua gen datamodel <table> <file.aqua>     # → stdout: json-ui DataModel JSON
```

命令按三层(组→表→字段)拆分命名,不用重载的 `list`。`<file.aqua>` 位置参数由 AI 传入。参数最终形态在 design.md 敲定。

## Requirements

- **R1 查询(省 token)**:`groups`(表组)、`tables [--group]`(表)、`show <table>`(单表结构)。避免整份 `.aqua` 进 AI 上下文。
- **R2 生成**:`gen entity`(java)、`gen datamodel`(frontend_json),stdout 输出,符合 dba/json-ui 规范(生成器已对齐)。
- **R3 复用 aqua-core**,不重复实现 schema/生成逻辑。
- **R4 SKILL**:见 D8。

## Acceptance Criteria

- [ ] 新增 `crates/aqua-cli`,以 `aqua-core` 为依赖,无重复 schema/生成实现(R3)。
- [ ] `aqua groups` / `aqua tables [--group <g>]` / `aqua show <table>` 正确输出(R1)。
- [ ] `aqua gen entity <table>` / `gen datamodel <table>` stdout 输出与现有生成器一致的产物(R2,D7)。
- [ ] `release.yml` 产出 `mac-arm64` + `win-x64` CLI 二进制(D2)。
- [ ] `aqua` SKILL 含 aqua 简介 + schema 速览 + 命令清单 + 典型触发,**不含 `.aqua` 文件位置**;可放入消费项目 `.claude/skills/aqua/` 并在目标平台跑通(D1/D2/D8)。

## Out of Scope（本期不做）

- CLI 的 DDL / insert / H2 方言 / `gen constants`(D6)。
- 写入 `.aqua`(结构化 add、validate)。
- 连真实数据库核对结构;重写/大改现有生成器。

## Related / 后续任务

- **frs `DbTest`/`MemoryDba`**(Java,不同 repo):读 `.aqua` 生成 H2 DDL + `.data` 生成 insert 建内存表(D5)。
- **`gen constants`**:DatabaseConstants 生成(strconst 现成,后续暴露)。
- **AI 设计新表**:直接编辑 `.aqua` JSON;配套 `validate` 命令。

## Open Questions

- 无阻塞项。命令名/参数细节在 design.md 敲定。
