import { spawn } from "node:child_process";
import { expect, test, describe } from "vitest";

const SERVER_PATH = "../target/debug/web-search-mcp-rust";

async function sendRpc(process: any, method: string, params?: any, id = 1) {
  return new Promise((resolve, reject) => {
    const request = JSON.stringify({
      jsonrpc: "2.0",
      method,
      params,
      id,
    }) + "\n";

    process.stdin.write(request);

    let output = "";
    const onData = (data: any) => {
      const chunk = data.toString();
      output += chunk;
      if (output.trim().endsWith("}")) {
        try {
          const response = JSON.parse(output.trim());
          process.stdout.removeListener("data", onData);
          resolve(response);
        } catch (e) {
          // keep reading
        }
      }
    };

    process.stdout.on("data", onData);

    setTimeout(() => {
      process.stdout.removeListener("data", onData);
      reject(new Error(`Timeout waiting for response to ${method}`));
    }, 10000);
  });
}

async function createInitializedServer() {
  const process = spawn(SERVER_PATH, [], {
    stdio: ["pipe", "pipe", "pipe"],
  });

  await sendRpc(process, "initialize", {
    protocolVersion: "2025-11-25",
    capabilities: {},
    clientInfo: {
      name: "test-client",
      version: "1.0.0",
    },
  });

  return process;
}

async function assertToolResponse(response: any, toolName: string) {
  if (response.error) {
    if (
      response.error.message.includes("bots use DuckDuckGo too") ||
      response.error.message.includes("error sending request")
    ) {
      return;
    }
    throw new Error(`Tool ${toolName} returned an error: ${response.error.message}`);
  }
  if (!response.result || !response.result.content) {
    throw new Error(`Tool ${toolName} returned an unexpected response format: ${JSON.stringify(response)}`);
  }
  expect(response.result.content[0].type).toBe("text");
}



describe("MCP Server Stdio", () => {
  test("should initialize via stdio", async () => {
    const process = spawn(SERVER_PATH, [], {
      stdio: ["pipe", "pipe", "pipe"],
    });
    const response = await sendRpc(process, "initialize", {
      protocolVersion: "2025-11-25",
      capabilities: {},
      clientInfo: {
        name: "test-client",
        version: "1.0.0",
      },
    });
    expect(response.result.protocolVersion).toBe("2025-11-25");
    process.kill();
  });

  test("should call search_web via stdio", async () => {
    const process = await createInitializedServer();
    const response = await sendRpc(process, "tools/call", {
      name: "search_web",
      arguments: { query: "Hello world" }
    });
    await assertToolResponse(response, "search_web");
    process.kill();
  });

  test("should call fetch_page via stdio", async () => {
    const process = await createInitializedServer();
    const response = await sendRpc(process, "tools/call", {
      name: "fetch_page",
      arguments: { url: "https://example.com" }
    });
    await assertToolResponse(response, "fetch_page");
    process.kill();
  });

  test("should call search_domain via stdio", async () => {
    const process = await createInitializedServer();
    const response = await sendRpc(process, "tools/call", {
      name: "search_domain",
      arguments: { query: "Rust", domain: "rust-lang.org" }
    });
    await assertToolResponse(response, "search_domain");
    process.kill();
  });

  test("should return error for unknown tool", async () => {
    const process = await createInitializedServer();
    const response = await sendRpc(process, "tools/call", {
      name: "unknown_tool",
      arguments: {}
    });
    expect(response.error).toBeDefined();
    expect(response.error.message).toBe("tool not found");
    process.kill();
  });
});
