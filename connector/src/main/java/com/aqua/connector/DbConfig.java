package com.aqua.connector;

import java.util.List;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

/**
 * 数据库连接配置(从 stdin JSON 反序列化)。
 *
 * 字段名对齐 Rust DbConfig(驼峰)。
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class DbConfig {
    public String dialect;
    public String host;
    public int port;
    public String user;
    public String password;
    public String database;
    public String schema;
    /** getColumns/listIndexes/queryRows 用的表名 */
    public String table;
    /** importTables 用的表名列表(批量导入,一条连接反解多表) */
    public List<String> tables;
    /** executeUpdate 用的 SQL */
    public String sql;
}
