CREATE TABLE items (
    id         UUID PRIMARY KEY,
    title      TEXT        NOT NULL CHECK (char_length(title) BETWEEN 1 AND 120),
    done       BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
