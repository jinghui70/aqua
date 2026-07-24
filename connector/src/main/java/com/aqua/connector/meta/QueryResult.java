package com.aqua.connector.meta;

import java.util.List;

/**
 * 查询结果(queryRows 返回):列名 + 行数据(每行 = 值数组,顺序对应 columns)。
 * 值统一为字符串或 null(数据集 JSONL 兼容)。
 */
public class QueryResult {
    public List<String> columns;
    public List<List<Object>> rows;

    public QueryResult() {}

    public QueryResult(List<String> columns, List<List<Object>> rows) {
        this.columns = columns;
        this.rows = rows;
    }
}
