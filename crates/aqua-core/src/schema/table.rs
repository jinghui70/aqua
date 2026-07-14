//! §3.3 表模型(Table)与索引(Index)。

use crate::schema::field::Field;
use serde::{Deserialize, Deserializer, Serialize};

/// §3.3 索引字段排序方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    Asc,
    Desc,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::Asc => "ASC",
            Direction::Desc => "DESC",
        }
    }
}

fn default_direction() -> Direction {
    Direction::Asc
}

/// §3.3 索引字段(code + 排序方向)。
/// 序列化为 `{code, direction}`;反序列化兼容旧格式纯字符串(默认 ASC)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexField {
    pub code: String,
    pub direction: Direction,
}

impl<'de> Deserialize<'de> for IndexField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Str(String),
            Obj {
                code: String,
                #[serde(default = "default_direction")]
                direction: Direction,
            },
        }
        match Raw::deserialize(deserializer)? {
            Raw::Str(code) => Ok(IndexField {
                code,
                direction: Direction::Asc,
            }),
            Raw::Obj { code, direction } => Ok(IndexField { code, direction }),
        }
    }
}

/// §3.3 索引 Index。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub fields: Vec<IndexField>,
    pub unique: bool,
}

/// §3.3 表模型 Table。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub code: String,
    pub name: String,
    pub group: String,
    pub fields: Vec<Field>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexes: Option<Vec<Index>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 旧格式 fields 为纯字符串数组,反序列化默认 ASC。
    #[test]
    fn index_field_legacy_string_defaults_asc() {
        let json = r#"[{"fields":["a","b"],"unique":false}]"#;
        let idx: Vec<Index> = serde_json::from_str(json).unwrap();
        assert_eq!(idx[0].fields.len(), 2);
        assert_eq!(idx[0].fields[0].code, "a");
        assert_eq!(idx[0].fields[0].direction, Direction::Asc);
        assert_eq!(idx[0].fields[1].code, "b");
        assert_eq!(idx[0].fields[1].direction, Direction::Asc);
    }

    /// 新格式 {code,direction} 保留方向。
    #[test]
    fn index_field_new_format_keeps_direction() {
        let json = r#"[{"fields":[{"code":"a","direction":"DESC"}],"unique":false}]"#;
        let idx: Vec<Index> = serde_json::from_str(json).unwrap();
        assert_eq!(idx[0].fields[0].code, "a");
        assert_eq!(idx[0].fields[0].direction, Direction::Desc);
    }

    /// 序列化输出 {code,direction}。
    #[test]
    fn index_field_serializes_with_direction() {
        let idx = Index {
            name: None,
            fields: vec![IndexField {
                code: "a".to_string(),
                direction: Direction::Desc,
            }],
            unique: false,
        };
        let json = serde_json::to_string(&idx).unwrap();
        assert!(json.contains(r#""code":"a""#), "{json}");
        assert!(json.contains(r#""direction":"DESC""#), "{json}");
    }
}
