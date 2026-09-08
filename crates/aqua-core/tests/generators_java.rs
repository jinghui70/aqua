//! Java 实体生成器集成测试。

use aqua_core::generators::java::{generate_java_entity, JavaOptions};
use aqua_core::schema::parse_project;
use std::fs;

/// 加载 fixture。
fn load_fixture(name: &str) -> aqua_core::schema::Project {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    let json_str = fs::read_to_string(&path).expect("读取 fixture 失败");
    let value: serde_json::Value = serde_json::from_str(&json_str).expect("JSON 解析失败");
    parse_project(value).expect("Project 校验失败")
}

/// 取生成产物中的实体(首项)内容。
fn entity_content(files: &[aqua_core::generators::java::JavaFile]) -> String {
    files
        .first()
        .expect("应有实体文件")
        .content
        .clone()
}

#[test]
fn test_generate_java_entity_with_lombok() {
    let project = load_fixture("valid-full.json");
    let options = JavaOptions {
        package: Some("com.example.core.entity".to_string()),
        ..Default::default()
    };
    let files = generate_java_entity(&project, "SYS_USER", &options).expect("生成失败");
    let java_code = entity_content(&files);

    // 验证 package
    assert!(java_code.contains("package com.example.core.entity;"), "应包含 package 声明");

    // 验证 import(默认类名 SysUser 能反推 SYS_USER → 省略 @Table,故不 import Table)
    assert!(!java_code.contains("io.github.jinghui70.rainbow.dbaccess.annotation.Table"),
        "默认类名省略 @Table,不应 import Table");
    assert!(java_code.contains("import lombok.Data"));
    assert!(
        java_code.contains("import java.time.LocalDateTime"),
        "应导入 LocalDateTime"
    );
    assert!(
        java_code.contains("import java.math.BigDecimal"),
        "应导入 BigDecimal"
    );

    // 验证注解(默认类名 SysUser↔SYS_USER 反推匹配 → 省略 @Table)
    assert!(!java_code.contains("@Table"), "默认类名应省略 @Table");
    assert!(java_code.contains("@Data"));
    assert!(java_code.contains("@Id"), "主键字段应有 @Id");

    // 验证类定义
    assert!(
        java_code.contains("public class SysUser"),
        "类名应为 SysUser"
    );

    // 验证字段
    assert!(java_code.contains("private Long id"), "应有 Long id 字段");
    assert!(
        java_code.contains("private String userName"),
        "应有 String userName 字段"
    );
    assert!(
        java_code.contains("private BigDecimal amount"),
        "应有 BigDecimal amount 字段"
    );
    assert!(
        java_code.contains("private LocalDateTime createTime"),
        "应有 LocalDateTime createTime 字段"
    );

    // Lombok 模式不应有 getter/setter
    assert!(
        !java_code.contains("public Long getId()"),
        "Lombok 模式不应生成 getter"
    );
}

#[test]
fn test_generate_java_entity_without_lombok() {
    let project = load_fixture("valid-full.json");
    let options = JavaOptions {
        use_lombok: false,
        ..Default::default()
    };

    let files = generate_java_entity(&project, "SYS_USER", &options).expect("生成失败");
    let java_code = entity_content(&files);

    // 不应有 @Data
    assert!(!java_code.contains("@Data"));
    assert!(!java_code.contains("import lombok.Data"));

    // 应有 getter/setter
    assert!(java_code.contains("public Long getId()"));
    assert!(java_code.contains("public void setId(Long id)"));
    assert!(java_code.contains("public String getUserName()"));
    assert!(java_code.contains("public void setUserName(String userName)"));
}

#[test]
fn test_custom_package_and_class_name() {
    let project = load_fixture("valid-full.json");
    let options = JavaOptions {
        use_lombok: true,
        package: Some("com.example.entity".to_string()),
        class_name: Some("User".to_string()),
    };

    let files = generate_java_entity(&project, "SYS_USER", &options).expect("生成失败");
    let java_code = entity_content(&files);

    assert!(java_code.contains("package com.example.entity;"));
    assert!(java_code.contains("public class User {"));
    // 自定义类名 User 不能反推 SYS_USER → 必须写 @Table + import
    assert!(java_code.contains("@Table(name = \"SYS_USER\")"), "自定义类名应写 @Table");
    assert!(java_code.contains("io.github.jinghui70.rainbow.dbaccess.annotation.Table"),
        "写 @Table 时应 import Table");
}

#[test]
fn test_table_not_found() {
    let project = load_fixture("valid-full.json");
    let result = generate_java_entity(&project, "NONEXISTENT_TABLE", &JavaOptions::default());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Table not found"));
}

#[test]
fn test_clob_blob_sql_type_annotation() {
    // Clob->Types.CLOB, Blob->Types.BLOB(防回归: 此前 Clob 误写成 Types.BLOB)
    let project_json = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "groups": [{ "code": "core", "name": "核心" }],
        "tables": [{
            "code": "T", "name": "T", "group": "core",
            "fields": [
                { "prop": "content", "code": "CONTENT", "name": "内容", "dataType": "CLOB" },
                { "prop": "data", "code": "DATA", "name": "数据", "dataType": "BLOB" }
            ]
        }]
    });
    let project = parse_project(project_json).expect("Project 校验失败");
    let files = generate_java_entity(&project, "T", &JavaOptions::default()).expect("生成失败");
    let java_code = entity_content(&files);

    assert!(
        java_code.contains("@Column(sqlType = Types.CLOB)"),
        "CLOB 字段应生成 Types.CLOB:\n{}", java_code
    );
    assert!(
        java_code.contains("@Column(sqlType = Types.BLOB)"),
        "BLOB 字段应生成 Types.BLOB:\n{}", java_code
    );
}

#[test]
fn test_generate_field_with_auto_generate() {
    // autoGenerate 字段应生成 @GeneratedValue,参数等于默认值(strategy=default/timing=INSERT/无 param)即省略
    let value = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "groups": [{ "code": "core", "name": "核心" }],
        "tables": [{
            "code": "SYS_LOG",
            "name": "日志",
            "group": "core",
            "fields": [{
                "code": "ID",
                "prop": "id",
                "name": "主键",
                "dataType": "LONG",
                "isKey": true,
                "autoGenerate": { "strategy": "snowflake", "timing": "INSERT" }
            }, {
                "code": "GMT_MODIFIED",
                "prop": "gmtModified",
                "name": "修改时间",
                "dataType": "DATETIME",
                "autoGenerate": { "strategy": "now", "param": "yyyy", "timing": "INSERT_UPDATE" }
            }, {
                "code": "GMT_CREATE",
                "prop": "gmtCreate",
                "name": "创建时间",
                "dataType": "DATETIME",
                "autoGenerate": { "strategy": "default", "timing": "INSERT" }
            }, {
                "code": "NAME",
                "prop": "name",
                "name": "名称",
                "dataType": "VARCHAR",
                "length": 64,
                "autoGenerate": null
            }]
        }]
    });
    let project = parse_project(value).expect("Project 校验失败");
    let files = generate_java_entity(&project, "SYS_LOG", &JavaOptions::default()).expect("生成失败");
    let java_code = entity_content(&files);

    // strategy 非默认、timing=INSERT 省略、无 param
    assert!(
        java_code.contains("@GeneratedValue(strategy = \"snowflake\")"),
        "snowflake+INSERT 应只输出 strategy:\n{}", java_code
    );
    // 三个参数都非默认:全输出
    assert!(
        java_code.contains("@GeneratedValue(strategy = \"now\", param = \"yyyy\", timing = GenerationTiming.INSERT_UPDATE)"),
        "now 字段应全输出:\n{}", java_code
    );
    // 全默认(strategy=default + timing=INSERT + 无 param):无括号
    assert!(
        java_code.contains("@GeneratedValue\n"),
        "全默认应输出无括号 @GeneratedValue:\n{}", java_code
    );
    assert!(
        !java_code.contains("@GeneratedValue()"),
        "全默认不应带空括号:\n{}", java_code
    );
    // enabled=false 不输出;共 3 个 @GeneratedValue
    assert!(
        java_code.matches("@GeneratedValue").count() == 3,
        "enabled=false 不输出,应共 3 个 @GeneratedValue:\n{}", java_code
    );
    // timing=INSERT_UPDATE 引用枚举常量,需 import GenerationTiming
    assert!(
        java_code.contains("import io.github.jinghui70.rainbow.dbaccess.annotation.GenerationTiming;"),
        "timing 枚举引用应 import GenerationTiming:\n{}", java_code
    );
}

#[test]
fn test_enum_field_generates_enum_class() {
    // hasCode=true 枚举:实体字段类型=枚举类名,并产独立枚举文件(CodeEnum)
    let value = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "groups": [{ "code": "core", "name": "核心" }],
        "tables": [{
            "code": "USER_INFO",
            "name": "用户",
            "group": "core",
            "fields": [{
                "prop": "gender", "code": "GENDER", "name": "性别",
                "dataType": "VARCHAR", "length": 8,
                "enum": {
                    "name": "性别", "hasCode": true,
                    "values": [
                        { "id": "MALE", "name": "男", "code": "M" },
                        { "id": "FEMALE", "name": "女", "code": "F" }
                    ]
                }
            }]
        }]
    });
    let project = parse_project(value).expect("Project 校验失败");
    let files = generate_java_entity(&project, "USER_INFO", &JavaOptions::default()).expect("生成失败");

    // 实体字段类型应为枚举类名(非 String)
    let entity = entity_content(&files);
    assert!(entity.contains("private Gender gender;"), "枚举字段类型应为 Gender:\n{}", entity);

    // 应有独立的 Gender 枚举文件
    assert_eq!(files.len(), 2, "应有实体 + 枚举共 2 个文件");
    let enum_file = &files[1];
    assert_eq!(enum_file.path, "Gender.java");
    let enum_content = &enum_file.content;
    assert!(enum_content.contains("import io.github.jinghui70.rainbow.dbaccess.object.CodeEnum;"));
    assert!(enum_content.contains("public enum Gender implements CodeEnum {"));
    assert!(enum_content.contains("MALE(\"M\"), // 男"));
    assert!(enum_content.contains("FEMALE(\"F\"); // 女"));
    assert!(enum_content.contains("public String code() { return code; }"));
}

#[test]
fn test_plain_enum_generates_no_codeenum() {
    // hasCode=false 枚举:普通枚举,项后注释,无 CodeEnum
    let value = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "groups": [{ "code": "core", "name": "核心" }],
        "tables": [{
            "code": "ORDER",
            "name": "订单",
            "group": "core",
            "fields": [{
                "prop": "status", "code": "STATUS", "name": "状态",
                "dataType": "VARCHAR", "length": 8,
                "enum": {
                    "name": "状态", "hasCode": false,
                    "values": [{ "id": "ACTIVE", "name": "启用" }]
                }
            }]
        }]
    });
    let project = parse_project(value).expect("Project 校验失败");
    let files = generate_java_entity(&project, "ORDER", &JavaOptions::default()).expect("生成失败");

    assert_eq!(files.len(), 2);
    let enum_content = &files[1].content;
    assert!(enum_content.contains("public enum Status {"));
    assert!(enum_content.contains("ACTIVE; // 启用"));
    assert!(!enum_content.contains("implements"));
    assert!(!enum_content.contains("code()"));
}

#[test]
fn test_enum_reference_generates_no_enum_file() {
    // 跨表引用: 表ORDER 的 status 引用 表DICT 的 type 枚举 → 引用方不产枚举文件,字段类型指向目标类名
    let value = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "groups": [{ "code": "core", "name": "核心" }],
        "tables": [{
            "code": "DICT",
            "name": "字典",
            "group": "core",
            "fields": [{
                "prop": "type", "code": "DICT_TYPE", "name": "类型",
                "dataType": "VARCHAR", "length": 8,
                "enum": {
                    "name": "类型", "hasCode": true,
                    "values": [{ "id": "A", "name": "甲", "code": "1" }]
                }
            }]
        }, {
            "code": "ORDER",
            "name": "订单",
            "group": "core",
            "fields": [{
                "prop": "orderType", "code": "ORDER_TYPE", "name": "订单类型",
                "dataType": "VARCHAR", "length": 8,
                "enum": { "name": "类型", "ref": { "code": "DICT", "prop": "type" } }
            }]
        }]
    });
    let project = parse_project(value).expect("Project 校验失败");

    // 引用方(ORDER): 只产实体一个文件,字段类型=Type(目标类名)
    let order_files = generate_java_entity(&project, "ORDER", &JavaOptions::default()).expect("生成失败");
    assert_eq!(order_files.len(), 1, "引用方不应产枚举文件");
    let order = entity_content(&order_files);
    assert!(order.contains("private Type orderType;"), "引用方字段类型应为 Type:\n{}", order);

    // 定义方(DICT): 产实体 + 枚举
    let dict_files = generate_java_entity(&project, "DICT", &JavaOptions::default()).expect("生成失败");
    assert_eq!(dict_files.len(), 2, "定义方应产枚举文件");
    assert_eq!(dict_files[1].path, "Type.java");
}

#[test]
fn test_enum_class_name_override() {
    // className 覆盖派生名
    let value = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "groups": [{ "code": "core", "name": "核心" }],
        "tables": [{
            "code": "USER_INFO",
            "name": "用户",
            "group": "core",
            "fields": [{
                "prop": "gender", "code": "GENDER", "name": "性别",
                "dataType": "VARCHAR", "length": 8,
                "enum": {
                    "name": "性别", "className": "GenderEnum", "hasCode": true,
                    "values": [{ "id": "MALE", "name": "男", "code": "M" }]
                }
            }]
        }]
    });
    let project = parse_project(value).expect("Project 校验失败");
    let files = generate_java_entity(&project, "USER_INFO", &JavaOptions::default()).expect("生成失败");

    let entity = entity_content(&files);
    assert!(entity.contains("private GenderEnum gender;"), "字段类型应为覆盖类名:\n{}", entity);
    assert_eq!(files[1].path, "GenderEnum.java");
    assert!(files[1].content.contains("public enum GenderEnum implements CodeEnum {"));
}

#[test]
fn test_cross_group_ref_enum_imported() {
    // 跨组引用:引用方实体 import 定义方包下的枚举类;同组(同包)则不 import
    let value = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "groups": [{ "code": "core", "name": "核心" }, { "code": "order", "name": "订单" }],
        "tables": [{
            "code": "USER",
            "name": "用户",
            "group": "core",
            "fields": [{
                "prop": "gender", "code": "GENDER", "name": "性别",
                "dataType": "VARCHAR", "length": 8,
                "enum": {
                    "name": "性别", "hasCode": true,
                    "values": [{ "id": "MALE", "name": "男", "code": "M" }]
                }
            }]
        }, {
            "code": "ORDER",
            "name": "订单",
            "group": "order",
            "fields": [{
                "prop": "buyerGender", "code": "BUYER_GENDER", "name": "购买人性别",
                "dataType": "VARCHAR", "length": 8,
                "enum": { "name": "性别", "ref": { "code": "USER", "prop": "gender" } }
            }]
        }]
    });
    let project = parse_project(value).expect("Project 校验失败");

    // 跨组:ORDER 实体包 != USER 枚举包 -> import
    let files = generate_java_entity(
        &project,
        "ORDER",
        &JavaOptions { package: Some("com.example.order.entity".into()), ..Default::default() },
    )
    .expect("生成失败");
    let entity = entity_content(&files);
    assert!(
        entity.contains("import com.example.core.entity.Gender;"),
        "跨组引用应 import 定义方枚举类:\n{}", entity
    );

    // 同组:同包 -> 不 import(同包 import 冗余)
    let same_pkg = generate_java_entity(
        &project,
        "ORDER",
        &JavaOptions { package: Some("com.example.core.entity".into()), ..Default::default() },
    )
    .expect("生成失败");
    let entity2 = entity_content(&same_pkg);
    assert!(
        !entity2.contains("import com.example.core.entity.Gender;"),
        "同包引用不应 import:\n{}", entity2
    );
}

#[test]
fn test_ref_enum_import_uses_saved_java_package() {
    // 定义表存了 javaPackage(用户在 JavaTab 改过)-> 引用方 import 用它,而非默认规则
    let value = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "groups": [{ "code": "core", "name": "核心" }, { "code": "order", "name": "订单" }],
        "tables": [{
            "code": "SYS_USER",
            "name": "用户",
            "group": "core",
            "javaPackage": "com.example.sys.user.entity",
            "fields": [{
                "prop": "gender", "code": "GENDER", "name": "性别",
                "dataType": "VARCHAR", "length": 8,
                "enum": {
                    "name": "性别", "hasCode": true,
                    "values": [{ "id": "MALE", "name": "男", "code": "M" }]
                }
            }]
        }, {
            "code": "ORDER",
            "name": "订单",
            "group": "order",
            "fields": [{
                "prop": "buyerGender", "code": "BUYER_GENDER", "name": "购买人性别",
                "dataType": "VARCHAR", "length": 8,
                "enum": { "name": "性别", "ref": { "code": "SYS_USER", "prop": "gender" } }
            }]
        }]
    });
    let project = parse_project(value).expect("Project 校验失败");

    let files = generate_java_entity(
        &project,
        "ORDER",
        &JavaOptions { package: Some("com.example.order.entity".into()), ..Default::default() },
    )
    .expect("生成失败");
    let entity = entity_content(&files);
    assert!(
        entity.contains("import com.example.sys.user.entity.Gender;"),
        "引用枚举 import 应使用定义表持久化的 javaPackage:\n{}", entity
    );
    assert!(
        !entity.contains("com.example.core.entity.Gender"),
        "不应再用默认规则包名:\n{}", entity
    );
}
