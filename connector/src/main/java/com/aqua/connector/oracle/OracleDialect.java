package com.aqua.connector.oracle;

import java.sql.Connection;
import java.sql.SQLException;

import com.aqua.connector.AbstractJdbcDialect;
import com.aqua.connector.DataType;
import com.aqua.connector.DbConfig;

/**
 * Oracle 数据库方言实现 - 继承 AbstractJdbcDialect。
 *
 * 特化:
 * - URL 格式(jdbc:oracle:thin:@//...)
 * - 类型映射(OracleTypeMapping,NUMBER 按 precision/scale 反解)
 * - schema 解析(Oracle schema=登录用户名,用 conn.getSchema())
 */
public class OracleDialect extends AbstractJdbcDialect {

    @Override
    public String name() {
        return "oracle";
    }

    @Override
    protected String getDriverClass() {
        return "oracle.jdbc.OracleDriver";
    }

    @Override
    protected String buildUrl(DbConfig config) {
        return "jdbc:oracle:thin:@//" + config.host + ":" + config.port + "/" + config.database;
    }

    @Override
    protected DataType mapType(int jdbcType, String typeName, Integer precision, Integer scale) {
        return OracleTypeMapping.map(jdbcType, typeName, precision, scale);
    }

    @Override
    protected String resolveSchema(Connection conn, String schema) throws SQLException {
        // Oracle schema=用户名,用连接的实际 schema,不用传入值
        return conn.getSchema();
    }
}
