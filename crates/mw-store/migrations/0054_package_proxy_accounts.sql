-- Proxy session account IDs are namespaced by their upstream authority and do
-- not require a native IMAP/POP3 accounts row. Retain native deletion cleanup.
CREATE TABLE package_account_next (
    account_id TEXT PRIMARY KEY NOT NULL,
    state TEXT NOT NULL,
    revision TEXT NOT NULL,
    native_account_id TEXT REFERENCES accounts(id) ON DELETE CASCADE
);
INSERT INTO package_account_next(account_id,state,revision,native_account_id)
    SELECT p.account_id,p.state,p.revision,a.id FROM package_account p LEFT JOIN accounts a ON a.id=p.account_id;
DROP TABLE package_account;
ALTER TABLE package_account_next RENAME TO package_account;
