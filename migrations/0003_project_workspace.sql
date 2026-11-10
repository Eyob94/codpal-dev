CREATE TYPE project_source AS ENUM ('scratch', 'git');
CREATE TYPE project_status AS ENUM ('cloning', 'ready', 'failed');

-- Existing rows predate the workspace, so they default to a ready scratch project.
ALTER TABLE projects
    ADD COLUMN source         project_source NOT NULL DEFAULT 'scratch',
    ADD COLUMN status         project_status NOT NULL DEFAULT 'ready',
    ADD COLUMN status_message TEXT;
