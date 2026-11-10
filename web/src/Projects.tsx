// Views only: render ProjectView from Rust, forward intents back. No domain logic here.
import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, ApiFailure } from "./api";
import type { FieldError } from "./generated/FieldError";
import type { ProjectView } from "./generated/ProjectView";

function useFieldErrors() {
  const [errors, setErrors] = useState<FieldError[]>([]);
  // Validation is decided by Rust; we only display what it returns.
  const onError = (e: unknown) =>
    setErrors(
      e instanceof ApiFailure
        ? e.error.fields.length > 0
          ? e.error.fields
          : [{ field: "", message: e.error.message }]
        : [{ field: "", message: "Something went wrong." }],
    );
  return { errors, setErrors, onError };
}

function Errors({ errors }: { errors: FieldError[] }) {
  return (
    <>
      {errors.map((f) => (
        <p key={`${f.field}:${f.message}`} className="mb-1 text-sm text-red-600">
          {f.message}
        </p>
      ))}
    </>
  );
}

interface FormValues {
  name: string;
  repo_url: string;
  description: string;
}

function ProjectForm(props: {
  initial: FormValues;
  submitLabel: string;
  errors: FieldError[];
  pending: boolean;
  onSubmit: (v: FormValues) => void;
  onCancel?: () => void;
}) {
  const [v, setV] = useState(props.initial);
  const input = "w-full rounded border px-3 py-2";
  return (
    <form
      className="space-y-2"
      onSubmit={(e) => {
        e.preventDefault();
        props.onSubmit(v);
      }}
    >
      <input
        className={input}
        value={v.name}
        onChange={(e) => setV({ ...v, name: e.target.value })}
        placeholder="Name"
        aria-label="Name"
      />
      <input
        className={input}
        value={v.repo_url}
        onChange={(e) => setV({ ...v, repo_url: e.target.value })}
        placeholder="Repository URL (optional)"
        aria-label="Repository URL"
      />
      <textarea
        className={input}
        value={v.description}
        onChange={(e) => setV({ ...v, description: e.target.value })}
        placeholder="Description (optional)"
        aria-label="Description"
      />
      <Errors errors={props.errors} />
      <div className="flex gap-2">
        <button
          className="rounded bg-black px-4 py-2 text-white disabled:opacity-50"
          type="submit"
          disabled={props.pending}
        >
          {props.submitLabel}
        </button>
        {props.onCancel && (
          <button className="px-4 py-2 underline" type="button" onClick={props.onCancel}>
            Cancel
          </button>
        )}
      </div>
    </form>
  );
}

const EMPTY: FormValues = { name: "", repo_url: "", description: "" };

function ProjectRow({ project }: { project: ProjectView }) {
  const qc = useQueryClient();
  const [editing, setEditing] = useState(false);
  const { errors, setErrors, onError } = useFieldErrors();
  const refresh = () => void qc.invalidateQueries({ queryKey: ["projects"] });

  const update = useMutation({
    mutationFn: (v: FormValues) =>
      // Empty strings clear optional fields; Rust interprets that.
      api.updateProject(project.id, { name: v.name, repo_url: v.repo_url, description: v.description }),
    onSuccess: () => {
      setEditing(false);
      setErrors([]);
      refresh();
    },
    onError,
  });
  const remove = useMutation({ mutationFn: () => api.deleteProject(project.id), onSuccess: refresh, onError });

  if (editing) {
    return (
      <li className="rounded border p-3">
        <ProjectForm
          initial={{
            name: project.name,
            repo_url: project.repo_url ?? "",
            description: project.description ?? "",
          }}
          submitLabel="Save"
          errors={errors}
          pending={update.isPending}
          onSubmit={(v) => update.mutate(v)}
          onCancel={() => {
            setEditing(false);
            setErrors([]);
          }}
        />
      </li>
    );
  }

  return (
    <li className="rounded border p-3">
      <div className="flex items-start justify-between gap-2">
        <div>
          <p className="font-medium">{project.name}</p>
          {project.repo_url && <p className="text-sm text-gray-600">{project.repo_url}</p>}
          {project.description && <p className="mt-1 text-sm">{project.description}</p>}
        </div>
        <div className="flex gap-3 text-sm">
          <button className="underline" onClick={() => setEditing(true)}>
            Edit
          </button>
          <button className="underline" onClick={() => remove.mutate()} disabled={remove.isPending}>
            Delete
          </button>
        </div>
      </div>
      <Errors errors={errors} />
    </li>
  );
}

export function Projects() {
  const qc = useQueryClient();
  const { errors, setErrors, onError } = useFieldErrors();
  const [formKey, setFormKey] = useState(0);
  const projects = useQuery({ queryKey: ["projects"], queryFn: api.listProjects });

  const create = useMutation({
    mutationFn: (v: FormValues) =>
      api.createProject({
        name: v.name,
        repo_url: v.repo_url || null,
        description: v.description || null,
      }),
    onSuccess: () => {
      setErrors([]);
      setFormKey((k) => k + 1); // reset the form
      void qc.invalidateQueries({ queryKey: ["projects"] });
    },
    onError,
  });

  return (
    <section className="mt-10">
      <h2 className="mb-4 text-xl font-semibold">Projects</h2>
      <div className="mb-6">
        <ProjectForm
          key={formKey}
          initial={EMPTY}
          submitLabel="Create project"
          errors={errors}
          pending={create.isPending}
          onSubmit={(v) => create.mutate(v)}
        />
      </div>
      {projects.isPending && <p>Loading…</p>}
      {projects.isError && <p className="text-red-600">Failed to load.</p>}
      <ul className="space-y-2">
        {projects.data?.map((p) => <ProjectRow key={p.id} project={p} />)}
      </ul>
    </section>
  );
}
