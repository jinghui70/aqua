package com.aqua.connector.registry;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests {@link RegistryLoader} — loading registry.json and scanning the drivers/ directory.
 */
public class RegistryLoaderTest {

    @Test
    void shouldLoadBuiltinRegistry() throws Exception {
        RegistryLoader loader = new RegistryLoader();
        loader.load();

        assertNotNull(loader.getRegistry());
        // All 7 preset entries must be present
        assertNotNull(loader.getEntry("mysql"));
        assertNotNull(loader.getEntry("postgresql"));
        assertNotNull(loader.getEntry("oracle"));
        assertNotNull(loader.getEntry("dm"));
        assertNotNull(loader.getEntry("kingbase"));
        assertNotNull(loader.getEntry("gbase"));
        assertNotNull(loader.getEntry("h2"));

        // Verify key entries have expected driver class
        assertEquals("com.mysql.cj.jdbc.Driver", loader.getEntry("mysql").getDriverClass());
        assertEquals("org.postgresql.Driver", loader.getEntry("postgresql").getDriverClass());
        assertEquals("oracle.jdbc.OracleDriver", loader.getEntry("oracle").getDriverClass());
        assertEquals("org.h2.Driver", loader.getEntry("h2").getDriverClass());
    }

    @Test
    void shouldPresetDriverClassForChineseDbs() throws Exception {
        RegistryLoader loader = new RegistryLoader();
        loader.load();

        assertEquals("dm.jdbc.driver.DmDriver", loader.getEntry("dm").getDriverClass());
        assertEquals("com.kingbase8.Driver", loader.getEntry("kingbase").getDriverClass());
        assertEquals("com.gbasedbt.jdbc.Driver", loader.getEntry("gbase").getDriverClass());
    }

    @Test
    void shouldHaveUrlPatterns() throws Exception {
        RegistryLoader loader = new RegistryLoader();
        loader.load();

        assertNotNull(loader.getEntry("mysql").getUrlPattern());
        assertTrue(loader.getEntry("h2").getUrlPattern().contains("jdbc:h2:mem:"));
    }

    @Test
    void shouldReturnNullForUnknownType() throws Exception {
        RegistryLoader loader = new RegistryLoader();
        loader.load();

        assertNull(loader.getEntry("unknown_db"));
    }

    @Test
    void shouldHaveNonNullDriverClassLoader() throws Exception {
        RegistryLoader loader = new RegistryLoader();
        loader.load();

        assertNotNull(loader.getDriverClassLoader());
    }
}
