package com.aqua.connector.meta;

import java.util.List;

/**
 * 索引元数据(反解结果,对齐 Rust IndexMeta)。
 */
public class IndexMeta {
    public String name;
    public List<String> fields;
    public boolean unique;

    public IndexMeta() {}

    public IndexMeta(String name, List<String> fields, boolean unique) {
        this.name = name;
        this.fields = fields;
        this.unique = unique;
    }
}
