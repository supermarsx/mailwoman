-- Durable feature-profile targets derived only from verified primary sign-ins.
-- Raw upstream account IDs remain directory labels, never policy authorities.
CREATE TABLE feature_user_subject (
 subject TEXT PRIMARY KEY NOT NULL,
 account_id TEXT NOT NULL,
 address TEXT NOT NULL,
 binding_id TEXT NOT NULL REFERENCES identity_binding(id) DEFERRABLE INITIALLY DEFERRED
);
CREATE INDEX feature_user_subject_account ON feature_user_subject(account_id);
