export class APIError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}
export async function api<T = any>(
  path: string,
  body?: unknown,
  method = body === undefined ? "GET" : "POST",
): Promise<T> {
  const response = await fetch(`/api${path}`, {
    method,
    headers: { "Content-Type": "application/json", "X-CloudLab-Client": "web" },
    credentials: "same-origin",
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const value = await response.json().catch(() => ({
    error: "The coordinator returned an unreadable response.",
  }));
  if (!response.ok)
    throw new APIError(
      response.status,
      value.error || `Request failed (${response.status})`,
    );
  return value as T;
}
