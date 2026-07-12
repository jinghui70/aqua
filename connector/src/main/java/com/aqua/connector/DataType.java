package com.aqua.connector;

import com.fasterxml.jackson.annotation.JsonValue;

/**
 * aqua 逻辑类型枚举(对齐 Rust aqua_core::schema::DataType)。
 *
 * 序列化为大写字符串,与 Rust 端一致。
 */
public enum DataType {
    VARCHAR,
    CLOB,
    TINYINT,
    INT,
    LONG,
    DECIMAL,
    DATE,
    DATETIME,
    BLOB;

    @JsonValue
    public String value() {
        return name();
    }
}
