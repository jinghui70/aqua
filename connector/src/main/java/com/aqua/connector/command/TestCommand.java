package com.aqua.connector.command;

import com.aqua.connector.jdbc.ConnectionFactory;
import com.aqua.connector.protocol.DataSource;
import com.aqua.connector.protocol.Response;

import java.sql.Connection;

/**
 * Handles the "test" command: verify database connectivity.
 */
public class TestCommand {

    private final ConnectionFactory connectionFactory;

    public TestCommand(ConnectionFactory connectionFactory) {
        this.connectionFactory = connectionFactory;
    }

    public Response execute(DataSource dataSource) {
        Connection conn = null;
        try {
            conn = connectionFactory.connect(dataSource);
            return Response.ok("connected");
        } catch (Exception e) {
            return Response.fail("CONNECTION_FAILED", e.getMessage());
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
