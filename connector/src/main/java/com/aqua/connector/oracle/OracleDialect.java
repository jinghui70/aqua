package com.aqua.connector.oracle;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.HashMap;
import java.util.Map;

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

    @Override
    protected Map<String, String> resolveColumnComments(Connection conn, String schema, String table) throws SQLException {
        // Oracle 列注释存在 USER_COL_COMMENTS,DatabaseMetaData.getColumns 的 REMARKS 常为空,需补查
        Map<String, String> map = new HashMap<>();
        String sql = "SELECT COLUMN_NAME, COMMENTS FROM USER_COL_COMMENTS WHERE TABLE_NAME = ?";
        try (PreparedStatement ps = conn.prepareStatement(sql)) {
            ps.setString(1, table);
            try (ResultSet rs = ps.executeQuery()) {
                while (rs.next()) {
                    String col = rs.getString("COLUMN_NAME");
                    String c = rs.getString("COMMENTS");
                    if (c != null && !c.isBlank()) {
                        map.put(col, c);
                    }
                }
            }
        }
        return map;
    }
}
