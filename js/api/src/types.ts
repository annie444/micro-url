export type LogoutResponseType =
  | BasicError
  | BasicError
  | BasicResponse
  | BasicResponse
  | BasicError;

export interface UserLinkWithViews {
  id: string;
  short_url: string;
  original_url: string;
  user_id: string;
  expiry_date?: string;
  created_at: string;
  updated_at: string;
  views: UserView[];
}

export interface Views {
  id: number;
  short_link: string;
  headers?: JsonValue;
  ip?: string;
  cache_hit: boolean;
  created_at: string;
}

export type UpdateUrlResponse = BasicError | BasicError | null | ShortLink;

export type GetUrlResponse =
  | BasicError
  | null
  | string
  | BasicError
  | BasicError;

export interface UserView {
  id: number;
  headers?: Partial<Record<string, string[]>>;
  ip?: string;
  cache_hit: boolean;
  created_at: string;
}

export type LoginResponseType = BasicError | BasicError | User | BasicError;

export interface ShortLink {
  id: string;
  short_url: string;
  original_url: string;
  user_id?: string;
  expiry_date?: string;
  created_at: string;
  updated_at: string;
}

export type NewUrlResponse =
  | BasicError
  | BasicError
  | null
  | BasicError
  | ShortLink;

export interface AuthUrl {
  url: string;
  name: string;
}

export type UserLinksResponse =
  | BasicError
  | BasicError
  | UserLinksAndViews
  | UserLink[];

export type UserProfileResponse = BasicError | BasicError | User;

export interface UserLinksAndViews {
  urls: UserLinkWithViews[];
}

export type DeleteUrlResponse = BasicError | BasicError | null | null;

export interface UserPass {
  id: number;
  user_id: string;
  password: string;
}

export interface BasicResponse {
  message: string;
}

export type HeaderMapDef = Partial<Record<string, string[]>>;

export type ImageFormats = "png" | "webp" | "jpeg";

export interface QrCodeParams {
  format?: ImageFormats;
  bg_red?: number;
  bg_green?: number;
  bg_blue?: number;
  bg_alpha?: number;
  fg_red?: number;
  fg_green?: number;
  fg_blue?: number;
  fg_alpha?: number;
}

export type QrCodeResponse =
  | BasicError
  | BasicError
  | null
  | BasicError
  | BasicError
  | BasicError
  | number[]
  | number[]
  | number[];

export interface UserLink {
  id: string;
  short_url: string;
  original_url: string;
  user_id: string;
  expiry_date?: string;
  created_at: string;
  updated_at: string;
  views: bigint;
}

export type AuthUrls = AuthUrl[];

export type OidcCallbackResponseType =
  | BasicError
  | BasicError
  | BasicError
  | BasicError
  | BasicError
  | BasicError
  | BasicError
  | BasicError
  | BasicError
  | string
  | BasicError
  | BasicError
  | BasicError;

export interface BasicError {
  error: string;
}

export interface NewUserRequest {
  name: string;
  email: string;
  password: string;
}

export type GetUrlInfoResponse = BasicError | null | ShortLink;

export interface OidcName {
  name: string;
}

export type OidcNameResponse = OidcName;

export type NewUserResponse = BasicError | BasicError | BasicError | User;

export interface LoginRequest {
  email: string;
  password: string;
}

export interface NewUrlRequest {
  url: string;
  short?: string | null;
  user?: string | null;
  expiry?: string | null;
}

export type JsonValue =
  | number
  | string
  | boolean
  | JsonValue[]
  | { [key in string]?: JsonValue }
  | null;

export interface User {
  user_id: string;
  name: string;
  email: string;
  created_at: string;
  updated_at: string;
}

export interface Sessions {
  id: number;
  session_id: string;
  user_id: string;
  expiry: string;
}

export interface Paginate {
  page: bigint;
  size: bigint;
}
