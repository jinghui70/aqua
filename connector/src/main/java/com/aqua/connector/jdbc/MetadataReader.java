package com.aqua.connector.jdbc;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.sql.*;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Reads raw JDBC metadata (tables, columns, primary keys, indexes).
 *
 * <p>Outputs <em>raw</em> physical types — NO type resolution (that happens
 * in the main process via resolver JS).
 */
public class MetadataReader {

    private static final ObjectMapper MAPPER = new ObjectMapper();

    public MetadataReader() {}

    /**
     * Read metadata for all tables (or a filtered subset) and return as JSON.
     *
     * @param conn   active JDBC connection
     * @param tables optional table filter (may be null/empty for "all")
     * @return JSON ObjectNode with {@code { tables: [...] }}
     */
    public ObjectNode readMetadata(Connection conn, List<String> tables) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        ObjectNode root = MAPPER.createObjectNode();
        ArrayNode tablesArr = root.putArray("tables");

        // Determine which tables to process
        List<String[]> tableInfos = new ArrayList<>();
        String catalog = conn.getCatalog();
        String schema = getSchema(conn, meta);

        try (ResultSet rs = meta.getTables(catalog, schema, "%", new String[]{"TABLE", "VIEW"})) {
            while (rs.next()) {
                String tableName = rs.getString("TABLE_NAME");
                if (tables != null && !tables.isEmpty() && !tables.contains(tableName)) {
                    continue;
                }
                tableInfos.add(new String[]{
                        rs.getString("TABLE_CAT"),
                        rs.getString("TABLE_SCHEM"),
                        tableName,
                        rs.getString("REMARKS")
                });
            }
        }

        for (String[] ti : tableInfos) {
            String tabCat = ti[0];
            String tabSchem = ti[1];
            String tabName = ti[2];
            String remarks = ti[3];

            ObjectNode tableNode = tablesArr.addObject();
            tableNode.put("code", tabName);
            tableNode.put("name", (remarks != null && !remarks.isEmpty()) ? remarks : tabName);

            // Columns
            ArrayNode columnsArr = tableNode.putArray("columns");
            List<String> pkCols = readPkColumns(meta, tabCat, tabSchem, tabName);

            try (ResultSet colRs = meta.getColumns(tabCat, tabSchem, tabName, "%")) {
                while (colRs.next()) {
                    ObjectNode colNode = columnsArr.addObject();
                    String colName = colRs.getString("COLUMN_NAME");
                    colNode.put("name", colName);
                    // Raw JDBC physical type name (e.g. VARCHAR, NUMBER)
                    colNode.put("rawType", colRs.getString("TYPE_NAME"));
                    colNode.put("length", colRs.getInt("COLUMN_SIZE"));
                    colNode.put("precision", colRs.getInt("COLUMN_SIZE"));
                    colNode.put("scale", colRs.getInt("DECIMAL_DIGITS"));
                    colNode.put("nullable",
                            colRs.getInt("NULLABLE") == DatabaseMetaData.columnNullable);
                    colNode.put("pk", pkCols.contains(colName));
                }
            }

            // Indexes
            ArrayNode indexesArr = tableNode.putArray("indexes");
            Map<String, ObjectNode> indexMap = new LinkedHashMap<>();
            try (ResultSet idxRs = meta.getIndexInfo(tabCat, tabSchem, tabName, false, false)) {
                while (idxRs.next()) {
                    String idxName = idxRs.getString("INDEX_NAME");
                    if (idxName == null) continue; // skip table statistics entries
                    boolean nonUnique = idxRs.getBoolean("NON_UNIQUE");
                    String colName = idxRs.getString("COLUMN_NAME");
                    ObjectNode idxNode = indexMap.computeIfAbsent(idxName, k -> {
                        ObjectNode n = MAPPER.createObjectNode();
                        n.put("name", k);
                        n.putArray("fields");
                        n.put("unique", !nonUnique);
                        return n;
                    });
                    ((ArrayNode) idxNode.get("fields")).add(colName);
                }
            }
            for (ObjectNode idxNode : indexMap.values()) {
                indexesArr.add(idxNode);
            }
        }

        return root;
    }

    private List<String> readPkColumns(DatabaseMetaData meta,
                                        String catalog, String schema, String table)
            throws SQLException {
        List<String> pks = new ArrayList<>();
        try (ResultSet rs = meta.getPrimaryKeys(catalog, schema, table)) {
            while (rs.next()) {
                pks.add(rs.getString("COLUMN_NAME"));
            }
        }
        return pks;
    }

    private String getSchema(Connection conn, DatabaseMetaData meta) throws SQLException {
        try {
            return conn.getSchema();
        } catch (Exception e) {
            return null;
        }
    }
}
