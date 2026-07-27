package com.aqua.aquah2;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.InputStream;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

/**
 * 读 .aqua(表结构) + .data(数据集),生成可直接在 H2 执行的 DDL + INSERT SQL。
 * 供消费项目(frs common-test)copy 源码,建 H2 内存表跑单元测试。
 *
 * <p>纯逻辑:只接 String/InputStream(内容),出 SQL String。不碰 IO/不绑 Spring/不建库。
 * 仅依赖 Jackson。类型映射照搬 aqua-core {@code map_h2}(10 种逻辑类型)。
 */
public class AquaH2 {

    private static final ObjectMapper M = new ObjectMapper();

    private final Project project;
    private final List<DataRow> data = new ArrayList<>();
    private final Set<String> tables = new LinkedHashSet<>();
    private final Set<String> groups = new LinkedHashSet<>();

    public AquaH2(String aquaJson) {
        try {
            this.project = M.readValue(aquaJson, Project.class);
        } catch (Exception e) {
            throw new RuntimeException("解析 .aqua 失败", e);
        }
    }

    public AquaH2(InputStream aquaIn) {
        try {
            this.project = M.readValue(aquaIn, Project.class);
        } catch (Exception e) {
            throw new RuntimeException("解析 .aqua 失败", e);
        }
    }

    /** 追加数据集(JSONL,每行 {table, row})。可多次调用累加。 */
    public AquaH2 dataset(String jsonl) {
        for (String line : jsonl.split("\n")) {
            line = line.trim();
            if (line.isEmpty()) continue;
            try {
                data.add(M.readValue(line, DataRow.class));
            } catch (Exception e) {
                throw new RuntimeException("解析 .data 行失败: " + line, e);
            }
        }
        return this;
    }

    public AquaH2 dataset(InputStream in) {
        try {
            return dataset(new String(in.readAllBytes()));
        } catch (Exception e) {
            throw new RuntimeException("读 .data 失败", e);
        }
    }

    /** 累加要导出的表(code)。 */
    public AquaH2 table(String... tables) {
        Collections.addAll(this.tables, tables);
        return this;
    }

    /** 累加要导出的分组(group code,导出该组下所有表)。 */
    public AquaH2 group(String... groups) {
        Collections.addAll(this.groups, groups);
        return this;
    }

    /**
     * 导出 SQL。table/group 都空 = 全部表;可混合(累加,合并去重)。
     * 数据集只插过滤范围内表的数据;空表只 DDL 不 INSERT。
     */
    public String export() {
        List<Table> targets = resolveTables();
        StringBuilder sb = new StringBuilder();
        Set<String> targetCodes = new HashSet<>();
        for (Table t : targets) {
            targetCodes.add(t.code.toUpperCase());
            sb.append(createTable(t));
            if (t.indexes != null) {
                for (Index idx : t.indexes) sb.append(createIndex(t, idx));
            }
        }
        for (DataRow row : data) {
            if (row.table == null) continue;
            if (!targetCodes.contains(row.table.toUpperCase())) continue;
            Table t = findTable(targets, row.table);
            if (t != null) sb.append(insert(t, row.row));
        }
        return sb.toString();
    }

    private List<Table> resolveTables() {
        if (tables.isEmpty() && groups.isEmpty()) {
            return project.tables != null ? project.tables : List.of();
        }
        Set<String> want = new LinkedHashSet<>(tables);
        if (project.tables != null) {
            for (Table t : project.tables) {
                if (t.group != null && groups.contains(t.group)) want.add(t.code);
            }
        }
        List<Table> result = new ArrayList<>();
        if (project.tables != null) {
            for (Table t : project.tables) {
                if (want.contains(t.code)) result.add(t);
            }
        }
        return result;
    }

    private Table findTable(List<Table> targets, String code) {
        for (Table t : targets) {
            if (t.code.equalsIgnoreCase(code)) return t;
        }
        return null;
    }

    // ===== DDL =====

    /** CREATE TABLE(列定义 + PRIMARY KEY)。 */
    private String createTable(Table t) {
        List<String> defs = new ArrayList<>();
        for (Field f : t.fields) defs.add("  " + fieldDef(f));
        List<String> pk = new ArrayList<>();
        for (Field f : t.fields) {
            if (Boolean.TRUE.equals(f.isKey)) pk.add(f.code.toUpperCase());
        }
        if (!pk.isEmpty()) defs.add("  PRIMARY KEY (" + String.join(", ", pk) + ")");
        return "CREATE TABLE " + t.code.toUpperCase() + " (\n"
                + String.join(",\n", defs) + "\n);\n";
    }

    /** 字段定义: CODE TYPE [DEFAULT xxx] [NOT NULL]。DEFAULT 在 NOT NULL 前(Oracle 兼容)。 */
    private String fieldDef(Field f) {
        String type = mapH2(f);
        String expr = defaultExpr(f.defaultValue);
        String def = expr != null ? " DEFAULT " + expr : "";
        String notNull = Boolean.TRUE.equals(f.notNull) ? " NOT NULL" : "";
        return f.code.toUpperCase() + " " + type + def + notNull;
    }

    /**
     * DEFAULT 表达式: 数字/布尔/NULL/已引号/函数(含括号或 CURRENT_/NOW)原值;
     * 裸字符串(如用户未加引号的 Abc)自动加引号,兼容 .aqua 数据。
     */
    private String defaultExpr(String v) {
        if (v == null || v.isEmpty()) return null;
        String upper = v.toUpperCase();
        if (v.startsWith("'")
                || v.matches("-?\\d+(\\.\\d+)?")
                || upper.equals("NULL") || upper.equals("TRUE") || upper.equals("FALSE")
                || upper.startsWith("CURRENT_") || upper.equals("NOW")
                || upper.contains("(")) {
            return v;
        }
        return "'" + escape(v) + "'";
    }

    /** CREATE [UNIQUE] INDEX name ON table (F1 [DESC], ...)。name 空则 IDX_TABLE_F1_F2。 */
    private String createIndex(Table t, Index idx) {
        List<String> cols = new ArrayList<>();
        for (IndexField f : idx.fields) {
            String c = f.code.toUpperCase();
            cols.add("DESC".equalsIgnoreCase(f.direction) ? c + " DESC" : c);
        }
        String unique = idx.unique ? "UNIQUE " : "";
        String name = (idx.name != null && !idx.name.isEmpty()) ? idx.name.toUpperCase()
                : ("IDX_" + t.code + "_" + idx.fields.stream().map(f -> f.code).reduce((a, b) -> a + "_" + b).orElse(""))
                        .toUpperCase();
        return "CREATE " + unique + "INDEX " + name + " ON " + t.code.toUpperCase()
                + " (" + String.join(", ", cols) + ");\n";
    }

    // ===== INSERT =====

    private String insert(Table t, Map<String, Object> row) {
        List<String> cols = new ArrayList<>();
        List<String> vals = new ArrayList<>();
        for (Map.Entry<String, Object> e : row.entrySet()) {
            cols.add(e.getKey().toUpperCase());
            vals.add(literal(e.getValue()));
        }
        return "INSERT INTO " + t.code.toUpperCase() + " (" + String.join(", ", cols)
                + ") VALUES (" + String.join(", ", vals) + ");\n";
    }

    /** 值字面量: null->NULL, 数字/布尔直, 其余字符串(单引号转义)。 */
    private String literal(Object v) {
        if (v == null) return "NULL";
        if (v instanceof Number || v instanceof Boolean) return v.toString();
        return "'" + escape(v.toString()) + "'";
    }

    private String escape(String s) {
        return s.replace("'", "''");
    }

    // ===== 类型映射(照搬 aqua-core map_h2,10 种) =====

    private String mapH2(Field f) {
        String dt = f.dataType == null ? "" : f.dataType.toUpperCase();
        switch (dt) {
            case "VARCHAR": return "VARCHAR(" + (f.length != null ? f.length : 255) + ")";
            case "CLOB": return "CLOB";
            case "TINYINT": return "TINYINT";
            case "INT": return "INT";
            case "LONG": return "BIGINT";
            case "DECIMAL":
                if (f.precision != null) {
                    return "DECIMAL(" + f.precision + ", " + (f.scale != null ? f.scale : 0) + ")";
                }
                return "DECIMAL";
            case "DOUBLE": return "DOUBLE";
            case "DATE": return "DATE";
            case "DATETIME": return "TIMESTAMP";
            case "BLOB": return "BLOB";
            default: return "VARCHAR(255)";
        }
    }

    // ===== record POJO(对齐 aqua schema,未知字段忽略) =====

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Project(List<Table> tables, List<Group> groups) {}

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Table(String code, String name, String group,
                        List<Field> fields, List<Index> indexes, String comment) {}

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Field(String code, String prop, String name, String dataType,
                        Integer length, Integer precision, Integer scale,
                        Boolean isKey, Boolean notNull, String defaultValue, String comment) {}

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Index(String name, List<IndexField> fields, boolean unique) {}

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record IndexField(String code, String direction) {}

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Group(String code, String name) {}

    @JsonIgnoreProperties(ignoreUnknown = true)
    private record DataRow(String table, Map<String, Object> row) {}
}
