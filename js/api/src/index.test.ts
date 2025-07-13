import { test, describe, beforeAll, afterAll, expect } from "@jest/globals";
import {
  checkHealth,
  newUrl,
  getQrCode,
  getUrlInfo,
  updateUrl,
  deleteUrl,
  registerLocalUser,
  loginLocalUser,
  getUserInfo,
  getUserUrls,
  logout,
  getUserUrlsPaged,
  getOidcProvider,
  getOidcLoginUrl,
} from "./index";
import { ShortLink, User } from "./types";
import { Ok } from "./result";
import assert from "assert";
import {
  spawn,
  spawnSync,
  ChildProcessWithoutNullStreams,
} from "child_process";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function isOk<T>(result: any): result is Ok<T> {
  expect(result).toBeDefined();
  expect(result._tag).toBe("ok");
  expect(result).toHaveProperty("value");
  assert(result instanceof Ok);
  expect(result.value).toBeDefined();
  return true;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function isShortLink(result: any): result is ShortLink {
  const attributes = [
    "id",
    "short_url",
    "original_url",
    "created_at",
    "updated_at",
  ];
  for (const attr of attributes) {
    expect(result).toHaveProperty(attr);
  }
  expect(result.id).toBeDefined();
  expect(typeof result.id).toBe("string");
  expect(result.short_url).toBeDefined();
  expect(typeof result.short_url).toBe("string");
  expect(result.original_url).toBeDefined();
  expect(typeof result.original_url).toBe("string");
  expect(result.created_at).toBeDefined();
  expect(typeof result.created_at).toBe("string");
  expect(result.updated_at).toBeDefined();
  expect(typeof result.updated_at).toBe("string");
  return true;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function isUser(result: any): result is User {
  const attributes = ["user_id", "name", "email", "created_at", "updated_at"];
  for (const attr of attributes) {
    expect(result).toHaveProperty(attr);
  }
  expect(result.user_id).toBeDefined();
  expect(typeof result.user_id).toBe("string");
  expect(result.name).toBeDefined();
  expect(typeof result.name).toBe("string");
  expect(result.email).toBeDefined();
  expect(typeof result.email).toBe("string");
  expect(result.created_at).toBeDefined();
  expect(typeof result.created_at).toBe("string");
  expect(result.updated_at).toBeDefined();
  expect(typeof result.updated_at).toBe("string");
  return true;
}
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
      proc = spawn("shuttle", ["run", "--port", "8081"], {
        shell: true,
        cwd: __dirname,
        env: { ...process.env },
      });
      const timeout = setTimeout(() => {
        reject(new Error("Timed out waiting for server to be ready"));
      }, wait); // 10 second timeout

      const end = () => {
        setTimeout(() => {
          console.log("Server is ready, resolving...");
          resolve();
        }, 2000); // Wait 3 seconds before resolving
      };

      // const logRegex =
      //   /.*([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})\.([0-9]{3}).*\[app\].*INFO.*shuttle_runtime::rt:.*Starting service.*/;

      const onData = (data: Buffer) => {
        const line = data.toString();

        if (line.includes("Starting service")) {
          clearTimeout(timeout);
          if (proc) {
            proc.stdout.off("data", onData);
            proc.stderr.off("data", onData);
          }
          end();
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

  describe("URL API", () => {
    let id: string | undefined;

    test("newUrl", async () => {
      const url = await newUrl({
        url: "https://example.com",
      });
      expect(isOk(url)).toBeTruthy();
      assert(isOk(url));
      expect(isShortLink(url.value)).toBeTruthy();
      assert(isShortLink(url.value));
      id = url.value.id;
    });

    test("getQrCode", async () => {
      expect(id).toBeDefined();
      expect(typeof id).toBe("string");
      assert(id !== undefined);
      const qrCode = await getQrCode(id, {
        format: "png",
        bg_red: 255,
        bg_green: 255,
        bg_blue: 255,
        bg_alpha: 1,
        fg_red: 0,
        fg_green: 0,
        fg_blue: 0,
        fg_alpha: 1,
      });
      expect(isOk(qrCode)).toBeTruthy();
      assert(isOk(qrCode));
      console.log(typeof qrCode.value);
      const header = qrCode.value.slice(0, 8);
      expect(header).toBeDefined();
      expect(header).toEqual(
        new Blob([new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10])]),
      ); // PNG signature
    });

    test("getUrlInfo", async () => {
      expect(id).toBeDefined();
      expect(typeof id).toBe("string");
      assert(id !== undefined);
      const urlInfo = await getUrlInfo(id);
      expect(isOk(urlInfo)).toBeTruthy();
      assert(isOk(urlInfo));
      expect(isShortLink(urlInfo.value)).toBeTruthy();
    });

    test("updateUrl", async () => {
      expect(id).toBeDefined();
      expect(typeof id).toBe("string");
      assert(id !== undefined);
      const updatedUrl = await updateUrl(id, {
        url: "https://updated-example.com",
      });
      expect(isOk(updatedUrl)).toBeTruthy();
      assert(isOk(updatedUrl));
      expect(isShortLink(updatedUrl.value)).toBeTruthy();
      assert(isShortLink(updatedUrl.value));
      expect(updatedUrl.value.id).toBe(id);
    });

    test("deleteUrl", async () => {
      expect(id).toBeDefined();
      expect(typeof id).toBe("string");
      assert(id !== undefined);
      const response = await deleteUrl(id);
      expect(isOk(response)).toBeTruthy();
      assert(isOk(response));
      expect(response.value).toBeDefined();
      expect(response.value).toHaveProperty("message");
      expect(response.value.message).toBe("OK");
    });
  });

  describe("User API", () => {
    let userId: string | undefined;

    test("registerLocalUser", async () => {
      const user = await registerLocalUser({
        name: "Test User",
        password: "password",
        email: "test@example.com",
      });
      expect(isOk(user)).toBeTruthy();
      assert(isOk(user));
      expect(isUser(user.value)).toBeTruthy();
      assert(isUser(user.value));
      userId = user.value.user_id;
    });

    test("loginLocalUser", async () => {
      expect(userId).toBeDefined();
      expect(typeof userId).toBe("string");
      assert(userId !== undefined);
      const login = await loginLocalUser({
        email: "test@example.com",
        password: "password",
      });
      expect(isOk(login)).toBeTruthy();
      assert(isOk(login));
      expect(login.value).toBeDefined();
      assert(login.value);
      expect(login.value.user_id).toBe(userId);
    });

    test("getUserInfo", async () => {
      expect(userId).toBeDefined();
      expect(typeof userId).toBe("string");
      assert(userId !== undefined);
      const userInfo = await getUserInfo();
      expect(isOk(userInfo)).toBeTruthy();
      assert(isOk(userInfo));
      expect(isUser(userInfo.value)).toBeTruthy();
      assert(isUser(userInfo.value));
      expect(userInfo.value.user_id).toBe(userId);
    });

    test("User's can 'own' URLs", async () => {
      expect(userId).toBeDefined();
      expect(typeof userId).toBe("string");
      assert(userId !== undefined);
      const urls = await newUrl({
        url: "https://user-owned-url.com",
        user: userId,
      });
      expect(isOk(urls)).toBeTruthy();
      assert(isOk(urls));
      expect(isShortLink(urls.value)).toBeTruthy();
      assert(isShortLink(urls.value));
      expect(urls.value.user_id).toBe(userId);
    });

    test("getUserUrls", async () => {
      expect(userId).toBeDefined();
      expect(typeof userId).toBe("string");
      assert(userId !== undefined);
      const userUrls = await getUserUrls();
      expect(isOk(userUrls)).toBeTruthy();
      assert(isOk(userUrls));
      expect(Array.isArray(userUrls.value)).toBeTruthy();
      userUrls.value.forEach((url) => {
        expect(isShortLink(url)).toBeTruthy();
        assert(isShortLink(url));
      });
    });

    test("getUserUrlsPaged", async () => {
      expect(userId).toBeDefined();
      expect(typeof userId).toBe("string");
      assert(userId !== undefined);
      const userUrlsPaged = await getUserUrlsPaged({ page: 0, size: 10 });
      expect(isOk(userUrlsPaged)).toBeTruthy();
      assert(isOk(userUrlsPaged));
      expect(Array.isArray(userUrlsPaged.value.urls)).toBeTruthy();
      userUrlsPaged.value.urls.forEach((url) => {
        expect(isShortLink(url)).toBeTruthy();
        assert(isShortLink(url));
      });
    });

    describe("OIDC", () => {
      test("getOidcProvider", async () => {
        const provider = await getOidcProvider();
        expect(isOk(provider)).toBeTruthy();
        assert(isOk(provider));
        expect(provider.value).toBeDefined();
        expect(provider.value).toHaveProperty("name");
        expect(typeof provider.value.name).toBe("string");
      });

      test("getOidcLoginUrl", () => {
        const loginUrl = getOidcLoginUrl();
        expect(loginUrl).toBeDefined();
        expect(typeof loginUrl).toBe("string");
        expect(loginUrl).toContain("http://localhost:8081");
      });
    });

    test("logout", async () => {
      const response = await logout();
      expect(isOk(response)).toBeTruthy();
      assert(isOk(response));
      expect(response.value).toBeDefined();
      expect(response.value).toHaveProperty("message");
      expect(response.value.message).toBe("User logged out");
    });
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
