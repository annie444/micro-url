import axios, { AxiosError, type AxiosResponse, AxiosRequestConfig } from "axios";
import { BasicError } from "./types";

function isBasicError(error: unknown | BasicError): error is BasicError {
  let err: BasicError | unknown = error;
  if (err instanceof AxiosError) {
    if (
      err.response &&
      "error" in err.response?.data &&
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

function toBasicError(maybeError: unknown | BasicError): BasicError {
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

export async function makeGetCall<T, P = unknown>(
  url: string,
  params?: P,
): Promise<T> {
  try {
    const response: AxiosResponse<T> = await axios.get(url, { params });
    return response.data;
  } catch (error) {
    if (isBasicError(error)) {
      throw toBasicError(error);
    } else {
      throw toBasicError(error);
    }
  }
}
