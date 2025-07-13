import axios, {
  AxiosError,
  type AxiosResponse,
  type AxiosRequestConfig,
} from "axios";
import { BasicError } from "./types";
import { Result, ok, err } from "./result";

function isBasicError(error: unknown | BasicError): error is BasicError {
  let err: BasicError | unknown = error;
  if (err instanceof AxiosError) {
    if (
      err.response &&
      "error" in err.response.data &&
      typeof (err.response.data as Record<string, unknown>).error === "string"
    ) {
      err = err.response.data;
    }
  }
  return (
    typeof err === "object" &&
    err !== null &&
    "error" in err &&
    typeof (err as Record<string, unknown>).error === "string"
  );
}

export function toBasicError(maybeError: unknown | BasicError): BasicError {
  if (isBasicError(maybeError)) return maybeError;

  try {
    return {
      error: JSON.stringify(maybeError),
    };
  } catch {
    // fallback in case there's an error stringifying the maybeError
    // like with circular references for example.
    return {
      error: String(maybeError),
    };
  }
}

export async function makeGetCall<R, E = BasicError, P = unknown>(
  url: string,
  params?: P,
  config?: AxiosRequestConfig,
): Promise<Result<R, E | BasicError>> {
  try {
    const response: AxiosResponse<R> = await axios.get(url, {
      params,
      ...config,
    });
    return ok(response.data);
  } catch (error) {
    if (process.env.NODE_ENV !== "production") {
      console.error(`GET ${url}`, error);
    }
    return err(toBasicError(error));
  }
}

export async function makePostCall<B, R, E = BasicError, P = unknown>(
  url: string,
  body: B,
  params?: P,
  config?: AxiosRequestConfig,
): Promise<Result<R, E | BasicError>> {
  try {
    const response: AxiosResponse<R> = await axios.post(url, body, {
      params,
      ...config,
    });
    return ok(response.data);
  } catch (error) {
    if (process.env.NODE_ENV !== "production") {
      console.error(`POST ${url}`, error);
    }
    return err(toBasicError(error));
  }
}

export async function makeDeleteCall<R, E = BasicError, P = unknown>(
  url: string,
  params?: P,
  config?: AxiosRequestConfig,
): Promise<Result<R, E | BasicError>> {
  try {
    const response: AxiosResponse<R> = await axios.delete(url, {
      params,
      ...config,
    });
    return ok(response.data);
  } catch (error) {
    if (process.env.NODE_ENV !== "production") {
      console.error(`DELETE ${url}`, error);
    }
    return err(toBasicError(error));
  }
}

export async function makePutCall<B, R, E = BasicError, P = unknown>(
  url: string,
  body: B,
  params?: P,
  config?: AxiosRequestConfig,
): Promise<Result<R, E | BasicError>> {
  try {
    const response: AxiosResponse<R> = await axios.put(url, body, {
      params,
      ...config,
    });
    return ok(response.data);
  } catch (error) {
    if (process.env.NODE_ENV !== "production") {
      console.error(`PUT ${url}`, error);
    }
    return err(toBasicError(error));
  }
}
