ALTER TABLE ssh_tunnel ADD COLUMN targets_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE ssh_tunnel ADD COLUMN allow_private BIGINT NOT NULL DEFAULT 0;
ALTER TABLE ssh_tunnel ADD COLUMN auth_kind TEXT DEFAULT 'privateKey';
ALTER TABLE ssh_tunnel ADD COLUMN revision TEXT NOT NULL DEFAULT 'legacy';
UPDATE ssh_tunnel SET revision='legacy-' || id;
UPDATE ssh_tunnel SET auth_kind=NULL WHERE length(sealed_private_key)=0;
