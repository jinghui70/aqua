package com.aqua.connector;

import com.aqua.connector.command.ImportCommand;
import com.aqua.connector.command.QueryCommand;
import com.aqua.connector.command.TestCommand;
import com.aqua.connector.jdbc.ConnectionFactory;
import com.aqua.connector.protocol.Request;
import com.aqua.connector.protocol.Response;
import com.aqua.connector.registry.RegistryLoader;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

/**
 * Entry point for the db-connector subprocess.
 *
 * <p>Reads a single JSON request from stdin, dispatches to the appropriate
 * command handler, writes the JSON response to stdout, and exits.
 */
public class Main {

    private static final ObjectMapper MAPPER = new ObjectMapper();

    public static void main(String[] args) {
        int exitCode = 0;
        try {
            // 1. Load registry + external drivers
            RegistryLoader registryLoader = new RegistryLoader();
            registryLoader.load();

            ConnectionFactory connectionFactory = new ConnectionFactory(registryLoader);

            // 2. Read full stdin into a string
            String stdinContent = readStdin();

            // 3. Parse request
            Request request;
            try {
                request = MAPPER.readValue(stdinContent, Request.class);
            } catch (Exception e) {
                Response resp = Response.fail("INVALID_REQUEST",
                        "Cannot parse request JSON: " + e.getMessage());
                System.out.println(MAPPER.writeValueAsString(resp));
                System.exit(1);
                return;
            }

            // 4. Dispatch command
            Response response = dispatch(request, connectionFactory);

            // 5. Write response to stdout
            System.out.println(MAPPER.writeValueAsString(response));

        } catch (Exception e) {
            Response resp = Response.fail("INTERNAL_ERROR", e.getMessage());
            try {
                System.out.println(MAPPER.writeValueAsString(resp));
            } catch (Exception ignored) {}
            exitCode = 1;
        }
        System.exit(exitCode);
    }

    private static Response dispatch(Request request, ConnectionFactory connectionFactory) {
        String cmd = request.getCommand();
        if (cmd == null) {
            return Response.fail("INVALID_REQUEST", "Missing 'command' field");
        }

        switch (cmd.toLowerCase()) {
            case "import":
                return new ImportCommand(connectionFactory)
                        .execute(request.getDataSource(), request.getParams());

            case "query":
                return new QueryCommand(connectionFactory)
                        .execute(request.getDataSource(), request.getParams());

            case "test":
                return new TestCommand(connectionFactory)
                        .execute(request.getDataSource());

            default:
                return Response.fail("UNKNOWN_COMMAND",
                        "Unrecognised command: " + cmd + " (expected import/query/test)");
        }
    }

    private static String readStdin() throws Exception {
        ByteArrayOutputStream buffer = new ByteArrayOutputStream();
        InputStream in = System.in;
        byte[] tmp = new byte[4096];
        int n;
        while ((n = in.read(tmp)) != -1) {
            buffer.write(tmp, 0, n);
            // Stop when we read enough (handle pipe not closing)
            if (n < tmp.length) break;
        }
        return buffer.toString(StandardCharsets.UTF_8);
    }
}
