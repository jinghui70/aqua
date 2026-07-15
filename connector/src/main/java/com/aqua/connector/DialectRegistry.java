package com.aqua.connector;

import java.util.HashMap;
import java.util.Map;

import com.aqua.connector.h2.H2Dialect;
import com.aqua.connector.oracle.OracleDialect;

/**
 * Dialect 注册表(name -> Dialect 实例)。
 *
 * 新增数据库支持:实现 Dialect 接口 + 在此注册。
 */
public class DialectRegistry {
    private static final Map<String, Dialect> DIALECTS = new HashMap<>();

    static {
        register(new H2Dialect());
        register(new OracleDialect());
    }

    public static void register(Dialect dialect) {
        DIALECTS.put(dialect.name().toLowerCase(), dialect);
    }

    public static Dialect get(String name) {
        if (name == null) return null;
        return DIALECTS.get(name.toLowerCase());
    }
}
