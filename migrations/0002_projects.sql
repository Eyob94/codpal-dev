CREATE TABLE projects (
    id          UUID PRIMARY KEY,
    name        TEXT        NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
    repo_url    TEXT        CHECK (repo_url ~ '^(https?://|ssh://|git@[^:/]+:)'),
    description TEXT        CHECK (char_length(description) BETWEEN 1 AND 1000),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX projects_name_lower_idx ON projects (lower(name));
