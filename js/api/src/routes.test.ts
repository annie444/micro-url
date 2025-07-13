import { describe, test, expect } from "@jest/globals";
import { routes } from "./routes";

describe("API Routes", () => {
  test("should have a /health route", () => {
    expect(routes).toHaveProperty("health");
    expect(routes.health).toBe("http://localhost:8081/api/health");
  });

  describe("URL Routes", () => {
    test("should have a /url/new route", () => {
      expect(routes).toHaveProperty("url");
      expect(routes.url).toHaveProperty("newUrl");
      expect(routes.url.newUrl).toBe("http://localhost:8081/api/url/new");
    });

    test("should have a /url/delete/:id route", () => {
      const id = "123";
      expect(routes).toHaveProperty("url");
      expect(routes.url).toHaveProperty("deleteUrl");
      expect(routes.url.deleteUrl(id)).toBe(
        `http://localhost:8081/api/url/delete/${id}`,
      );
    });

    test("should have a /url/update/:id route", () => {
      const id = "123";
      expect(routes).toHaveProperty("url");
      expect(routes.url).toHaveProperty("updateUrl");
      expect(routes.url.updateUrl(id)).toBe(
        `http://localhost:8081/api/url/update/${id}`,
      );
    });

    test("should have a /url/:id route", () => {
      const id = "123";
      expect(routes).toHaveProperty("url");
      expect(routes.url).toHaveProperty("urlInfo");
      expect(routes.url.urlInfo(id)).toBe(
        `http://localhost:8081/api/url/${id}`,
      );
    });

    test("should have a /url/qr/:id route", () => {
      const id = "123";
      expect(routes).toHaveProperty("url");
      expect(routes.url).toHaveProperty("urlQrCode");
      expect(routes.url.urlQrCode(id)).toBe(
        `http://localhost:8081/api/url/qr/${id}`,
      );
    });
  });

  describe("User Routes", () => {
    test("should have a /user/get route", () => {
      expect(routes).toHaveProperty("user");
      expect(routes.user).toHaveProperty("getUser");
      expect(routes.user.getUser).toBe("http://localhost:8081/api/user");
    });

    test("should have a /user/logout route", () => {
      expect(routes).toHaveProperty("user");
      expect(routes.user).toHaveProperty("logout");
      expect(routes.user.logout).toBe("http://localhost:8081/api/user/logout");
    });

    test("should have a /user/urls route", () => {
      expect(routes).toHaveProperty("user");
      expect(routes.user).toHaveProperty("userUrls");
      expect(routes.user.userUrls).toBe("http://localhost:8081/api/user/urls");
    });

    test("should have a /user/urls/paged route", () => {
      expect(routes).toHaveProperty("user");
      expect(routes.user).toHaveProperty("userUrlsPaged");
      expect(routes.user.userUrlsPaged).toBe(
        "http://localhost:8081/api/user/urls/page",
      );
    });

    describe("OIDC Routes", () => {
      test("should have a /user/oidc/provider route", () => {
        expect(routes).toHaveProperty("user");
        expect(routes.user).toHaveProperty("oidc");
        expect(routes.user.oidc).toHaveProperty("provider");
        expect(routes.user.oidc.provider).toBe(
          "http://localhost:8081/api/user/oidc/provider",
        );
      });

      test("should have a /user/oidc/login route", () => {
        expect(routes).toHaveProperty("user");
        expect(routes.user).toHaveProperty("oidc");
        expect(routes.user.oidc).toHaveProperty("login");
        expect(routes.user.oidc.login).toBe(
          "http://localhost:8081/api/user/oidc/login",
        );
      });
    });

    describe("Local Routes", () => {
      test("should have a /user/local/register route", () => {
        expect(routes).toHaveProperty("user");
        expect(routes.user).toHaveProperty("local");
        expect(routes.user.local).toHaveProperty("register");
        expect(routes.user.local.register).toBe(
          "http://localhost:8081/api/user/local/register",
        );
      });

      test("should have a /user/local/login route", () => {
        expect(routes).toHaveProperty("user");
        expect(routes.user).toHaveProperty("local");
        expect(routes.user.local).toHaveProperty("login");
        expect(routes.user.local.login).toBe(
          "http://localhost:8081/api/user/local/login",
        );
      });
    });
  });
});
