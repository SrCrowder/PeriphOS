import type { IpcError } from "@/types/app";

function isIpcError(value: unknown): value is IpcError {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    typeof (value as IpcError).message === "string"
  );
}

/** Normalize anything thrown by a command into an {@link IpcError}. */
export function toIpcError(error: unknown): IpcError {
  if (isIpcError(error)) return error;
  if (error instanceof Error) return { code: "unknown", message: error.message };
  return { code: "unknown", message: String(error) };
}

export function errorMessage(error: unknown): string {
  return toIpcError(error).message;
}
