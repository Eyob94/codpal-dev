// Thin typed fetch wrapper. Types come exclusively from ts-rs (src/generated).
import type { ApiError } from "./generated/ApiError";
import type { CreateItem } from "./generated/CreateItem";
import type { ItemView } from "./generated/ItemView";
import type { CreateProject } from "./generated/CreateProject";
import type { ProjectView } from "./generated/ProjectView";
import type { UpdateProject } from "./generated/UpdateProject";

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
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export const api = {
  listItems: () => request<ItemView[]>("/items"),
  createItem: (intent: CreateItem) =>
    request<ItemView>("/items", { method: "POST", body: JSON.stringify(intent) }),
  completeItem: (id: string) => request<ItemView>(`/items/${id}/complete`, { method: "POST" }),
  listProjects: () => request<ProjectView[]>("/projects"),
  createProject: (intent: CreateProject) =>
    request<ProjectView>("/projects", { method: "POST", body: JSON.stringify(intent) }),
  updateProject: (id: string, intent: UpdateProject) =>
    request<ProjectView>(`/projects/${id}`, { method: "PATCH", body: JSON.stringify(intent) }),
  deleteProject: (id: string) => request<void>(`/projects/${id}`, { method: "DELETE" }),
};
