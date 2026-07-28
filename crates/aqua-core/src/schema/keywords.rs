//! SQL 保留字(跨库并集) + Java 关键字,用于 code/prop 命名校验(§保存时校验)。
//!
//! SQL 列表取 ANSI + MySQL + PG + Oracle + H2 的 reserved 并集,偏严不偏松(D4)。
//! 若误判常用词,从此处移除即可。

/// SQL 保留字并集(大写存储,查时大写化 code,大小写不敏感)。
const SQL_RESERVED: &[&str] = &[
    "ACCESS", "ACCESSIBLE", "ADD", "ALL", "ALTER", "ANALYSE", "ANALYZE", "AND", "ANY",
    "ARRAY", "AS", "ASC", "ASENSITIVE", "ASYMMETRIC", "AT", "ATOMIC", "AUDIT", "AUTHORIZATION",
    "BEFORE", "BEGIN", "BETWEEN", "BIGINT", "BINARY", "BIT", "BLOB", "BOOLEAN", "BOTH", "BY",
    "CALL", "CALLED", "CASCADE", "CASCADED", "CASE", "CAST", "CEIL", "CEILING", "CHANGE",
    "CHAR", "CHAR_LENGTH", "CHARACTER", "CHARACTER_LENGTH", "CHECK", "CLUSTER", "CLOB", "CLOSE",
    "COALESCE", "COLLATE", "COLLATION", "COLLECT", "COLUMN", "COMMENT", "COMMIT", "COMPRESS",
    "CONDITION", "CONNECT", "CONNECTION", "CONSTRAINT", "CONSTRAINTS", "CONTINUE", "CONVERT",
    "CORR", "CORRESPONDING", "COUNT", "CREATE", "CROSS", "CUBE", "CUME_DIST",
    "CURRENT_CATALOG", "CURRENT_DATE", "CURRENT_PATH", "CURRENT_ROLE", "CURRENT_SCHEMA",
    "CURRENT_TIME", "CURRENT_TIMESTAMP", "CURRENT_USER", "CURSOR", "CYCLE",
    "DATABASE", "DATABASES", "DATE", "DAY", "DAY_HOUR", "DAY_MICROSECOND", "DAY_MINUTE",
    "DAY_SECOND", "DEALLOCATE", "DEC", "DECIMAL", "DECLARE", "DEFAULT", "DELAYED", "DELETE",
    "DENSE_RANK", "DEREF", "DESC", "DESCRIBE", "DETERMINISTIC", "DISCONNECT", "DISTINCT",
    "DISTINCTROW", "DIV", "DO", "DOUBLE", "DROP", "DUAL", "DYNAMIC",
    "EACH", "ELEMENT", "ELSE", "ELSEIF", "EMPTY", "ENCLOSED", "END", "ESCAPE", "ESCAPED",
    "EVERY", "EXCEPT", "EXCLUSIVE", "EXEC", "EXECUTE", "EXISTS", "EXIT", "EXPLAIN", "EXP",
    "EXTERNAL", "EXTRACT",
    "FALSE", "FETCH", "FILE", "FILTER", "FIRST_VALUE", "FLOAT", "FLOAT4", "FLOAT8", "FLOOR",
    "FOR", "FORCE", "FOREIGN", "FREE", "FROM", "FULL", "FULLTEXT", "FUNCTION", "FUSION",
    "GENERATED", "GET", "GLOBAL", "GRANT", "GROUP", "GROUPING", "GROUPS",
    "HAVING", "HIGH_PRIORITY", "HOLD", "HOUR", "HOUR_MICROSECOND", "HOUR_MINUTE", "HOUR_SECOND",
    "IDENTIFIED", "IF", "IGNORE", "ILIKE", "IMMEDIATE", "IN", "INCREMENT", "INDEX", "INDEXES",
    "INDICATOR", "INFILE", "INNER", "INOUT", "INSENSITIVE", "INSERT", "INT", "INT1", "INT2",
    "INT3", "INT4", "INT8", "INTEGER", "INTERSECT", "INTERSECTION", "INTERVAL", "INTO", "IS",
    "ISNULL", "ITERATE",
    "JOIN", "JSON_TABLE",
    "KEY", "KEYS", "KILL",
    "LAG", "LANGUAGE", "LARGE", "LAST_VALUE", "LATERAL", "LEAD", "LEADING", "LEAVE", "LEFT",
    "LEVEL", "LIKE", "LIKE_REGEX", "LIMIT", "LINEAR", "LINES", "LN", "LOAD", "LOCAL",
    "LOCALTIME", "LOCALTIMESTAMP", "LOCATION", "LOCK", "LONG", "LONGBLOB", "LONGTEXT", "LOOP",
    "LOW_PRIORITY",
    "MASTER_BIND", "MATCH", "MAX", "MAXEXTENTS", "MAXVALUE", "MEDIUMBLOB", "MEDIUMINT",
    "MEDIUMTEXT", "MEMBER", "MERGE", "METHOD", "MIDDLEINT", "MIN", "MINUS", "MINUTE",
    "MINUTE_MICROSECOND", "MINUTE_SECOND", "MLSLABEL", "MOD", "MODE", "MODIFIES", "MODIFY",
    "MODULE", "MONTH", "MULTISET",
    "NATIONAL", "NATURAL", "NCHAR", "NCLOB", "NEW", "NO", "NO_WRITE_TO_BINLOG", "NONE", "NOT",
    "NOTNULL", "NOWAIT", "NTH_VALUE", "NTILE", "NULL", "NULLIF", "NUMERIC", "NVARCHAR2",
    "OCTET_LENGTH", "OF", "OFFLINE", "OFFSET", "OLD", "ON", "ONLINE", "ONLY", "OPEN", "OPTIMIZE",
    "OPTIMIZER_COSTS", "OPTION", "OPTIONALLY", "OR", "ORDER", "OUT", "OUTER", "OUTFILE", "OVER",
    "OVERLAPS", "OVERLAY",
    "PARAMETER", "PARTITION", "PCTFREE", "PERCENT", "PERCENT_RANK", "PERCENTILE_CONT",
    "PERCENTILE_DISC", "PLACING", "POSITION", "POSITION_REGEX", "POWER", "PRECISION", "PREPARE",
    "PRIOR", "PRIVILEGES", "PROCEDURE", "PUBLIC", "PURGE",
    "RANGE", "RANK", "READ", "READS", "READ_WRITE", "REAL", "RECURSIVE", "REF", "REFERENCES",
    "REFERENCING", "REGEXP", "RELEASE", "RENAME", "REPEAT", "REPLACE", "REQUIRE", "RESIGNAL",
    "RESOURCE", "RESTRICT", "RESULT", "RETURN", "RETURNING", "RETURNS", "REVOKE", "RIGHT",
    "RLIKE", "ROLE", "ROLLBACK", "ROLLUP", "ROW", "ROWID", "ROWNUM", "ROW_NUMBER", "ROWS",
    "SAVEPOINT", "SCHEMA", "SCHEMAS", "SCOPE", "SCROLL", "SEARCH", "SECOND", "SECOND_MICROSECOND",
    "SELECT", "SENSITIVE", "SEPARATOR", "SEQUENCE", "SESSION", "SESSION_USER", "SET", "SHARE",
    "SHOW", "SIGNAL", "SIMILAR", "SIZE", "SMALLINT", "SOME", "SPATIAL", "SPECIFIC",
    "SPECIFICTYPE", "SQL", "SQLEXCEPTION", "SQLSTATE", "SQLWARNING", "SQL_BIG_RESULT",
    "SQL_CALC_FOUND_ROWS", "SQL_SMALL_RESULT", "SSL", "START", "STARTING", "STATISTICS",
    "STDDEV_POP", "STDDEV_SAMP", "STORED", "STRAIGHT_JOIN", "SUBMULTISET", "SUBSTRING",
    "SUBSTRING_REGEX", "SUCCESSFUL", "SUM", "SYMMETRIC", "SYNONYM", "SYSDATE", "SYSTEM",
    "SYSTEM_TIME", "SYSTEM_USER",
    "TABLE", "TABLESAMPLE", "TERMINATED", "THEN", "TIME", "TIMESTAMP", "TIMEZONE_HOUR",
    "TIMEZONE_MINUTE", "TINYBLOB", "TINYINT", "TINYTEXT", "TO", "TODAY", "TRAILING", "TRANSLATE",
    "TRANSLATE_REGEX", "TRANSLATION", "TREAT", "TRIGGER", "TRIM", "TRIM_ARRAY", "TRUE", "TRUNCATE",
    "TYPE",
    "UESCAPE", "UID", "UNDO", "UNION", "UNIQUE", "UNLOCK", "UNNEST", "UNSIGNED", "UPDATE",
    "UPPER", "USAGE", "USE", "USER", "USING", "UTC_DATE", "UTC_TIME", "UTC_TIMESTAMP",
    "VALUE", "VALUES", "VAR_POP", "VAR_SAMP", "VARBINARY", "VARCHAR", "VARCHAR2", "VARCHARACTER",
    "VARIADIC", "VARYING", "VERBOSE", "VIEW", "VIRTUAL",
    "WHEN", "WHENEVER", "WHERE", "WHILE", "WIDTH_BUCKET", "WINDOW", "WITH", "WITHIN", "WITHOUT",
    "WRITE",
    "XOR",
    "YEAR", "YEAR_MONTH",
    "ZEROFILL",
];

/// Java 关键字(JLS,全小写)。prop 精确匹配(大小写敏感:class 是关键字,Class 不是)。
const JAVA_KEYWORDS: &[&str] = &[
    "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class", "const",
    "continue", "default", "do", "double", "else", "enum", "extends", "final", "finally",
    "float", "for", "goto", "if", "implements", "import", "instanceof", "int", "interface",
    "long", "native", "new", "package", "private", "protected", "public", "return", "short",
    "static", "strictfp", "super", "switch", "synchronized", "this", "throw", "throws",
    "transient", "try", "void", "volatile", "while", "true", "false", "null",
];

/// code 是否撞 SQL 保留字(大小写不敏感)。
pub fn is_sql_reserved(code: &str) -> bool {
    if code.is_empty() {
        return false;
    }
    let upper = code.to_uppercase();
    SQL_RESERVED.contains(&upper.as_str())
}

/// prop 是否撞 Java 关键字(精确匹配)。
pub fn is_java_keyword(prop: &str) -> bool {
    JAVA_KEYWORDS.contains(&prop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_reserved_case_insensitive() {
        assert!(is_sql_reserved("VALUE"));
        assert!(is_sql_reserved("value"));
        assert!(is_sql_reserved("COMMENT"));
        assert!(is_sql_reserved("Key"));
        assert!(!is_sql_reserved("USER_NAME"));
        assert!(!is_sql_reserved(""));
    }

    #[test]
    fn java_keyword_exact() {
        assert!(is_java_keyword("class"));
        assert!(is_java_keyword("int"));
        assert!(!is_java_keyword("Class")); // 大小写敏感
        assert!(!is_java_keyword("userId"));
    }
}
