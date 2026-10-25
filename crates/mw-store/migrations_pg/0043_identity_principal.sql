-- Identity bindings are permanent tombstones: a retired verified subject is never
-- automatically assigned to a new principal. Email addresses are not identity keys.
CREATE TABLE identity_serialization (id INTEGER PRIMARY KEY CHECK (id=1));
INSERT INTO identity_serialization(id) VALUES(1);
CREATE TABLE identity_principal (
 id TEXT PRIMARY KEY NOT NULL,
 state TEXT NOT NULL CHECK (state IN ('active','disabled','retired')),
 created_at BIGINT NOT NULL,
 retired_at BIGINT
);
CREATE TABLE identity_binding (
 id TEXT PRIMARY KEY NOT NULL,
 principal_id TEXT NOT NULL REFERENCES identity_principal(id),
 kind TEXT NOT NULL,
 authority TEXT NOT NULL,
 subject TEXT NOT NULL,
 created_at BIGINT NOT NULL,
 retired_at BIGINT,
 UNIQUE(kind,authority,subject)
);
CREATE INDEX identity_binding_principal ON identity_binding(principal_id);
CREATE TABLE session_principal (
 session_id TEXT PRIMARY KEY NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
 principal_id TEXT NOT NULL REFERENCES identity_principal(id),
 binding_id TEXT NOT NULL REFERENCES identity_binding(id),
 primary_account_id TEXT NOT NULL,
 auth_time BIGINT NOT NULL,
 authentication_method TEXT NOT NULL
);
CREATE INDEX session_principal_owner ON session_principal(principal_id);
