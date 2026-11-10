// Views only: render view models from Rust, forward intents back. No domain logic here.
import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, ApiFailure } from "./api";
import type { FieldError } from "./generated/FieldError";
import { Projects } from "./Projects";

export function App() {
  const qc = useQueryClient();
  const [title, setTitle] = useState("");
  const [fieldErrors, setFieldErrors] = useState<FieldError[]>([]);

  const items = useQuery({ queryKey: ["items"], queryFn: api.listItems });

  const create = useMutation({
    mutationFn: api.createItem,
    onSuccess: () => {
      setTitle("");
      setFieldErrors([]);
      void qc.invalidateQueries({ queryKey: ["items"] });
    },
    // Validation is decided by Rust; we only display what it returns.
    onError: (e) => setFieldErrors(e instanceof ApiFailure ? e.error.fields : []),
  });

  const complete = useMutation({
    mutationFn: api.completeItem,
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["items"] }),
  });

  return (
    <main className="mx-auto max-w-xl p-6">
      <h1 className="mb-4 text-2xl font-semibold">codpal</h1>

      <form
        className="mb-6 flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          create.mutate({ title });
        }}
      >
        <input
          className="flex-1 rounded border px-3 py-2"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="New item"
        />
        <button className="rounded bg-black px-4 py-2 text-white" type="submit">
          Add
        </button>
      </form>
      {fieldErrors.map((f) => (
        <p key={f.field} className="mb-2 text-sm text-red-600">
          {f.message}
        </p>
      ))}

      {items.isPending && <p>Loading…</p>}
      {items.isError && <p className="text-red-600">Failed to load.</p>}
      <ul className="space-y-2">
        {items.data?.map((item) => (
          <li key={item.id} className="flex items-center justify-between rounded border p-3">
            <span>
              {item.label} <em className="text-xs text-gray-500">{item.badge}</em>
            </span>
            {item.can_complete && (
              <button className="text-sm underline" onClick={() => complete.mutate(item.id)}>
                Complete
              </button>
            )}
          </li>
        ))}
      </ul>

      <Projects />
    </main>
  );
}
