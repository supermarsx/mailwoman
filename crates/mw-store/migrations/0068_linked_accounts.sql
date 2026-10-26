-- Internal sessions are never cookie/bearer credentials. Their original sign-in
-- clocks remain unchanged; authorization also requires the owning live session.
ALTER TABLE sessions ADD COLUMN linked_internal INTEGER NOT NULL DEFAULT 0 CHECK(linked_internal IN (0,1,2));
CREATE TABLE linked_account (
 id TEXT PRIMARY KEY NOT NULL,
 owner_principal_id TEXT NOT NULL REFERENCES identity_principal(id) DEFERRABLE INITIALLY DEFERRED,
 child_principal_id TEXT NOT NULL REFERENCES identity_principal(id) DEFERRABLE INITIALLY DEFERRED,
 child_subject TEXT NOT NULL,
 account_id TEXT NOT NULL,
 display_name TEXT NOT NULL,
 sealed_session BLOB NOT NULL,
 revision TEXT NOT NULL,
 created_at BIGINT NOT NULL,
 UNIQUE(owner_principal_id,child_subject)
);
CREATE TABLE linked_context (
 session_id TEXT PRIMARY KEY NOT NULL REFERENCES sessions(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
 owner_session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
 link_id TEXT NOT NULL REFERENCES linked_account(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
 link_revision TEXT NOT NULL,
 UNIQUE(owner_session_id,link_id)
);
CREATE INDEX linked_context_link ON linked_context(link_id);
CREATE TABLE linked_challenge (
 id TEXT PRIMARY KEY NOT NULL,
 owner_session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
 sealed_pending BLOB NOT NULL,
 display_name TEXT NOT NULL,
 expires_at BIGINT NOT NULL
);
CREATE INDEX linked_challenge_expiry ON linked_challenge(expires_at);
