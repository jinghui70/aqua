//! 命名转换: snake_case ↔ camelCase/PascalCase。

/// 蛇形 → 驼峰 (USER_NAME → userName)。
pub fn snake_to_camel(code: &str) -> String {
    let parts: Vec<&str> = code.split('_').filter(|p| !p.is_empty()).collect();

    if parts.is_empty() {
        return String::new();
    }

    let mut result = parts[0].to_lowercase();
    for part in &parts[1..] {
        if !part.is_empty() {
            result.push_str(&capitalize_first(part));
        }
    }
    result
}

/// 蛇形 → PascalCase (USER_INFO → UserInfo)。
pub fn snake_to_pascal(code: &str) -> String {
    code.split('_')
        .filter(|p| !p.is_empty())
        .map(capitalize_first)
        .collect::<String>()
}

/// 驼峰 prop → PascalCase (gender → Gender, userName → UserName)。
/// 枚举类名派生源为字段 prop(驼峰),语义是"首字母大写",保留其余大小写。
pub fn prop_to_pascal(prop: &str) -> String {
    if prop.is_empty() {
        return String::new();
    }
    let first = prop[..1].to_uppercase();
    first + &prop[1..]
}

/// 首字母大写,其余小写。
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut result = first.to_uppercase().to_string();
            result.push_str(&chars.as_str().to_lowercase());
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("USER_NAME"), "userName");
        assert_eq!(snake_to_camel("ID"), "id");
        assert_eq!(snake_to_camel("USER_ID"), "userId");
    }

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("USER_INFO"), "UserInfo");
        assert_eq!(snake_to_pascal("SYS_USER"), "SysUser");
        assert_eq!(snake_to_pascal("ID"), "Id");
    }

    #[test]
    fn test_prop_to_pascal() {
        assert_eq!(prop_to_pascal("gender"), "Gender");
        assert_eq!(prop_to_pascal("userName"), "UserName");
        assert_eq!(prop_to_pascal("status"), "Status");
        assert_eq!(prop_to_pascal(""), "");
    }
}
