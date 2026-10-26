-- Pre-login OIDC requests are not authenticated sessions. Only hashed, bounded
-- browser-bound handles are stored; consent later binds a real primary session.
CREATE TABLE oidc_browser_flow (
    flow_hash TEXT PRIMARY KEY,
    browser_hash TEXT NOT NULL,
    source_hash TEXT NOT NULL,
    request_json TEXT NOT NULL,
    initial_session_id TEXT,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    consumed_at BIGINT
);
CREATE INDEX oidc_browser_flow_expiry ON oidc_browser_flow(expires_at);
CREATE INDEX oidc_browser_flow_source ON oidc_browser_flow(source_hash,created_at);
CREATE INDEX oidc_browser_flow_browser ON oidc_browser_flow(browser_hash,expires_at);
