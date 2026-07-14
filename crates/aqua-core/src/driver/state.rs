//! 数据库支持状态持久化(drivers/databases.json)。
//!
//! 应用级:记录用户隐藏了哪些数据库、装了哪些 JDBC 驱动。
//! 与项目级 `.dbconfig.json`(数据源连接)分开。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::dialects::{find_dialect, DbCategory, ALL_DATABASES};

/// 已安装的 JDBC 驱动记录。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledDriver {
    pub dialect: String,
    pub driver_jar: String,
    pub driver_class: String,
}

/// `databases.json` 结构。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseState {
    #[serde(default)]
    pub hidden: Vec<String>,
    #[serde(default)]
    pub installed: Vec<InstalledDriver>,
}

/// 前端用的数据库信息(清单 + 状态合并)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub name: String,
    pub label: String,
    pub category: String,
    pub default_port: u16,
    pub needs_schema: bool,
    pub generate_as: Option<String>,
    pub driver_class: Option<String>,
    pub reverse_supported: bool,
    pub builtin_driver: bool,
    pub hidden: bool,
    pub installed: bool,
    pub installed_jar: Option<String>,
}

fn state_path(dir: &Path) -> PathBuf {
    dir.join("databases.json")
}

/// 读 `databases.json`,不存在返回默认空状态。
pub fn load_state(dir: &Path) -> DatabaseState {
    match std::fs::read(state_path(dir)) {
        Ok(data) => serde_json::from_slice(&data).unwrap_or_default(),
        Err(_) => DatabaseState::default(),
    }
}

/// 写 `databases.json`。
pub fn save_state(dir: &Path, state: &DatabaseState) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("创建 drivers 目录失败: {}", e))?;
    let json = serde_json::to_string_pretty(state).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(state_path(dir), json).map_err(|e| format!("写 databases.json 失败: {}", e))
}

/// 合并清单 + 状态,返回前端用的完整列表。
pub fn list_databases_with_state(dir: &Path) -> Vec<DatabaseInfo> {
    let state = load_state(dir);
    ALL_DATABASES
        .iter()
        .map(|d| {
            let installed = state.installed.iter().find(|i| i.dialect == d.name);
            DatabaseInfo {
                name: d.name.to_string(),
                label: d.label.to_string(),
                category: match d.category {
                    DbCategory::Native => "native",
                    DbCategory::Jdbc => "jdbc",
                }
                .to_string(),
                default_port: d.default_port,
                needs_schema: d.needs_schema,
                generate_as: d.generate_as.map(|s| s.to_string()),
                driver_class: d.driver_class.map(|s| s.to_string()),
                reverse_supported: d.reverse_supported,
                builtin_driver: d.builtin_driver,
                hidden: state.hidden.contains(&d.name.to_string()),
                installed: installed.is_some(),
                installed_jar: installed.map(|i| i.driver_jar.clone()),
            }
        })
        .collect()
}

/// 安装 JDBC 驱动:copy jar 到 drivers/,写 installed 记录。
///
/// 仅 `reverse_supported && !builtin_driver` 的数据库可装(本任务即 Oracle)。
pub fn install_driver(dir: &Path, dialect: &str, jar_path: &str) -> Result<(), String> {
    let info = find_dialect(dialect).ok_or(format!("未知数据库: {}", dialect))?;
    if !info.reverse_supported {
        return Err(format!("{} 暂不支持反解,无法安装驱动", dialect));
    }
    if info.builtin_driver {
        return Err(format!("{} 驱动内置,无需安装", dialect));
    }
    let src = Path::new(jar_path);
    let jar_name = src
        .file_name()
        .ok_or("jar 路径无效")?
        .to_string_lossy()
        .to_string();
    std::fs::create_dir_all(dir).map_err(|e| format!("创建 drivers 目录失败: {}", e))?;
    std::fs::copy(src, dir.join(&jar_name)).map_err(|e| format!("复制 jar 失败: {}", e))?;
    let mut state = load_state(dir);
    state.installed.retain(|i| i.dialect != dialect);
    state.installed.push(InstalledDriver {
        dialect: dialect.to_string(),
        driver_jar: jar_name,
        driver_class: info.driver_class.unwrap_or("").to_string(),
    });
    save_state(dir, &state)
}

/// 卸载驱动:删 jar + 删 installed 记录。未安装则无操作。
pub fn uninstall_driver(dir: &Path, dialect: &str) -> Result<(), String> {
    let mut state = load_state(dir);
    if let Some(idx) = state.installed.iter().position(|i| i.dialect == dialect) {
        let jar = state.installed[idx].driver_jar.clone();
        let _ = std::fs::remove_file(dir.join(&jar));
        state.installed.remove(idx);
        save_state(dir, &state)
    } else {
        Ok(())
    }
}

/// 设置数据库显示/隐藏。
pub fn set_hidden(dir: &Path, dialect: &str, hidden: bool) -> Result<(), String> {
    let mut state = load_state(dir);
    state.hidden.retain(|h| h != dialect);
    if hidden {
        state.hidden.push(dialect.to_string());
    }
    save_state(dir, &state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aqua_db_test_{}", nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_state_roundtrip() {
        let dir = tmp_dir();
        let state = DatabaseState {
            hidden: vec!["gbase".into()],
            installed: vec![InstalledDriver {
                dialect: "oracle".into(),
                driver_jar: "ojdbc8.jar".into(),
                driver_class: "oracle.jdbc.OracleDriver".into(),
            }],
        };
        save_state(&dir, &state).unwrap();
        let loaded = load_state(&dir);
        assert_eq!(loaded.hidden, vec!["gbase".to_string()]);
        assert_eq!(loaded.installed.len(), 1);
        assert_eq!(loaded.installed[0].dialect, "oracle");
        // 验证 camelCase 序列化
        let raw = std::fs::read_to_string(state_path(&dir)).unwrap();
        assert!(raw.contains("driverJar"));
        assert!(raw.contains("driverClass"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_load_missing_returns_default() {
        let dir = tmp_dir();
        let state = load_state(&dir);
        assert!(state.hidden.is_empty());
        assert!(state.installed.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_list_databases_with_state() {
        let dir = tmp_dir();
        let state = DatabaseState {
            hidden: vec!["gbase".into()],
            installed: vec![InstalledDriver {
                dialect: "oracle".into(),
                driver_jar: "ojdbc8.jar".into(),
                driver_class: "oracle.jdbc.OracleDriver".into(),
            }],
        };
        save_state(&dir, &state).unwrap();
        let list = list_databases_with_state(&dir);
        assert_eq!(list.len(), 11);
        let oracle = list.iter().find(|d| d.name == "oracle").unwrap();
        assert!(oracle.installed);
        assert_eq!(oracle.installed_jar.as_deref(), Some("ojdbc8.jar"));
        assert!(!oracle.hidden);
        let gbase = list.iter().find(|d| d.name == "gbase").unwrap();
        assert!(gbase.hidden);
        assert!(!gbase.installed);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_set_hidden_toggle() {
        let dir = tmp_dir();
        set_hidden(&dir, "tidb", true).unwrap();
        assert!(load_state(&dir).hidden.contains(&"tidb".to_string()));
        set_hidden(&dir, "tidb", false).unwrap();
        assert!(!load_state(&dir).hidden.contains(&"tidb".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_install_driver_rejects_unsupported() {
        let dir = tmp_dir();
        // dm 不支持反解
        let err = install_driver(&dir, "dm", "/tmp/x.jar").unwrap_err();
        assert!(err.contains("暂不支持反解"));
        // mysql 内置
        let err = install_driver(&dir, "mysql", "/tmp/x.jar").unwrap_err();
        assert!(err.contains("内置"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_install_and_uninstall_driver() {
        let dir = tmp_dir();
        let jar = dir.join("fake_ojdbc.jar");
        std::fs::write(&jar, b"fake").unwrap();
        let drivers = dir.join("drivers");
        install_driver(&drivers, "oracle", jar.to_str().unwrap()).unwrap();
        let state = load_state(&drivers);
        assert_eq!(state.installed.len(), 1);
        assert_eq!(state.installed[0].driver_jar, "fake_ojdbc.jar");
        assert!(drivers.join("fake_ojdbc.jar").exists());

        // 重复安装覆盖(不累计)
        install_driver(&drivers, "oracle", jar.to_str().unwrap()).unwrap();
        assert_eq!(load_state(&drivers).installed.len(), 1);

        // 卸载
        uninstall_driver(&drivers, "oracle").unwrap();
        assert!(load_state(&drivers).installed.is_empty());
        assert!(!drivers.join("fake_ojdbc.jar").exists());

        std::fs::remove_dir_all(&dir).ok();
    }
}
