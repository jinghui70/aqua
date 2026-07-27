package com.aqua.aquah2;

import org.junit.jupiter.api.Test;

import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.Statement;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * AquaH2 实跑验证:用 test.aqua + test.data,H2 内存库执行 export 的 SQL。
 */
class AquaH2Test {

    private String readResource(String name) throws Exception {
        try (InputStream in = getClass().getClassLoader().getResourceAsStream("aquah2/" + name)) {
            assertNotNull(in, "资源不存在: " + name);
            return new String(in.readAllBytes(), StandardCharsets.UTF_8);
        }
    }

    @Test
    void exportAll_h2BuildsAndInserts() throws Exception {
        AquaH2 h2 = new AquaH2(readResource("test.aqua")).dataset(readResource("test.data"));
        String sql = h2.export();

        assertFalse(sql.isBlank(), "export 不应为空");
        assertTrue(sql.contains("CREATE TABLE"), "应含 CREATE TABLE");
        assertTrue(sql.contains("INSERT INTO"), "应含 INSERT");

        // H2 内存库逐句执行,验证 DDL + INSERT 全部成功
        try (Connection conn = DriverManager.getConnection("jdbc:h2:mem:aquah2test")) {
            Statement st = conn.createStatement();
            for (String stmt : sql.split(";")) {
                String s = stmt.trim();
                if (!s.isEmpty()) st.execute(s);
            }
            ResultSet tables = conn.getMetaData().getTables(null, null, "%", new String[]{"TABLE"});
            assertTrue(tables.next(), "应至少建出一张表");
        }
    }

    @Test
    void tableFilter_onlySpecifiedTable() throws Exception {
        String aqua = readResource("test.aqua");
        AquaH2.Project proj = new com.fasterxml.jackson.databind.ObjectMapper()
                .readValue(aqua, AquaH2.Project.class);
        assertFalse(proj.tables().isEmpty(), "test.aqua 应有表");

        String first = proj.tables().get(0).code();
        String another = null;
        for (AquaH2.Table t : proj.tables()) {
            if (!t.code().equalsIgnoreCase(first)) { another = t.code(); break; }
        }

        String sql = new AquaH2(aqua).dataset(readResource("test.data")).table(first).export();
        assertTrue(sql.contains("CREATE TABLE " + first.toUpperCase()), "应含选定表: " + first);
        if (another != null) {
            assertFalse(sql.contains("CREATE TABLE " + another.toUpperCase()),
                    "不应含未选表: " + another);
        }
    }
}
