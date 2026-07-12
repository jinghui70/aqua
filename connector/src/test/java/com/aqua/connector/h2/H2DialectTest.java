package com.aqua.connector.h2;

import java.sql.Connection;
import java.sql.Statement;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

import com.aqua.connector.DataType;
import com.aqua.connector.DbConfig;
import com.aqua.connector.meta.ColumnMeta;
import com.aqua.connector.meta.IndexMeta;
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
        List<String> tables = dialect.listTables(conn, null);
        assertTrue(tables.contains("SYS_USER"), "应包含 SYS_USER: " + tables);
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
    }

    private ColumnMeta find(List<ColumnMeta> columns, String name) {
        return columns.stream()
                .filter(c -> name.equalsIgnoreCase(c.name))
                .findFirst()
                .orElseThrow(() -> new AssertionError("未找到列: " + name));
    }
}
