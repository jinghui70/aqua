package com.aqua.connector.protocol;

/**
 * Standardised error codes for JSON error responses.
 */
public enum ErrorCode {
    INVALID_REQUEST("INVALID_REQUEST", "Malformed or missing fields in request JSON"),
    UNKNOWN_COMMAND("UNKNOWN_COMMAND", "Unrecognised command name"),
    UNSUPPORTED_DB("UNSUPPORTED_DB", "Database type not found in registry.json"),
    CONNECTION_FAILED("CONNECTION_FAILED", "JDBC connection could not be established"),
    QUERY_FAILED("QUERY_FAILED", "SQL execution error"),
    IMPORT_FAILED("IMPORT_FAILED", "Metadata reading error"),
    INTERNAL_ERROR("INTERNAL_ERROR", "Unexpected internal error");

    private final String code;
    private final String defaultMessage;

    ErrorCode(String code, String defaultMessage) {
        this.code = code;
        this.defaultMessage = defaultMessage;
    }

    public String getCode() { return code; }
    public String getDefaultMessage() { return defaultMessage; }
}
