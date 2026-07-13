# 技术设计:数据源持久化

## 分层边界

- **aqua-core**(纯逻辑):加解密原语 + `.dbconfig.json` 读写。不感知平台目录,key 文件路径由调用方传入。
- **src-tauri**(壳):用 Tauri path API 解析 app data dir → key 路径;暴露无状态 command,签名 `(project_dir, key_path, ...)`。
- **前端**:store 增删改后调 command 落盘;打开项目时加载。

## aqua-core: `datasource` 模块

新建 `crates/aqua-core/src/datasource/mod.rs`。

### 类型

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct DataSourceConfig {
    pub dialect: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,   // 内存态明文;文件态密文
    pub database: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DbConfigFile {
    pub sources: BTreeMap<String, DataSourceConfig>,  // key = sourceName,稳定排序
}
```

### 加密(私有 helper)

```rust
fn load_or_create_key(key_path: &Path) -> Result<[u8; 32], DataSourceError>
```
- 存在:读 32 字节;长度不符报错。
- 不存在:`OsRng` 生成 32 字节 → 创建父目录 → 写文件 → Unix 下 `set_permissions(0o600)`。

```rust
fn encrypt(key: &[u8;32], plain: &str) -> Result<String, _>   // 空串直接返回空串
fn decrypt(key: &[u8;32], token: &str) -> Result<String, _>   // 空串直接返回空串
```
- 加密:随机 12 字节 nonce;`Aes256Gcm::encrypt` → `base64(nonce ‖ ct)`。
- 解密:base64 解码 → 拆 nonce(前 12 字节)+ ct → decrypt。长度不足/解码失败/tag 校验失败 → `DecryptFailed`。

### 对外 API

```rust
pub fn load_db_config(project_dir: &str, key_path: &str) -> Result<Vec<(String, DataSourceConfig)>, DataSourceError>
pub fn save_db_config(project_dir: &str, key_path: &str, sources: Vec<(String, DataSourceConfig)>) -> Result<(), DataSourceError>
```
- `load`:拼 `project_dir/.dbconfig.json`;不存在返回空 Vec;读 → 反序列化 → 逐条 decrypt password → 返回按 name 排序的列表。
- `save`:load_or_create_key → 逐条 encrypt password → 序列化 pretty → 写 `.dbconfig.json`。

### 错误

```rust
#[derive(thiserror::Error, Debug)]
pub enum DataSourceError {
    #[error("IO 错误: {0}")] Io(#[from] std::io::Error),
    #[error("JSON 错误: {0}")] Json(#[from] serde_json::Error),
    #[error("密钥长度非法")] BadKey,
    #[error("密码解密失败(密钥不匹配或数据损坏)")] DecryptFailed,
    #[error("base64 解码失败")] Base64,
}
```

### 依赖

`aes-gcm = "0.10"`(已在 Cargo.toml),用 `Aes256Gcm`、`aead::{Aead, KeyInit, OsRng, AeadCore}`。`base64` 已在。

## src-tauri: command

新建 `src-tauri/src/commands/datasource.rs`,注册进 `mod.rs` + `lib.rs`。

```rust
fn key_path(app: &AppHandle) -> Result<String, String>  // app_data_dir()/key

#[tauri::command]
pub async fn datasource_load(app: AppHandle, project_dir: String) -> Result<Vec<(String, DataSourceConfig)>, String>

#[tauri::command]
pub async fn datasource_save(app: AppHandle, project_dir: String, sources: Vec<(String, DataSourceConfig)>) -> Result<(), String>
```

Tauri `AppHandle::path().app_data_dir()` 拿平台数据目录。

## 前端改造

### `stores/datasource.ts`

- `add/update/remove` 改为触发一次 `persist()`(内部调 `datasource_save`,project_dir 由 currentPath 派生)。
- 新增 `load(projectDir)`:调 `datasource_load` 填充 `sources`。
- 无 currentPath 时 `persist()` 跳过落盘(仅内存,符合"保存项目后落盘")。
- wire 结构是 `Vec<(name, config)>`;store 内部保持现有 `DataSource`(含 sourceName)数组,转换在 store 边界做。

### `stores/project.ts`

- `openProject` 成功后调 `datasource.load(dirOf(path))`。
- `saveProject`(尤其首次另存)后调 `datasource.persist()`,把内存数据源落盘到新目录。
- `newProject` 清空数据源。
- `dirOf(path)`:取文件所在目录(前端用字符串截断到最后一个分隔符;兼顾 `/` 与 `\`)。

### `useTauri.ts`

加 `datasourceLoad(projectDir)` / `datasourceSave(projectDir, sources)`。

## 密钥策略说明

设计文档 §7 写"密钥从机器特征派生"。改为**用户数据目录随机密钥**:机器特征(machine-uid)在重装系统/容器/换硬件后会变,导致自己也解不开;随机密钥落盘更稳定可控,权限 600 防其他用户读取。换机器需拷 key 文件——可接受(与拷项目同理)。已与用户确认。

## 测试(aqua-core 单测)

- `encrypt`→`decrypt` roundtrip 得原文;空串 roundtrip 得空串。
- 篡改密文 → `DecryptFailed`;不同 key 解密 → `DecryptFailed`。
- `save_db_config` 后文件中 password ≠ 明文;`load_db_config` 还原明文(用临时目录 + 临时 key)。
- key 文件不存在 → save 自动生成;二次 save 复用同 key。
