package com.aqua.connector.jdbc;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.sql.*;
import java.util.Base64;

/**
 * Executes SQL queries via {@link PreparedStatement} and returns results
 * as JSON ({@code columns + rows} or {@code affectedRows}).
 */
public class QueryExecutor {

    private static final ObjectMapper MAPPER = new ObjectMapper();

    public QueryExecutor() {}

    /**
     * Execute a query (SELECT or DML).
     *
     * @param conn active JDBC connection
     * @param sql  the SQL statement
     * @param args optional positional parameters for the PreparedStatement
     * @return JSON result
     */
    public ObjectNode execute(Connection conn, String sql, Object[] args) throws SQLException {
        try (PreparedStatement stmt = conn.prepareStatement(sql)) {
            if (args != null) {
                for (int i = 0; i < args.length; i++) {
                    stmt.setObject(i + 1, args[i]);
                }
            }

            boolean isResultSet = stmt.execute();
            if (isResultSet) {
                return readResultSet(stmt.getResultSet());
            } else {
                ObjectNode result = MAPPER.createObjectNode();
                result.put("affectedRows", stmt.getUpdateCount());
                return result;
            }
        }
    }

    private ObjectNode readResultSet(ResultSet rs) throws SQLException {
        ResultSetMetaData rsmd = rs.getMetaData();
        int colCount = rsmd.getColumnCount();

        ObjectNode result = MAPPER.createObjectNode();
        ArrayNode columns = result.putArray("columns");
        for (int i = 1; i <= colCount; i++) {
            columns.add(rsmd.getColumnName(i));
        }

        ArrayNode rows = result.putArray("rows");
        while (rs.next()) {
            ArrayNode row = rows.addArray();
            for (int i = 1; i <= colCount; i++) {
                Object val = rs.getObject(i);
                if (val == null) {
                    row.addNull();
                } else if (val instanceof byte[]) {
                    // BLOB / binary -> base64（数据集规范；toString 会输出 [B@hash 丢数据）
                    row.add(Base64.getEncoder().encodeToString((byte[]) val));
                } else if (val instanceof Blob) {
                    Blob blob = (Blob) val;
                    byte[] bytes = blob.getBytes(1, (int) blob.length());
                    row.add(Base64.getEncoder().encodeToString(bytes));
                } else {
                    row.add(val.toString());
                }
            }
        }

        return result;
    }
}
