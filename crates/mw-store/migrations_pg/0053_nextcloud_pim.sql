CREATE TABLE nextcloud_pim_binding (
    account_id TEXT NOT NULL,
    collection_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('calendar','tasks','contacts')),
    instance_id TEXT NOT NULL REFERENCES nextcloud_instance(id),
    link_revision TEXT NOT NULL,
    href TEXT NOT NULL,
    enabled BIGINT NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)),
    PRIMARY KEY(account_id,collection_id),
    UNIQUE(account_id,instance_id,kind,href)
);
CREATE INDEX nextcloud_pim_by_account ON nextcloud_pim_binding(account_id,enabled);
