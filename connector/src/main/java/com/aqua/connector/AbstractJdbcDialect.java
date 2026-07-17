package com.aqua.connector;

import java.sql.Connection;
import java.sql.DatabaseMetaData;
import java.sql.Driver;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Properties;

import com.aqua.connector.meta.ColumnMeta;
import com.aqua.connector.meta.IndexMeta;

/**
 * JDBC 方言抽象基类 - 提取标准 JDBC 元数据遍历逻辑。
 *
 * 子类只需填三个"洞":
 * - buildUrl(DbConfig): 构造 JDBC URL
 * - mapType(jdbcType, typeName, precision, scale): 物理类型 -> aqua 逻辑类型
 * - resolveSchema(Connection, schema): schema 解析钩子(默认直接用传入值)
 */
public abstract class AbstractJdbcDialect implements Dialect {

    /** 子类返回 JDBC 驱动类全限定名 */
    protected abstract String getDriverClass();

    /** 子类构造 JDBC URL */
    protected abstract String buildUrl(DbConfig config);

    /** 子类映射类型(jdbcType 来自 java.sql.Types, precision/scale 可能为 null) */
    protected abstract DataType mapType(int jdbcType, String typeName, Integer precision, Integer scale);

    /** 子类覆写 schema 解析逻辑(默认直接用传入 schema,Oracle 覆写为 conn.getSchema()) */
    protected String resolveSchema(Connection conn, String schema) throws SQLException {
        return schema;
    }

    /**
     * 解析表的列注释补充(默认空,用 ResultSetMetaData REMARKS)。
     * Oracle 等方言覆写:REMARKS 常为空,从数据字典(如 USER_COL_COMMENTS)批量补查。
     * 返回 columnName -> comment;getColumns 中 REMARKS 为空时取用。
     */
    protected Map<String, String> resolveColumnComments(Connection conn, String schema, String table) throws SQLException {
        return Collections.emptyMap();
    }

    @Override
    public final Connection connect(DbConfig config) throws SQLException {
        String url = buildUrl(config);
        try {
            ClassLoader cl = Thread.currentThread().getContextClassLoader();
            if (cl == null) cl = getClass().getClassLoader();
            Driver driver = (Driver) Class.forName(getDriverClass(), true, cl)
                    .getDeclaredConstructor().newInstance();
            Properties props = new Properties();
            props.setProperty("user", config.user);
            props.setProperty("password", config.password);
            return driver.connect(url, props);
        } catch (SQLException e) {
            throw e;
        } catch (Exception e) {
            throw new SQLException("加载驱动失败(需先安装 " + name() + " 驱动): " + e.getMessage(), e);
        }
    }

    @Override
    public final List<String> listTables(Connection conn, String schema) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        String resolvedSchema = resolveSchema(conn, schema);
        List<String> tables = new ArrayList<>();
        try (ResultSet rs = meta.getTables(conn.getCatalog(), resolvedSchema, "%", new String[]{"TABLE"})) {
            while (rs.next()) {
                tables.add(rs.getString("TABLE_NAME"));
            }
        }
        return tables;
    }

    @Override
    public final List<ColumnMeta> getColumns(Connection conn, String table) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        String schema = resolveSchema(conn, conn.getSchema());

        // 先取主键列表
        Map<String, Boolean> pkMap = new HashMap<>();
        try (ResultSet pkRs = meta.getPrimaryKeys(conn.getCatalog(), schema, table)) {
            while (pkRs.next()) {
                pkMap.put(pkRs.getString("COLUMN_NAME"), true);
            }
        }

        List<ColumnMeta> columns = new ArrayList<>();
        // Oracle 等方言 REMARKS 为空,从数据字典批量补查列注释
        Map<String, String> extraComments = resolveColumnComments(conn, schema, table);
        try (ResultSet rs = meta.getColumns(conn.getCatalog(), schema, table, "%")) {
            while (rs.next()) {
                String colName = rs.getString("COLUMN_NAME");
                int jdbcType = rs.getInt("DATA_TYPE");
                String typeName = rs.getString("TYPE_NAME");
                int length = rs.getInt("COLUMN_SIZE");
                int precision = length;
                int scale = rs.getInt("DECIMAL_DIGITS");
                boolean nullable = rs.getInt("NULLABLE") == DatabaseMetaData.columnNullable;
                String defaultValue = rs.getString("COLUMN_DEF");
                String remarks = rs.getString("REMARKS");
                String comment = (remarks != null && !remarks.isBlank())
                        ? remarks
                        : extraComments.get(colName);

                columns.add(new ColumnMeta(
                        colName,
                        mapType(jdbcType, typeName, precision, scale),
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
    public final List<IndexMeta> getIndexes(Connection conn, String table) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        String schema = resolveSchema(conn, conn.getSchema());

        // 取主键约束名,用于跳过主键索引
        String pkName = null;
        try (ResultSet pkRs = meta.getPrimaryKeys(conn.getCatalog(), schema, table)) {
            if (pkRs.next()) {
                pkName = pkRs.getString("PK_NAME");
            }
        }

        // 按索引名分组字段
        Map<String, List<String>> idxFields = new HashMap<>();
        Map<String, Boolean> idxUnique = new HashMap<>();
        try (ResultSet rs = meta.getIndexInfo(conn.getCatalog(), schema, table, false, false)) {
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
