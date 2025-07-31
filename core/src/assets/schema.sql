CREATE TABLE IF NOT EXISTS project_groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audio_projects (
    id TEXT PRIMARY KEY,
    group_id TEXT NOT NULL REFERENCES project_groups(id),
    name TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    date TEXT NOT NULL,
    type TEXT NOT NULL,
    language TEXT NOT NULL
);

DROP TABLE IF EXISTS project_notes;
CREATE VIRTUAL TABLE IF NOT EXISTS project_notes USING fts5(
    project_id UNINDEXED,
    transcript,
    summary,
    email
);