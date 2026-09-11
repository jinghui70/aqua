//! 版本兼容性检查。

use std::cmp::Ordering;

/// aqua 产品当前版本(与 workspace Cargo.toml 保持同步,release 时更新)。
pub const AQUA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 版本兼容性检查结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionCheck {
    /// 文件版本 <= 当前版本,兼容可打开
    Compatible,
    /// 文件版本 > 当前版本,需要升级 app
    NeedUpgrade { file_version: String },
    /// 文件版本 < 当前版本,可打开但会在保存时升级
    CanOpen { file_version: String },
}

/// 检查文件版本与当前 app 版本的兼容性。
pub fn check_version_compatibility(file_version: &str) -> VersionCheck {
    match compare_version(file_version, AQUA_VERSION) {
        Ordering::Greater => VersionCheck::NeedUpgrade {
            file_version: file_version.to_string(),
        },
        Ordering::Equal => VersionCheck::Compatible,
        Ordering::Less => VersionCheck::CanOpen {
            file_version: file_version.to_string(),
        },
    }
}

/// 简单的 semver 比较(major.minor.patch),忽略 pre-release/build metadata。
fn compare_version(a: &str, b: &str) -> Ordering {
    let parse = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = s.split('.').collect();
        let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };
    parse(a).cmp(&parse(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_version() {
        assert_eq!(compare_version("1.0.0", "1.0.0"), Ordering::Equal);
        assert_eq!(compare_version("1.0.1", "1.0.0"), Ordering::Greater);
        assert_eq!(compare_version("1.0.0", "1.0.1"), Ordering::Less);
        assert_eq!(compare_version("1.1.0", "1.0.9"), Ordering::Greater);
        assert_eq!(compare_version("2.0.0", "1.9.9"), Ordering::Greater);
    }

    #[test]
    fn test_check_compatibility() {
        // 假设当前版本是 AQUA_VERSION(从 Cargo.toml 读取,测试时为 workspace version)
        let current = AQUA_VERSION;

        // 相同版本
        assert_eq!(
            check_version_compatibility(current),
            VersionCheck::Compatible
        );

        // 低版本文件
        let result = check_version_compatibility("1.0.0");
        assert!(matches!(result, VersionCheck::CanOpen { .. }));

        // 高版本文件(假设一个未来版本)
        let result = check_version_compatibility("99.0.0");
        assert!(matches!(result, VersionCheck::NeedUpgrade { .. }));
    }
}
