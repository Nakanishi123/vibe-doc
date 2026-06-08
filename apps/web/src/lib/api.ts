import type { ApiError } from "./api-types";
import type { LoadState } from "./app-types";

export async function loadJson<T>(path: string): Promise<LoadState<T>> {
  try {
    const response = await fetch(path, { headers: { Accept: "application/json" } });
    const payload: unknown = await response.json();
    if (!response.ok) {
      return { status: "error", message: apiErrorMessage(payload, response.status) };
    }
    return { status: "ready", data: payload as T };
  } catch (error) {
    return {
      status: "error",
      message: error instanceof Error ? error.message : "Request failed",
    };
  }
}

function apiErrorMessage(payload: unknown, status: number) {
  const maybeError = payload as { error?: ApiError };
  if (maybeError.error?.message) {
    return maybeError.error.message;
  }
  return `Request failed with HTTP ${status}`;
}
