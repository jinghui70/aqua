# aqua CLI 与 SKILL — 技术设计

> 需求/决策见 `prd.md`。本文件只写技术落地。

## 1. 架构与边界

- 新增 **`crates/aqua-cli`**(bin crate),依赖 `aqua-core`。不碰 `src-tauri`、不引 tauri。
- CLI 是 aqua-core 之上的**薄命令层**:解析参数 → 调 aqua-core → 格式化到 stdout。无任何业务/生成逻辑自研(R3)。
- workspace:`Cargo.toml` 的 `[workspace].members` 增加 `crates/aqua-cli`。

## 2. crate 布局

```
crates/aqua-cli/
  Cargo.toml            # deps: aqua-core, clap(derive), serde_json, anyhow
  src/
    main.rs             # clap 解析 + 分发 + 统一错误→stderr/退出码
    load.rs             # 读文件 → serde_json::Value → parse_project(带校验)
    commands/
      query.rs          # groups / tables / show
      gen.rs            # gen entity / gen datamodel
```

- 参数解析用 **clap(derive)**。
- 统一错误处理:任何步骤失败 → 打印到 **stderr** + 非零退出码;stdout 只放"给 AI 用"的正常产物,避免污染。

## 3. 输入解析(load.rs)

```rust
let json = std::fs::read_to_string(path)?;                     // IO 错误
let value: serde_json::Value = serde_json::from_str(&json)?;   // JSON 错误
let project = aqua_core::schema::validate::parse_project(value)?; // schema/校验错误
```

- `parse_project`(`schema/validate.rs:91`)已含反序列化 + 校验,复用它,CLI 不另写校验。
- `ParseError::Validate` 携带结构化错误 → CLI 转成可读文本到 stderr。

## 4. 命令 → aqua-core API 映射

| 命令 | 数据来源 / 调用 | stdout 输出 |
|---|---|---|
| `groups <file>` | `project.groups: Vec<GroupDefine{code,name}>` | 每行 `code  name` |
| `tables [--group <g>] <file>` | `project.tables` 按 `t.group == g` 过滤 | 每行 `code  name  (group)` |
| `show <table> <file>` | `project.tables.find(code==table)` | 字段表:`code/name/data_type/length/scale/is_key/not_null/biz_type` + 索引(`table.indexes`) |
| `gen entity <table> <file>` | `generators::java::generate_java_entity(&project, table, &JavaOptions::default())` → `Result<String,String>` | entity Java 源码 |
| `gen datamodel <table> <file>` | `generators::frontend_json::generate_frontend_json(&project, &FrontendJsonOptions{ table: Some(table) })` → `String` | DataModel JSON |

- **包名**:`JavaOptions.package = None` → 生成器按 `basePackage` + 分组自算(`strconst.rs` 同款逻辑),CLI 不传、AI 不传。
- **`show` 输出格式**:人类/AI 可读的紧凑文本(非 JSON),目的就是省 token。逻辑类型/bizType 用 `.aqua` 里的原值展示。
- 查询类命令(groups/tables/show)输出**文本**;gen 类**原样输出生成器产物**(Java / JSON)。

## 5. 分发(D1/D2)

- **release.yml**:在现有 mac/win matrix 里,`pnpm tauri build` 之后加一步 `cargo build --release -p aqua-cli`,上传二进制(`aqua-mac-arm64` / `aqua-win-x64.exe`)到 Release。
- **skill 目录结构**(消费项目 `.claude/skills/aqua/`):
  ```
  aqua/
    SKILL.md
    bin/aqua-mac-arm64
    bin/aqua-win-x64.exe
  ```
- 无 wrapper:AI 按自身 OS 直接调 `bin/` 下二进制(SKILL.md 说明),消除 windows 的 bash 依赖。

## 6. SKILL.md 骨架(D8)

```
# aqua — 数据表结构工具
（一句话:aqua 管理项目数据表结构;本 skill 是命令手册)

## schema 速览（精简）
- 逻辑类型:Int/Long/Decimal/Tinyint/Varchar/Clob/Blob/Date/Datetime
- bizType:业务类型（如 Bool）；分组=模块；表=code/name；字段=code/prop/name/类型...
（够读懂 show 输出即可，不写完整 DSL）

## 命令
groups / tables / show / gen entity / gen datamodel — 用法 + 示例 + stdout 说明

## 典型用法
- 不知有哪些模块 → groups
- 看某模块的表 → tables --group
- 写业务代码要懂表 → show
- 生成 entity / DataModel → gen ...（stdout，AI 落到项目规定位置）
```

- **不写 `.aqua` 文件位置**(项目特有,归 frs spec)。
- 落位提示(entity 进功能包等)属 frs 流程,SKILL 只提"输出到你项目规范的位置",不硬编码路径。

## 7. 待实现期确认(非阻塞)

- `generate_java_entity` 返回 `Result<_, String>`,错误语义在 CLI 层包装。
- `show` 的具体列与对齐格式(实现时定,以 AI 易读为准)。
- clap 版本与 workspace 依赖对齐(`Cargo.toml` workspace deps)。

## 8. 不做(见 prd Out of Scope)

DDL / insert / H2 / constants / 写入 / validate 命令,均不在本 crate。
