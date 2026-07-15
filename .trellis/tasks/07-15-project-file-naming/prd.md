# 项目文件命名规则调整：支持同目录多项目

## Goal

允许同一目录下存放多个 aqua 项目，通过文件名前缀（项目文件名去扩展名）关联项目文件、配置文件、数据集文件。

## Background

### 当前实现

- **项目文件**：扩展名 `.json`，新建保存默认名 `schema.json`
- **数据源配置**：固定为 `.dbconfig.json`，存于项目文件所在目录
- **数据集**：`dataset.json` / `dataset.db`（默认名），用户可自定义，无强制关联

从代码看：
- `project.ts:83` 打开项目调用 `datasource.load(dirOf(path))`，数据源按目录加载
- `datasource/mod.rs:51` 配置文件名硬编码为 `.dbconfig.json`
- `useFileDialog.ts:5,19` 项目文件过滤器为 `{name: "schema", extensions: ["json"]}`，默认名 `schema.json`
- 数据集无项目级自动关联

### 现有问题

1. **同目录多项目冲突**：项目 A `projectA.json` 和项目 B `projectB.json` 放同一目录时，共享同一个 `.dbconfig.json`
2. **扩展名不专属**：`.json` 太通用，应改为项目级扩展名 `.aqua`
3. **默认文件名不友好**：`schema.json` 语义不清
4. **saveAs 无法独立**：配置文件未按项目隔离

## Requirements

### R1 项目文件扩展名改为 `.aqua`

- 项目文件扩展名从 `.json` 改为 `.aqua`
- 文件对话框过滤器改为 `{name: "Aqua 项目", extensions: ["aqua"]}`
- 新建项目保存时**不提供默认文件名**（`pickSaveFile()` 调用时不传 `defaultName` 参数）

### R2 文件名关联规则

- **项目文件名前缀** = 去掉 `.aqua` 扩展名
  - 如 `myproject.aqua` 前缀为 `myproject`
  - 如 `my.project.aqua` 前缀为 `my.project`
- **数据源配置文件**：`<前缀>.aqua.conf`
  - 如 `myproject.aqua` → `myproject.aqua.conf`
- **数据集文件**：
  - SQLite 格式：`<前缀>.<数据集名>.aqua.db`（不入 git）
  - JSON 格式：`<前缀>.<数据集名>.json`（可入 git）
  - 示例：`myproject.aqua` + 数据集名 `default` → `myproject.default.aqua.db` 或 `myproject.default.json`

### R3 数据源配置文件命名调整

- 打开项目 `<prefix>.aqua` 时，数据源加载路径改为 `<dir>/<prefix>.aqua.conf`
- 保存项目时，数据源持久化到 `<prefix>.aqua.conf`
- 修改位置：
  - `datasource/mod.rs:51` 的 `DBCONFIG_NAME` 改为动态拼接
  - 新增 `config_path_for_project(project_path: &str)` 函数提取前缀
  - `datasource_load/save` commands 传入项目完整路径而非目录

### R4 .gitignore 自动管理

保存/saveAs 项目时，自动处理项目目录下的 `.gitignore`：
- 如不存在 → 创建并写入 `*.aqua.conf` 和 `*.aqua.db`
- 如存在 → 检查是否已包含这两个模式，缺失则追加（每行独立，不重复）
- 追加时保留原文件末尾换行，新行追加到文件末尾
- **不包含** `*.json`（JSON 数据集可入 git）

### R5 数据集目录扫描

数据集通过**目录扫描**发现，不持久化到项目文件：
- 打开项目时，扫描同目录下匹配 `<prefix>.*.aqua.db` 和 `<prefix>.*.json` 的文件
- 提取数据集名和格式：
  - `<prefix>.<name>.aqua.db` → 数据集名 `<name>`，格式 `db`
  - `<prefix>.<name>.json` → 数据集名 `<name>`，格式 `json`
- 数据集管理界面显示扫描到的列表，用户可新建数据集（指定名称和格式）
- 加载/保存数据集时，路径拼接：
  - SQLite: `<dir>/<prefix>.<dataset_name>.aqua.db`
  - JSON: `<dir>/<prefix>.<dataset_name>.json`

## Acceptance Criteria

- [ ] 文件对话框过滤器为"Aqua 项目"+ `.aqua` 扩展名，保存时无默认文件名
- [ ] 保存项目 `myproject.aqua` 时，数据源配置自动写入 `myproject.aqua.conf`
- [ ] 打开项目 `myproject.aqua` 时，数据源从 `myproject.aqua.conf` 加载
- [ ] saveAs 生成独立配置文件（如 `another.aqua` → `another.aqua.conf`），旧配置文件不变
- [ ] 同目录存在 `projectA.aqua` + `projectA.aqua.conf` 和 `projectB.aqua` + `projectB.aqua.conf`，互不干扰
- [ ] 保存/saveAs 时自动创建/更新 `.gitignore`，包含 `*.aqua.conf` 和 `*.aqua.db`（不含 `*.json`）
- [ ] 数据集文件命名：SQLite 为 `myproject.default.aqua.db`，JSON 为 `myproject.test.json`
- [ ] 数据集界面通过目录扫描显示数据集列表

## Out of Scope

- 旧 `.json` 项目文件兼容（用户需手动改扩展名 + 重命名配置文件）
- 数据集文件的自动迁移/改名（saveAs 不处理数据集文件）

## Decisions

### D1: 数据集列表不持久化
**决策**：打开项目时扫描目录，不存入项目文件。
**原因**：数据集文件部分入 git（JSON）、部分不入（DB），列表持久化易不同步。

### D2: 配置文件名后缀
**决策**：`<prefix>.aqua.conf`（两段后缀）
**原因**：清晰标识 aqua 项目配置，与 `.aqua.db` 风格一致。

### D3: .gitignore 自动管理
**决策**：保存/saveAs 时自动创建/更新 `.gitignore`，追加 `*.aqua.conf` 和 `*.aqua.db`。
**原因**：配置文件含密码，SQLite 数据集体积大不宜入库；JSON 数据集可入库用于共享种子数据。

### D4: 数据集文件后缀
**决策**：
- SQLite 格式：`.aqua.db`，不入 git
- JSON 格式：`.json`（单段后缀），可入 git
**原因**：JSON 文件保持常规命名，便于 git diff 和团队协作。

### D5: 文件名前缀允许 `.`
**决策**：允许。提取逻辑为去掉 `.aqua` 后缀（`Path::file_stem()`）。
