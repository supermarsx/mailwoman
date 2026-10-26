-- No legacy raw-account rows are attributed automatically.
CREATE TABLE personal_identity (
    owner_subject TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    email TEXT NOT NULL,
    reply_to TEXT,
    signature_html TEXT,
    signature_text TEXT,
    signature_name TEXT,
    sent_mailbox_id TEXT,
    source TEXT NOT NULL DEFAULT 'configured',
    PRIMARY KEY (owner_subject,id)
);
CREATE TABLE personal_notification_rules (
    owner_subject TEXT PRIMARY KEY NOT NULL,
    rule_json TEXT NOT NULL,
    quiet_hours_json TEXT NOT NULL,
    enabled BIGINT NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);
CREATE TABLE personal_push_subscription (
    id TEXT PRIMARY KEY NOT NULL,
    owner_subject TEXT NOT NULL,
    transport TEXT NOT NULL,
    endpoint TEXT NOT NULL UNIQUE,
    p256dh TEXT,
    auth TEXT,
    app_id TEXT,
    expires_at TEXT,
    created_at TEXT NOT NULL,
    last_wake_at TEXT
);
CREATE INDEX personal_push_owner_idx ON personal_push_subscription(owner_subject);
