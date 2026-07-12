package com.aqua.connector.h2;

import java.sql.Types;

import com.aqua.connector.DataType;

/**
 * H2 JDBC 类型 -> aqua 逻辑类型反解。
 */
public final class H2TypeMapping {

    private H2TypeMapping() {}

    public static DataType map(int jdbcType, String typeName) {
        switch (jdbcType) {
            case Types.VARCHAR:
            case Types.CHAR:
            case Types.LONGVARCHAR:
                return DataType.VARCHAR;
            case Types.CLOB:
                return DataType.CLOB;
            case Types.TINYINT:
                return DataType.TINYINT;
            case Types.SMALLINT:
            case Types.INTEGER:
                return DataType.INT;
            case Types.BIGINT:
                return DataType.LONG;
            case Types.DECIMAL:
            case Types.NUMERIC:
                return DataType.DECIMAL;
            case Types.DATE:
                return DataType.DATE;
            case Types.TIMESTAMP:
            case Types.TIMESTAMP_WITH_TIMEZONE:
                return DataType.DATETIME;
            case Types.BLOB:
            case Types.BINARY:
            case Types.VARBINARY:
            case Types.LONGVARBINARY:
                return DataType.BLOB;
            default:
                // 兜底:按类型名推断
                return mapByName(typeName);
        }
    }

    private static DataType mapByName(String typeName) {
        if (typeName == null) return DataType.VARCHAR;
        String upper = typeName.toUpperCase();
        if (upper.contains("CLOB") || upper.contains("TEXT")) return DataType.CLOB;
        if (upper.contains("BLOB") || upper.contains("BINARY")) return DataType.BLOB;
        if (upper.contains("DATE") && !upper.contains("TIME")) return DataType.DATE;
        if (upper.contains("TIMESTAMP") || upper.contains("DATETIME")) return DataType.DATETIME;
        return DataType.VARCHAR;
    }
}
