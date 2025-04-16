import axios, { AxiosError, type AxiosResponse } from "axios";
import type {
  BasicError,
  LoginRequest,
  NewUserRequest,
  Paginate,
  User,
  UserLink,
  UserLinksAndViews,
} from "@/lib/types";
import { userRoutes } from "./routes";

export function getUser(): Promise<User> {
  return new Promise(
    (resolve: (value: User) => void, reject: (reason: BasicError) => void) => {
      axios
        .get(userRoutes.getUser)
        .then((data: AxiosResponse<User>) => {
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

export function getUserUrls(): Promise<UserLink[]> {
  return new Promise(
    (
      resolve: (value: UserLink[]) => void,
      reject: (reason: BasicError) => void,
    ) => {
      axios
        .get(userRoutes.userUrls)
        .then((data: AxiosResponse<UserLink[]>) => {
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

export function getUserUrlsPaged(
  size: bigint,
  page: bigint,
): Promise<UserLinksAndViews> {
  const paging: Paginate = {
    page: page,
    size: size,
  };
  return new Promise(
    (
      resolve: (value: UserLinksAndViews) => void,
      reject: (reason: BasicError) => void,
    ) => {
      axios
        .get(userRoutes.userUrlsPaged(paging))
        .then((data: AxiosResponse<UserLinksAndViews>) => {
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

export function getOidcProvider(): Promise<string> {
  return new Promise(
    (
      resolve: (value: string) => void,
      reject: (reason: BasicError) => void,
    ) => {
      axios
        .get(userRoutes.oidc.provider)
        .then((data: AxiosResponse<string>) => {
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

export function loginOidc(): Promise<null> {
  return new Promise(
    (resolve: (value: null) => void, reject: (reason: BasicError) => void) => {
      axios
        .get(userRoutes.oidc.login)
        .then(() => {
          resolve(null);
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

export function registerUser(new_user: NewUserRequest): Promise<User> {
  return new Promise(
    (resolve: (value: User) => void, reject: (reason: BasicError) => void) => {
      axios
        .post(userRoutes.local.register, new_user)
        .then((data: AxiosResponse<User>) => {
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

export function loginLocal(creds: LoginRequest): Promise<User> {
  return new Promise(
    (resolve: (value: User) => void, reject: (reason: BasicError) => void) => {
      axios
        .post(userRoutes.local.register, creds)
        .then((data: AxiosResponse<User>) => {
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
