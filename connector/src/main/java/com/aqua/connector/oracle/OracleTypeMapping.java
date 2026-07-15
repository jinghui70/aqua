package com.aqua.connector.oracle;

import java.sql.Types;

import com.aqua.connector.DataType;

/**
 * Oracle JDBC 类型 -> aqua 逻辑类型反解。
 *
 * Oracle NUMBER 按精度/scale 反解为整数(Tinyint/Int/Long)或小数(Decimal)。
 */
public final class OracleTypeMapping {

    private OracleTypeMapping() {}

    public static DataType map(int jdbcType, String typeName, int precision, int scale) {
        switch (jdbcType) {
            case Types.VARCHAR:
            case Types.CHAR:
            case Types.LONGVARCHAR:
            case Types.NVARCHAR:
            case Types.NCHAR:
                return DataType.VARCHAR;
            case Types.CLOB:
            case Types.NCLOB:
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
                // Oracle NUMBER 按精度反解为整数/小数
                if (scale == 0) {
                    if (precision <= 3) return DataType.TINYINT;
                    if (precision <= 10) return DataType.INT;
                    if (precision <= 19) return DataType.LONG;
                }
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
                return mapByName(typeName);
        }
    }

    private static DataType mapByName(String typeName) {
        if (typeName == null) return DataType.VARCHAR;
        String upper = typeName.toUpperCase();
        if (upper.contains("CLOB")) return DataType.CLOB;
        if (upper.contains("BLOB") || upper.contains("BINARY")) return DataType.BLOB;
        if (upper.contains("TIMESTAMP")) return DataType.DATETIME;
        if (upper.contains("DATE")) return DataType.DATE;
        if (upper.contains("NUMBER")) return DataType.DECIMAL;
        return DataType.VARCHAR;
    }
}
