package com.aqua.connector.jdbc;

import com.aqua.connector.protocol.DataSource;
import com.aqua.connector.registry.RegistryEntry;
import com.aqua.connector.registry.RegistryLoader;

import java.net.URLClassLoader;
import java.sql.Connection;
import java.sql.Driver;
import java.sql.DriverManager;
import java.util.Properties;

/**
 * Creates JDBC connections dynamically using registry config and external drivers.
 */
public class ConnectionFactory {

    private final RegistryLoader registryLoader;

    public ConnectionFactory(RegistryLoader registryLoader) {
        this.registryLoader = registryLoader;
    }

    /**
     * Open a JDBC connection for the given {@link DataSource}.
     *
     * @throws Exception if the db type is unsupported, driver class not found, or connection fails
     */
    public Connection connect(DataSource ds) throws Exception {
        RegistryEntry entry = registryLoader.getEntry(ds.getType());
        if (entry == null) {
            throw new IllegalArgumentException("Unsupported database type: " + ds.getType()
                    + " (add it to registry.json)");
        }

        String url = buildUrl(entry.getUrlPattern(), ds);

        // Try to load the driver — first via the external URLClassLoader (drivers/),
        // then fallback to Class.forName (catches H2 which is built-in).
        Driver driver = loadDriver(entry.getDriverClass());

        Properties props = new Properties();
        props.setProperty("user", ds.getUser());
        props.setProperty("password", ds.getPassword());

        if (driver != null) {
            return driver.connect(url, props);
        }
        return DriverManager.getConnection(url, props);
    }

    private Driver loadDriver(String driverClass) throws Exception {
        URLClassLoader cl = registryLoader.getDriverClassLoader();
        if (cl != null) {
            try {
                Class<?> cls = Class.forName(driverClass, true, cl);
                return (Driver) cls.getDeclaredConstructor().newInstance();
            } catch (ClassNotFoundException e) {
                // Not in external classloader, try built-in
            }
        }
        try {
            Class<?> cls = Class.forName(driverClass);
            return (Driver) cls.getDeclaredConstructor().newInstance();
        } catch (ClassNotFoundException e) {
            throw new ClassNotFoundException("JDBC driver class not found: " + driverClass
                    + ". Make sure the driver jar is in drivers/ or on classpath.");
        }
    }

    String buildUrl(String urlPattern, DataSource ds) {
        return urlPattern
                .replace("{host}", ds.getHost())
                .replace("{port}", String.valueOf(ds.getPort()))
                .replace("{database}", ds.getDatabase());
    }
}
