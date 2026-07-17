package com.aqua.connector;

import java.io.File;
import java.io.FileDescriptor;
import java.io.FileOutputStream;
import java.io.PrintStream;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.sql.Connection;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import com.aqua.connector.meta.ColumnMeta;
import com.aqua.connector.meta.IndexMeta;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

/**
 * connector.jar 入口:读 stdin JSON,分发 action,写 stdout JSON。
 *
 * 协议(对齐 Rust driver/jdbc.rs):
 * 请求: {action, dialect, host, port, user, password, database, schema, table}
 * action: testConnection / listTables / getColumns / listIndexes
 * 响应: {status:"ok"} / {tables:[...]} / {columns:[...]} / {indexes:[...]} / {error:"..."}
 */
public class Main {

    private static final ObjectMapper MAPPER = new ObjectMapper();

    public static void main(String[] args) {
        // Windows 中文系统 System.out 默认 GBK 编码,JSON 响应含中文(如列注释)时输出 GBK 字节,
        // Rust 端按 UTF-8 解析失败("invalid unicode code point")。强制 stdout/stderr 为 UTF-8。
        System.setOut(new PrintStream(new FileOutputStream(FileDescriptor.out), true, StandardCharsets.UTF_8));
        System.setErr(new PrintStream(new FileOutputStream(FileDescriptor.err), true, StandardCharsets.UTF_8));
        try {
            // 1. 读 stdin
            String stdin = readStdin();
            JsonNode root = MAPPER.readTree(stdin);
            DbConfig config = MAPPER.treeToValue(root, DbConfig.class);
            String action = root.get("action").asText();
            String driversDir = root.has("driversDir") ? root.get("driversDir").asText() : null;

            // 1.5 加载外置 JDBC 驱动(drivers/*.jar,Oracle 等)
            if (driversDir != null) {
                loadDrivers(driversDir);
            }

            // 2. 获取 Dialect
            Dialect dialect = DialectRegistry.get(config.dialect);
            if (dialect == null) {
                writeError("不支持的方言: " + config.dialect);
                System.exit(1);
                return;
            }

            // 3. 分发 action
            ObjectNode response;
            try (Connection conn = dialect.connect(config)) {
                response = dispatch(action, dialect, conn, config);
            }
            System.out.println(MAPPER.writeValueAsString(response));
        } catch (Exception e) {
            try {
                writeError(e.getMessage() == null ? e.toString() : e.getMessage());
            } catch (Exception ignored) {}
            System.exit(1);
        }
    }

    private static ObjectNode dispatch(String action, Dialect dialect, Connection conn, DbConfig config)
            throws Exception {
        switch (action) {
            case "testConnection": {
                ObjectNode resp = MAPPER.createObjectNode();
                resp.put("status", "ok");
                return resp;
            }
            case "listTables": {
                List<String> tables = dialect.listTables(conn, config.schema);
                ObjectNode resp = MAPPER.createObjectNode();
                ArrayNode arr = resp.putArray("tables");
                tables.forEach(arr::add);
                return resp;
            }
            case "getColumns": {
                List<ColumnMeta> columns = dialect.getColumns(conn, config.table);
                ObjectNode resp = MAPPER.createObjectNode();
                ArrayNode arr = resp.putArray("columns");
                for (ColumnMeta c : columns) {
                    arr.addPOJO(c);
                }
                return resp;
            }
            case "listIndexes": {
                List<IndexMeta> indexes = dialect.getIndexes(conn, config.table);
                ObjectNode resp = MAPPER.createObjectNode();
                ArrayNode arr = resp.putArray("indexes");
                for (IndexMeta i : indexes) {
                    arr.addPOJO(i);
                }
                return resp;
            }
            default:
                throw new IllegalArgumentException("未知 action: " + action);
        }
    }

    private static void writeError(String msg) throws Exception {
        ObjectNode resp = MAPPER.createObjectNode();
        resp.put("error", msg);
        System.out.println(MAPPER.writeValueAsString(resp));
    }

    /** 读 drivers/databases.json,用 URLClassLoader 加载 installed 的 JDBC jar。 */
    private static void loadDrivers(String driversDir) throws Exception {
        File dbFile = new File(driversDir, "databases.json");
        if (!dbFile.exists()) return;
        JsonNode root = MAPPER.readTree(dbFile);
        JsonNode installed = root.get("installed");
        if (installed == null || !installed.isArray() || installed.size() == 0) return;

        List<URL> urls = new ArrayList<>();
        for (JsonNode item : installed) {
            String jar = item.get("driverJar").asText();
            urls.add(new File(driversDir, jar).toURI().toURL());
        }
        URLClassLoader cl = new URLClassLoader(urls.toArray(new URL[0]), Main.class.getClassLoader());
        Thread.currentThread().setContextClassLoader(cl);
        // 显式加载 Driver 类触发 DriverManager 注册
        for (JsonNode item : installed) {
            if (item.has("driverClass") && !item.get("driverClass").asText().isEmpty()) {
                Class.forName(item.get("driverClass").asText(), true, cl);
            }
        }
    }

    private static String readStdin() throws Exception {
        StringBuilder sb = new StringBuilder();
        int ch;
        while ((ch = System.in.read()) != -1) {
            sb.append((char) ch);
            if (System.in.available() == 0) break;
        }
        return sb.toString();
    }
}
