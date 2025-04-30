import axios, { type AxiosError, type AxiosResponse } from "axios";
import type {
  NewUrlRequest,
  ShortLink,
  BasicError,
  QrCodeParams,
} from "@/lib/types";
import { urlRoutes } from "@/lib/api";

export function newUrl(urlRequest: NewUrlRequest): Promise<ShortLink> {
  return new Promise(
    (
      resolve: (value: ShortLink) => void,
      reject: (reason: BasicError) => void,
    ) => {
      axios
        .post(urlRoutes.newUrl, urlRequest)
        .then((data: AxiosResponse<ShortLink>) => {
          resolve(data.data);
        })
        .catch((error: AxiosError<BasicError>) => {
          if (error.response && error.response.data) {
            reject(error.response.data);
          } else {
            reject({
              error: "",
            });
          }
        });
    },
  );
}

export function updateUrl(
  id: string,
  urlRequest: NewUrlRequest,
): Promise<ShortLink> {
  return new Promise(
    (
      resolve: (value: ShortLink) => void,
      reject: (reason: BasicError) => void,
    ) => {
      axios
        .put(urlRoutes.updateUrl(id), urlRequest)
        .then((data: AxiosResponse<ShortLink>) => {
          resolve(data.data);
        })
        .catch((error: AxiosError<BasicError>) => {
          if (error.response && error.response.data) {
            reject(error.response.data);
          } else {
            reject({
              error: "",
            });
          }
        });
    },
  );
}

export function urlInfo(id: string): Promise<ShortLink> {
  return new Promise(
    (
      resolve: (value: ShortLink) => void,
      reject: (reason: BasicError) => void,
    ) => {
      axios
        .get(urlRoutes.urlInfo(id))
        .then((data: AxiosResponse<ShortLink>) => {
          resolve(data.data);
        })
        .catch((error: AxiosError<BasicError>) => {
          if (error.response && error.response.data) {
            reject(error.response.data);
          } else {
            reject({
              error: "",
            });
          }
        });
    },
  );
}

export interface Rgba {
  red?: number; // 0-255
  green?: number; // 0-255
  blue?: number; // 0-255
  alpha?: number; // 0-255
}

export interface QrParams {
  bg?: Rgba;
  fg?: Rgba;
  format: "png" | "jpeg" | "webp";
}

export function qrCode(id: string, params: QrParams): Promise<File> {
  const query: QrCodeParams = {
    format: params.format,
  };
  if (params.bg) {
    if (params.bg.red) {
      query.bg_red = params.bg.red;
    }
    if (params.bg.green) {
      query.bg_green = params.bg.green;
    }
    if (params.bg.blue) {
      query.bg_blue = params.bg.blue;
    }
    if (params.bg.alpha) {
      query.bg_alpha = params.bg.alpha;
    }
  }
  if (params.fg) {
    if (params.fg.red) {
      query.fg_red = params.fg.red;
    }
    if (params.fg.green) {
      query.fg_green = params.fg.green;
    }
    if (params.fg.blue) {
      query.fg_blue = params.fg.blue;
    }
    if (params.fg.alpha) {
      query.fg_alpha = params.fg.alpha;
    }
  }
  return new Promise(
    (resolve: (value: File) => void, reject: (reason: BasicError) => void) => {
      axios
        .get(urlRoutes.urlQrCode(id, query), {
          responseType: "blob",
          headers: {
            Accept: `image/${params.format}`,
          },
        })
        .then((data: AxiosResponse<BlobPart[]>) => {
          const contentDisposition = data.headers["content-disposition"];
          const fileName = contentDisposition.match(/filename="(.+)"/)[1];
          const file = new File(data.data, fileName);
          resolve(file);
        })
        .catch((error: AxiosError<BasicError>) => {
          if (error.response && error.response.data) {
            reject(error.response.data);
          } else {
            reject({
              error: "",
            });
          }
        });
    },
  );
}
