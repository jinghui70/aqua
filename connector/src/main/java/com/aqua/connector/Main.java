package com.aqua.connector;

import java.sql.Connection;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import com.aqua.connector.meta.ColumnMeta;
import com.aqua.connector.meta.IndexMeta;
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
        try {
            // 1. 读 stdin
            String stdin = readStdin();
            DbConfig config = MAPPER.readValue(stdin, DbConfig.class);
            String action = MAPPER.readTree(stdin).get("action").asText();

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
