CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    password TEXT NOT NULL,
    salt TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    last_login TIMESTAMPTZ,
    created_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "group" (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS group_membership (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id INTEGER NOT NULL REFERENCES "group"(id) ON DELETE CASCADE,
    created_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);

CREATE TABLE IF NOT EXISTS permissions_group (
    id SERIAL PRIMARY KEY,
    resource TEXT NOT NULL,
    group_id INTEGER NOT NULL REFERENCES "group"(id) ON DELETE CASCADE,
    perm_value INTEGER NOT NULL,
    note TEXT,
    created_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS permissions_type (
    id INTEGER PRIMARY KEY,
    type TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS permission_methods (
    id SERIAL PRIMARY KEY,
    method TEXT NOT NULL UNIQUE,
    perm_type INTEGER NOT NULL REFERENCES permissions_type(id)
);

CREATE TABLE IF NOT EXISTS data_catalog (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    base_query TEXT NOT NULL,
    created_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO "group" (name, description)
VALUES
    ('All Users', 'Everyone'),
    ('Administrators', 'Admin group'),
    ('Editors', 'The moderators')
ON CONFLICT (id) DO NOTHING;

INSERT INTO users (email, first_name, last_name, password, salt, status)
VALUES
    ('admin@sample.com', 'admin', 'super', '$argon2id$v=19$m=19456,t=2,p=1$WVdaelp6RXlaMlpv$DfW5+RL//bsrEcyZ4QXOQQ', 'YWZzZzEyZ2Zo', 'active')
ON CONFLICT (id) DO NOTHING;

INSERT INTO group_membership (user_id, group_id)
VALUES
    (1, 1),
    (1, 2)
ON CONFLICT DO NOTHING;

INSERT INTO permissions_group (resource, group_id, perm_value, note)
VALUES
    ('/*', 2, 7, 'Admin permission'),
    ('/v1/users/*', 3, 1, 'Editors can read users')
ON CONFLICT DO NOTHING;

INSERT INTO permissions_type (id, type) VALUES
    (1, 'read'),
    (2, 'write'),
    (3, 'read_write'),
    (4, 'delete'),
    (5, 'delete_read'),
    (6, 'delete_write'),
    (7, 'read_write_delete')
ON CONFLICT (id) DO NOTHING;

INSERT INTO permission_methods (method, perm_type) VALUES
    ('GET', 1),
    ('POST', 2),
    ('PUT', 2),
    ('DELETE', 4)
ON CONFLICT (method) DO NOTHING;

