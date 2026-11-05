-- Cookie consent: the consent log, admin classification rules for cookies and
-- browser storage keys, and names of storage the browser scan found that no
-- inventory entry or rule covers. Names and counts only: no cookie or storage
-- value is ever stored. A consent row holds no address, IP or user agent; the
-- account id is present only when the person was signed in.
CREATE TABLE cookie_consent (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT,
    domain TEXT NOT NULL,
    policy_version BIGINT NOT NULL,
    decision TEXT NOT NULL,
    categories_json TEXT NOT NULL,
    region TEXT NOT NULL,
    signal TEXT NOT NULL,
    created_at BIGINT NOT NULL
);
CREATE INDEX cookie_consent_time ON cookie_consent(created_at);
CREATE INDEX cookie_consent_account ON cookie_consent(account_id, created_at);

CREATE TABLE cookie_rule (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    pattern TEXT NOT NULL,
    category TEXT NOT NULL,
    note TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    UNIQUE(kind, pattern)
);

CREATE TABLE cookie_detected (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    first_seen BIGINT NOT NULL,
    last_seen BIGINT NOT NULL,
    seen_count BIGINT NOT NULL,
    status TEXT NOT NULL,
    UNIQUE(kind, name)
);
