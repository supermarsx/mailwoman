-- t94: a user's own Focused inbox choices, "Always move to Focused" or "Always
-- move to Other" for one sender. Keyed by the personal owner subject (the same
-- owner as personal_signature), so each user sees only their own. `sender` is
-- a lower-cased mailbox address validated by the server; `tab` is 'focused' or
-- 'other'; `updated_at` is Unix milliseconds. At most 1000 rows per owner
-- (enforced by the store).
CREATE TABLE focused_sender (
 owner_subject TEXT NOT NULL,
 sender TEXT NOT NULL,
 tab TEXT NOT NULL CHECK (tab IN ('focused', 'other')),
 updated_at BIGINT NOT NULL,
 PRIMARY KEY (owner_subject, sender)
);
