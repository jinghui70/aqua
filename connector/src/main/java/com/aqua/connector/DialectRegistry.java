package com.aqua.connector;

import java.util.HashMap;
import java.util.Map;

import com.aqua.connector.h2.H2Dialect;
import com.aqua.connector.oracle.OracleDialect;

/**
 * Dialect 注册表(name -> Dialect 实例)。
 *
 * 新增数据库支持:实现 Dialect 接口 + 在此注册。
 * 无专门实现的外置 JDBC 库走 GenericJdbcDialect 通用兜底。
 */
public class DialectRegistry {
    private static final Map<String, Dialect> DIALECTS = new HashMap<>();

    static {
        // 内置驱动
        register(new H2Dialect());
        // 外置 JDBC - 专门实现
        register(new OracleDialect());
        // 外置 JDBC - 通用兜底(DM/KingBase/GBase/SQLServer)
        register(new GenericJdbcDialect("dm", "dm.jdbc.driver.DmDriver"));
        register(new GenericJdbcDialect("kingbase", "com.kingbase8.Driver"));
        register(new GenericJdbcDialect("gbase", "com.gbase.jdbc.Driver"));
        register(new GenericJdbcDialect("sqlserver", "com.microsoft.sqlserver.jdbc.SQLServerDriver"));
    }

    public static void register(Dialect dialect) {
        DIALECTS.put(dialect.name().toLowerCase(), dialect);
    }

    public static Dialect get(String name) {
        if (name == null) return null;
        return DIALECTS.get(name.toLowerCase());
    }
}
