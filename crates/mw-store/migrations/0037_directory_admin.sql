-- LDAP administration: explicit network consent and optimistic revision.
ALTER TABLE directory_config ADD COLUMN connection_mode TEXT NOT NULL DEFAULT 'blocked';
ALTER TABLE directory_config ADD COLUMN allow_private INTEGER NOT NULL DEFAULT 0;
ALTER TABLE directory_config ADD COLUMN revision TEXT NOT NULL DEFAULT '0';
