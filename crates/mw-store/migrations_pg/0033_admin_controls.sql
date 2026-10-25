-- 0033 (26.20 t32-e10a): the admin-controls schema — POSTGRES variant.
-- Behaviourally identical to the SQLite `migrations/0033_admin_controls.sql`, whose
-- header carries the full rationale (conventions, upgrade backfills, what is left
-- to the consuming lanes). ADDITIVE over 0001..0032 — NEVER edit an earlier
-- migration.
--
-- Dialect differences only:
-- * INTEGER -> BIGINT (the uniform i64 bind/read path) and BLOB -> BYTEA;
-- * every foreign key is `DEFERRABLE INITIALLY IMMEDIATE`, as in 0002 onward, so
--   `migrate-store` can defer them to commit (`SET CONSTRAINTS ALL DEFERRED`);
-- * `ALTER TABLE … ADD COLUMN` with a constant default does not rewrite the table
--   on Postgres 11+, so the ALTERs below are cheap on large tables.
--
-- New timestamps are unix seconds; booleans are 0/1 integers; ids are TEXT minted
-- by the application. Uniqueness uses NAMED unique indexes so the names match the
-- SQLite variant. `audit_log.seq` stays NULL on existing rows (assigned by the
-- audit-chain lane at its genesis checkpoint).

-- ── C/AG settings model ─────────────────────────────────────────────────────
-- Global value and lock per registry key; the scoped values below it.
CREATE TABLE IF NOT EXISTS setting_policy (
    key                  TEXT PRIMARY KEY NOT NULL,
    global_value         TEXT,
    allow_user_override  BIGINT NOT NULL DEFAULT 1,
    global_locked        BIGINT NOT NULL DEFAULT 0,
    updated_by           TEXT NOT NULL,
    updated_at           BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS setting_value (
    scope_kind  TEXT NOT NULL CHECK (scope_kind IN ('domain', 'profile', 'group', 'admin_user', 'user')),
    scope_id    TEXT NOT NULL,
    key         TEXT NOT NULL,
    value       TEXT NOT NULL,
    locked      BIGINT NOT NULL DEFAULT 0,
    updated_by  TEXT NOT NULL,
    updated_at  BIGINT NOT NULL,
    PRIMARY KEY (scope_kind, scope_id, key)
);
CREATE INDEX IF NOT EXISTS idx_setting_value_key ON setting_value (key);

-- ── D security controls ─────────────────────────────────────────────────────
-- Sign-in failure counters (DB-backed so they survive a restart and are shared
-- by every process on one database).
CREATE TABLE IF NOT EXISTS auth_failure (
    surface       TEXT NOT NULL CHECK (surface IN ('admin', 'webmail')),
    key_kind      TEXT NOT NULL CHECK (key_kind IN ('ip', 'account')),
    key           TEXT NOT NULL,
    window_start  BIGINT NOT NULL,
    count         BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (surface, key_kind, key)
);
CREATE INDEX IF NOT EXISTS idx_auth_failure_window ON auth_failure (window_start);

-- IP/CIDR blocks; `blocked_until` NULL = permanent.
CREATE TABLE IF NOT EXISTS ip_block (
    id             TEXT PRIMARY KEY NOT NULL,
    ip_or_cidr     TEXT NOT NULL,
    surface        TEXT NOT NULL CHECK (surface IN ('admin', 'webmail', 'all')),
    reason         TEXT NOT NULL CHECK (reason IN ('auto', 'manual')),
    offences       BIGINT NOT NULL DEFAULT 1,
    blocked_until  BIGINT,
    created_by     TEXT NOT NULL,
    created_at     BIGINT NOT NULL,
    note           TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_ip_block_until ON ip_block (blocked_until);
CREATE INDEX IF NOT EXISTS idx_ip_block_ip ON ip_block (ip_or_cidr);

-- IP allowlists, global or per domain/account (no rows = no restriction).
CREATE TABLE IF NOT EXISTS ip_allow (
    id          TEXT PRIMARY KEY NOT NULL,
    surface     TEXT NOT NULL CHECK (surface IN ('admin', 'webmail')),
    scope_kind  TEXT NOT NULL DEFAULT 'global' CHECK (scope_kind IN ('global', 'domain', 'account')),
    scope_id    TEXT NOT NULL DEFAULT '',
    cidr        TEXT NOT NULL,
    note        TEXT NOT NULL DEFAULT '',
    created_by  TEXT NOT NULL,
    created_at  BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ip_allow_scope ON ip_allow (surface, scope_kind, scope_id);

-- ── E/G domains ─────────────────────────────────────────────────────────────
ALTER TABLE domains ADD COLUMN enabled BIGINT NOT NULL DEFAULT 1;
ALTER TABLE domains ADD COLUMN preferred BIGINT NOT NULL DEFAULT 0;
ALTER TABLE domains ADD COLUMN settings_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE domains ADD COLUMN updated_at BIGINT;
-- At most one preferred domain.
CREATE UNIQUE INDEX IF NOT EXISTS idx_domains_preferred ON domains (preferred) WHERE preferred = 1;

CREATE TABLE IF NOT EXISTS domain_alias (
    alias   TEXT PRIMARY KEY NOT NULL,
    domain  TEXT NOT NULL REFERENCES domains(name) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE
);
CREATE INDEX IF NOT EXISTS idx_domain_alias_domain ON domain_alias (domain);

CREATE TABLE IF NOT EXISTS domain_host (
    host    TEXT PRIMARY KEY NOT NULL,
    domain  TEXT NOT NULL REFERENCES domains(name) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE
);
CREATE INDEX IF NOT EXISTS idx_domain_host_domain ON domain_host (domain);

-- ── J SSH tunnels ───────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ssh_tunnel (
    id                  TEXT PRIMARY KEY NOT NULL,
    name                TEXT NOT NULL,
    host                TEXT NOT NULL,
    port                BIGINT NOT NULL,
    username            TEXT NOT NULL,
    sealed_private_key  BYTEA NOT NULL,                  -- SEALED
    public_key          TEXT NOT NULL,
    key_algorithm       TEXT NOT NULL,
    host_key_pins_json  TEXT NOT NULL DEFAULT '[]',
    keepalive_secs      BIGINT NOT NULL DEFAULT 30,
    connect_timeout_ms  BIGINT NOT NULL DEFAULT 5000,
    max_channels        BIGINT NOT NULL DEFAULT 16,
    enabled             BIGINT NOT NULL DEFAULT 1,
    created_by          TEXT NOT NULL,
    created_at          BIGINT NOT NULL,
    updated_at          BIGINT NOT NULL
);

-- ── M admin accounts and roles ──────────────────────────────────────────────
-- `admin_users` (0007) holds provisioned MAIL users, so admin accounts get a new
-- name; nothing is renamed. `password_hash` NULL = an env-bootstrap account whose
-- password lives in the environment.
CREATE TABLE IF NOT EXISTS admin_account (
    id             TEXT PRIMARY KEY NOT NULL,
    username       TEXT NOT NULL,
    display_name   TEXT NOT NULL,
    password_hash  TEXT,
    disabled       BIGINT NOT NULL DEFAULT 0,
    source         TEXT NOT NULL CHECK (source IN ('db', 'env-bootstrap')),
    created_at     BIGINT NOT NULL,
    updated_at     BIGINT NOT NULL,
    last_login_at  BIGINT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_account_username ON admin_account (username);

CREATE TABLE IF NOT EXISTS admin_totp (
    account_id     TEXT PRIMARY KEY NOT NULL REFERENCES admin_account(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    sealed_secret  BYTEA NOT NULL,                        -- SEALED
    last_step      BIGINT NOT NULL DEFAULT 0,           -- replay guard
    enrolled_at    BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS admin_role (
    id       TEXT PRIMARY KEY NOT NULL,
    name     TEXT NOT NULL,
    builtin  BIGINT NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_role_name ON admin_role (name);

CREATE TABLE IF NOT EXISTS admin_role_capability (
    role_id     TEXT NOT NULL REFERENCES admin_role(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    capability  TEXT NOT NULL,
    PRIMARY KEY (role_id, capability)
);

-- `domains_json` NULL = the role applies to every domain.
CREATE TABLE IF NOT EXISTS admin_account_role (
    account_id    TEXT NOT NULL REFERENCES admin_account(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    role_id       TEXT NOT NULL REFERENCES admin_role(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    domains_json  TEXT,
    PRIMARY KEY (account_id, role_id)
);
CREATE INDEX IF NOT EXISTS idx_admin_account_role_role ON admin_account_role (role_id);

CREATE TABLE IF NOT EXISTS admin_capability_override (
    account_id  TEXT NOT NULL REFERENCES admin_account(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    capability  TEXT NOT NULL,
    effect      TEXT NOT NULL CHECK (effect IN ('allow', 'deny')),
    PRIMARY KEY (account_id, capability)
);

-- ── AD/M audit ──────────────────────────────────────────────────────────────
-- `audit_log.id` is TEXT (0007), not an integer, so the DB-ordered position in
-- the hash chain is the new `seq` column. The integer cursors below
-- (`audit_chain_head.last_id`, `audit_checkpoint.pruned_through_id`,
-- `audit_sink.cursor_id`) are `seq` values.
ALTER TABLE audit_log ADD COLUMN admin_role TEXT;
ALTER TABLE audit_log ADD COLUMN capability TEXT;
ALTER TABLE audit_log ADD COLUMN category TEXT;
ALTER TABLE audit_log ADD COLUMN request_id TEXT;
ALTER TABLE audit_log ADD COLUMN before_json TEXT;
ALTER TABLE audit_log ADD COLUMN after_json TEXT;
ALTER TABLE audit_log ADD COLUMN prev_hash TEXT;
ALTER TABLE audit_log ADD COLUMN hash TEXT;
ALTER TABLE audit_log ADD COLUMN seq BIGINT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_audit_log_seq ON audit_log (seq);
CREATE INDEX IF NOT EXISTS idx_audit_log_request ON audit_log (request_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_category ON audit_log (category, ts);

-- The single chain head row, locked by every append.
CREATE TABLE IF NOT EXISTS audit_chain_head (
    id       BIGINT PRIMARY KEY NOT NULL DEFAULT 1 CHECK (id = 1),
    last_id  BIGINT NOT NULL DEFAULT 0,
    hash     TEXT
);

CREATE TABLE IF NOT EXISTS audit_checkpoint (
    id                   TEXT PRIMARY KEY NOT NULL,
    kind                 TEXT NOT NULL CHECK (kind IN ('genesis', 'prune')),
    pruned_through_id    BIGINT NOT NULL DEFAULT 0,
    pruned_through_hash  TEXT,
    ts                   BIGINT NOT NULL
);

-- `days` NULL = keep forever.
CREATE TABLE IF NOT EXISTS audit_retention (
    category    TEXT PRIMARY KEY NOT NULL,
    days        BIGINT,
    legal_hold  BIGINT NOT NULL DEFAULT 0,
    updated_by  TEXT NOT NULL,
    updated_at  BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_sink (
    id             TEXT PRIMARY KEY NOT NULL,
    kind           TEXT NOT NULL CHECK (kind IN ('syslog', 'webhook')),
    sealed_config  BYTEA NOT NULL,                        -- SEALED
    enabled        BIGINT NOT NULL DEFAULT 1,
    cursor_id      BIGINT NOT NULL DEFAULT 0,
    last_ok_at     BIGINT,
    last_error     TEXT,
    created_by     TEXT NOT NULL,
    created_at     BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS admin_saved_query (
    id                TEXT PRIMARY KEY NOT NULL,
    admin_account_id  TEXT NOT NULL REFERENCES admin_account(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    name              TEXT NOT NULL,
    filter_json       TEXT NOT NULL,
    created_at        BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_admin_saved_query_account ON admin_saved_query (admin_account_id);

CREATE TABLE IF NOT EXISTS alert_rule (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL,
    condition_json  TEXT NOT NULL,
    channels_json   TEXT NOT NULL,
    throttle_secs   BIGINT NOT NULL DEFAULT 900,
    enabled         BIGINT NOT NULL DEFAULT 1,
    created_by      TEXT NOT NULL,
    created_at      BIGINT NOT NULL,
    updated_at      BIGINT NOT NULL
);

-- History: no foreign key, so deleting a rule keeps what it fired.
CREATE TABLE IF NOT EXISTS alert_event (
    id              TEXT PRIMARY KEY NOT NULL,
    rule_id         TEXT NOT NULL,
    fired_at        BIGINT NOT NULL,
    summary         TEXT NOT NULL,
    delivered_json  TEXT NOT NULL,
    audit_id        TEXT
);
CREATE INDEX IF NOT EXISTS idx_alert_event_rule ON alert_event (rule_id, fired_at);

-- ── AD sessions and sign-in history ─────────────────────────────────────────
ALTER TABLE sessions ADD COLUMN user_agent TEXT;
ALTER TABLE sessions ADD COLUMN ip TEXT;
ALTER TABLE sessions ADD COLUMN client_type TEXT;
ALTER TABLE sessions ADD COLUMN device_label TEXT;
ALTER TABLE sessions ADD COLUMN client_version TEXT;

CREATE TABLE IF NOT EXISTS signin_event (
    id                  TEXT PRIMARY KEY NOT NULL,
    account_id          TEXT,
    username_attempted  TEXT NOT NULL,
    ts                  BIGINT NOT NULL,
    result              TEXT NOT NULL CHECK (result IN ('success', 'failure', 'blocked')),
    method              TEXT NOT NULL,
    ip                  TEXT NOT NULL,
    user_agent          TEXT NOT NULL,
    geo_country         TEXT,
    reason              TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_signin_event_account ON signin_event (account_id, ts);
CREATE INDEX IF NOT EXISTS idx_signin_event_ts ON signin_event (ts);

CREATE TABLE IF NOT EXISTS remembered_device (
    id            TEXT PRIMARY KEY NOT NULL,
    account_id    TEXT NOT NULL,
    token_hash    TEXT NOT NULL,                         -- SHA-256 of the token; never the token
    label         TEXT NOT NULL,
    created_at    BIGINT NOT NULL,
    expires_at    BIGINT NOT NULL,
    last_used_at  BIGINT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_remembered_device_token ON remembered_device (token_hash);
CREATE INDEX IF NOT EXISTS idx_remembered_device_account ON remembered_device (account_id);

ALTER TABLE webauthn_credentials ADD COLUMN discoverable BIGINT NOT NULL DEFAULT 0;

-- ── P/Q/Z/X/AF branding and stored assets ───────────────────────────────────
CREATE TABLE IF NOT EXISTS branding (
    scope        TEXT PRIMARY KEY NOT NULL,
    config_json  TEXT NOT NULL,
    updated_by   TEXT NOT NULL,
    updated_at   BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS brand_asset (
    id             TEXT PRIMARY KEY NOT NULL,
    scope          TEXT NOT NULL,
    kind           TEXT NOT NULL CHECK (kind IN ('logo', 'background', 'icon_master', 'font', 'preset',
                                                 'user_background', 'signature', 'template_attachment',
                                                 'sound')),
    owner_account  TEXT,
    sha256         TEXT NOT NULL,
    content_type   TEXT NOT NULL,
    bytes          BIGINT NOT NULL,
    width          BIGINT,
    height         BIGINT,
    duration_ms    BIGINT,
    licence_ack    TEXT,
    data           BYTEA NOT NULL,
    created_by     TEXT NOT NULL,
    created_at     BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_brand_asset_owner ON brand_asset (owner_account, kind);
CREATE INDEX IF NOT EXISTS idx_brand_asset_scope ON brand_asset (scope, kind);

CREATE TABLE IF NOT EXISTS custom_css (
    scope_kind  TEXT NOT NULL CHECK (scope_kind IN ('global_login', 'global_app', 'domain_login',
                                                    'domain_app', 'user')),
    scope_id    TEXT NOT NULL,
    source      TEXT NOT NULL,
    sanitized   TEXT NOT NULL,
    sha256      TEXT NOT NULL,
    updated_by  TEXT NOT NULL,
    updated_at  BIGINT NOT NULL,
    PRIMARY KEY (scope_kind, scope_id)
);

-- ── V2 custom themes and accents ────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS custom_theme (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    appearance   TEXT NOT NULL CHECK (appearance IN ('light', 'dark')),
    tokens_json  TEXT NOT NULL,
    enabled      BIGINT NOT NULL DEFAULT 1,
    scope        TEXT NOT NULL DEFAULT 'global',
    created_by   TEXT NOT NULL,
    created_at   BIGINT NOT NULL,
    updated_at   BIGINT NOT NULL
);

-- `shades_json`: theme id -> hex colour.
CREATE TABLE IF NOT EXISTS custom_accent (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    shades_json  TEXT NOT NULL,
    enabled      BIGINT NOT NULL DEFAULT 1,
    scope        TEXT NOT NULL DEFAULT 'global',
    created_by   TEXT NOT NULL,
    created_at   BIGINT NOT NULL,
    updated_at   BIGINT NOT NULL
);

-- ── S Sieve scripts ─────────────────────────────────────────────────────────
-- `state_json`: vacation `:days`/`:seconds` and `duplicate` state.
CREATE TABLE IF NOT EXISTS sieve_script (
    account_id  TEXT NOT NULL,
    name        TEXT NOT NULL,
    content     TEXT NOT NULL,
    active      BIGINT NOT NULL DEFAULT 0,
    size        BIGINT NOT NULL,
    state_json  TEXT NOT NULL DEFAULT '{}',
    updated_at  BIGINT NOT NULL,
    PRIMARY KEY (account_id, name)
);
-- At most one active script per account.
CREATE UNIQUE INDEX IF NOT EXISTS idx_sieve_script_active ON sieve_script (account_id) WHERE active = 1;

-- ── R2 egress routes ────────────────────────────────────────────────────────
ALTER TABLE egress_proxy ADD COLUMN kind TEXT NOT NULL DEFAULT 'http'
    CHECK (kind IN ('http', 'socks5', 'socks5h', 'ssh'));
ALTER TABLE egress_proxy ADD COLUMN tunnel_id TEXT;
ALTER TABLE egress_proxy ADD COLUMN tls_to_proxy BIGINT NOT NULL DEFAULT 0;
ALTER TABLE egress_proxy ADD COLUMN remote_dns BIGINT NOT NULL DEFAULT 0;
ALTER TABLE egress_proxy ADD COLUMN remote_dns_risk_accepted_by TEXT;
ALTER TABLE egress_proxy ADD COLUMN remote_dns_risk_accepted_at BIGINT;
ALTER TABLE egress_proxy ADD COLUMN chain_next_id TEXT;
ALTER TABLE egress_proxy ADD COLUMN scope TEXT NOT NULL DEFAULT 'global';
ALTER TABLE egress_proxy ADD COLUMN traffic_classes_json TEXT NOT NULL DEFAULT '["remote_content","ics_feeds"]';
-- An existing proxy keeps its protocol: `scheme` was 'http' | 'socks5'.
UPDATE egress_proxy SET kind = scheme WHERE scheme IN ('http', 'socks5');

CREATE TABLE IF NOT EXISTS egress_internal_allow (
    id             TEXT PRIMARY KEY NOT NULL,
    traffic_class  TEXT NOT NULL CHECK (traffic_class IN ('siem', 'webhooks')),
    destination    TEXT NOT NULL,
    note           TEXT NOT NULL DEFAULT '',
    created_by     TEXT NOT NULL,
    created_at     BIGINT NOT NULL
);

-- ── W S/MIME trust and revocation ───────────────────────────────────────────
CREATE TABLE IF NOT EXISTS smime_trust_root (
    id           TEXT PRIMARY KEY NOT NULL,
    scope        TEXT NOT NULL,
    der          BYTEA NOT NULL,
    fingerprint  TEXT NOT NULL,
    label        TEXT NOT NULL,
    added_by     TEXT NOT NULL,
    added_at     BIGINT NOT NULL
);

-- `next_update` is optional in both OCSP (RFC 6960) and CRLs (RFC 5280).
CREATE TABLE IF NOT EXISTS pki_revocation_cache (
    issuer_key_hash  TEXT NOT NULL,
    serial           TEXT NOT NULL,
    status           TEXT NOT NULL CHECK (status IN ('good', 'revoked', 'unknown')),
    source           TEXT NOT NULL CHECK (source IN ('ocsp', 'crl')),
    this_update      BIGINT NOT NULL,
    next_update      BIGINT,
    checked_at       BIGINT NOT NULL,
    PRIMARY KEY (issuer_key_hash, serial)
);

-- ── X signatures ────────────────────────────────────────────────────────────
ALTER TABLE signatures ADD COLUMN html TEXT;
ALTER TABLE signatures ADD COLUMN text TEXT;
ALTER TABLE signatures ADD COLUMN text_edited BIGINT NOT NULL DEFAULT 0;
ALTER TABLE signatures ADD COLUMN images_json TEXT NOT NULL DEFAULT '[]';
UPDATE signatures SET text = body;

CREATE TABLE IF NOT EXISTS identity_signature_default (
    identity_id     TEXT NOT NULL,
    mode            TEXT NOT NULL CHECK (mode IN ('new', 'reply', 'forward')),
    signature_name  TEXT NOT NULL,
    placement       TEXT NOT NULL CHECK (placement IN ('below', 'above')),
    separator       BIGINT NOT NULL DEFAULT 1,
    PRIMARY KEY (identity_id, mode)
);

CREATE TABLE IF NOT EXISTS domain_signature_template (
    domain         TEXT NOT NULL,
    name           TEXT NOT NULL,
    html           TEXT NOT NULL,
    text           TEXT NOT NULL,
    user_editable  BIGINT NOT NULL,
    updated_by     TEXT NOT NULL,
    updated_at     BIGINT NOT NULL,
    PRIMARY KEY (domain, name)
);

CREATE TABLE IF NOT EXISTS domain_disclaimer (
    domain      TEXT PRIMARY KEY NOT NULL,
    html        TEXT NOT NULL,
    text        TEXT NOT NULL,
    enabled     BIGINT NOT NULL,
    updated_by  TEXT NOT NULL,
    updated_at  BIGINT NOT NULL
);

-- ── Y3 read and delivery receipts ───────────────────────────────────────────
CREATE TABLE IF NOT EXISTS mdn_sender_rule (
    account_id  TEXT NOT NULL,
    address     TEXT NOT NULL,
    decision    TEXT NOT NULL CHECK (decision IN ('always', 'never')),
    updated_at  BIGINT NOT NULL,
    PRIMARY KEY (account_id, address)
);

-- `reported_at` / `report_email_id` stay NULL until a report arrives.
CREATE TABLE IF NOT EXISTS receipt_status (
    account_id       TEXT NOT NULL,
    sent_email_id    TEXT NOT NULL,
    recipient        TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('mdn', 'dsn')),
    status           TEXT NOT NULL,
    reported_at      BIGINT,
    report_email_id  TEXT,
    PRIMARY KEY (account_id, sent_email_id, recipient, kind)
);

-- ── AB linked accounts, providers, delegation ───────────────────────────────
CREATE TABLE IF NOT EXISTS child_account (
    owner_account_id  TEXT NOT NULL,
    account_id        TEXT NOT NULL,
    provider_id       TEXT,
    display_name      TEXT NOT NULL,
    accent_id         TEXT,
    position          BIGINT NOT NULL,
    paused            BIGINT NOT NULL DEFAULT 0,
    added_at          BIGINT NOT NULL,
    PRIMARY KEY (owner_account_id, account_id)
);
CREATE INDEX IF NOT EXISTS idx_child_account_account ON child_account (account_id);

CREATE TABLE IF NOT EXISTS provider_profile (
    id               TEXT PRIMARY KEY NOT NULL,
    name             TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('imap', 'pop3', 'bridge')),
    endpoints_json   TEXT NOT NULL,
    oauth_client_id  TEXT,
    tunnel_id        TEXT,
    egress_route_id  TEXT,
    scope            TEXT NOT NULL DEFAULT 'global',
    enabled          BIGINT NOT NULL DEFAULT 0,
    preset           TEXT,
    created_by       TEXT NOT NULL,
    updated_at       BIGINT NOT NULL
);

-- Distinct from the 0007 `oauth_clients` (clients of THIS server's OAuth
-- provider): these are this server's registrations AT an upstream provider.
CREATE TABLE IF NOT EXISTS oauth_client (
    id                    TEXT PRIMARY KEY NOT NULL,
    provider_id           TEXT NOT NULL,
    client_id             TEXT NOT NULL,
    sealed_client_secret  BYTEA,                          -- SEALED; NULL = public client
    auth_url              TEXT NOT NULL,
    token_url             TEXT NOT NULL,
    device_url            TEXT,
    scopes                TEXT NOT NULL,
    redirect              TEXT NOT NULL,
    created_by            TEXT NOT NULL,
    updated_at            BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_oauth_client_provider ON oauth_client (provider_id);

CREATE TABLE IF NOT EXISTS account_oauth (
    account_id            TEXT PRIMARY KEY NOT NULL,
    provider_id           TEXT NOT NULL,
    sealed_refresh_token  BYTEA NOT NULL,                 -- SEALED
    expires_at            BIGINT NOT NULL,
    scope                 TEXT NOT NULL,
    needs_signin          BIGINT NOT NULL DEFAULT 0,
    updated_at            BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS delegation_grant (
    id                         TEXT PRIMARY KEY NOT NULL,
    owner_account_id           TEXT NOT NULL,
    delegate_owner_account_id  TEXT,
    delegate_group_id          TEXT,
    scope_kind                 TEXT NOT NULL CHECK (scope_kind IN ('account', 'mailboxes')),
    mailbox_ids_json           TEXT NOT NULL,
    permissions_json           TEXT NOT NULL,
    expires_at                 BIGINT,
    created_at                 BIGINT NOT NULL,
    revoked_at                 BIGINT
);
CREATE INDEX IF NOT EXISTS idx_delegation_grant_owner ON delegation_grant (owner_account_id);
CREATE INDEX IF NOT EXISTS idx_delegation_grant_delegate ON delegation_grant (delegate_owner_account_id);
CREATE INDEX IF NOT EXISTS idx_delegation_grant_group ON delegation_grant (delegate_group_id);

CREATE TABLE IF NOT EXISTS shared_mailbox (
    account_id  TEXT PRIMARY KEY NOT NULL,
    domain      TEXT NOT NULL,
    name        TEXT NOT NULL,
    created_by  TEXT NOT NULL,
    created_at  BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_shared_mailbox_domain ON shared_mailbox (domain);

-- ── AC saved searches and organisation views ────────────────────────────────
ALTER TABLE saved_searches ADD COLUMN query_text TEXT;
ALTER TABLE saved_searches ADD COLUMN builder_json TEXT;
ALTER TABLE saved_searches ADD COLUMN accounts_json TEXT;
ALTER TABLE saved_searches ADD COLUMN settings_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE saved_searches ADD COLUMN position BIGINT NOT NULL DEFAULT 0;
ALTER TABLE saved_searches ADD COLUMN icon TEXT;
ALTER TABLE saved_searches ADD COLUMN accent_id TEXT;
ALTER TABLE saved_searches ADD COLUMN start_page BIGINT NOT NULL DEFAULT 0;
ALTER TABLE saved_searches ADD COLUMN source TEXT NOT NULL DEFAULT 'user';
ALTER TABLE saved_searches ADD COLUMN org_view_id TEXT;

CREATE TABLE IF NOT EXISTS org_view (
    id             TEXT PRIMARY KEY NOT NULL,
    scope          TEXT NOT NULL,
    name           TEXT NOT NULL,
    query_text     TEXT NOT NULL,
    settings_json  TEXT NOT NULL,
    mandatory      BIGINT NOT NULL DEFAULT 0,
    created_by     TEXT NOT NULL,
    updated_at     BIGINT NOT NULL
);

-- ── AE identities ───────────────────────────────────────────────────────────
-- The existing `identities.source` takes the new values without a CHECK change.
ALTER TABLE identities ADD COLUMN auto_bcc TEXT;
ALTER TABLE identities ADD COLUMN color TEXT;
ALTER TABLE identities ADD COLUMN pgp_key_id TEXT;
ALTER TABLE identities ADD COLUMN smime_cert_id TEXT;
ALTER TABLE identities ADD COLUMN request_receipt BIGINT NOT NULL DEFAULT 0;
ALTER TABLE identities ADD COLUMN verified_at BIGINT;
ALTER TABLE identities ADD COLUMN sender_header TEXT;

CREATE TABLE IF NOT EXISTS identity_verification (
    token_hash   TEXT PRIMARY KEY NOT NULL,              -- SHA-256 of the token; never the token
    identity_id  TEXT NOT NULL,
    sent_at      BIGINT NOT NULL,
    expires_at   BIGINT NOT NULL,
    consumed     BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_identity_verification_identity ON identity_verification (identity_id);

-- ── AF templates ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS template (
    id                TEXT PRIMARY KEY NOT NULL,
    owner_kind        TEXT NOT NULL CHECK (owner_kind IN ('user', 'domain', 'group')),
    owner_id          TEXT NOT NULL,
    folder            TEXT NOT NULL,
    name              TEXT NOT NULL,
    subject           TEXT NOT NULL,
    html              TEXT NOT NULL,
    text              TEXT NOT NULL,
    attachments_json  TEXT NOT NULL,
    identity_id       TEXT,
    recipients_json   TEXT NOT NULL,
    fields_json       TEXT NOT NULL,
    mandatory         BIGINT NOT NULL DEFAULT 0,
    user_copyable     BIGINT NOT NULL DEFAULT 1,
    version           BIGINT NOT NULL DEFAULT 1,
    usage_count       BIGINT NOT NULL DEFAULT 0,
    created_by        TEXT NOT NULL,
    created_at        BIGINT NOT NULL,
    updated_at        BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_template_owner ON template (owner_kind, owner_id);

CREATE TABLE IF NOT EXISTS template_version (
    template_id       TEXT NOT NULL REFERENCES template(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    version           BIGINT NOT NULL,
    subject           TEXT NOT NULL,
    html              TEXT NOT NULL,
    text              TEXT NOT NULL,
    attachments_json  TEXT NOT NULL,
    created_by        TEXT NOT NULL,
    created_at        BIGINT NOT NULL,
    PRIMARY KEY (template_id, version)
);

-- ── AG groups and feature profiles ──────────────────────────────────────────
CREATE TABLE IF NOT EXISTS user_group (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    domain      TEXT NOT NULL,
    source      TEXT NOT NULL CHECK (source IN ('manual', 'ldap')),
    ldap_dn     TEXT,
    created_at  BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_user_group_domain ON user_group (domain);

CREATE TABLE IF NOT EXISTS user_group_member (
    group_id    TEXT NOT NULL REFERENCES user_group(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    account_id  TEXT NOT NULL,
    PRIMARY KEY (group_id, account_id)
);
CREATE INDEX IF NOT EXISTS idx_user_group_member_account ON user_group_member (account_id);

CREATE TABLE IF NOT EXISTS feature_profile (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    values_json  TEXT NOT NULL,
    builtin      BIGINT NOT NULL DEFAULT 0,
    updated_at   BIGINT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_feature_profile_name ON feature_profile (name);

CREATE TABLE IF NOT EXISTS feature_profile_assignment (
    scope_kind  TEXT NOT NULL CHECK (scope_kind IN ('domain', 'group', 'user')),
    scope_id    TEXT NOT NULL,
    profile_id  TEXT NOT NULL REFERENCES feature_profile(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    priority    BIGINT NOT NULL,
    PRIMARY KEY (scope_kind, scope_id, profile_id)
);
CREATE INDEX IF NOT EXISTS idx_feature_profile_assignment_profile ON feature_profile_assignment (profile_id);

-- ── Q users, sign-in policy, usage (t32-q) ──────────────────────────────────
-- The user directory: one row per canonical address. Filled by the boot backfill
-- (e66); `status` mirrors the account-gate flags, which stay authoritative.
CREATE TABLE IF NOT EXISTS mail_user (
    address            TEXT PRIMARY KEY NOT NULL,
    domain             TEXT NOT NULL,
    local_part         TEXT NOT NULL,
    origin             TEXT NOT NULL CHECK (origin IN ('provisioned', 'signin', 'sso', 'import')),
    status             TEXT NOT NULL DEFAULT 'active'
                         CHECK (status IN ('active', 'disabled', 'deleting', 'deleted')),
    display_name       TEXT,
    note               TEXT,
    created_at         BIGINT NOT NULL,
    created_by         TEXT,
    first_signin_at    BIGINT,
    last_signin_at     BIGINT,
    status_changed_at  BIGINT,
    status_changed_by  TEXT,
    status_reason      TEXT,
    expires_at         BIGINT,
    deleted_at         BIGINT
);
CREATE INDEX IF NOT EXISTS idx_mail_user_domain ON mail_user (domain, address);
CREATE INDEX IF NOT EXISTS idx_mail_user_status ON mail_user (status);
CREATE INDEX IF NOT EXISTS idx_mail_user_last_signin ON mail_user (last_signin_at);

-- Session/engine account ids -> canonical address. `child` (t32-q A16): a linked
-- child account, gated by its owner.
CREATE TABLE IF NOT EXISTS mail_user_account (
    account_id  TEXT PRIMARY KEY NOT NULL,
    address     TEXT NOT NULL,
    mode        TEXT NOT NULL CHECK (mode IN ('engine', 'proxy', 'header', 'sso', 'child')),
    first_seen  BIGINT NOT NULL,
    last_seen   BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_mail_user_account_address ON mail_user_account (address);

-- First-sign-in allow patterns (local-part globs) and invitations.
CREATE TABLE IF NOT EXISTS signin_allow (
    id          TEXT PRIMARY KEY NOT NULL,
    domain      TEXT NOT NULL,
    pattern     TEXT NOT NULL,
    note        TEXT,
    expires_at  BIGINT,
    max_uses    BIGINT,
    uses        BIGINT NOT NULL DEFAULT 0,
    created_by  TEXT NOT NULL,
    created_at  BIGINT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_signin_allow_domain_pattern ON signin_allow (domain, pattern);

-- Daily counters; `day` = UTC day number.
CREATE TABLE IF NOT EXISTS usage_counter (
    address  TEXT NOT NULL,
    domain   TEXT NOT NULL,
    metric   TEXT NOT NULL,
    day      BIGINT NOT NULL,
    value    BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (address, metric, day)
);
CREATE INDEX IF NOT EXISTS idx_usage_counter_domain ON usage_counter (domain, metric, day);
CREATE INDEX IF NOT EXISTS idx_usage_counter_day ON usage_counter (day);

-- Measured storage per user. `measuring_until` is the measurer's claim (t32-q
-- A18): a crashed run's claim expires instead of leaving the row stuck.
CREATE TABLE IF NOT EXISTS usage_storage (
    address               TEXT PRIMARY KEY NOT NULL,
    domain                TEXT NOT NULL,
    blob_bytes            BIGINT NOT NULL DEFAULT 0,
    blob_count            BIGINT NOT NULL DEFAULT 0,
    content_bytes         BIGINT NOT NULL DEFAULT 0,
    cache_bytes           BIGINT NOT NULL DEFAULT 0,
    cache_messages        BIGINT NOT NULL DEFAULT 0,
    search_docs           BIGINT,
    breakdown_json        TEXT,
    measured_at           BIGINT,
    measuring_until       BIGINT,
    upstream_used_bytes   BIGINT,
    upstream_limit_bytes  BIGINT,
    upstream_json         TEXT,
    upstream_status       TEXT,
    upstream_checked_at   BIGINT,
    warn_level            BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_usage_storage_domain ON usage_storage (domain);

-- The measurer's per-account aggregate and the delete cascade.
CREATE INDEX IF NOT EXISTS idx_bodies_account ON bodies (account_id);

-- ── CFG configuration, maintenance, jobs, diagnostics, bans (t32-cfg) ───────
CREATE TABLE IF NOT EXISTS config_snapshot (
    id          TEXT PRIMARY KEY NOT NULL,
    created_at  BIGINT NOT NULL,
    created_by  TEXT NOT NULL,
    reason      TEXT NOT NULL CHECK (reason IN ('export', 'pre-import', 'manual')),
    sha256      TEXT NOT NULL,
    json        TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_config_snapshot_created ON config_snapshot (created_at);

-- `kind` has no CHECK (t32-cfg A16): kinds are validated in code, so a new
-- integration needs no SQLite table rebuild. Disabled by default: the env
-- configuration stays in force.
CREATE TABLE IF NOT EXISTS integration_config (
    kind           TEXT PRIMARY KEY NOT NULL,
    config_json    TEXT NOT NULL DEFAULT '{}',
    sealed_secret  BYTEA,                                 -- SEALED
    secret_hash    TEXT,
    enabled        BIGINT NOT NULL DEFAULT 0,
    updated_by     TEXT NOT NULL,
    updated_at     BIGINT NOT NULL
);

-- `occurrence_ended_at`: `POST …/end` on a recurring window ends the current
-- occurrence; cancelling ends the series.
CREATE TABLE IF NOT EXISTS maintenance_window (
    id                   TEXT PRIMARY KEY NOT NULL,
    scope_kind           TEXT NOT NULL DEFAULT 'global' CHECK (scope_kind IN ('global', 'domain')),
    scope_id             TEXT NOT NULL DEFAULT '',
    mode                 TEXT NOT NULL CHECK (mode IN ('announce', 'read_only', 'signin_closed', 'full')),
    starts_at            BIGINT NOT NULL,
    ends_at              BIGINT,
    tz                   TEXT NOT NULL DEFAULT 'UTC',
    recurrence_json      TEXT,
    announce_minutes     BIGINT NOT NULL DEFAULT 60,
    message_json         TEXT NOT NULL DEFAULT '{}',
    created_by           TEXT NOT NULL,
    created_at           BIGINT NOT NULL,
    updated_by           TEXT NOT NULL,
    updated_at           BIGINT NOT NULL,
    occurrence_ended_at  BIGINT,
    cancelled_at         BIGINT,
    cancelled_by         TEXT
);
CREATE INDEX IF NOT EXISTS idx_maintenance_window_starts ON maintenance_window (starts_at);

-- Nothing enabled by default. `lease_owner`/`lease_until`: the DB lease that makes
-- a due task run once across processes.
CREATE TABLE IF NOT EXISTS scheduled_task (
    id              TEXT PRIMARY KEY NOT NULL,
    kind            TEXT NOT NULL,
    schedule_json   TEXT NOT NULL,
    params_json     TEXT NOT NULL DEFAULT '{}',
    enabled         BIGINT NOT NULL DEFAULT 0,
    only_in_window  BIGINT NOT NULL DEFAULT 0,
    created_by      TEXT NOT NULL,
    created_at      BIGINT NOT NULL,
    updated_at      BIGINT NOT NULL,
    last_run_at     BIGINT,
    last_status     TEXT,
    next_run_at     BIGINT,
    lease_owner     TEXT,
    lease_until     BIGINT
);
CREATE INDEX IF NOT EXISTS idx_scheduled_task_next ON scheduled_task (next_run_at);

-- `instance` = host name + process start time. A `running` row whose heartbeat is
-- older than 3x the lease is marked `abandoned` (t32-cfg A31).
CREATE TABLE IF NOT EXISTS job_run (
    id            TEXT PRIMARY KEY NOT NULL,
    task_id       TEXT,
    kind          TEXT NOT NULL,
    instance      TEXT NOT NULL,
    started_at    BIGINT NOT NULL,
    heartbeat_at  BIGINT NOT NULL,
    finished_at   BIGINT,
    status        TEXT NOT NULL CHECK (status IN ('running', 'ok', 'failed', 'cancelled', 'abandoned')),
    summary       TEXT,
    detail_json   TEXT,
    triggered_by  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_job_run_kind ON job_run (kind, started_at);

CREATE TABLE IF NOT EXISTS diag_run (
    id            TEXT PRIMARY KEY NOT NULL,
    suite         TEXT NOT NULL,
    target        TEXT,
    started_at    BIGINT NOT NULL,
    finished_at   BIGINT,
    status        TEXT NOT NULL CHECK (status IN ('running', 'pass', 'warn', 'fail', 'error')),
    results_json  TEXT NOT NULL DEFAULT '[]',
    triggered_by  TEXT NOT NULL,
    task_id       TEXT
);
CREATE INDEX IF NOT EXISTS idx_diag_run_suite ON diag_run (suite, started_at);

CREATE TABLE IF NOT EXISTS diag_probe_account (
    domain            TEXT PRIMARY KEY NOT NULL REFERENCES domains(name) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    address           TEXT NOT NULL,
    sealed_password   BYTEA NOT NULL,                     -- SEALED
    created_by        TEXT NOT NULL,
    updated_at        BIGINT NOT NULL,
    last_loop_at      BIGINT,
    last_loop_ms      BIGINT,
    last_loop_status  TEXT
);

-- Identity and geo bans (IP/CIDR bans stay in `ip_block`).
CREATE TABLE IF NOT EXISTS account_ban (
    id           TEXT PRIMARY KEY NOT NULL,
    kind         TEXT NOT NULL CHECK (kind IN ('user', 'pattern', 'domain', 'asn', 'country')),
    value        TEXT NOT NULL,
    surface      TEXT NOT NULL DEFAULT 'webmail' CHECK (surface IN ('webmail', 'admin', 'all')),
    reason       TEXT NOT NULL,
    user_note    TEXT,
    starts_at    BIGINT NOT NULL,
    expires_at   BIGINT,
    source       TEXT NOT NULL DEFAULT 'manual' CHECK (source IN ('manual', 'import', 'rule')),
    created_by   TEXT NOT NULL,
    created_at   BIGINT NOT NULL,
    lifted_at    BIGINT,
    lifted_by    TEXT,
    lift_reason  TEXT,
    hits         BIGINT NOT NULL DEFAULT 0,
    last_hit_at  BIGINT
);
CREATE INDEX IF NOT EXISTS idx_account_ban_kind ON account_ban (kind, value);
CREATE INDEX IF NOT EXISTS idx_account_ban_expires ON account_ban (expires_at);

-- Hourly rollups, one row per series per hour per instance.
CREATE TABLE IF NOT EXISTS metric_rollup (
    series      TEXT NOT NULL,
    hour        BIGINT NOT NULL,
    instance    TEXT NOT NULL,
    value_json  TEXT NOT NULL,
    PRIMARY KEY (series, hour, instance)
);
CREATE INDEX IF NOT EXISTS idx_metric_rollup_hour ON metric_rollup (hour);

CREATE TABLE IF NOT EXISTS data_export (
    id                TEXT PRIMARY KEY NOT NULL,
    address           TEXT NOT NULL,
    requested_by      TEXT NOT NULL,
    status            TEXT NOT NULL CHECK (status IN ('queued', 'running', 'ready', 'downloaded',
                                                      'expired', 'failed')),
    item_counts_json  TEXT NOT NULL DEFAULT '{}',
    bytes             BIGINT,
    blob_ref          TEXT,
    created_at        BIGINT NOT NULL,
    expires_at        BIGINT NOT NULL,
    downloaded_at     BIGINT
);
CREATE INDEX IF NOT EXISTS idx_data_export_address ON data_export (address);
CREATE INDEX IF NOT EXISTS idx_data_export_expires ON data_export (expires_at);

-- ── CFG columns on existing tables ──────────────────────────────────────────
ALTER TABLE directory_config ADD COLUMN sealed_bind_password BYTEA;   -- SEALED
-- `client_version` sits with the AD session columns above.
