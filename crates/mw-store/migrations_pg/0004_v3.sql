-- V3 (PIM) schema — POSTGRES variant (t6-e1; plan §1.1, §2.1). Behaviourally
-- identical to the SQLite `migrations/0004_v3.sql`. Dialect mapping: INTEGER →
-- BIGINT (incl. boolean-as-0/1: is_visible/is_overlay/is_default/is_favorite/
-- pinned), BLOB → BYTEA (json / body_*_sealed), FKs DEFERRABLE INITIALLY
-- IMMEDIATE. `body_*_sealed` remain sealed-at-rest (mw-store seal), NOT
-- zero-access.

CREATE TABLE IF NOT EXISTS calendars (
    id          TEXT PRIMARY KEY NOT NULL,
    account_id  TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    name        TEXT NOT NULL,
    color       TEXT NOT NULL DEFAULT '',
    sort_order  BIGINT NOT NULL DEFAULT 0,
    is_visible  BIGINT NOT NULL DEFAULT 1,
    role        TEXT,
    caldav_url  TEXT,
    sync_token  TEXT,
    ctag        TEXT,
    is_overlay  BIGINT NOT NULL DEFAULT 0,
    component   TEXT NOT NULL DEFAULT 'VEVENT'
);
CREATE INDEX IF NOT EXISTS idx_calendars_account ON calendars (account_id);

CREATE TABLE IF NOT EXISTS calendar_shares (
    calendar_id TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    principal   TEXT NOT NULL,
    access      TEXT NOT NULL DEFAULT 'read'
);
CREATE INDEX IF NOT EXISTS idx_calendar_shares ON calendar_shares (calendar_id);

CREATE TABLE IF NOT EXISTS events (
    id          TEXT PRIMARY KEY NOT NULL,
    calendar_id TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    uid         TEXT NOT NULL,
    etag        TEXT,
    ical_raw    TEXT NOT NULL,
    start_utc   TEXT,
    end_utc     TEXT,
    tzid        TEXT,
    rrule       TEXT,
    status      TEXT NOT NULL DEFAULT 'confirmed',
    json        BYTEA
);
CREATE INDEX IF NOT EXISTS idx_events_calendar ON events (calendar_id);

CREATE TABLE IF NOT EXISTS event_instances (
    event_id           TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    instance_start_utc TEXT NOT NULL,
    instance_end_utc   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_event_instances_range
    ON event_instances (instance_start_utc, instance_end_utc);

CREATE TABLE IF NOT EXISTS tasks (
    id               TEXT PRIMARY KEY NOT NULL,
    list_id          TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    uid              TEXT NOT NULL,
    etag             TEXT,
    due_utc          TEXT,
    start_utc        TEXT,
    priority         BIGINT NOT NULL DEFAULT 0,
    percent_complete BIGINT NOT NULL DEFAULT 0,
    status           TEXT NOT NULL DEFAULT 'needs-action',
    parent_id        TEXT,
    my_day_date      TEXT,
    ical_raw         TEXT NOT NULL,
    json             BYTEA
);
CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks (list_id);
CREATE INDEX IF NOT EXISTS idx_tasks_due ON tasks (due_utc);
CREATE INDEX IF NOT EXISTS idx_tasks_myday ON tasks (my_day_date);

CREATE TABLE IF NOT EXISTS notebooks (
    id         TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    name       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notebooks_account ON notebooks (account_id);

CREATE TABLE IF NOT EXISTS notes (
    id               TEXT PRIMARY KEY NOT NULL,
    account_id       TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    notebook_id      TEXT,
    title            TEXT NOT NULL DEFAULT '',
    tags_json        TEXT NOT NULL DEFAULT '[]',
    color            TEXT NOT NULL DEFAULT '',
    pinned           BIGINT NOT NULL DEFAULT 0,
    body_html_sealed BYTEA,
    body_text_sealed BYTEA,
    links_json       TEXT NOT NULL DEFAULT '[]',
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notes_account ON notes (account_id);
CREATE INDEX IF NOT EXISTS idx_notes_pinned ON notes (pinned);

CREATE TABLE IF NOT EXISTS address_books (
    id          TEXT PRIMARY KEY NOT NULL,
    account_id  TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    name        TEXT NOT NULL,
    is_default  BIGINT NOT NULL DEFAULT 0,
    carddav_url TEXT,
    sync_token  TEXT,
    ctag        TEXT
);
CREATE INDEX IF NOT EXISTS idx_address_books_account ON address_books (account_id);

CREATE TABLE IF NOT EXISTS contacts (
    id              TEXT PRIMARY KEY NOT NULL,
    address_book_id TEXT NOT NULL REFERENCES address_books(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    uid             TEXT NOT NULL,
    etag            TEXT,
    vcard_raw       TEXT NOT NULL,
    json            BYTEA,
    full_name       TEXT NOT NULL DEFAULT '',
    is_favorite     BIGINT NOT NULL DEFAULT 0,
    photo_blob_id   TEXT,
    pgp_key         TEXT,
    smime_cert      TEXT
);
CREATE INDEX IF NOT EXISTS idx_contacts_book ON contacts (address_book_id);
CREATE INDEX IF NOT EXISTS idx_contacts_fullname ON contacts (full_name);
CREATE INDEX IF NOT EXISTS idx_contacts_favorite ON contacts (is_favorite);

CREATE TABLE IF NOT EXISTS contact_groups (
    id              TEXT PRIMARY KEY NOT NULL,
    address_book_id TEXT NOT NULL REFERENCES address_books(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    name            TEXT NOT NULL,
    member_ids_json TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_contact_groups_book ON contact_groups (address_book_id);

CREATE TABLE IF NOT EXISTS pim_changes (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE,
    type       TEXT NOT NULL,
    state      BIGINT NOT NULL,
    object_id  TEXT NOT NULL,
    op         TEXT NOT NULL,
    at         TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_pim_changes
    ON pim_changes (account_id, type, state);
