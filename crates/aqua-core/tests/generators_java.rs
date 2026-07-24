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

#[test]
fn test_generate_java_entity_with_lombok() {
    let project = load_fixture("valid-full.json");
    let java_code =
        generate_java_entity(&project, "SYS_USER", &JavaOptions::default()).expect("生成失败");

    println!("\n=== Generated Java (Lombok) ===\n{}\n", java_code);

    // 验证 package
    assert!(java_code.contains("package"), "应包含 package 声明");

    // 验证 import(默认类名 SysUser 能反推 SYS_USER → 省略 @Table,故不 import Table)
    assert!(!java_code.contains("import io.github.rainbow.dbaccess.annotation.Table"),
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

    let java_code = generate_java_entity(&project, "SYS_USER", &options).expect("生成失败");

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

    let java_code = generate_java_entity(&project, "SYS_USER", &options).expect("生成失败");

    assert!(java_code.contains("package com.example.entity;"));
    assert!(java_code.contains("public class User {"));
    // 自定义类名 User 不能反推 SYS_USER → 必须写 @Table + import
    assert!(java_code.contains("@Table(name = \"SYS_USER\")"), "自定义类名应写 @Table");
    assert!(java_code.contains("import io.github.rainbow.dbaccess.annotation.Table"),
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
    let java_code = generate_java_entity(&project, "SYS_LOG", &JavaOptions::default()).expect("生成失败");

    // strategy 非默认、timing=INSERT 省略、无 param
    assert!(
        java_code.contains("@GeneratedValue(strategy = \"snowflake\")"),
        "snowflake+INSERT 应只输出 strategy:\n{}", java_code
    );
    // 三个参数都非默认:全输出
    assert!(
        java_code.contains("@GeneratedValue(strategy = \"now\", param = \"yyyy\", timing = \"INSERT_UPDATE\")"),
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
}
