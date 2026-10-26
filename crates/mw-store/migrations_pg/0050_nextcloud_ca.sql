-- Public CA trust material belongs to an exact administrator-approved instance.
ALTER TABLE nextcloud_instance ADD COLUMN ca_pem TEXT NOT NULL DEFAULT '';
