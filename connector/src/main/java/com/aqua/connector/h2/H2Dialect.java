package com.aqua.connector.h2;

import com.aqua.connector.AbstractJdbcDialect;
import com.aqua.connector.DataType;
import com.aqua.connector.DbConfig;

/**
 * H2 数据库方言实现 - 继承 AbstractJdbcDialect。
 *
 * 特化:URL 格式(内存库/文件库/TCP)、类型映射(H2TypeMapping)。
 */
public class H2Dialect extends AbstractJdbcDialect {

    @Override
    public String name() {
        return "h2";
    }

    @Override
    protected String getDriverClass() {
        return "org.h2.Driver";
    }

    @Override
    protected String buildUrl(DbConfig config) {
        // H2 内存库: jdbc:h2:mem:<database>
        // H2 文件库: jdbc:h2:file:<path>
        // H2 TCP:    jdbc:h2:tcp://<host>:<port>/<database>
        if (config.host == null || config.host.isEmpty() || "mem".equalsIgnoreCase(config.host)) {
            return "jdbc:h2:mem:" + config.database + ";DB_CLOSE_DELAY=-1";
        } else {
            return "jdbc:h2:tcp://" + config.host + ":" + config.port + "/" + config.database;
        }
    }

    @Override
    protected DataType mapType(int jdbcType, String typeName, Integer precision, Integer scale) {
        return H2TypeMapping.map(jdbcType, typeName);
    }
}
