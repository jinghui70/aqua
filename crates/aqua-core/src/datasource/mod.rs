//! datasource 模块 - 数据源配置持久化。
//!
//! 落盘到项目目录 `<prefix>.aqua.conf`,密码用 AES-256-GCM 加密。
//! 密钥为用户数据目录下的 32 字节随机 key(路径由调用方传入,核心不感知平台目录)。
//! 见 `docs/design.md` §7 与本任务 design.md。

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// datasource 错误类型。
#[derive(Error, Debug)]
pub enum DataSourceError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("密钥长度非法(应为 32 字节)")]
    BadKey,
    #[error("密码解密失败(密钥不匹配或数据损坏)")]
    DecryptFailed,
    #[error("base64 解码失败")]
    Base64,
}

/// 单个数据源配置。password 在内存态为明文,文件态为密文。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DataSourceConfig {
    pub dialect: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub schema: Option<String>,
}

/// `.dbconfig.json` 文件结构。key = sourceName,BTreeMap 保证稳定排序。
#[derive(Serialize, Deserialize, Default)]
pub struct DbConfigFile {
    pub sources: BTreeMap<String, DataSourceConfig>,
}

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

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
        DataSourceError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, "无效项目路径"))
    )?;
    let prefix = extract_project_prefix(project_path).ok_or_else(||
        DataSourceError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, "无法提取文件名前缀"))
    )?;
    Ok(dir.join(format!("{}.aqua.conf", prefix)))
}

/// 读取或首次生成 32 字节随机密钥。
fn load_or_create_key(key_path: &str) -> Result<[u8; KEY_LEN], DataSourceError> {
    let path = Path::new(key_path);
    if path.exists() {
        let bytes = std::fs::read(path)?;
        if bytes.len() != KEY_LEN {
            return Err(DataSourceError::BadKey);
        }
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&bytes);
        return Ok(key);
    }
    // 生成新密钥
    let key = Aes256Gcm::generate_key(OsRng);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, key.as_slice())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    let mut out = [0u8; KEY_LEN];
    out.copy_from_slice(key.as_slice());
    Ok(out)
}

/// AES-256-GCM 加密。空串直接返回空串。密文格式 base64(nonce ‖ ciphertext+tag)。
fn encrypt(key: &[u8; KEY_LEN], plain: &str) -> Result<String, DataSourceError> {
    if plain.is_empty() {
        return Ok(String::new());
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let ct = cipher
        .encrypt(&nonce, plain.as_bytes())
        .map_err(|_| DataSourceError::DecryptFailed)?;
    let mut combined = Vec::with_capacity(NONCE_LEN + ct.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ct);
    Ok(BASE64_STANDARD.encode(combined))
}

/// AES-256-GCM 解密。空串直接返回空串。
fn decrypt(key: &[u8; KEY_LEN], token: &str) -> Result<String, DataSourceError> {
    if token.is_empty() {
        return Ok(String::new());
    }
    let combined = BASE64_STANDARD
        .decode(token)
        .map_err(|_| DataSourceError::Base64)?;
    if combined.len() < NONCE_LEN {
        return Err(DataSourceError::DecryptFailed);
    }
    let (nonce_bytes, ct) = combined.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher
        .decrypt(nonce, ct)
        .map_err(|_| DataSourceError::DecryptFailed)?;
    String::from_utf8(plain).map_err(|_| DataSourceError::DecryptFailed)
}

/// 加载项目对应的数据源配置,解密密码,返回按 name 排序的列表。
/// 文件不存在时返回空列表。
pub fn load_db_config(
    project_path: &str,
    key_path: &str,
) -> Result<Vec<(String, DataSourceConfig)>, DataSourceError> {
    let config_file = config_path_for_project(project_path)?;
    if !config_file.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&config_file)?;
    let file: DbConfigFile = serde_json::from_str(&content)?;
    let key = load_or_create_key(key_path)?;
    let mut out = Vec::with_capacity(file.sources.len());
    for (name, mut cfg) in file.sources {
        cfg.password = decrypt(&key, &cfg.password)?;
        out.push((name, cfg));
    }
    Ok(out)
}

/// 保存数据源配置到项目对应的配置文件,密码加密。
pub fn save_db_config(
    project_path: &str,
    key_path: &str,
    sources: Vec<(String, DataSourceConfig)>,
) -> Result<(), DataSourceError> {
    let key = load_or_create_key(key_path)?;
    let mut file = DbConfigFile::default();
    for (name, mut cfg) in sources {
        cfg.password = encrypt(&key, &cfg.password)?;
        file.sources.insert(name, cfg);
    }
    let json = serde_json::to_string_pretty(&file)?;
    let config_file = config_path_for_project(project_path)?;
    std::fs::write(config_file, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aqua_ds_test_{}", uid()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// 测试临时目录唯一后缀:PID + 进程内原子递增。
    /// (原用纳秒时间戳,并发测试同一纳秒会碰撞 → 共用目录 → key 文件互相干扰导致 flaky。)
    fn uid() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{}_{}", std::process::id(), n)
    }

    fn sample(name: &str, pwd: &str) -> (String, DataSourceConfig) {
        (
            name.to_string(),
            DataSourceConfig {
                dialect: "mysql".into(),
                host: "localhost".into(),
                port: 3306,
                user: "root".into(),
                password: pwd.into(),
                database: "mydb".into(),
                schema: None,
            },
        )
    }

    #[test]
    fn test_extract_project_prefix_normal() {
        assert_eq!(extract_project_prefix("/path/to/myproject.aqua"), Some("myproject".to_string()));
        assert_eq!(extract_project_prefix("myproject.aqua"), Some("myproject".to_string()));
    }

    #[test]
    fn test_extract_project_prefix_with_dots() {
        assert_eq!(extract_project_prefix("/path/to/my.project.aqua"), Some("my.project".to_string()));
        assert_eq!(extract_project_prefix("a.b.c.aqua"), Some("a.b.c".to_string()));
    }

    #[test]
    fn test_extract_project_prefix_edge_cases() {
        // .aqua 也有 file_stem，返回 ".aqua"
        assert_eq!(extract_project_prefix(".aqua"), Some(".aqua".to_string()));
        assert_eq!(extract_project_prefix(""), None);
        assert_eq!(extract_project_prefix("/"), None);
    }

    #[test]
    fn test_config_path_for_project() {
        let result = config_path_for_project("/path/to/myproject.aqua").unwrap();
        assert_eq!(result, PathBuf::from("/path/to/myproject.aqua.conf"));

        let result = config_path_for_project("/path/to/my.project.aqua").unwrap();
        assert_eq!(result, PathBuf::from("/path/to/my.project.aqua.conf"));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [7u8; KEY_LEN];
        let ct = encrypt(&key, "s3cr3t").unwrap();
        assert_ne!(ct, "s3cr3t");
        assert_eq!(decrypt(&key, &ct).unwrap(), "s3cr3t");
    }

    #[test]
    fn test_empty_password_roundtrip() {
        let key = [7u8; KEY_LEN];
        assert_eq!(encrypt(&key, "").unwrap(), "");
        assert_eq!(decrypt(&key, "").unwrap(), "");
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = [7u8; KEY_LEN];
        let ct = encrypt(&key, "hello").unwrap();
        let mut bad = ct.clone();
        // 篡改最后一个字符
        bad.pop();
        bad.push(if ct.ends_with('A') { 'B' } else { 'A' });
        assert!(matches!(
            decrypt(&key, &bad),
            Err(DataSourceError::DecryptFailed) | Err(DataSourceError::Base64)
        ));
    }

    #[test]
    fn test_wrong_key_fails() {
        let ct = encrypt(&[1u8; KEY_LEN], "hello").unwrap();
        assert!(matches!(
            decrypt(&[2u8; KEY_LEN], &ct),
            Err(DataSourceError::DecryptFailed)
        ));
    }

    #[test]
    fn test_file_roundtrip_and_key_reuse() {
        let dir = tmp_dir();
        let project_path = dir.join("test.aqua");
        let project_path_str = project_path.to_str().unwrap();
        let key_path = dir.join("key");
        let key_str = key_path.to_str().unwrap();

        // key 不存在 → save 自动生成
        assert!(!key_path.exists());
        save_db_config(
            project_path_str,
            key_str,
            vec![sample("dev", "pwd_dev"), sample("prod", "pwd_prod")],
        )
        .unwrap();
        assert!(key_path.exists());

        // 文件中 password 为密文
        let config_file = config_path_for_project(project_path_str).unwrap();
        let raw = std::fs::read_to_string(&config_file).unwrap();
        assert!(!raw.contains("pwd_dev"));
        assert!(!raw.contains("pwd_prod"));

        // load 还原明文,且按 name 排序
        let loaded = load_db_config(project_path_str, key_str).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].0, "dev");
        assert_eq!(loaded[0].1.password, "pwd_dev");
        assert_eq!(loaded[1].0, "prod");
        assert_eq!(loaded[1].1.password, "pwd_prod");

        // 二次 save 复用同 key(未报 BadKey)
        save_db_config(project_path_str, key_str, vec![sample("dev", "pwd2")]).unwrap();
        let reloaded = load_db_config(project_path_str, key_str).unwrap();
        assert_eq!(reloaded[0].1.password, "pwd2");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_load_missing_file_returns_empty() {
        let dir = tmp_dir();
        let project_path = dir.join("missing.aqua");
        let out = load_db_config(project_path.to_str().unwrap(), dir.join("key").to_str().unwrap()).unwrap();
        assert!(out.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_bad_key_length() {
        let dir = tmp_dir();
        let key_path = dir.join("key");
        std::fs::write(&key_path, b"tooshort").unwrap();
        let err = load_or_create_key(key_path.to_str().unwrap()).unwrap_err();
        assert!(matches!(err, DataSourceError::BadKey));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// wire 契约:前端传 Array<[name, cfg]>,Rust 端 Vec<(String, DataSourceConfig)>
    /// 必须能从 JSON 二元组数组反序列化,且序列化回同样形态。
    #[test]
    fn test_wire_tuple_roundtrip() {
        let src = vec![sample("dev", "p1"), sample("prod", "p2")];
        let json = serde_json::to_value(&src).unwrap();
        // 确认序列化为 [[name, cfg], ...] 形态(与前端 Array<[string, DbConfig]> 一致)
        assert!(json.is_array());
        assert!(json[0].is_array() && json[0].as_array().unwrap().len() == 2);
        assert_eq!(json[0][0], "dev");

        // 反序列化回来
        let back: Vec<(String, DataSourceConfig)> = serde_json::from_value(json).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].0, "dev");
        assert_eq!(back[0].1.database, "mydb");
    }
}
