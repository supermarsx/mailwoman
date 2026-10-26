-- Explicit plugin-private-host permission. A plugin hostname grant alone never enables private IPs.
CREATE TABLE egress_internal_allow_next (
    id TEXT PRIMARY KEY NOT NULL,
    traffic_class TEXT NOT NULL CHECK (traffic_class IN ('siem', 'webhooks', 'analytics', 'nextcloud', 'plugins')),
    destination TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
INSERT INTO egress_internal_allow_next SELECT id, traffic_class, destination, note, created_by, created_at FROM egress_internal_allow;
DROP TABLE egress_internal_allow;
ALTER TABLE egress_internal_allow_next RENAME TO egress_internal_allow;
