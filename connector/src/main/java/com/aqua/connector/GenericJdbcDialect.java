package com.aqua.connector;

import java.sql.Types;

/**
 * 通用 JDBC 方言兜底实现 - 零特化,适用于任何标准 JDBC 驱动。
 *
 * 类型映射走 java.sql.Types 标准,得到粗粒度逻辑类型(如所有 NUMERIC→DECIMAL)。
 * URL 格式为 jdbc:<dialect>://<host>:<port>/<database>。
 * schema 直接用传入值。
 *
 * 用于没有专门 Dialect 子类的数据库,装了 jar 即可反解。
 */
public class GenericJdbcDialect extends AbstractJdbcDialect {
    private final String dialectName;
    private final String driverClass;

    public GenericJdbcDialect(String dialectName, String driverClass) {
        this.dialectName = dialectName;
        this.driverClass = driverClass;
    }

    @Override
    public String name() {
        return dialectName;
    }

    @Override
    protected String getDriverClass() {
        return driverClass;
    }

    @Override
    protected String buildUrl(DbConfig config) {
        return String.format("jdbc:%s://%s:%d/%s", dialectName, config.host, config.port, config.database);
    }

    @Override
    protected DataType mapType(int jdbcType, String typeName, Integer precision, Integer scale) {
        // 按 java.sql.Types 标准映射(保守)
        switch (jdbcType) {
            case Types.TINYINT:
                return DataType.TINYINT;
            case Types.SMALLINT:
            case Types.INTEGER:
                return DataType.INT;
            case Types.BIGINT:
                return DataType.LONG;
            case Types.DECIMAL:
            case Types.NUMERIC:
            case Types.FLOAT:
            case Types.REAL:
            case Types.DOUBLE:
                return DataType.DECIMAL;
            case Types.VARCHAR:
            case Types.CHAR:
            case Types.LONGVARCHAR:
            case Types.NVARCHAR:
            case Types.NCHAR:
                return DataType.VARCHAR;
            case Types.CLOB:
            case Types.NCLOB:
                return DataType.CLOB;
            case Types.BLOB:
            case Types.BINARY:
            case Types.VARBINARY:
            case Types.LONGVARBINARY:
                return DataType.BLOB;
            case Types.DATE:
                return DataType.DATE;
            case Types.TIME:
            case Types.TIMESTAMP:
                return DataType.DATETIME;
            case Types.BOOLEAN:
            case Types.BIT:
                // aqua DataType 无 BOOLEAN,映射为 TINYINT
                return DataType.TINYINT;
            default:
                // 未知类型兜底为 VARCHAR
                return DataType.VARCHAR;
        }
    }
}
