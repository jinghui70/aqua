package com.aqua.connector.oracle;

import java.sql.Connection;
import java.sql.DatabaseMetaData;
import java.sql.Driver;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Properties;

import com.aqua.connector.DataType;
import com.aqua.connector.DbConfig;
import com.aqua.connector.Dialect;
import com.aqua.connector.meta.ColumnMeta;
import com.aqua.connector.meta.IndexMeta;

/**
 * Oracle 数据库方言实现。
 *
 * 反解 Oracle JDBC 元数据为 aqua 逻辑类型。
 * 连接 URL: jdbc:oracle:thin:@//host:port/service_name
 */
public class OracleDialect implements Dialect {

    @Override
    public String name() {
        return "oracle";
    }

    @Override
    public Connection connect(DbConfig config) throws SQLException {
        String url = "jdbc:oracle:thin:@//" + config.host + ":" + config.port + "/" + config.database;
        // 外置 ojdbc 由 Main 的 URLClassLoader 加载并设为 contextClassLoader。
        // 直接调 driver.connect 绕过 DriverManager —— 其类加载器隔离会跳过
        // URLClassLoader 加载的 driver,导致 "No suitable driver found"。
        try {
            ClassLoader cl = Thread.currentThread().getContextClassLoader();
            Driver driver = (Driver) Class.forName("oracle.jdbc.OracleDriver", true, cl)
                    .getDeclaredConstructor().newInstance();
            Properties props = new Properties();
            props.setProperty("user", config.user);
            props.setProperty("password", config.password);
            return driver.connect(url, props);
        } catch (SQLException e) {
            throw e;
        } catch (Exception e) {
            throw new SQLException("加载 Oracle 驱动失败(需先安装 ojdbc): " + e.getMessage(), e);
        }
    }

    @Override
    public List<String> listTables(Connection conn, String schema) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        // Oracle schema = 当前登录用户;传入的 schema 实为 service name(database),
        // 不可用作 schema 过滤,否则 getTables 查不到表。与 getColumns/getIndexes 一致用当前用户。
        String schemaPattern = conn.getSchema();
        List<String> tables = new ArrayList<>();
        try (ResultSet rs = meta.getTables(null, schemaPattern, "%", new String[]{"TABLE"})) {
            while (rs.next()) {
                tables.add(rs.getString("TABLE_NAME"));
            }
        }
        return tables;
    }

    @Override
    public List<ColumnMeta> getColumns(Connection conn, String table) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        String schema = conn.getSchema();
        Map<String, Boolean> pkMap = new HashMap<>();
        try (ResultSet pkRs = meta.getPrimaryKeys(null, schema, table)) {
            while (pkRs.next()) {
                pkMap.put(pkRs.getString("COLUMN_NAME"), true);
            }
        }

        List<ColumnMeta> columns = new ArrayList<>();
        try (ResultSet rs = meta.getColumns(null, schema, table, "%")) {
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

                DataType dt = OracleTypeMapping.map(jdbcType, typeName, precision, scale);
                Integer lenVal = (jdbcType == java.sql.Types.VARCHAR || jdbcType == java.sql.Types.CHAR)
                        ? length : null;
                Integer precVal = (jdbcType == java.sql.Types.DECIMAL || jdbcType == java.sql.Types.NUMERIC)
                        ? precision : null;
                Integer scaleVal = (jdbcType == java.sql.Types.DECIMAL || jdbcType == java.sql.Types.NUMERIC)
                        ? scale : null;

                columns.add(new ColumnMeta(
                        colName, dt, lenVal, precVal, scaleVal,
                        nullable, pkMap.getOrDefault(colName, false),
                        defaultValue, comment
                ));
            }
        }
        return columns;
    }

    @Override
    public List<IndexMeta> getIndexes(Connection conn, String table) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        String schema = conn.getSchema();
        String pkName = null;
        try (ResultSet pkRs = meta.getPrimaryKeys(null, schema, table)) {
            if (pkRs.next()) {
                pkName = pkRs.getString("PK_NAME");
            }
        }

        Map<String, List<String>> idxFields = new HashMap<>();
        Map<String, Boolean> idxUnique = new HashMap<>();
        try (ResultSet rs = meta.getIndexInfo(null, schema, table, false, false)) {
            while (rs.next()) {
                String idxName = rs.getString("INDEX_NAME");
                if (idxName == null) continue;
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
