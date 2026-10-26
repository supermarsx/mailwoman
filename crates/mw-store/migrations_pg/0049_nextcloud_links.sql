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
    sealed_password BYTEA NOT NULL,
    revision TEXT NOT NULL,
    PRIMARY KEY (account_id, instance_id)
);

ALTER TABLE egress_internal_allow DROP CONSTRAINT egress_internal_allow_traffic_class_check;
ALTER TABLE egress_internal_allow ADD CONSTRAINT egress_internal_allow_traffic_class_check
    CHECK (traffic_class IN ('siem', 'webhooks', 'analytics', 'nextcloud'));
