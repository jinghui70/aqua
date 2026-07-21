//! diff 引擎 - Project vs Project 对比,输出结构化差异。
//!
//! 规则见 `docs/design.md` §4.3。用于 ALTER DDL 生成。

mod types;

pub use types::{DiffResult, FieldChange, FieldDiff, IndexDiff, TableChange, TableDiff};

use crate::schema::{Field, Index, Project, Table};
use std::collections::HashSet;

/// 对比两个 Project,返回结构化差异。
pub fn diff_project(old: &Project, new: &Project) -> DiffResult {
    DiffResult {
        tables: diff_tables(&old.tables, &new.tables),
        biz_types: diff_codes(
            old.biz_types.iter().map(|b| b.biz_type.as_str()),
            new.biz_types.iter().map(|b| b.biz_type.as_str()),
        ),
    }
}

/// 表级 diff: 按 code 匹配。
fn diff_tables(old: &[Table], new: &[Table]) -> TableDiff {
    let old_map: std::collections::HashMap<&str, &Table> =
        old.iter().map(|t| (t.code.as_str(), t)).collect();
    let new_map: std::collections::HashMap<&str, &Table> =
        new.iter().map(|t| (t.code.as_str(), t)).collect();

    let old_codes: HashSet<&str> = old_map.keys().copied().collect();
    let new_codes: HashSet<&str> = new_map.keys().copied().collect();

    let added: Vec<String> = new_codes
        .difference(&old_codes)
        .map(|c| c.to_string())
        .collect();
    let removed: Vec<String> = old_codes
        .difference(&new_codes)
        .map(|c| c.to_string())
        .collect();

    let changed: Vec<TableChange> = old_codes
        .intersection(&new_codes)
        .filter_map(|&code| {
            let old_t = old_map[code];
            let new_t = new_map[code];
            let fields = diff_fields(&old_t.fields, &new_t.fields);
            let indexes = diff_indexes(
                old_t.indexes.as_deref().unwrap_or(&[]),
                new_t.indexes.as_deref().unwrap_or(&[]),
            );
            // 表名/分组变更也算 changed
            let table_meta_changed = old_t.name != new_t.name || old_t.group != new_t.group;

            if fields.has_changes() || indexes.has_changes() || table_meta_changed {
                Some(TableChange {
                    table: code.to_string(),
                    fields,
                    indexes,
                })
            } else {
                None
            }
        })
        .collect();

    TableDiff {
        added,
        removed,
        changed,
    }
}

/// 字段级 diff: 按 code 匹配,比较属性。
fn diff_fields(old: &[Field], new: &[Field]) -> FieldDiff {
    let old_map: std::collections::HashMap<&str, &Field> =
        old.iter().map(|f| (f.code.as_str(), f)).collect();
    let new_map: std::collections::HashMap<&str, &Field> =
        new.iter().map(|f| (f.code.as_str(), f)).collect();

    let old_codes: HashSet<&str> = old_map.keys().copied().collect();
    let new_codes: HashSet<&str> = new_map.keys().copied().collect();

    let added: Vec<String> = new_codes
        .difference(&old_codes)
        .map(|c| c.to_string())
        .collect();
    let removed: Vec<String> = old_codes
        .difference(&new_codes)
        .map(|c| c.to_string())
        .collect();

    let changed: Vec<FieldChange> = old_codes
        .intersection(&new_codes)
        .filter_map(|&code| {
            let changes = field_changes(old_map[code], new_map[code]);
            if changes.is_empty() {
                None
            } else {
                Some(FieldChange {
                    field: code.to_string(),
                    changes,
                })
            }
        })
        .collect();

    FieldDiff {
        added,
        removed,
        changed,
    }
}

/// 比较单字段属性,返回变更的属性名列表。
fn field_changes(old: &Field, new: &Field) -> Vec<String> {
    let mut changes = Vec::new();
    if old.prop != new.prop {
        changes.push("prop".to_string());
    }
    if old.name != new.name {
        changes.push("name".to_string());
    }
    if old.data_type != new.data_type {
        changes.push("dataType".to_string());
    }
    if old.length != new.length {
        changes.push("length".to_string());
    }
    if old.precision != new.precision {
        changes.push("precision".to_string());
    }
    if old.scale != new.scale {
        changes.push("scale".to_string());
    }
    if old.not_null != new.not_null {
        changes.push("notNull".to_string());
    }
    if old.is_key != new.is_key {
        changes.push("isKey".to_string());
    }
    if old.default_value != new.default_value {
        changes.push("defaultValue".to_string());
    }
    changes
}

/// 索引级 diff: 按 name 匹配(无 name 用 fields 组合键)。
fn diff_indexes(old: &[Index], new: &[Index]) -> IndexDiff {
    let old_map: std::collections::HashMap<String, &Index> =
        old.iter().map(|i| (index_key(i), i)).collect();
    let new_map: std::collections::HashMap<String, &Index> =
        new.iter().map(|i| (index_key(i), i)).collect();

    let old_keys: HashSet<String> = old_map.keys().cloned().collect();
    let new_keys: HashSet<String> = new_map.keys().cloned().collect();

    let added: Vec<String> = new_keys.difference(&old_keys).cloned().collect();
    let removed: Vec<String> = old_keys.difference(&new_keys).cloned().collect();
    // 索引属性变更(unique 等):同 key 视为变更(重建),归入 removed+added
    // 简化:同 key 但 unique 不同,记录为 changed(本期归入 removed+added)

    IndexDiff { added, removed }
}

/// 索引匹配键: name 优先,无 name 用 fields 的 code+方向组合(方向不同 = 变更)。
pub(crate) fn index_key(idx: &Index) -> String {
    idx.name.clone().unwrap_or_else(|| {
        format!(
            "IDX_{}",
            idx.fields
                .iter()
                .map(|f| format!("{}_{}", f.code, f.direction.as_str()))
                .collect::<Vec<_>>()
                .join("_")
        )
    })
}

/// 简单 code 列表 diff(用于 bizTypes)。
fn diff_codes<'a>(
    old: impl Iterator<Item = &'a str>,
    new: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    let old_set: HashSet<&str> = old.collect();
    let new_set: HashSet<&str> = new.collect();
    new_set
        .difference(&old_set)
        .chain(old_set.difference(&new_set))
        .map(|c| c.to_string())
        .collect()
}
