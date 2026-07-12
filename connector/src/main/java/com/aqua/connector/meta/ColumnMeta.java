package com.aqua.connector.meta;

import com.aqua.connector.DataType;

/**
 * 列元数据(反解结果,对齐 Rust ColumnMeta)。
 */
public class ColumnMeta {
    public String name;
    public DataType dataType;
    public Integer length;
    public Integer precision;
    public Integer scale;
    public boolean nullable;
    public boolean isKey;
    public String defaultValue;
    public String comment;

    public ColumnMeta() {}

    public ColumnMeta(String name, DataType dataType, Integer length, Integer precision,
                      Integer scale, boolean nullable, boolean isKey, String defaultValue, String comment) {
        this.name = name;
        this.dataType = dataType;
        this.length = length;
        this.precision = precision;
        this.scale = scale;
        this.nullable = nullable;
        this.isKey = isKey;
        this.defaultValue = defaultValue;
        this.comment = comment;
    }
}
