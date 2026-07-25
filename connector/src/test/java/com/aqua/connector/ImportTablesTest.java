package com.aqua.connector;

import java.sql.Connection;
import java.sql.Statement;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

import com.aqua.connector.h2.H2Dialect;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

/**
 * importTables 批量反解单测(H2 内存库)。
 *
 * 验证 Main.dispatch 的 importTables 分支:一条连接反解多表,
 * 返回 {tables:[{name, columns:[...], indexes:[...]}]},顺序对齐请求。
 *
 * 注意:dispatch 用 addPOJO 存列/索引,得到的是 POJONode——能正确序列化成 JSON 文本
 * (正是 Rust 侧消费的 wire 格式),但不支持树遍历 get(field)。故断言前先序列化再 readTree,
 * 还原真实 wire 结构。
 */
class ImportTablesTest {

    private static final ObjectMapper MAPPER = new ObjectMapper();

    private H2Dialect dialect;
    private Connection conn;
    private DbConfig config;

    @BeforeEach
    void setUp() throws Exception {
        dialect = new H2Dialect();
        config = new DbConfig();
        config.dialect = "h2";
        config.database = "aqua_import_test";
        config.user = "sa";
        config.password = "";
        conn = dialect.connect(config);

        try (Statement st = conn.createStatement()) {
            st.execute("DROP TABLE IF EXISTS SYS_USER");
            st.execute("DROP TABLE IF EXISTS SYS_ORDER");
            st.execute("CREATE TABLE SYS_USER (" +
                    "ID BIGINT NOT NULL, USER_NAME VARCHAR(64) NOT NULL, PRIMARY KEY (ID))");
            st.execute("CREATE UNIQUE INDEX IDX_SYS_USER_USER_NAME ON SYS_USER(USER_NAME)");
            st.execute("CREATE TABLE SYS_ORDER (" +
                    "ID BIGINT NOT NULL, AMOUNT DECIMAL(12,2), PRIMARY KEY (ID))");
        }
    }

    @AfterEach
    void tearDown() throws Exception {
        conn.close();
    }

    @Test
    void testImportTablesReturnsPerTableColumnsAndIndexes() throws Exception {
        config.tables = List.of("SYS_USER", "SYS_ORDER");

        ObjectNode raw = Main.dispatch("importTables", dialect, conn, config);
        // 序列化再解析,还原 Rust 侧实际消费的 wire 结构(addPOJO 的 POJONode 展开为可遍历 ObjectNode)
        JsonNode resp = MAPPER.readTree(MAPPER.writeValueAsString(raw));

        JsonNode tables = resp.get("tables");
        assertNotNull(tables, "响应应含 tables");
        assertEquals(2, tables.size(), "应返回两个表");

        // 顺序对齐请求
        assertEquals("SYS_USER", tables.get(0).get("name").asText());
        assertEquals("SYS_ORDER", tables.get(1).get("name").asText());

        // SYS_USER: 两列 + 唯一索引 IDX_SYS_USER_USER_NAME;主键索引已跳过,故索引恰好 1 条
        JsonNode user = tables.get(0);
        assertEquals(2, user.get("columns").size(), "SYS_USER 两列");
        assertEquals(1, user.get("indexes").size(), "SYS_USER 索引恰 1 条(主键索引已跳过): " + user.get("indexes"));
        JsonNode userIdx = findIndex(user.get("indexes"), "IDX_SYS_USER_USER_NAME");
        assertNotNull(userIdx, "SYS_USER 应含 IDX_SYS_USER_USER_NAME: " + user.get("indexes"));
        assertTrue(userIdx.get("unique").asBoolean(), "IDX_SYS_USER_USER_NAME 唯一");
        assertEquals("USER_NAME", userIdx.get("fields").get(0).asText());

        // SYS_ORDER: 两列,仅有主键 → 主键索引跳过后索引为空
        JsonNode order = tables.get(1);
        assertEquals(2, order.get("columns").size(), "SYS_ORDER 两列");
        assertEquals(0, order.get("indexes").size(), "SYS_ORDER 仅主键,索引应为空: " + order.get("indexes"));
    }

    /** 按 name 在 indexes 数组查一条,找不到返回 null。 */
    private JsonNode findIndex(JsonNode indexes, String name) {
        for (JsonNode i : indexes) {
            if (name.equals(i.get("name").asText())) return i;
        }
        return null;
    }
}
