-- Proxy session account IDs have no required native account row.
ALTER TABLE package_account DROP CONSTRAINT package_account_account_id_fkey;
ALTER TABLE package_account ADD COLUMN native_account_id TEXT REFERENCES accounts(id) ON DELETE CASCADE;
UPDATE package_account SET native_account_id=accounts.id FROM accounts WHERE accounts.id=package_account.account_id;
