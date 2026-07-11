package com.aqua.connector.protocol;

import com.fasterxml.jackson.annotation.JsonInclude;

/**
 * Error payload in a failed Response.
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public class ErrorInfo {
    private String code;
    private String message;

    public ErrorInfo() {}

    public ErrorInfo(String code, String message) {
        this.code = code;
        this.message = message;
    }

    public String getCode() { return code; }
    public void setCode(String code) { this.code = code; }

    public String getMessage() { return message; }
    public void setMessage(String message) { this.message = message; }
}
