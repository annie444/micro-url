import {
  BasicError,
  NewUrlRequest,
  Paginate,
  QrCodeParams,
  ShortLink,
  BasicResponse,
  User,
  UserLink,
  UserLinksAndViews,
  OidcName,
  NewUserRequest,
  LoginRequest,
} from "./types";
import {
  makeGetCall,
  makePostCall,
  makeDeleteCall,
  makePutCall,
} from "./utils";
import { Result } from "./result";
import { routes } from "./routes";

export function checkHealth(): Promise<Result<string, BasicError>> {
  return makeGetCall<string>(routes.health);
}

export function getQrCode(
  urlId: string,
  params: QrCodeParams,
): Promise<Result<number[], BasicError>> {
  return makeGetCall<number[], BasicError, QrCodeParams>(
    routes.url.urlQrCode(urlId),
    params,
  );
}

export function newUrl(
  newUrl: NewUrlRequest,
): Promise<Result<ShortLink, BasicError>> {
  return makePostCall<NewUrlRequest, ShortLink>(routes.url.newUrl, newUrl);
}

export function deleteUrl(
  urlId: string,
): Promise<Result<BasicResponse, BasicError>> {
  return makeDeleteCall<BasicResponse>(routes.url.deleteUrl(urlId));
}

export function updateUrl(
  urlId: string,
  updatedUrl: NewUrlRequest,
): Promise<Result<ShortLink, BasicError>> {
  return makePutCall<NewUrlRequest, ShortLink>(
    routes.url.updateUrl(urlId),
    updatedUrl,
  );
}

export function getUrlInfo(
  urlId: string,
): Promise<Result<ShortLink, BasicError>> {
  return makeGetCall<ShortLink>(routes.url.urlInfo(urlId));
}

export function logout(): Promise<Result<BasicResponse, BasicError>> {
  return makeGetCall<BasicResponse>(routes.user.logout);
}

export function getUserInfo(): Promise<Result<User, BasicError>> {
  return makeGetCall<User>(routes.user.getUser);
}

export function getUserUrls(): Promise<Result<UserLink[], BasicError>> {
  return makeGetCall<UserLink[]>(routes.user.userUrls);
}

export function getUserUrlsPaged(
  params: Paginate,
): Promise<Result<UserLinksAndViews, BasicError>> {
  return makeGetCall<UserLinksAndViews, BasicError, Paginate>(
    routes.user.userUrlsPaged,
    params,
  );
}

export function getOidcProvider(): Promise<Result<OidcName, BasicError>> {
  return makeGetCall<OidcName>(routes.user.oidc.provider);
}

export function getOidcLoginUrl(): string {
  return routes.user.oidc.login;
}

export function registerLocalUser(
  user: NewUserRequest,
): Promise<Result<User, BasicError>> {
  return makePostCall<NewUserRequest, User>(routes.user.local.register, user);
}

export function loginLocalUser(
  user: LoginRequest,
): Promise<Result<User, BasicError>> {
  return makePostCall<LoginRequest, User>(routes.user.local.login, user);
}
