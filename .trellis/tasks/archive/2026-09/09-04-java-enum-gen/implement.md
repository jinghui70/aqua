# Implement — Java 枚举类生成 + DataModel Options 映射

## 前置确认(实现第 0 步)
- [ ] `rg -i "CodeEnum" ~/.gradle ~/.m2 crates connector` / 查 rainbow-dbaccess 坐标,敲定 `CodeEnum` 接口全限定名;写入 design.md「开放确认点」。

## 有序清单
### A. 数据模型 + 校验(`crates/aqua-core/src/schema/`)
1. `enum_def.rs`:`InlineEnum` 加 `class_name: Option<String>`、`r#ref: Option<InlineEnumRef>`,`values` 加 `#[serde(default)]`;新增 `InlineEnumRef { code, prop }`。
2. `validate.rs`:两遍校验——收集定义索引;引用存在性(目标存在 / 为定义方);类型一致性(dataType + length 与目标一致)。
3. 新增 fixture `tests/fixtures/valid-enum-ref.json`(表B 字段 ref 表A GENDER),及 invalid 用例(ref 缺失 / 指向非枚举 / 类型不一致)。

### B. 定义索引(共享工具)
4. 新增 `generators/java/` 内或 schema 层的枚举索引构建:`Project → HashMap<(String,String), EnumMeta>`,`EnumMeta{ class_name, has_code, values }`;Java 与 frontend_json 共用。

### C. Java 生成器(`generators/java/`)
5. `naming.rs`:补 prop→PascalCase(首字母大写)helper(或复用 snake_to_pascal 适配驼峰输入)。
6. `mod.rs` + `entity.rs`:返回 `Vec<JavaFile>`;`generate_entity_class` 产实体 + 本表定义枚举文件;`java_type_for` 枚举分支;新增 `enum.rs` 产枚举类文本(普通 / CodeEnum 两形态)。
7. 调用方适配:`src-tauri/src/commands/generate.rs` `generate_java_command` 返回 `Vec<JavaFile>`;`crates/aqua-cli/src/commands/gen.rs` `entity()` 多文件输出。

### D. DataModel 生成器(`generators/frontend_json.rs`)
8. `transform_field`/`transform_table`/`generate_frontend_json` 透传枚举索引;枚举字段输出 `bizType:"Options"` + `bizTypeData:[{id,name}]`(引用方解析定义方)。

### E. 前端(`app/`)
9. `types/schema.ts`:`InlineEnum` 加 `className?`、`ref?{code,prop}`;`values` 可空。
10. `views/table-editor/JavaTab.vue` + `composables/useTauri`:适配 `JavaFile[]` 多文件预览(文件切换)/保存(逐个或批量导出);`generateJava` 返回类型同步。
11. `views/table-editor/BizTypeEditDialog.vue`:枚举区加「枚举来源」radio(新建定义 / 引用已有)+ `className` 输入 + 二级级联引用选择器(引用表→引用字段)+ 目标只读预览 + 选定后 length 同步;保存校验引用完整性。(设计见 design.md §五)

### F. 文档
12. `docs/design.md` §3.5:更新 Java 生成示例为「描述作注释、单参构造」;补充 ref 引用、className、frontend_json Options 映射说明。

## 验证命令
- `cargo test -p aqua-core`(枚举/校验/两生成器全绿)
- `cargo build`(工作区,含 src-tauri 调用方)
- `pnpm -C app type-check`(或项目既有 tsc 脚本)确认前端类型
- 手验:`cargo run -p aqua-cli -- <file.aqua> gen entity <表>` 输出实体 + 枚举多文件

## 风险 / 回滚点
- **破坏性签名**:`generate_java_entity` 返回类型变更牵动 2 处(cli/tauri)+ 前端——集中一个提交内改完再验,避免半程编译断裂。
- **前端多文件 UX**(步骤 10)与**引用级联选择器**(步骤 11)是前端主要工作量,已在 design.md §五 定死交互,按图施工。
- fixture 改动可能影响既有 ddl/alter 测试(共用 valid-full.json)——新增独立 fixture,勿改 valid-full 现有字段。
