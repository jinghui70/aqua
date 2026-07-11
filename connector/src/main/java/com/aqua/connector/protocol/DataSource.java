package com.aqua.connector.protocol;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Database connection parameters from stdin JSON request.
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class DataSource {
    @JsonProperty(required = true)
    private String type;

    @JsonProperty(required = true)
    private String host;

    @JsonProperty(required = true)
    private int port;

    @JsonProperty(required = true)
    private String database;

    @JsonProperty(required = true)
    private String user;

    @JsonProperty(required = true)
    private String password;

    public DataSource() {}

    public String getType() { return type; }
    public void setType(String type) { this.type = type; }

    public String getHost() { return host; }
    public void setHost(String host) { this.host = host; }

    public int getPort() { return port; }
    public void setPort(int port) { this.port = port; }

    public String getDatabase() { return database; }
    public void setDatabase(String database) { this.database = database; }

    public String getUser() { return user; }
    public void setUser(String user) { this.user = user; }

    public String getPassword() { return password; }
    public void setPassword(String password) { this.password = password; }
}
