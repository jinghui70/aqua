# 执行计划:数据源持久化

## 顺序(后端 → command → 前端 → 验证)

1. **aqua-core `datasource` 模块**
   - 新建 `crates/aqua-core/src/datasource/mod.rs`:类型、错误、key 加载/生成、encrypt/decrypt、load/save_db_config。
   - `crates/aqua-core/src/lib.rs` 加 `pub mod datasource;`。
   - 写单测(roundtrip / 篡改 / 换 key / 文件 roundtrip / key 生成复用)。
   - `cargo test -p aqua-core` + `cargo clippy -p aqua-core`。

2. **src-tauri command**
   - 新建 `src-tauri/src/commands/datasource.rs`:`key_path` helper + `datasource_load` / `datasource_save`。
   - `commands/mod.rs` 加 `pub mod datasource;`;`lib.rs` invoke_handler 注册两 command。
   - `cargo build`。

3. **前端**
   - `useTauri.ts`:加 `datasourceLoad` / `datasourceSave`。
   - `stores/datasource.ts`:加 `load` / `persist`,改造 add/update/remove。
   - `stores/project.ts`:open/save/new 联动。
   - `pnpm build`。

4. **验证**
   - clippy 0 warning、cargo test 全绿、pnpm build 通过。
   - 人工:新建项目→加数据源→保存→重开→数据源恢复且能测连接(GUI,需用户实测)。

## 风险 / 注意

- aes-gcm 0.10 的 `OsRng` 来自 `aes_gcm::aead::OsRng`(re-export),避免额外引 rand。
- Windows 无 `set_permissions(0o600)`,用 `#[cfg(unix)]` 包裹。
- wire 类型 `Vec<(String, DataSourceConfig)>` 序列化为 JSON 数组套二元组;前端按 `[name, config]` 解构。
- `DataSourceConfig.port` 用 `u16`;前端 DbConfig.port 是 number,注意范围。
