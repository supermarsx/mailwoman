-- Account-bound Nextcloud authorities. Secrets are sealed with account/instance/revision AAD.
CREATE TABLE nextcloud_instance (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    enabled BIGINT NOT NULL DEFAULT 0,
    revision TEXT NOT NULL
);
CREATE TABLE nextcloud_account_link (
    account_id TEXT NOT NULL,
    instance_id TEXT NOT NULL REFERENCES nextcloud_instance(id) ON DELETE CASCADE,
    instance_revision TEXT NOT NULL,
    username TEXT NOT NULL,
    sealed_password BLOB NOT NULL,
    revision TEXT NOT NULL,
    PRIMARY KEY (account_id, instance_id)
);

-- Preserve the audited entries while adding one narrowly scoped traffic class.
CREATE TABLE egress_internal_allow_next (
    id TEXT PRIMARY KEY NOT NULL,
    traffic_class TEXT NOT NULL CHECK (traffic_class IN ('siem', 'webhooks', 'analytics', 'nextcloud')),
    destination TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
INSERT INTO egress_internal_allow_next SELECT id, traffic_class, destination, note, created_by, created_at FROM egress_internal_allow;
DROP TABLE egress_internal_allow;
ALTER TABLE egress_internal_allow_next RENAME TO egress_internal_allow;
