# 执行计划:内置业务类型加载

## 顺序

1. **结构扩展(参数默认值)**
   - `crates/aqua-core/src/schema/biz_type.rs`:`BizTypeDataField` 加 `default_value: Option<serde_json::Value>`(serde rename "default",skip_serializing_if)。
   - `app/src/types/schema.ts`:`BizTypeDataField` 加 `default?: string | number`。
   - `cargo test -p aqua-core` 确认旧测试不破。

2. **清单文件 + 资源配置**
   - 新建 `src-tauri/resources/builtin-biztypes.json`(含 Date 示例,见 design.md)。
   - `tauri.conf.json` `bundle` 加 `"resources": ["resources/builtin-biztypes.json"]`。

3. **src-tauri command**
   - 新建 `src-tauri/src/commands/builtin.rs`:`builtin_biztypes_load`。
   - `commands/mod.rs` 加 `pub mod builtin;`;`lib.rs` 注册 command。
   - `cargo build` 确认(含 Tauri path API 用法)。

4. **前端**
   - `useTauri.ts` 加 `builtinBiztypesLoad`。
   - 新建 `stores/builtin.ts`。
   - `App.vue` onMounted 调 `builtin.load()`。
   - `BizTypeManage.vue`:合并展示、内置只读禁删、新建重名校验含内置。
   - `FieldDetailDialog.vue`:bizType 下拉合并内置 + 自定义;`initBizTypeData` + `onBizTypeChange` 默认值初始化。
   - `pnpm build`。

5. **验证**
   - clippy 0 warning、cargo test、cargo build、pnpm build 通过。
   - 人工:启动 dev,BizTypeManage 见 Date 只读条目;字段详情选 Date,format 自动填 YYYYMMDD、dataType 锁 VARCHAR、length 填 8(需用户实测)。

## 注意

- Tauri 2 `BaseDirectory::Resource` dev 解析路径若不符,改用 `app.path().resource_dir()?.join("resources/builtin-biztypes.json")`。
- 内置条目在 BizTypeManage 选中时,右侧表单设为只读(disabled)+ 顶部提示"内置业务类型不可编辑"。
- 不改 `BizTypeDefine` Rust/TS 结构。
