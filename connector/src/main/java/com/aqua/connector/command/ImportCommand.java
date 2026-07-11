package com.aqua.connector.command;

import com.aqua.connector.jdbc.ConnectionFactory;
import com.aqua.connector.jdbc.MetadataReader;
import com.aqua.connector.protocol.DataSource;
import com.aqua.connector.protocol.Response;

import java.sql.Connection;
import java.util.List;
import java.util.Map;

/**
 * Handles the "import" command: read raw JDBC metadata and return as JSON.
 */
public class ImportCommand {

    private final ConnectionFactory connectionFactory;
    private final MetadataReader metadataReader;

    public ImportCommand(ConnectionFactory connectionFactory) {
        this.connectionFactory = connectionFactory;
        this.metadataReader = new MetadataReader();
    }

    public Response execute(DataSource dataSource, Map<String, Object> params) {
        Connection conn = null;
        try {
            conn = connectionFactory.connect(dataSource);

            @SuppressWarnings("unchecked")
            List<String> tables = null;
            if (params != null && params.containsKey("tables")) {
                Object tablesObj = params.get("tables");
                if (tablesObj instanceof List) {
                    tables = (List<String>) tablesObj;
                }
            }

            return Response.ok(metadataReader.readMetadata(conn, tables));
        } catch (Exception e) {
            return Response.fail("IMPORT_FAILED", e.getMessage());
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
