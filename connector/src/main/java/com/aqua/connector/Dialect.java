package com.aqua.connector;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;

import com.aqua.connector.meta.ColumnMeta;
import com.aqua.connector.meta.IndexMeta;

/**
 * 数据库方言接口(v2 架构:反解在 Java 侧)。
 *
 * 每种 JDBC 数据库实现此接口,封装连接 + 元数据反解(物理类型 -> aqua 逻辑类型)。
 * 新增数据库支持:实现此接口 + 注册到 DialectRegistry。
 *
 * 对齐 Rust Driver trait(test_connection/list_tables/get_columns/list_indexes)。
 */
public interface Dialect {
    /** 方言标识(如 "h2", "oracle"),与请求 JSON dialect 字段对应。 */
    String name();

    /** 建立 JDBC 连接。 */
    Connection connect(DbConfig config) throws SQLException;

    /** 列出所有表名。 */
    List<String> listTables(Connection conn, String schema) throws SQLException;

    /** 获取表的列元数据(反解为 aqua 逻辑类型)。 */
    List<ColumnMeta> getColumns(Connection conn, String table) throws SQLException;

    /** 获取表的索引元数据。 */
    List<IndexMeta> getIndexes(Connection conn, String table) throws SQLException;
}
