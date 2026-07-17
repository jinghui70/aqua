//! 数据库配置与元数据类型。

use crate::schema::DataType;
use serde::{Deserialize, Serialize};

/// 数据库连接配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// 数据库方言: "mysql" | "postgresql" | "oracle" | "dm" | ...
    pub dialect: String,
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 用户名
    pub user: String,
    /// 密码
    pub password: String,
    /// 数据库名(MySQL)/schema 名(Oracle/PG)
    pub database: String,
    /// schema 名(可选,部分数据库需要)
    pub schema: Option<String>,
}

/// 列元数据(反解结果)。
#[derive(Debug, Clone)]
pub struct ColumnMeta {
    /// 列名
    pub name: String,
    /// aqua 逻辑类型(反解后的类型,不是物理类型)
    pub data_type: DataType,
    /// 长度(VARCHAR 用)
    pub length: Option<u32>,
    /// 精度(DECIMAL 用)
    pub precision: Option<u32>,
    /// 小数位数(DECIMAL 用)
    pub scale: Option<u32>,
    /// 是否可空
    pub nullable: bool,
    /// 是否主键
    pub is_key: bool,
    /// 默认值(字符串形式)
    pub default_value: Option<String>,
    /// 注释
    pub comment: Option<String>,
}

/// 表信息(表名 + 注释,listTables 返回,选表显示 + import 复用,避免再 spawn 取注释)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// 表名
    pub name: String,
    /// 表注释
    pub comment: Option<String>,
}

/// 索引元数据(反解结果)。
#[derive(Debug, Clone)]
pub struct IndexMeta {
    /// 索引名
    pub name: String,
    /// 字段列表(列名数组)
    pub fields: Vec<String>,
    /// 是否唯一索引
    pub unique: bool,
}
