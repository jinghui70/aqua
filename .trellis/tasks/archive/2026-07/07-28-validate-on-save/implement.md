# 实现计划

## 步骤

1. 后端新建 `crates/aqua-core/src/schema/keywords.rs`：`SQL_RESERVED`（ANSI + MySQL/PG/Oracle/H2 保留字并集）+ `JAVA_KEYWORDS`（JLS 列表）const 集合，导出 `is_sql_reserved`/`is_java_keyword`；`schema/mod.rs` 注册 `mod keywords; pub use keywords::*;`。
2. 后端扩展 `schema/validate.rs`：在现有规则后追加 5 条（code 表内重复 / code SQL 保留字 / prop Java 关键字 / DECIMAL p<s / 索引名表内重复），复用 `ValidationError::new`，不短路收集。
3. 后端 `src-tauri/src/commands/project.rs:37` `project_validate` 改返回 `Result<Vec<ValidationError>, String>`（序列化 JSON 数组）。
4. 前端 `app/src/composables/useTauri.ts`：`projectValidate` 改 `invoke<ValidationError[]>`；`app/src/types/schema.ts` 加 `ValidationError` 类型（`{path, message}`）。
5. 前端 `app/src/stores/project.ts`：
   - 加 `cleanupProject(p)`（R1 清理）。
   - `saveProject` 改 `Promise<boolean>`：cleanup -> projectValidate -> 有错 `showValidateErrors` confirm -> 取消 return false / 落盘 return true。
   - 加 `showValidateErrors(errors)`（ElMessageBox.confirm，dangerouslyUseHTMLString，按表分组）。
6. 前端 `app/src/views/table-editor/IndexTab.vue`：索引字段 el-select 选项过滤当前索引已选 code（除自身），预防重复（R8）。
7. 前端调用方适配：
   - `useMenuActions.ts:90` -- 不处理返回（取消即停）。
   - `NewProjectDialog.vue:38` -- 判断返回，取消不进"已创建"。
   - `confirmIfDirty`(:140) -- `return await saveProject(target)`。
8. 验证。

## 验证命令

- `cargo test -p aqua-core`（后端校验单测，含新增规则 + 关键字）。
- `pnpm -C app exec vue-tsc --noEmit`（前端类型）。
- `pnpm dev` 手测：
  - 新增空 code 字段 + 空索引 -> 保存 -> 被清理删除。
  - 同表两个同 code 字段 -> 保存 -> 报 code 重复。
  - code=VALUE -> 报 SQL 保留字；prop 字段名转成 class/int -> 报 Java 关键字。
  - DECIMAL p=2 s=4 -> 报 p<s。
  - 两个同名索引 -> 报索引名重复。
  - 有错 confirm：仍保存落盘 / 取消去修不落盘。
  - 关闭项目时自动保存校验取消 -> 不关闭。

## 风险点 / 回滚

- **关键字列表整理量大**：SQL 保留字并集（MySQL/PG/Oracle/H2 官方 keywords 文档取并集，几百个）。偏严不偏松（D4）。若误判常用词为保留字，用户反馈后从集合移除。
- **saveProject 返回值改**：3 个调用方逐一适配，勿漏 confirmIfDirty（否则取消仍关闭项目）。
- **后端 project_validate 返回类型改**：前端 invoke 类型同步，否则运行时解析错。
- **清理删除用户数据**：code 空字段被删是预期（用户已同意），但 confirmIfDirty 自动保存路径也会清理（用户关闭时静默删空字段）--若不接受可改为仅手动保存清理。
- 回滚：git revert，validate_project 旧规则保留不受影响。
