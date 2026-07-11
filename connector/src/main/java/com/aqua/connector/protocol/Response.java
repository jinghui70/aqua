package com.aqua.connector.protocol;

import com.fasterxml.jackson.annotation.JsonInclude;

/**
 * Outgoing JSON response (stdout).
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public class Response {
    private boolean success;
    private Object data;
    private ErrorInfo error;

    public Response() {}

    public static Response ok(Object data) {
        Response r = new Response();
        r.success = true;
        r.data = data;
        return r;
    }

    public static Response fail(String code, String message) {
        Response r = new Response();
        r.success = false;
        r.error = new ErrorInfo(code, message);
        return r;
    }

    public boolean isSuccess() { return success; }
    public void setSuccess(boolean success) { this.success = success; }

    public Object getData() { return data; }
    public void setData(Object data) { this.data = data; }

    public ErrorInfo getError() { return error; }
    public void setError(ErrorInfo error) { this.error = error; }
}
