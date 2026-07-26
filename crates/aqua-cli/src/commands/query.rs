//! 查询命令:groups / tables / show —— 省 token 地读结构,输出可读文本。

use crate::load::load;
use anyhow::{anyhow, Result};
use aqua_core::schema::{Field, Table};

/// 列出所有表组。
pub fn groups(file: &str) -> Result<()> {
    let project = load(file)?;
    for g in &project.groups {
        println!("{}\t{}", g.code, g.name);
    }
    Ok(())
}

/// 列出表(可按组过滤)。
pub fn tables(file: &str, group: Option<&str>) -> Result<()> {
    let project = load(file)?;
    for t in &project.tables {
        if let Some(g) = group {
            if t.group != g {
                continue;
            }
        }
        println!("{}\t{}\t({})", t.code, t.name, t.group);
    }
    Ok(())
}

/// 显示单表结构(字段 + 索引)。
pub fn show(file: &str, table_code: &str) -> Result<()> {
    let project = load(file)?;
    let table = project
        .tables
        .iter()
        .find(|t| t.code == table_code)
        .ok_or_else(|| anyhow!("表不存在: {table_code}"))?;
    print_table(table);
    Ok(())
}

fn print_table(t: &Table) {
    println!("Table: {} ({})  group={}", t.code, t.name, t.group);
    if let Some(c) = &t.comment {
        if !c.is_empty() {
            println!("  comment: {c}");
        }
    }
    println!("Fields:");
    for f in &t.fields {
        let key = if f.is_key.unwrap_or(false) { "PK" } else { "" };
        let nn = if f.not_null.unwrap_or(false) {
            "notNull"
        } else {
            ""
        };
        let biz = f.biz_type.as_deref().unwrap_or("-");
        println!(
            "  {:<22} {:<24} {:<14} {:<3} {:<8} bizType={}",
            f.code,
            f.name,
            fmt_type(f),
            key,
            nn,
            biz
        );
    }
    match &t.indexes {
        Some(idxs) if !idxs.is_empty() => {
            println!("Indexes:");
            for idx in idxs {
                let name = idx.name.as_deref().unwrap_or("(unnamed)");
                let cols: Vec<String> = idx
                    .fields
                    .iter()
                    .map(|f| format!("{} {}", f.code, f.direction.as_str()))
                    .collect();
                let uniq = if idx.unique { "[unique] " } else { "" };
                println!("  {uniq}{name}: {}", cols.join(", "));
            }
        }
        _ => {}
    }
}

/// 逻辑类型显示为 UPPERCASE(对齐 .aqua 里的原值),带长度/精度。
fn fmt_type(f: &Field) -> String {
    let base = format!("{:?}", f.data_type).to_uppercase();
    if let Some(l) = f.length {
        return format!("{base}({l})");
    }
    match (f.precision, f.scale) {
        (Some(p), Some(s)) => format!("{base}({p},{s})"),
        (Some(p), None) => format!("{base}({p})"),
        _ => base,
    }
}
