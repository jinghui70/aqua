package com.aqua.connector.registry;

/**
 * Entry in registry.json: type -> driverClass + urlPattern.
 */
public class RegistryEntry {
    private String driverClass;
    private String urlPattern;

    public RegistryEntry() {}

    public RegistryEntry(String driverClass, String urlPattern) {
        this.driverClass = driverClass;
        this.urlPattern = urlPattern;
    }

    public String getDriverClass() { return driverClass; }
    public void setDriverClass(String driverClass) { this.driverClass = driverClass; }

    public String getUrlPattern() { return urlPattern; }
    public void setUrlPattern(String urlPattern) { this.urlPattern = urlPattern; }
}
