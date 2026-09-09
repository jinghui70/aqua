package com.aqua.connector.h2;

import java.sql.Connection;
import java.sql.Statement;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

import com.aqua.connector.DataType;
import com.aqua.connector.DbConfig;
import com.aqua.connector.meta.ColumnMeta;
import com.aqua.connector.meta.IndexMeta;
import com.aqua.connector.meta.TableInfo;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/**
 * H2Dialect 单元测试(H2 内存库)。
 *
 * 建表 -> 反解 -> 断言逻辑类型/主键/索引。
 */
class H2DialectTest {

    private H2Dialect dialect;
    private Connection conn;

    @BeforeEach
    void setUp() throws Exception {
        dialect = new H2Dialect();
        DbConfig config = new DbConfig();
        config.dialect = "h2";
        config.database = "aqua_test";
        config.user = "sa";
        config.password = "";
        conn = dialect.connect(config);

        // 建测试表(对齐 valid-full fixture 的 SYS_USER)
        try (Statement st = conn.createStatement()) {
            st.execute("DROP TABLE IF EXISTS SYS_USER");
            st.execute("CREATE TABLE SYS_USER (" +
                    "ID BIGINT NOT NULL, " +
                    "USER_NAME VARCHAR(64) NOT NULL, " +
                    "AMOUNT DECIMAL(12, 2), " +
                    "REMARK CLOB, " +
                    "CREATE_TIME TIMESTAMP, " +
                    "PRIMARY KEY (ID))");
            st.execute("CREATE UNIQUE INDEX IDX_SYS_USER_USER_NAME ON SYS_USER(USER_NAME)");
        }
    }

    @AfterEach
    void tearDown() throws Exception {
        conn.close();
    }

    @Test
    void testListTables() throws Exception {
        List<TableInfo> tables = dialect.listTables(conn, null);
        assertTrue(tables.stream().anyMatch(t -> t.name.equals("SYS_USER")), "应包含 SYS_USER: " + tables);
    }

    @Test
    void testGetColumns() throws Exception {
        List<ColumnMeta> columns = dialect.getColumns(conn, "SYS_USER");

        // ID: BIGINT -> LONG, 主键
        ColumnMeta id = find(columns, "ID");
        assertEquals(DataType.LONG, id.dataType, "BIGINT -> LONG");
        assertTrue(id.isKey, "ID 应为主键");

        // USER_NAME: VARCHAR(64)
        ColumnMeta userName = find(columns, "USER_NAME");
        assertEquals(DataType.VARCHAR, userName.dataType);
        assertEquals(64, userName.length);
        assertFalse(userName.nullable, "USER_NAME NOT NULL");

        // AMOUNT: DECIMAL(12,2)
        ColumnMeta amount = find(columns, "AMOUNT");
        assertEquals(DataType.DECIMAL, amount.dataType);
        assertEquals(12, amount.precision);
        assertEquals(2, amount.scale);

        // REMARK: CLOB
        ColumnMeta remark = find(columns, "REMARK");
        assertEquals(DataType.CLOB, remark.dataType);

        // CREATE_TIME: TIMESTAMP -> DATETIME
        ColumnMeta createTime = find(columns, "CREATE_TIME");
        assertEquals(DataType.DATETIME, createTime.dataType);
    }

    @Test
    void testGetIndexes() throws Exception {
        List<IndexMeta> indexes = dialect.getIndexes(conn, "SYS_USER");

        // 应有唯一索引 IDX_SYS_USER_USER_NAME(主键索引已跳过)
        IndexMeta idx = indexes.stream()
                .filter(i -> "IDX_SYS_USER_USER_NAME".equals(i.name))
                .findFirst()
                .orElseThrow(() -> new AssertionError("未找到 IDX_SYS_USER_USER_NAME: " + indexes));
        assertTrue(idx.unique, "应为唯一索引");
        assertTrue(idx.fields.contains("USER_NAME"));

        // 主键背后的索引必须跳过:H2 主键索引名形如 PRIMARY_KEY_x,列集=主键列集(ID),不得混入
        assertTrue(
                indexes.stream().noneMatch(i -> i.fields.size() == 1 && i.fields.contains("ID")),
                "主键索引(单列 ID)应被跳过,实际: " + indexes);
    }

    @Test
    void testBuildUrlFileMode() {
        // host=file -> 文件库 URL(AUTO_SERVER 允许多进程并发打开)
        DbConfig config = new DbConfig();
        config.dialect = "h2";
        config.host = "file";
        config.database = "/tmp/aqua_test_file";
        assertEquals("jdbc:h2:file:/tmp/aqua_test_file;AUTO_SERVER=TRUE", dialect.buildUrl(config));
    }

    @Test
    void testConnectWithJdbcUrlOverride() throws Exception {
        // jdbcUrl 直填模式:跳过 buildUrl,直接用完整 URL 连接(内存库,测试后自动回收)
        DbConfig config = new DbConfig();
        config.dialect = "h2";
        config.host = "localhost"; // 应被忽略
        config.port = 9092;        // 应被忽略
        config.database = "should_be_ignored";
        config.user = "sa";
        config.password = "";
        config.jdbcUrl = "jdbc:h2:mem:aqua_url_test;DB_CLOSE_DELAY=-1";
        try (Connection urlConn = dialect.connect(config)) {
            try (Statement st = urlConn.createStatement()) {
                st.execute("CREATE TABLE URL_MODE_TEST (ID INT PRIMARY KEY)");
            }
            List<TableInfo> tables = dialect.listTables(urlConn, null);
            assertTrue(tables.stream().anyMatch(t -> t.name.equals("URL_MODE_TEST")),
                    "jdbcUrl 直连的内存库应可见建表结果: " + tables);
        }
    }

    private ColumnMeta find(List<ColumnMeta> columns, String name) {
        return columns.stream()
                .filter(c -> name.equalsIgnoreCase(c.name))
                .findFirst()
                .orElseThrow(() -> new AssertionError("未找到列: " + name));
    }
}
