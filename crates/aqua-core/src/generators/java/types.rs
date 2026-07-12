//! Java 类型映射与选项。

use crate::schema::DataType;

/// Java 生成选项。
#[derive(Debug, Clone)]
pub struct JavaOptions {
    /// 是否使用 Lombok @Data 注解(默认 true)。
    pub use_lombok: bool,
    /// 自定义包名(为空则用默认规则: {basePackage}.{group}.entity)。
    pub package: Option<String>,
    /// 自定义类名(为空则从 table.code 派生 PascalCase)。
    pub class_name: Option<String>,
}

impl Default for JavaOptions {
    fn default() -> Self {
        Self {
            use_lombok: true,
            package: None,
            class_name: None,
        }
    }
}

/// 逻辑类型映射为 Java 类型。
pub fn map_java_type(data_type: DataType) -> &'static str {
    match data_type {
        DataType::Varchar => "String",
        DataType::Clob => "String",
        DataType::Tinyint => "Integer",
        DataType::Int => "Integer",
        DataType::Long => "Long",
        DataType::Decimal => "BigDecimal",
        DataType::Date => "LocalDate",
        DataType::Datetime => "LocalDateTime",
        DataType::Blob => "byte[]",
    }
}

/// 获取需要 import 的类型(非 java.lang)。
pub fn get_java_import(data_type: DataType) -> Option<&'static str> {
    match data_type {
        DataType::Decimal => Some("java.math.BigDecimal"),
        DataType::Date => Some("java.time.LocalDate"),
        DataType::Datetime => Some("java.time.LocalDateTime"),
        _ => None,
    }
}
