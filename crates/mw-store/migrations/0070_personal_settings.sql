-- Typed verified owners are separate from legacy arbitrary account/address IDs.
-- No legacy rows are attributed or deleted by this migration.
CREATE TABLE personal_setting_value (
    owner_subject TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (owner_subject, key)
);
