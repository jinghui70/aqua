package com.aqua.connector.command;

import com.aqua.connector.jdbc.ConnectionFactory;
import com.aqua.connector.jdbc.QueryExecutor;
import com.aqua.connector.protocol.DataSource;
import com.aqua.connector.protocol.Response;

import java.sql.Connection;
import java.util.List;
import java.util.Map;

/**
 * Handles the "query" command: execute SQL and return results.
 */
public class QueryCommand {

    private final ConnectionFactory connectionFactory;
    private final QueryExecutor queryExecutor;

    public QueryCommand(ConnectionFactory connectionFactory) {
        this.connectionFactory = connectionFactory;
        this.queryExecutor = new QueryExecutor();
    }

    public Response execute(DataSource dataSource, Map<String, Object> params) {
        if (params == null || !params.containsKey("sql")) {
            return Response.fail("INVALID_REQUEST", "Missing required param: sql");
        }

        String sql = (String) params.get("sql");
        if (sql == null || sql.isBlank()) {
            return Response.fail("INVALID_REQUEST", "Param 'sql' must be a non-empty string");
        }

        Object argsObj = params.get("args");
        Object[] args = null;
        if (argsObj instanceof List) {
            args = ((List<?>) argsObj).toArray();
        }

        Connection conn = null;
        try {
            conn = connectionFactory.connect(dataSource);
            return Response.ok(queryExecutor.execute(conn, sql, args));
        } catch (Exception e) {
            return Response.fail("QUERY_FAILED", e.getMessage());
        } finally {
            closeQuietly(conn);
        }
    }

    private void closeQuietly(Connection conn) {
        if (conn != null) {
            try { conn.close(); } catch (Exception ignored) {}
        }
    }
}
