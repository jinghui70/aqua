package com.aqua.connector.protocol;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.Map;

/**
 * Incoming JSON request (stdin).
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public class Request {
    @JsonProperty(required = true)
    private String command;

    @JsonProperty(required = true)
    private DataSource dataSource;

    @JsonProperty
    private Map<String, Object> params;

    public Request() {}

    public String getCommand() { return command; }
    public void setCommand(String command) { this.command = command; }

    public DataSource getDataSource() { return dataSource; }
    public void setDataSource(DataSource dataSource) { this.dataSource = dataSource; }

    public Map<String, Object> getParams() { return params; }
    public void setParams(Map<String, Object> params) { this.params = params; }
}
