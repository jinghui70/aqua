# 技术设计：项目文件命名规则调整

## 架构边界

### 改动范围

```
前端层 (app/src):
  - useFileDialog.ts: 文件对话框过滤器 + 移除默认文件名
  - stores/project.ts: saveProject 调用 gitignore 管理
  - stores/datasource.ts: load/persist 传参改为项目路径
  - composables/useTauri.ts: 更新接口签名

Rust 核心层 (crates/aqua-core/src):
  - datasource/mod.rs: 配置文件路径计算改为基于项目路径

Tauri 命令层 (src-tauri/src/commands):
  - datasource.rs: datasource_load/save 签名改为传项目路径
  - project.rs: 新增 update_gitignore command
```

### 不改动

- 项目文件内部 JSON 结构不变
- 数据集文件格式不变（SQLite `.aqua.db` / JSON `.json`）
- 数据源配置文件内部结构不变（仍为加密的 DbConfigFile）

## 核心设计

### 1. 项目文件名前缀提取

**位置**：`crates/aqua-core/src/datasource/mod.rs`

```rust
/// 从项目文件路径提取前缀（去掉 .aqua 扩展名 + 取文件名）
/// 如 "/path/to/myproject.aqua" → "myproject"
/// 如 "/path/to/my.project.aqua" → "my.project"
pub fn extract_project_prefix(project_path: &str) -> Option<String> {
    let path = Path::new(project_path);
    path.file_stem()?.to_str().map(|s| s.to_string())
}

/// 拼接配置文件路径：<dir>/<prefix>.aqua.conf
pub fn config_path_for_project(project_path: &str) -> Result<PathBuf, DataSourceError> {
    let path = Path::new(project_path);
    let dir = path.parent().ok_or_else(|| 
        DataSourceError::Io(io::Error::new(io::ErrorKind::InvalidInput, "无效项目路径"))
    )?;
    let prefix = extract_project_prefix(project_path).ok_or_else(||
        DataSourceError::Io(io::Error::new(io::ErrorKind::InvalidInput, "无法提取文件名前缀"))
    )?;
    Ok(dir.join(format!("{}.aqua.conf", prefix)))
}
```

### 2. 数据源接口变更

**当前签名**：
```rust
pub fn load_db_config(project_dir: &str, key_path: &str) -> Result<...>
pub fn save_db_config(project_dir: &str, key_path: &str, ...) -> Result<...>
```

**新签名**：
```rust
pub fn load_db_config(project_path: &str, key_path: &str) -> Result<...>
pub fn save_db_config(project_path: &str, key_path: &str, ...) -> Result<...>
```

**内部调整**：
```rust
pub fn load_db_config(project_path: &str, key_path: &str) -> Result<...> {
    let config_file = config_path_for_project(project_path)?;
    if !config_file.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&config_file)?;
    // ... 其余逻辑不变
}
```

### 3. Tauri 命令层

**位置**：`src-tauri/src/commands/datasource.rs`

```rust
#[tauri::command]
pub async fn datasource_load(project_path: String, key_path: String) -> Result<...> {
    aqua_core::datasource::load_db_config(&project_path, &key_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn datasource_save(project_path: String, key_path: String, sources: Vec<...>) -> Result<...> {
    aqua_core::datasource::save_db_config(&project_path, &key_path, sources)
        .map_err(|e| e.to_string())
}
```

### 4. .gitignore 管理

**位置**：`src-tauri/src/commands/project.rs`

```rust
/// 更新项目目录下的 .gitignore，确保包含指定模式
#[tauri::command]
pub async fn update_gitignore(project_path: String) -> Result<(), String> {
    let path = Path::new(&project_path);
    let dir = path.parent().ok_or("无效项目路径")?;
    let gitignore_path = dir.join(".gitignore");
    
    let patterns = vec!["*.aqua.conf", "*.aqua.db"];
    
    // 读取现有内容
    let mut content = if gitignore_path.exists() {
        tokio::fs::read_to_string(&gitignore_path)
            .await
            .map_err(|e| format!("读取 .gitignore 失败: {}", e))?
    } else {
        String::new()
    };
    
    // 检查并追加缺失的模式
    let mut modified = false;
    for pattern in patterns {
        if !content.lines().any(|line| line.trim() == pattern) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(pattern);
            content.push('\n');
            modified = true;
        }
    }
    
    // 仅在有变更时写入
    if modified || !gitignore_path.exists() {
        tokio::fs::write(&gitignore_path, content)
            .await
            .map_err(|e| format!("写入 .gitignore 失败: {}", e))?;
    }
    
    Ok(())
}
```

### 5. 前端 store 调整

**位置**：`app/src/stores/datasource.ts`

```typescript
export const useDataSourceStore = defineStore("datasource", () => {
  const projectPath = ref<string>("");  // 改为存储完整项目路径
  
  async function load(path: string) {
    projectPath.value = path;
    if (!path) {
      sources.value = [];
      return;
    }
    const wire = await tauri.datasourceLoad(path);
    sources.value = wire.map(fromWire);
  }
  
  function persist(): Promise<void> {
    if (!projectPath.value) return Promise.resolve();
    const run = () => tauri.datasourceSave(projectPath.value, sources.value.map(toWire));
    flushChain = flushChain.then(run, run);
    return flushChain;
  }
  
  async function bindDirAndPersist(path: string) {
    projectPath.value = path;
    await persist();
  }
});
```

**调用方**（`stores/project.ts`）：
```typescript
// openProject (line 83):
await datasource.load(path);  // 传完整路径

// saveProject (line 90):
async function saveProject(path?: string) {
  // ... 保存逻辑
  await tauri.projectSave(target, currentProject.value);
  
  // 更新 .gitignore
  try {
    await tauri.updateGitignore(target);
  } catch (err) {
    console.warn('更新 .gitignore 失败:', err);
  }
  
  // 首次保存或另存为
  if (firstBind) await datasource.bindDirAndPersist(target);
}
```

### 6. 数据集路径拼接

**位置**：前端数据集相关代码（未来实现扫描时使用）

```typescript
function getDatasetPath(projectPath: string, datasetName: string, format: 'db' | 'json'): string {
  const dir = dirOf(projectPath);
  const prefix = projectPath.replace(/\.aqua$/, '').split(/[/\\]/).pop()!;
  
  if (format === 'db') {
    return `${dir}/${prefix}.${datasetName}.aqua.db`;
  } else {
    return `${dir}/${prefix}.${datasetName}.json`;
  }
}
```

## 数据流

### 打开项目流程

```
用户选择 myproject.aqua
  ↓
project.openProject(path)
  ↓
tauri.projectOpen(path) → Project
  ↓
datasource.load(path)
  ↓
Rust: config_path_for_project("myproject.aqua") → "myproject.aqua.conf"
  ↓
加载 myproject.aqua.conf → 解密密码 → 返回数据源列表
```

### saveAs 流程

```
用户 saveAs: myproject.aqua → another.aqua
  ↓
project.saveProject(anotherPath)
  ↓
tauri.projectSave(anotherPath, project)
  ↓
tauri.updateGitignore(anotherPath) → 更新 .gitignore
  ↓
datasource.bindDirAndPersist(anotherPath)
  ↓
persist() → tauri.datasourceSave(anotherPath, sources)
  ↓
Rust: config_path_for_project("another.aqua") → "another.aqua.conf"
  ↓
保存到 another.aqua.conf
```

## 兼容性

### 向后兼容

- 配置文件不存在时返回空列表，不报错

### 不兼容变更

- 文件扩展名：旧 `.json` 项目需手动改为 `.aqua`
- 配置文件名：旧 `.dbconfig.json` 需手动改为 `<prefix>.aqua.conf`

## 风险点

1. **前缀提取边界**：文件名为 `.aqua` → `file_stem()` 返回 `None`，返回错误
2. **projectPath 为空串**：`load("")` / `persist()` 正确跳过
3. **.gitignore 并发写入**：同目录多项目同时保存 → 使用文件锁或接受最后写入胜出
