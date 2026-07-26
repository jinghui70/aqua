# aqua-cli — 后端开发规范

> `crates/aqua-cli`:给消费项目(如 frs)的 AI 用的命令行工具。**薄命令层**。

---

## 定位:薄包装,不是第二个核心

aqua-cli 是 `aqua-core` 之上的**薄命令层**,只做三件事:解析参数 → 调 aqua-core → 格式化输出。
所有 schema 解析、生成、校验逻辑**必须复用 aqua-core**,CLI 内不得自研。

**为什么**:aqua-core 是前后端字段契约的单源,且持续演进。CLI 若自写生成/解析逻辑,就会与 aqua-core 双源漂移——这是本工具设计时明确否决的(见 `07-25-aqua-cli-skill` task)。

---

## 铁律(新增命令前必读)

1. **复用 aqua-core**:生成/解析/校验一律调 aqua-core 函数;发现 aqua-core 缺能力,去 aqua-core 加,不在 CLI 里补。
2. **只读 + 生成,无副作用**:命令不得写回 `.aqua`、不落盘、不连数据库。产物一律打到 **stdout**(放哪由消费项目决定,只有 AI 知道)。
3. **错误到 stderr + 非零退出**:stdout 只放正常产物,避免污染 AI 读取。
4. **调 aqua-core 前防 panic**:部分生成器(如 `generate_frontend_json`)对非法输入会 panic,CLI 层要先校验(如表存在性)再调。

---

## 结构

```
src/
  main.rs            # clap 定义 + 分发 + 统一错误→stderr/退出码
  load.rs            # 读文件 → parse_project(复用 aqua-core 校验)
  commands/
    query.rs         # groups / tables / show(读结构)
    gen.rs           # gen entity / gen datamodel(调生成器)
```

## 分发

二进制随 `release.yml` 的 mac/win matrix 编译,命名为 `aqua-cli-mac-arm64` / `aqua-cli-win-x64.exe`,
随 `skills/aqua/`(SKILL.md,源码在此;无 wrapper,AI 按平台调 `bin/` 下二进制)一起放进消费项目的 `.claude/skills/aqua/`。

## Quality Check

- `cargo clippy -p aqua-cli --all-targets -- -D warnings` 零警告
- `cargo fmt -p aqua-cli -- --check` 干净
- 新增/改命令后,对真实 fixture(`~/work/frs/.../frs.aqua`)跑通,且 `gen` 产物与 aqua-core 生成器一致
