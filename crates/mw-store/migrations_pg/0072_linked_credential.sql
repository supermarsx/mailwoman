-- t48: the credential a linked mailbox is re-established from when its private
-- child session lapses (idle/absolute clocks). One row per link. The secret is
-- sealed with the server key (XChaCha20-Poly1305, purpose "linked-credential-v1")
-- and bound by AAD to owner principal + link id + child subject + child storage
-- authority, so a blob copied onto another row does not open. `state` is
-- 'active' (secret present) or 'rejected' (upstream refused it; secret erased,
-- the owner is offered "Renew access"). Removing the link removes the row.
CREATE TABLE linked_credential (
 link_id TEXT PRIMARY KEY NOT NULL REFERENCES linked_account(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
 owner_principal_id TEXT NOT NULL REFERENCES identity_principal(id) DEFERRABLE INITIALLY DEFERRED,
 owner_account_id TEXT NOT NULL,
 child_authority TEXT NOT NULL,
 login_name TEXT NOT NULL,
 state TEXT NOT NULL CHECK(state IN ('active','rejected')),
 reason TEXT,
 sealed_secret BYTEA,
 updated_at BIGINT NOT NULL
);
CREATE INDEX linked_credential_owner ON linked_credential(owner_principal_id);
CREATE INDEX linked_credential_owner_account ON linked_credential(owner_account_id);
