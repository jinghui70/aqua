package com.aqua.connector.h2;

import java.sql.Connection;
import java.sql.DatabaseMetaData;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import com.aqua.connector.DataType;
import com.aqua.connector.DbConfig;
import com.aqua.connector.Dialect;
import com.aqua.connector.meta.ColumnMeta;
import com.aqua.connector.meta.IndexMeta;

/**
 * H2 数据库方言实现。
 *
 * 反解 H2 JDBC 元数据为 aqua 逻辑类型(v2 架构:反解在 Java 侧)。
 */
public class H2Dialect implements Dialect {

    @Override
    public String name() {
        return "h2";
    }

    @Override
    public Connection connect(DbConfig config) throws SQLException {
        // H2 内存库: jdbc:h2:mem:<database>
        // H2 文件库: jdbc:h2:file:<path>
        // H2 TCP:    jdbc:h2:tcp://<host>:<port>/<database>
        String url;
        if (config.host == null || config.host.isEmpty() || "mem".equalsIgnoreCase(config.host)) {
            url = "jdbc:h2:mem:" + config.database + ";DB_CLOSE_DELAY=-1";
        } else {
            url = "jdbc:h2:tcp://" + config.host + ":" + config.port + "/" + config.database;
        }
        String user = config.user == null || config.user.isEmpty() ? "sa" : config.user;
        return DriverManager.getConnection(url, user, config.password == null ? "" : config.password);
    }

    @Override
    public List<String> listTables(Connection conn, String schema) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        List<String> tables = new ArrayList<>();
        try (ResultSet rs = meta.getTables(conn.getCatalog(), schema, "%",
                new String[]{"TABLE"})) {
            while (rs.next()) {
                tables.add(rs.getString("TABLE_NAME"));
            }
        }
        return tables;
    }

    @Override
    public List<ColumnMeta> getColumns(Connection conn, String table) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        // 先取主键列表
        Map<String, Boolean> pkMap = new HashMap<>();
        try (ResultSet pkRs = meta.getPrimaryKeys(conn.getCatalog(), conn.getSchema(), table)) {
            while (pkRs.next()) {
                pkMap.put(pkRs.getString("COLUMN_NAME"), true);
            }
        }

        List<ColumnMeta> columns = new ArrayList<>();
        try (ResultSet rs = meta.getColumns(conn.getCatalog(), conn.getSchema(), table, "%")) {
            while (rs.next()) {
                String colName = rs.getString("COLUMN_NAME");
                int jdbcType = rs.getInt("DATA_TYPE");
                String typeName = rs.getString("TYPE_NAME");
                int length = rs.getInt("COLUMN_SIZE");
                int precision = length;
                int scale = rs.getInt("DECIMAL_DIGITS");
                boolean nullable = rs.getInt("NULLABLE") == DatabaseMetaData.columnNullable;
                String defaultValue = rs.getString("COLUMN_DEF");
                String comment = rs.getString("REMARKS");

                columns.add(new ColumnMeta(
                        colName,
                        H2TypeMapping.map(jdbcType, typeName),
                        jdbcType == java.sql.Types.VARCHAR || jdbcType == java.sql.Types.CHAR ? length : null,
                        jdbcType == java.sql.Types.DECIMAL || jdbcType == java.sql.Types.NUMERIC ? precision : null,
                        jdbcType == java.sql.Types.DECIMAL || jdbcType == java.sql.Types.NUMERIC ? scale : null,
                        nullable,
                        pkMap.getOrDefault(colName, false),
                        defaultValue,
                        comment
                ));
            }
        }
        return columns;
    }

    @Override
    public List<IndexMeta> getIndexes(Connection conn, String table) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        // 取主键约束名,用于跳过主键索引
        String pkName = null;
        try (ResultSet pkRs = meta.getPrimaryKeys(conn.getCatalog(), conn.getSchema(), table)) {
            if (pkRs.next()) {
                pkName = pkRs.getString("PK_NAME");
            }
        }

        // 按索引名分组字段
        Map<String, List<String>> idxFields = new HashMap<>();
        Map<String, Boolean> idxUnique = new HashMap<>();
        try (ResultSet rs = meta.getIndexInfo(conn.getCatalog(), conn.getSchema(), table, false, false)) {
            while (rs.next()) {
                String idxName = rs.getString("INDEX_NAME");
                if (idxName == null) continue;
                // 跳过主键索引(主键已在 ColumnMeta.isKey 处理)
                if (pkName != null && pkName.equals(idxName)) continue;
                String colName = rs.getString("COLUMN_NAME");
                idxFields.computeIfAbsent(idxName, k -> new ArrayList<>()).add(colName);
                idxUnique.put(idxName, !rs.getBoolean("NON_UNIQUE"));
            }
        }
        List<IndexMeta> indexes = new ArrayList<>();
        for (Map.Entry<String, List<String>> e : idxFields.entrySet()) {
            indexes.add(new IndexMeta(e.getKey(), e.getValue(), idxUnique.getOrDefault(e.getKey(), false)));
        }
        return indexes;
    }
}
