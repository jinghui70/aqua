# 实施计划：项目文件命名规则调整

## 实施顺序

按依赖关系从底层到上层：Rust 核心 → Tauri 命令 → 前端

## 检查清单

### Phase 1: Rust 核心层改动

- [ ] **1.1 修改 `crates/aqua-core/src/datasource/mod.rs`**
  - [ ] 新增 `extract_project_prefix(project_path: &str) -> Option<String>`
  - [ ] 新增 `config_path_for_project(project_path: &str) -> Result<PathBuf, DataSourceError>`
  - [ ] 移除 `const DBCONFIG_NAME` 和 `config_path(project_dir: &str)` 函数
  - [ ] 修改 `load_db_config` 签名：`project_dir` → `project_path`
  - [ ] 修改 `save_db_config` 签名：`project_dir` → `project_path`
  - [ ] 更新单元测试

  **验证**：`cargo test -p aqua-core datasource`

### Phase 2: Tauri 命令层改动

- [ ] **2.1 修改 `src-tauri/src/commands/datasource.rs`**
  - [ ] `datasource_load` 签名：`dir: String` → `project_path: String`
  - [ ] `datasource_save` 签名：`dir: String` → `project_path: String`

- [ ] **2.2 修改 `src-tauri/src/commands/project.rs`**
  - [ ] 新增 `update_gitignore(project_path: String)` command
  - [ ] 逻辑：检查/创建 `.gitignore`，追加 `*.aqua.conf` 和 `*.aqua.db`

  **验证**：`cargo build -p aqua-app`

### Phase 3: 前端文件对话框

- [ ] **3.1 修改 `app/src/composables/useFileDialog.ts`**
  - [ ] 修改 `JSON_FILTER` 为 `AQUA_FILTER = [{ name: "Aqua 项目", extensions: ["aqua"] }]`
  - [ ] `pickSaveFile()` 移除 `defaultName` 参数

  **验证**：`cd app && pnpm tsc --noEmit`

### Phase 4: 前端数据源 store

- [ ] **4.1 修改 `app/src/stores/datasource.ts`**
  - [ ] 重命名 `projectDir` → `projectPath`
  - [ ] `load(dir)` → `load(path)`，调用 `tauri.datasourceLoad(path)`
  - [ ] `bindDirAndPersist(dir)` → `bindDirAndPersist(path)`
  - [ ] `persist()` 使用 `projectPath.value`

### Phase 5: 前端项目 store

- [ ] **5.1 修改 `app/src/stores/project.ts`**
  - [ ] `openProject` (line 83)：`datasource.load(path)` 传完整路径
  - [ ] `saveProject` (line 90)：调用 `tauri.updateGitignore(target)` 
  - [ ] `saveProject` (line 100)：`datasource.bindDirAndPersist(target)` 传完整路径

### Phase 6: 前端 Tauri 接口

- [ ] **6.1 修改 `app/src/composables/useTauri.ts`**
  - [ ] `datasourceLoad` 签名：`(dir, ...)` → `(projectPath, ...)`
  - [ ] `datasourceSave` 签名：`(dir, ...)` → `(projectPath, ...)`
  - [ ] 新增 `updateGitignore: (projectPath: string) => invoke<void>(...)`

  **验证**：`cd app && pnpm tsc --noEmit`

### Phase 7: 端到端测试

- [ ] **7.1 功能验证**
  - [ ] 新建项目：文件对话框无默认名，扩展名 `.aqua`，保存后生成 `<name>.aqua` + `<name>.aqua.conf`
  - [ ] 验证 `.gitignore` 自动创建，包含 `*.aqua.conf` 和 `*.aqua.db`
  - [ ] 打开项目：选择 `.aqua` 文件，数据源从 `<name>.aqua.conf` 加载
  - [ ] 另存为：新项目生成独立的 `<newname>.aqua` + `<newname>.aqua.conf`
  - [ ] 同目录多项目：`projectA.aqua` + `projectA.aqua.conf` 和 `projectB.aqua` + `projectB.aqua.conf` 互不干扰

- [ ] **7.2 边界测试**
  - [ ] 项目名含 `.`（如 `my.project.aqua`）：配置文件正确生成 `my.project.aqua.conf`
  - [ ] 配置文件不存在：打开项目时数据源为空，不报错
  - [ ] `.gitignore` 已存在：追加模式，不覆盖原有内容

### Phase 8: 清理

- [ ] 移除旧代码注释
- [ ] 确认所有 `dirOf()` 调用点已正确调整

## 验证命令汇总

```bash
# Rust 编译与测试
cargo test -p aqua-core datasource
cargo build --workspace

# 前端类型检查
cd app && pnpm tsc --noEmit

# 启动开发环境
cd app && pnpm tauri dev
```

## 回滚策略

如发现问题：
1. 恢复 `datasource/mod.rs` 的 `DBCONFIG_NAME` 常量和旧签名
2. 恢复前端 `dirOf()` 调用
3. 移除 `update_gitignore` command
