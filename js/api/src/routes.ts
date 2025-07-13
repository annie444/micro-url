let urlRouteBase = "";
let userRouteBase = "";
let oidcRouteBase = "";
let localRouteBase = "";
let healthRouteBase = "";

if (process.env.NODE_ENV !== "production") {
  console.log("Using development routes");
  healthRouteBase = "http://localhost:8081";
  urlRouteBase = "http://localhost:8081/api/url";
  userRouteBase = "http://localhost:8081/api/user";
  oidcRouteBase = `${userRouteBase}/oidc`;
  localRouteBase = `${userRouteBase}/local`;
} else if (process.env.NODE_ENV === "production") {
  urlRouteBase = "/api/url";
  userRouteBase = "/api/user";
  oidcRouteBase = `${userRouteBase}/oidc`;
  localRouteBase = `${userRouteBase}/local`;
}

export interface UrlRoutes {
  newUrl: string;
  deleteUrl: (id: string) => string;
  updateUrl: (id: string) => string;
  urlInfo: (id: string) => string;
  urlQrCode: (id: string) => string;
}

export const urlRoutes: UrlRoutes = {
  newUrl: `${urlRouteBase}/new`,
  deleteUrl: (id: string) => `${urlRouteBase}/delete/${id}`,
  updateUrl: (id: string) => `${urlRouteBase}/update/${id}`,
  urlInfo: (id: string) => `${urlRouteBase}/${id}`,
  urlQrCode: (id: string) => `${urlRouteBase}/qr/${id}`,
};

export interface OidcRoutes {
  provider: string;
  login: string;
}

export interface LocalRoutes {
  register: string;
  login: string;
}

export interface UserRoutes {
  getUser: string;
  logout: string;
  userUrls: string;
  userUrlsPaged: string;
  oidc: OidcRoutes;
  local: LocalRoutes;
}

export const oidcRoutes: OidcRoutes = {
  provider: `${oidcRouteBase}/provider`,
  login: `${oidcRouteBase}/login`,
};

export const localRoutes: LocalRoutes = {
  register: `${localRouteBase}/register`,
  login: `${localRouteBase}/login`,
};

export const userRoutes: UserRoutes = {
  getUser: `${userRouteBase}`,
  logout: `${userRouteBase}/logout`,
  userUrls: `${userRouteBase}/urls`,
  userUrlsPaged: `${userRouteBase}/urls/page`,
  oidc: oidcRoutes,
  local: localRoutes,
};

export type HealthRoute = string;

export const healthRoute: HealthRoute = `${healthRouteBase}/api/health`;

export interface Routes {
  url: UrlRoutes;
  user: UserRoutes;
  health: HealthRoute;
}

export const routes: Routes = {
  url: urlRoutes,
  user: userRoutes,
  health: healthRoute,
};
