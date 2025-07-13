import { test, describe, beforeAll, afterAll, expect } from "@jest/globals";
import { checkHealth, newUrl } from "./index";
import { Ok } from "./result";
import assert from "assert";
import {
  spawn,
  spawnSync,
  ChildProcessWithoutNullStreams,
} from "child_process";

describe("API Tests", () => {
  let proc: ChildProcessWithoutNullStreams | null = null;

  const killDb = () => {
    const dbContainer = spawnSync("podman", [
      "container",
      "exists",
      "shuttle_micro-url_shared_postgres",
    ]);
    if (dbContainer.status === 0) {
      console.log("Postgres container exists, killing it...");
      const stopDb = spawnSync("podman", [
        "stop",
        "shuttle_micro-url_shared_postgres",
      ]);
      if (stopDb.status !== 0) {
        console.error(
          "Failed to stop Postgres container:",
          stopDb.stderr.toString(),
        );
        throw new Error("Failed to stop Postgres container");
      }
      const rmDb = spawnSync("podman", [
        "rm",
        "shuttle_micro-url_shared_postgres",
      ]);
      if (rmDb.status !== 0) {
        console.error(
          "Failed to remove Postgres container:",
          rmDb.stderr.toString(),
        );
        throw new Error("Failed to remove Postgres container");
      }
    }
  };

  const setupOidc = () => {
    const oidcContainer = spawnSync("podman", ["container", "exists", "oidc"]);
    if (oidcContainer.status !== 0) {
      console.log("Creating oidc container...");
      const createOidc = spawnSync("podman", [
        "run",
        "-d",
        "--rm",
        "--name",
        "oidc",
        "--publish",
        "4011:8080",
        "--env",
        "ASPNETCORE_ENVIRONMENT=Development",
        "--env",
        'SERVER_OPTIONS_INLINE=\'{"AccessTokenJwtType":"JWT","Discovery":{"ShowKeySet":true},"Authentication":{"CookieSameSiteMode":"Lax","CheckSessionCookieSameSiteMode":"Lax"}}\'',
        "--env",
        "LOGIN_OPTIONS_INLINE='{\"AllowRememberLogin\":false}'",
        "--env",
        "LOGOUT_OPTIONS_INLINE='{\"AutomaticRedirectAfterSignOut\":true}'",
        "--env",
        'CLIENTS_CONFIGURATION_INLINE=\'[{"ClientId":"micro-url-mock","ClientSecrets":["micro-url-mock-secret"],"Description":"Client for authorization code flow","AllowedGrantTypes":["authorization_code"],"RequirePkce":true,"AllowAccessTokensViaBrowser":true,"RedirectUris":["http://localhost:8000/api/user/oidc/callback"],"AllowedScopes":["openid","profile","email"],"IdentityTokenLifetime":3600,"AccessTokenLifetime":3600,"RequireClientSecret":false}]\'',
        "--env",
        'USERS_CONFIGURATION_INLINE=\'[{"SubjectId":"1","Username":"User1","Password":"password","Claims":[{"Type":"name","Value":"Test User1","ValueType":"string"},{"Type":"email","Value":"testuser1@example.com","ValueType":"string"}]}]\'',
        "--env",
        'ASPNET_SERVICES_OPTIONS_INLINE=\'{"ForwardedHeadersOptions":{"ForwardedHeaders":"All"}}\'',
        "ghcr.io/soluto/oidc-server-mock:0.9.2",
      ]);
      if (createOidc.status !== 0) {
        console.error(
          "Failed to create oidc container:",
          createOidc.stderr.toString(),
        );
        throw new Error("Failed to create oidc container");
      }
    }
  };

  const setupApi = (wait: number) => {
    return new Promise<void>((resolve, reject) => {
      proc = spawn("shuttle", ["run"], {
        shell: true,
        cwd: __dirname,
        env: { ...process.env },
      });
      const timeout = setTimeout(() => {
        reject(new Error("Timed out waiting for server to be ready"));
      }, wait); // 10 second timeout

      const logRegex =
        /([0-9]*)-([0-9]*)-([0-9]*)T([0-9]*):([0-9]*):([0-9]*).([0-9]*).([0-9]*):([0-9]*) [app].*INFO shuttle_runtime::rt: Starting service/;

      const onData = (data: Buffer) => {
        const line = data.toString();
        console.log("[API]", line); // Optional: log server output for debugging

        if (logRegex.test(line)) {
          clearTimeout(timeout);
          console.log(`Server is ready ${line}`);
          proc?.stdout.off("data", onData);
          proc?.stderr.off("data", onData);
          resolve();
        }
      };

      proc.stdout.on("data", onData);
      proc.stderr.on("data", onData);

      proc.on("error", (err) => {
        clearTimeout(timeout);
        reject(err);
      });

      proc.on("exit", (code) => {
        if (code !== 0) {
          clearTimeout(timeout);
          reject(new Error(`Server exited with code ${code}`));
        }
      });
    });
  };

  beforeAll(async () => {
    killDb();
    setupOidc();
    await setupApi(30000); // Wait up to 30 seconds for the API to start
  }, 30000); // Set timeout for beforeAll to 30 seconds

  test("newUrl", async () => {
    const url = await newUrl({
      url: "https://example.com",
    });
    expect(url).toBeDefined();
  });

  test("getHealth", async () => {
    const health = await checkHealth();
    expect(health).toBeDefined();
    expect(health._tag).toBe("ok");
    expect(health).toHaveProperty("value");
    assert(health instanceof Ok);
    expect(health.value).toBeDefined();
    expect(health.value).toBe("ok");
  });

  afterAll(() => {
    if (proc) {
      proc.kill();
      proc = null;
    }
  });
});
