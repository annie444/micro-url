import type { Paginate, QrCodeParams } from "@/lib/types";

export const urlRoutes = {
  newUrl: "/api/url/new",
  deleteUrl: (id: string) => `/api/url/delete/${id}`,
  updateUrl: (id: string) => `/api/url/update/${id}`,
  urlInfo: (id: string) => `/api/url/${id}`,
  urlQrCode: (id: string, params: QrCodeParams) => {
    const url = `/api/url/${id}`;
    const query = new URLSearchParams();
    if (params.bg_red) {
      query.append("bg_red", `${params.bg_red}`);
    }
    if (params.bg_green) {
      query.append("bg_green", `${params.bg_green}`);
    }
    if (params.bg_blue) {
      query.append("bg_blue", `${params.bg_blue}`);
    }
    if (params.bg_alpha) {
      query.append("bg_alpha", `${params.bg_alpha}`);
    } else if (params.bg_red && params.bg_green && params.bg_blue) {
      query.append("bg_alpha", "255");
    }

    if (params.fg_red) {
      query.append("fg_red", `${params.fg_red}`);
    }
    if (params.fg_green) {
      query.append("fg_green", `${params.fg_green}`);
    }
    if (params.fg_blue) {
      query.append("fg_blue", `${params.fg_blue}`);
    }
    if (params.fg_alpha) {
      query.append("fg_alpha", `${params.fg_alpha}`);
    } else if (params.fg_red && params.fg_green && params.fg_blue) {
      query.append("fg_alpha", "255");
    }

    return `${url}?${query.toString()}`;
  },
};

export const userRoutes = {
  getUser: "/api/user",
  logout: "/api/user/logout",
  userUrls: "/api/user/urls",
  userUrlsPaged: (params: Paginate) => {
    const query = new URLSearchParams({
      page: `${params.page}`,
      size: `${params.size}`,
    });
    return "/api/user/urls/page?" + query.toString();
  },
  oidc: {
    provider: "/api/user/oidc/provider",
    login: "/api/user/oidc/login",
  },
  local: {
    register: "/api/user/local/register",
    login: "/api/user/local/login",
  },
};
