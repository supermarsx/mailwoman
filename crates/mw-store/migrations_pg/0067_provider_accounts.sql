-- Provider accounts retain existing IDs/FKs; no parent-table rebuild.
ALTER TABLE accounts ADD COLUMN backend_kind TEXT NOT NULL DEFAULT 'mail' CHECK (backend_kind IN ('mail','ews'));
ALTER TABLE accounts ADD COLUMN backend_authority TEXT NOT NULL DEFAULT '' CHECK ((backend_kind='mail' AND backend_authority='') OR (backend_kind='ews' AND length(backend_authority)=64));
DROP INDEX idx_accounts_identity;
CREATE UNIQUE INDEX idx_accounts_identity ON accounts (kind, (rtrim(translate(host, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), '.')), port, (translate(username, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz')), backend_kind, backend_authority);
CREATE TABLE account_provider (
    account_id TEXT PRIMARY KEY NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    provider TEXT NOT NULL CHECK (provider='ews'),
    endpoint_url TEXT NOT NULL,
    backend_username TEXT NOT NULL,
    login_address TEXT NOT NULL,
    domain_binding TEXT NOT NULL,
    ca_pem TEXT,
    sealed_credential BYTEA NOT NULL,
    revision TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
