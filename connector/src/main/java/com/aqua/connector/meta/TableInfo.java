package com.aqua.connector.meta;

/**
 * 表信息(表名 + 注释),listTables 返回,对齐 Rust TableInfo。
 */
public class TableInfo {
    public String name;
    public String comment;

    public TableInfo() {}

    public TableInfo(String name, String comment) {
        this.name = name;
        this.comment = comment;
    }
}
