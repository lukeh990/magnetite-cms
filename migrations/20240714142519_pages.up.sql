-- Add up migration script here
CREATE TABLE IF NOT EXISTS admins (
    id uuid NOT NULL,
    username text NOT NULL,
    enabled boolean NOT NULL,
    email text NOT NULL,
    PRIMARY KEY(id)
);

CREATE TABLE IF NOT EXISTS pages (
    path text NOT NULL,
    created_at timestamp default current_timestamp,
    created_by uuid references admins(id),
    modified_at timestamp default current_timestamp,
    modified_by uuid references admins(id),
    published boolean NOT NULL,
    body text NOT NULL,
    PRIMARY KEY (path)
);
