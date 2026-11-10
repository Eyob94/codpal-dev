// Thin typed fetch wrapper. Types come exclusively from ts-rs (src/generated).
import type { ApiError } from "./generated/ApiError";
import type { CreateItem } from "./generated/CreateItem";
import type { ItemView } from "./generated/ItemView";

export class ApiFailure extends Error {
  constructor(public readonly error: ApiError) {
    super(error.message);
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`/api/v1${path}`, {
    ...init,
    headers: { "content-type": "application/json", ...init?.headers },
  });
  if (!res.ok) throw new ApiFailure((await res.json()) as ApiError);
  return (await res.json()) as T;
}

export const api = {
  listItems: () => request<ItemView[]>("/items"),
  createItem: (intent: CreateItem) =>
    request<ItemView>("/items", { method: "POST", body: JSON.stringify(intent) }),
  completeItem: (id: string) => request<ItemView>(`/items/${id}/complete`, { method: "POST" }),
};
