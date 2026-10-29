-- t86: the owner's avatar for each of their mailboxes (a letter or an icon from
-- a fixed list, with foreground and background colours), kept next to the
-- order and nicknames as JSON keyed by layout key ("main" or a link id). The
-- server validates it on write and again on read. Rows written before this
-- migration hold no avatars.
ALTER TABLE linked_layout ADD COLUMN avatars_json TEXT NOT NULL DEFAULT '{}';
