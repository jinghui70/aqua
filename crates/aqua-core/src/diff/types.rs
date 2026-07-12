//! diff 引擎类型定义。

use serde::Serialize;

/// diff 结果(对齐 design.md §4.3 DiffResult)。
#[derive(Debug, Clone, Serialize)]
pub struct DiffResult {
    pub tables: TableDiff,
    /// bizType code 变更(added + removed)
    pub biz_types: Vec<String>,
    /// enum code 变更(added + removed)
    pub enums: Vec<String>,
}

/// 表级 diff。
#[derive(Debug, Clone, Serialize)]
pub struct TableDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<TableChange>,
}

/// 表变更详情。
#[derive(Debug, Clone, Serialize)]
pub struct TableChange {
    pub table: String,
    pub fields: FieldDiff,
    pub indexes: IndexDiff,
}

/// 字段级 diff。
#[derive(Debug, Clone, Serialize)]
pub struct FieldDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<FieldChange>,
}

/// 字段变更(记录变更的属性名)。
#[derive(Debug, Clone, Serialize)]
pub struct FieldChange {
    pub field: String,
    pub changes: Vec<String>,
}

/// 索引级 diff。
#[derive(Debug, Clone, Serialize)]
pub struct IndexDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

impl FieldDiff {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.changed.is_empty()
    }
}

impl IndexDiff {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty()
    }
}
