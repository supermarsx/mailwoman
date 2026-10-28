-- The mailbox an owner opens first: one of their verified links. No foreign key
-- on link_id: when that link is removed the row stays, so the next listing can
-- report the fallback to the primary account once.
CREATE TABLE linked_primary (
 owner_principal_id TEXT PRIMARY KEY NOT NULL REFERENCES identity_principal(id) DEFERRABLE INITIALLY DEFERRED,
 link_id TEXT NOT NULL,
 updated_at BIGINT NOT NULL
);
-- The owner's own arrangement of their mailboxes (t40 f/g): the order of the
-- sign-in account ("main") and their links, and display-only nicknames. Both are
-- JSON validated by the server against the current links; `revision` guards
-- concurrent edits from two devices.
CREATE TABLE linked_layout (
 owner_principal_id TEXT PRIMARY KEY NOT NULL REFERENCES identity_principal(id) DEFERRABLE INITIALLY DEFERRED,
 order_json TEXT NOT NULL,
 nicknames_json TEXT NOT NULL,
 revision TEXT NOT NULL,
 updated_at BIGINT NOT NULL
);
