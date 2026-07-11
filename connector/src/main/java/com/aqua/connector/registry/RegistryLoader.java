package com.aqua.connector.registry;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.*;
import java.net.URL;
import java.net.URLClassLoader;
import java.util.Collections;
import java.util.Map;

/**
 * Loads {@code registry.json} and scans the {@code drivers/} directory,
 * building a URLClassLoader for external JDBC drivers.
 *
 * <p>Loading sequence (on {@link #load()}):
 * <ol>
 *   <li>Read {@code registry.json} from classpath (built-in defaults for 6 dbs + H2).</li>
 *   <li>If a filesystem {@code registry.json} exists alongside the jar, merge/override.</li>
 *   <li>Scan {@code drivers/} directory (same dir as the jar), build a URLClassLoader
 *       that includes all {@code .jar} files found there.</li>
 * </ol>
 *
 * <p>This enables zero-code extension: drop a driver jar into {@code drivers/}
 * and add an entry to {@code registry.json}.
 */
public class RegistryLoader {

    private static final ObjectMapper MAPPER = new ObjectMapper();

    // Lazy-initialised after load()
    private Map<String, RegistryEntry> registry;
    private URLClassLoader driverClassLoader;

    public RegistryLoader() {}

    /**
     * Load registry configuration and scan external drivers directory.
     */
    public void load() throws IOException {
        // 1. Load built-in registry.json from classpath
        InputStream is = getClass().getClassLoader().getResourceAsStream("registry.json");
        if (is == null) {
            throw new FileNotFoundException("registry.json not found on classpath");
        }
        Map<String, RegistryEntry> builtin = MAPPER.readValue(is,
                new TypeReference<Map<String, RegistryEntry>>() {});
        is.close();

        // 2. Try to load filesystem registry.json override (next to the jar)
        Map<String, RegistryEntry> external = loadExternalRegistry();
        if (external != null) {
            builtin.putAll(external);
        }
        this.registry = builtin;

        // 3. Scan drivers/ directory and build URLClassLoader
        this.driverClassLoader = buildDriverClassLoader();
    }

    private Map<String, RegistryEntry> loadExternalRegistry() {
        File extFile = resolveRelative("registry.json");
        if (extFile != null && extFile.isFile()) {
            try {
                return MAPPER.readValue(extFile,
                        new TypeReference<Map<String, RegistryEntry>>() {});
            } catch (IOException e) {
                System.err.println("[WARN] Cannot parse external registry.json: " + e.getMessage());
            }
        }
        return null;
    }

    private URLClassLoader buildDriverClassLoader() {
        File driversDir = resolveRelative("drivers");
        if (driversDir == null || !driversDir.isDirectory()) {
            // No external drivers dir, return a ClassLoader with no extra URLs
            return new URLClassLoader(new URL[0], ClassLoader.getSystemClassLoader());
        }

        File[] jars = driversDir.listFiles((dir, name) -> name.toLowerCase().endsWith(".jar"));
        if (jars == null || jars.length == 0) {
            return new URLClassLoader(new URL[0], ClassLoader.getSystemClassLoader());
        }

        URL[] urls = new URL[jars.length];
        for (int i = 0; i < jars.length; i++) {
            try {
                urls[i] = jars[i].toURI().toURL();
            } catch (IOException e) {
                System.err.println("[WARN] Cannot convert jar to URL: " + jars[i].getAbsolutePath());
            }
        }
        return new URLClassLoader(urls, ClassLoader.getSystemClassLoader());
    }

    /**
     * Resolve a file/directory path relative to the jar's parent directory
     * (i.e., {@code System.getProperty("user.dir")}).
     */
    private File resolveRelative(String sub) {
        String userDir = System.getProperty("user.dir");
        if (userDir == null) return null;
        return new File(userDir, sub);
    }

    // ---- accessors ----

    public Map<String, RegistryEntry> getRegistry() {
        return Collections.unmodifiableMap(registry);
    }

    public RegistryEntry getEntry(String type) {
        return registry != null ? registry.get(type) : null;
    }

    public URLClassLoader getDriverClassLoader() {
        return driverClassLoader;
    }
}
