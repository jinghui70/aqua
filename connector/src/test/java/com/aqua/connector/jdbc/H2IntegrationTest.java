package com.aqua.connector.jdbc;

import com.aqua.connector.protocol.DataSource;
import com.aqua.connector.registry.RegistryLoader;
import org.junit.jupiter.api.*;

import java.io.File;
import java.nio.file.Files;
import java.sql.Connection;

import static org.junit.jupiter.api.Assertions.*;

/**
 * H2 integration tests: connection, metadata reading, and query execution.
 *
 * <p>Uses H2 in-memory database — no external drivers required.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class H2IntegrationTest {

    private static RegistryLoader registryLoader;
    private static ConnectionFactory connectionFactory;
    private static Connection conn;

    @BeforeAll
    static void setUp() throws Exception {
        registryLoader = new RegistryLoader();
        registryLoader.load();
        connectionFactory = new ConnectionFactory(registryLoader);

        // Initialize an H2 in-memory database with a schema
        DataSource ds = new DataSource();
        ds.setType("h2");
        ds.setHost("mem");
        ds.setPort(0);
        ds.setDatabase("testdb;DB_CLOSE_DELAY=-1");
        ds.setUser("sa");
        ds.setPassword("");

        conn = connectionFactory.connect(ds);

        // Create test schema
        conn.createStatement().execute(
                "CREATE TABLE users (" +
                "  id INT PRIMARY KEY," +
                "  username VARCHAR(50) NOT NULL," +
                "  full_name VARCHAR(100)," +
                "  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP" +
                ")"
        );
        conn.createStatement().execute(
                "CREATE INDEX idx_users_username ON users(username)"
        );
        conn.createStatement().execute(
                "INSERT INTO users (id, username, full_name) VALUES (1, 'alice', 'Alice Wang')"
        );
        conn.createStatement().execute(
                "INSERT INTO users (id, username, full_name) VALUES (2, 'bob', 'Bob Li')"
        );
    }

    @AfterAll
    static void tearDown() throws Exception {
        if (conn != null && !conn.isClosed()) {
            conn.close();
        }
    }

    @Test
    @Order(1)
    void shouldConnectToH2() throws Exception {
        assertNotNull(conn);
        assertFalse(conn.isClosed());
    }

    @Test
    @Order(2)
    void shouldReadMetadata() throws Exception {
        MetadataReader reader = new MetadataReader();
        var result = reader.readMetadata(conn, null);

        assertNotNull(result);
        assertTrue(result.has("tables"));
        var tables = result.get("tables");
        assertTrue(tables.isArray());
        assertTrue(tables.size() >= 1);

        // Find the "USERS" table (H2 returns uppercase by default)
        var table = tables.get(0);
        for (int i = 0; i < tables.size(); i++) {
            if ("USERS".equals(tables.get(i).get("code").asText())) {
                table = tables.get(i);
                break;
            }
        }

        assertEquals("USERS", table.get("code").asText());
        var columns = table.get("columns");
        assertTrue(columns.isArray());
        assertTrue(columns.size() >= 4);

        // Verify column structure: ID (PK, not null), USERNAME (not null), FULL_NAME, CREATED_AT
        var idCol = columns.get(0);
        assertEquals("ID", idCol.get("name").asText());
        assertTrue(idCol.get("pk").asBoolean());
        assertFalse(idCol.get("nullable").asBoolean());
        assertNotNull(idCol.get("rawType").asText()); // e.g. "INTEGER"

        // Verify index
        var indexes = table.get("indexes");
        assertTrue(indexes.isArray());
        assertTrue(indexes.size() >= 1);
    }

    @Test
    @Order(3)
    void shouldReadMetadataWithTableFilter() throws Exception {
        MetadataReader reader = new MetadataReader();
        var result = reader.readMetadata(conn, java.util.List.of("USERS"));

        var tables = result.get("tables");
        assertEquals(1, tables.size());
        assertEquals("USERS", tables.get(0).get("code").asText());
    }

    @Test
    @Order(4)
    void shouldExecuteQuery() throws Exception {
        QueryExecutor executor = new QueryExecutor();
        var result = executor.execute(conn, "SELECT * FROM users ORDER BY id", null);

        assertNotNull(result);
        assertTrue(result.has("columns"));
        assertTrue(result.has("rows"));

        var columns = result.get("columns");
        assertEquals(4, columns.size()); // id, username, full_name, created_at

        var rows = result.get("rows");
        assertEquals(2, rows.size());

        var row0 = rows.get(0);
        assertEquals("1", row0.get(0).asText());
        assertEquals("alice", row0.get(1).asText());
        assertEquals("Alice Wang", row0.get(2).asText());
    }

    @Test
    @Order(5)
    void shouldExecuteUpdateAndReturnAffectedRows() throws Exception {
        QueryExecutor executor = new QueryExecutor();
        var result = executor.execute(conn,
                "UPDATE users SET full_name = ? WHERE id = ?",
                new Object[]{"Alice Updated", 1});

        assertNotNull(result);
        assertTrue(result.has("affectedRows"));
        assertEquals(1, result.get("affectedRows").asInt());
    }

    @Test
    @Order(6)
    void shouldSerializeBlobAsBase64() throws Exception {
        conn.createStatement().execute(
                "CREATE TABLE blob_t (id INT PRIMARY KEY, avatar BLOB)"
        );
        // "ABC" -> bytes 0x41 0x42 0x43
        conn.createStatement().execute(
                "INSERT INTO blob_t (id, avatar) VALUES (1, X'414243')"
        );

        QueryExecutor executor = new QueryExecutor();
        var result = executor.execute(conn, "SELECT avatar FROM blob_t WHERE id = 1", null);

        var rows = result.get("rows");
        assertEquals(1, rows.size());
        // base64("ABC") == "QUJD"；toString 会是 [B@hash（丢数据）
        assertEquals("QUJD", rows.get(0).get(0).asText());
    }
}
