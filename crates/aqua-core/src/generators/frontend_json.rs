//! 前端 JSON 生成器 - 生成 json-ui 兼容格式(服务于外部 json-ui 项目)。
//!
//! 移植自 `~/work/aqua-legacy/packages/core/src/generators/frontend-json/`。
//! 规则见 `docs/design.md` §4.2.2。

use crate::schema::{DataType, EnumDefine, Field, Project, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// json-ui 粗粒度数据类型(4 种)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum JsonUiDataType {
    Number,
    String,
    Date,
    Datetime,
}

/// 10 逻辑类型 -> json-ui 4 粗粒度类型。
pub fn map_data_type(dt: DataType) -> JsonUiDataType {
    match dt {
        DataType::Int
        | DataType::Long
        | DataType::Decimal
        | DataType::Double
        | DataType::Tinyint => JsonUiDataType::Number,
        DataType::Varchar | DataType::Clob | DataType::Blob => JsonUiDataType::String,
        DataType::Date => JsonUiDataType::Date,
        DataType::Datetime => JsonUiDataType::Datetime,
    }
}

/// json-ui Field(排除 precision/comment)。
///
/// 字段声明顺序即序列化顺序:code/prop/name 靠前,bizType/bizTypeData 靠后。
/// 注意:序列化必须直接走 struct(见 generate_frontend_json),不能经 serde_json::Value 中转,
/// 否则 Value::Object 的 BTreeMap 会把键重排成字母序。
#[derive(Debug, Clone, Serialize)]
pub struct JsonUiField {
    pub code: String,
    pub prop: String,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: JsonUiDataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
    #[serde(rename = "isKey")]
    pub is_key: bool,
    #[serde(rename = "notNull")]
    pub not_null: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "autoGenerate")]
    pub auto_generate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizType")]
    pub biz_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizTypeData")]
    pub biz_type_data: Option<serde_json::Value>,
}

/// json-ui DataModel(JsonModelSchema:type/code/name/fields)。
/// 对齐 json-ui `src/schema/json-file.ts` 的 JsonModelSchema(存为 *.model.json)。
#[derive(Debug, Clone, Serialize)]
pub struct JsonUiModel {
    /// 固定 "model",json-ui FileEngine 据此识别模型文件。
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub code: String,
    pub name: String,
    pub fields: Vec<JsonUiField>,
}

/// 前端 JSON 生成选项。
#[derive(Debug, Clone, Default)]
pub struct FrontendJsonOptions {
    /// 单表过滤(为空则取首表;json-ui model 是单表概念)
    pub table: Option<String>,
}

/// 按数据类型规范前端 JSON 的 length/scale 输出(§3.1 + §4.2.2)。
///
/// Field 上的 length/scale 可能为脏值(反解残留/编辑未清),前端 JSON 不原样透传,
/// 而是按逻辑类型决定输出:
/// - VARCHAR: length(原值), 无 scale
/// - TINYINT/INT/LONG: 无 length, scale=0(整数,显式告知 json-ui 无小数位)
/// - DECIMAL: 无 length, scale(原值); precision 不输出(§4.2.2)
/// - DOUBLE: 均无(§3.1 不允许 precision/scale,IEEE 754 浮点无小数位概念)
/// - CLOB/BLOB/DATE/DATETIME: 均无
fn normalize_length_scale(
    dt: DataType,
    length: Option<u32>,
    scale: Option<u32>,
) -> (Option<u32>, Option<u32>) {
    match dt {
        DataType::Varchar => (length, None),
        DataType::Tinyint | DataType::Int | DataType::Long => (None, Some(0)),
        DataType::Decimal => (None, scale),
        DataType::Double
        | DataType::Clob
        | DataType::Blob
        | DataType::Date
        | DataType::Datetime => (None, None),
    }
}

/// Field -> JsonUiField 转换(排除 precision/comment)。
/// defs 为全项目枚举定义索引;枚举字段(定义或引用)输出 bizType="Options" + bizTypeData=[{id,name}]。
pub fn transform_field(field: &Field, defs: &HashMap<(String, String), EnumDefine>) -> JsonUiField {
    let (length, scale) = normalize_length_scale(field.data_type, field.length, field.scale);

    // 枚举字段 -> Options + bizTypeData(引用方解析到定义方取值)
    let (biz_type, biz_type_data) = enum_to_options(field, defs)
        .map(|vals| (Some("Options".to_string()), Some(serde_json::json!(vals))))
        .unwrap_or((field.biz_type.clone(), field.biz_type_data.clone()));

    JsonUiField {
        prop: field.prop.clone(),
        code: field.code.clone(),
        name: field.name.clone(),
        data_type: map_data_type(field.data_type),
        length,
        scale,
        biz_type,
        biz_type_data,
        is_key: field.is_key.unwrap_or(false),
        not_null: field.not_null.unwrap_or(false),
        auto_generate: field.auto_generate.is_some().then_some(true),
    }
}

/// 枚举字段的 Options 值列表[{id,name,color?}](定义方 or 引用方解析);非枚举返回 None。
/// color 仅在定义了时输出;code 不输出(仅 Java 用)。
fn enum_to_options(
    field: &Field,
    defs: &HashMap<(String, String), EnumDefine>,
) -> Option<Vec<serde_json::Value>> {
    let e = field.enum_ref.as_ref()?;
    let values = if let Some(r) = &e.r#ref {
        // 引用方:解析到定义方取值
        &defs.get(&(r.code.clone(), r.prop.clone()))?.r#enum.values
    } else {
        // 定义方:直接取本地 values
        &e.values
    };
    Some(
        values
            .iter()
            .map(|v| match &v.color {
                Some(c) => serde_json::json!({ "id": v.id, "name": v.name, "color": c }),
                None => serde_json::json!({ "id": v.id, "name": v.name }),
            })
            .collect(),
    )
}

/// Table -> JsonUiModel 转换(JsonModelSchema,type 固定 "model")。
pub fn transform_table(table: &Table, defs: &HashMap<(String, String), EnumDefine>) -> JsonUiModel {
    JsonUiModel {
        type_: "model",
        code: table.code.clone(),
        name: table.name.clone(),
        fields: table
            .fields
            .iter()
            .map(|f| transform_field(f, defs))
            .collect(),
    }
}

/// 前端 JSON 生成入口,返回单表 json-ui JsonModelSchema 文本。
/// options.table 指定表(为空取首表);model 是单表概念,不再包裹 tables 数组。
pub fn generate_frontend_json(project: &Project, options: &FrontendJsonOptions) -> String {
    let table: &Table = if let Some(ref table_code) = options.table {
        project
            .tables
            .iter()
            .find(|t| t.code == *table_code)
            .unwrap_or_else(|| panic!("Table not found: {}", table_code))
    } else {
        project.tables.first().expect("项目无表,无法生成 model")
    };

    let defs = project.enum_defs();
    // 直接序列化 struct 保持字段顺序;经 serde_json::Value 会被 BTreeMap 重排成字母序
    serde_json::to_string_pretty(&transform_table(table, &defs)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{AutoGenerate, GenerationTiming};
    use std::collections::HashMap;

    /// 空枚举索引(测试中无枚举字段)。
    fn no_defs() -> HashMap<(String, String), EnumDefine> {
        HashMap::new()
    }

    #[test]
    fn test_map_data_type() {
        assert_eq!(map_data_type(DataType::Int), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Long), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Decimal), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Double), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Tinyint), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Varchar), JsonUiDataType::String);
        assert_eq!(map_data_type(DataType::Clob), JsonUiDataType::String);
        assert_eq!(map_data_type(DataType::Blob), JsonUiDataType::String);
        assert_eq!(map_data_type(DataType::Date), JsonUiDataType::Date);
        assert_eq!(map_data_type(DataType::Datetime), JsonUiDataType::Datetime);
    }

    #[test]
    fn test_transform_field_excludes_precision() {
        let field = Field {
            prop: "amount".to_string(),
            code: "AMOUNT".to_string(),
            name: "金额".to_string(),
            data_type: DataType::Decimal,
            length: None,
            precision: Some(12),
            scale: Some(2),
            biz_type: None,
            biz_type_data: None,
            is_key: Some(false),
            not_null: Some(true),
            auto_generate: None,
            default_value: None,
            enum_ref: None,
            comment: Some("备注".to_string()),
        };

        let json = transform_field(&field, &no_defs());
        let serialized = serde_json::to_string(&json).unwrap();

        // 包含核心字段
        assert!(serialized.contains("\"prop\":\"amount\""));
        assert!(serialized.contains("\"code\":\"AMOUNT\""));
        assert!(serialized.contains("\"dataType\":\"NUMBER\""));
        assert!(serialized.contains("\"scale\":2"));
        assert!(serialized.contains("\"notNull\":true"));

        // 排除 precision/comment(auto_generate=None 时 autoGenerate 不输出)
        assert!(!serialized.contains("precision"));
        assert!(!serialized.contains("autoGenerate"));
        assert!(!serialized.contains("comment"));
        assert!(!serialized.contains("备注"));
    }

    #[test]
    fn test_transform_field_auto_generate_flag() {
        // 有 auto_generate -> 输出 "autoGenerate":true(布尔信号,json-ui 据此从表单排除该字段)
        let with_ag = Field {
            prop: "id".to_string(),
            code: "ID".to_string(),
            name: "主键".to_string(),
            data_type: DataType::Long,
            length: None,
            precision: None,
            scale: None,
            biz_type: None,
            biz_type_data: None,
            is_key: Some(true),
            not_null: Some(true),
            auto_generate: Some(AutoGenerate {
                strategy: "default".to_string(),
                param: None,
                timing: GenerationTiming::Insert,
            }),
            default_value: None,
            enum_ref: None,
            comment: None,
        };
        let serialized = serde_json::to_string(&transform_field(&with_ag, &no_defs())).unwrap();
        assert!(
            serialized.contains("\"autoGenerate\":true"),
            "有 auto_generate 应输出 \"autoGenerate\":true:\n{}",
            serialized
        );

        // 无 auto_generate -> 不输出 autoGenerate
        let without_ag = Field {
            auto_generate: None,
            ..with_ag.clone()
        };
        let serialized2 = serde_json::to_string(&transform_field(&without_ag, &no_defs())).unwrap();
        assert!(
            !serialized2.contains("autoGenerate"),
            "无 auto_generate 不应输出 autoGenerate:\n{}",
            serialized2
        );
    }

    #[test]
    fn test_field_order_code_prop_name_first_biztype_last() {
        // 序列化字段顺序: code/prop/name 靠前, bizType/bizTypeData 靠后
        // (防回归: 经 serde_json::Value 中转会被 BTreeMap 重排成字母序)
        let field = Field {
            code: "NAME".to_string(),
            prop: "name".to_string(),
            name: "名字".to_string(),
            data_type: DataType::Varchar,
            length: Some(8),
            precision: None,
            scale: None,
            biz_type: Some("Date".to_string()),
            biz_type_data: Some(serde_json::json!("YYYYMMDD")),
            is_key: Some(false),
            not_null: Some(true),
            auto_generate: None,
            default_value: None,
            enum_ref: None,
            comment: None,
        };
        let project = Project {
            version: "1.0.0".to_string(),
            name: None,
            base_package: "com.example".to_string(),
            tables: vec![Table {
                code: "T".to_string(),
                name: "表".to_string(),
                group: "g".to_string(),
                fields: vec![field],
                java_package: None,
                indexes: None,
                comment: None,
            }],
            biz_types: vec![],
            auto_gen_strategies: vec![],
            groups: vec![],
        };
        let json = generate_frontend_json(&project, &FrontendJsonOptions::default());

        // JsonModelSchema:顶层 type=model + code/name(单表,无 tables 包裹)
        assert!(
            json.contains("\"type\": \"model\""),
            "顶层应有 type:model:\n{}",
            json
        );
        assert!(
            !json.contains("\"tables\""),
            "不应再有 tables 包裹:\n{}",
            json
        );

        // 只看 field 对象片段(table 也有 name 字段,避免 find 匹配到 table.name)
        let field_json = &json[json.find("\"fields\"").unwrap()..];
        let pos = |k: &str| field_json.find(k).unwrap_or(usize::MAX);
        // code < prop < name < dataType
        assert!(
            pos("\"code\"") < pos("\"prop\""),
            "code 应在 prop 前:\n{}",
            json
        );
        assert!(
            pos("\"prop\"") < pos("\"name\""),
            "prop 应在 name 前:\n{}",
            json
        );
        assert!(
            pos("\"name\"") < pos("\"dataType\""),
            "name 应在 dataType 前:\n{}",
            json
        );
        // bizType/bizTypeData 靠后(在 notNull 之后)
        assert!(
            pos("\"notNull\"") < pos("\"bizType\""),
            "bizType 应靠后:\n{}",
            json
        );
    }

    /// 构造最小 Field(仅类型 + length/scale 不同),供 length/scale 输出测试复用。
    fn mk_field(dt: DataType, length: Option<u32>, scale: Option<u32>) -> Field {
        Field {
            prop: "f".to_string(),
            code: "F".to_string(),
            name: "f".to_string(),
            data_type: dt,
            length,
            precision: None,
            scale,
            biz_type: None,
            biz_type_data: None,
            is_key: Some(false),
            not_null: Some(false),
            auto_generate: None,
            default_value: None,
            enum_ref: None,
            comment: None,
        }
    }

    #[test]
    fn test_length_scale_by_data_type() {
        // VARCHAR: 保留 length, 不输出 scale(即便 Field 上有脏 scale)
        let s = serde_json::to_string(&transform_field(
            &mk_field(DataType::Varchar, Some(8), Some(2)),
            &no_defs(),
        ))
        .unwrap();
        assert!(s.contains("\"length\":8"), "VARCHAR 应输出 length:\n{}", s);
        assert!(!s.contains("scale"), "VARCHAR 不应输出 scale:\n{}", s);

        // TINYINT/INT/LONG: 不输出 length(即便有脏值), 输出 scale:0
        for dt in [DataType::Tinyint, DataType::Int, DataType::Long] {
            let s =
                serde_json::to_string(&transform_field(&mk_field(dt, Some(10), None), &no_defs()))
                    .unwrap();
            assert!(!s.contains("length"), "{:?} 不应输出 length:\n{}", dt, s);
            assert!(s.contains("\"scale\":0"), "{:?} 应输出 scale:0:\n{}", dt, s);
        }

        // DECIMAL: 不输出 length, 保留原 scale
        let s = serde_json::to_string(&transform_field(
            &mk_field(DataType::Decimal, Some(10), Some(2)),
            &no_defs(),
        ))
        .unwrap();
        assert!(!s.contains("length"), "DECIMAL 不应输出 length:\n{}", s);
        assert!(s.contains("\"scale\":2"), "DECIMAL 应输出原 scale:\n{}", s);

        // DOUBLE: 均不输出(§3.1 不允许 precision/scale)
        let s = serde_json::to_string(&transform_field(
            &mk_field(DataType::Double, Some(10), Some(2)),
            &no_defs(),
        ))
        .unwrap();
        assert!(!s.contains("length"), "DOUBLE 不应输出 length:\n{}", s);
        assert!(!s.contains("scale"), "DOUBLE 不应输出 scale:\n{}", s);

        // CLOB/BLOB/DATE/DATETIME: 均不输出
        for dt in [
            DataType::Clob,
            DataType::Blob,
            DataType::Date,
            DataType::Datetime,
        ] {
            let s = serde_json::to_string(&transform_field(
                &mk_field(dt, Some(10), Some(2)),
                &no_defs(),
            ))
            .unwrap();
            assert!(!s.contains("length"), "{:?} 不应输出 length:\n{}", dt, s);
            assert!(!s.contains("scale"), "{:?} 不应输出 scale:\n{}", dt, s);
        }
    }

    #[test]
    fn test_enum_field_emits_options() {
        // 枚举字段 -> bizType="Options" + bizTypeData=[{id,name}](不含 code/color)
        let mut field = mk_field(DataType::Varchar, Some(8), None);
        field.enum_ref = Some(crate::schema::InlineEnum {
            name: "性别".into(),
            has_code: Some(true),
            class_name: None,
            r#ref: None,
            values: vec![
                crate::schema::EnumValue {
                    id: "MALE".into(),
                    name: "男".into(),
                    code: Some("M".into()),
                    color: Some(crate::schema::EnumColor::Blue),
                },
                crate::schema::EnumValue {
                    id: "FEMALE".into(),
                    name: "女".into(),
                    code: Some("F".into()),
                    color: Some(crate::schema::EnumColor::Red),
                },
            ],
        });
        let defs = {
            let mut m = HashMap::new();
            // 定义方:键 = (表code, 字段prop)——此处 transform_field 定义方能直接取本地 values,
            // 不需在 defs 中出现;但为了构造走 defs 路径,这里放一个定义。
            let d = crate::schema::EnumDefine {
                table_code: "T".into(),
                field_prop: "gender".into(),
                field: field.clone(),
                r#enum: field.enum_ref.clone().unwrap(),
            };
            m.insert(("T".to_string(), "gender".to_string()), d);
            m
        };
        let json = serde_json::to_string(&transform_field(&field, &defs)).unwrap();
        assert!(
            json.contains("\"bizType\":\"Options\""),
            "枚举字段应输出 bizType=Options:\n{}",
            json
        );
        // 定义了 color 的项输出 color;code 不输出
        assert!(
            json.contains("[{\"id\":\"MALE\",\"name\":\"男\",\"color\":\"blue\"},{\"id\":\"FEMALE\",\"name\":\"女\",\"color\":\"red\"}]"),
            "bizTypeData 应含已定义的 color:\n{}", json
        );
        let data_seg = &json[json.find("bizTypeData").unwrap()..];
        assert!(
            !data_seg.contains("\"code\""),
            "bizTypeData 不应含 code:\n{}",
            data_seg
        );
    }

    #[test]
    fn test_enum_reference_field_emits_options() {
        // 引用方枚举字段: 解析到定义方取值 -> Options
        let mut ref_field = mk_field(DataType::Varchar, Some(8), None);
        ref_field.enum_ref = Some(crate::schema::InlineEnum {
            name: "性别".into(),
            has_code: None,
            class_name: None,
            r#ref: Some(crate::schema::InlineEnumRef {
                code: "A".into(),
                prop: "gender".into(),
            }),
            values: vec![],
        });
        let defs = {
            let mut m = HashMap::new();
            let d = crate::schema::EnumDefine {
                table_code: "A".into(),
                field_prop: "gender".into(),
                field: mk_field(DataType::Varchar, Some(8), None),
                r#enum: crate::schema::InlineEnum {
                    name: "性别".into(),
                    has_code: Some(true),
                    class_name: None,
                    r#ref: None,
                    values: vec![crate::schema::EnumValue {
                        id: "MALE".into(),
                        name: "男".into(),
                        code: Some("M".into()),
                        color: Some(crate::schema::EnumColor::Green),
                    }],
                },
            };
            m.insert(("A".to_string(), "gender".to_string()), d);
            m
        };
        let json = serde_json::to_string(&transform_field(&ref_field, &defs)).unwrap();
        assert!(
            json.contains("\"bizType\":\"Options\""),
            "引用方应输出 bizType=Options:\n{}",
            json
        );
        // 引用方解析定义方取值,含 color(定义方有);code 仍不输出
        assert!(
            json.contains("[{\"id\":\"MALE\",\"name\":\"男\",\"color\":\"green\"}]"),
            "引用方 bizTypeData 解析定义方(含 color):\n{}",
            json
        );
    }
}
