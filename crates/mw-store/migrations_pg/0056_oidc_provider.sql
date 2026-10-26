-- OIDC identity tokens are a distinct authority from mailbox OAuth tokens.
CREATE TABLE oidc_provider_config (
    id BIGINT PRIMARY KEY CHECK(id=1),
    issuer TEXT NOT NULL,
    enabled BIGINT NOT NULL CHECK(enabled IN (0,1)),
    revision TEXT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE TABLE oidc_client (
    client_id TEXT PRIMARY KEY REFERENCES oauth_clients(client_id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    config_json TEXT NOT NULL,
    enabled BIGINT NOT NULL CHECK(enabled IN (0,1)),
    secret_hash TEXT,
    revision TEXT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE TABLE oidc_signing_key (
    kid TEXT PRIMARY KEY,
    issuer TEXT NOT NULL,
    private_sealed BYTEA NOT NULL,
    public_jwk_json TEXT NOT NULL,
    active BIGINT NOT NULL CHECK(active IN (0,1)),
    created_at BIGINT NOT NULL,
    retain_until BIGINT NOT NULL
);
CREATE UNIQUE INDEX oidc_one_active_key ON oidc_signing_key(issuer) WHERE active=1;
CREATE TABLE oidc_consent (
    principal_id TEXT NOT NULL REFERENCES identity_principal(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    client_id TEXT NOT NULL REFERENCES oidc_client(client_id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    client_revision TEXT NOT NULL,
    scopes_json TEXT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY(principal_id,client_id)
);
CREATE TABLE oidc_authorization (
    request_hash TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    client_id TEXT NOT NULL REFERENCES oidc_client(client_id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    client_revision TEXT NOT NULL,
    provider_revision TEXT NOT NULL,
    request_json TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    consumed_at BIGINT
);
CREATE INDEX oidc_authorization_expiry ON oidc_authorization(expires_at);
CREATE TABLE oidc_grant (
    id TEXT PRIMARY KEY,
    client_id TEXT NOT NULL REFERENCES oidc_client(client_id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    client_revision TEXT NOT NULL,
    provider_revision TEXT NOT NULL,
    principal_id TEXT NOT NULL REFERENCES identity_principal(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    claims_json TEXT NOT NULL,
    scopes_json TEXT NOT NULL,
    nonce TEXT,
    auth_time BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    revoked_at BIGINT
);
CREATE INDEX oidc_grant_owner ON oidc_grant(principal_id,client_id);
CREATE INDEX oidc_grant_expiry ON oidc_grant(expires_at);
CREATE TABLE oidc_code (
    code_hash TEXT PRIMARY KEY,
    grant_id TEXT NOT NULL REFERENCES oidc_grant(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    redirect_uri TEXT NOT NULL,
    pkce_challenge TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    consumed_at BIGINT
);
CREATE TABLE oidc_token (
    token_hash TEXT PRIMARY KEY,
    grant_id TEXT NOT NULL REFERENCES oidc_grant(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    kind TEXT NOT NULL CHECK(kind IN ('access','refresh')),
    expires_at BIGINT NOT NULL,
    consumed_at BIGINT
);
CREATE INDEX oidc_token_expiry ON oidc_token(expires_at);
