-- No implicit private access: only a saved Plugins allowlist entry widens its address policy.
ALTER TABLE egress_internal_allow DROP CONSTRAINT egress_internal_allow_traffic_class_check;
ALTER TABLE egress_internal_allow ADD CONSTRAINT egress_internal_allow_traffic_class_check CHECK (traffic_class IN ('siem', 'webhooks', 'analytics', 'nextcloud', 'plugins'));
