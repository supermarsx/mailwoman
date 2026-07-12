-- V3 (PIM) schema (plan §2.4). Additive over 0001_init + 0002_v1_cache +
-- 0003_v2; never edit an earlier migration. Everything here is the calendar /
-- tasks / notes / contacts storage behind the Mailwoman PIM surface: CalDAV/
-- CardDAV-backed collections (etag/sync-token/ctag), the recurrence-expansion
-- index, sealed-at-rest note bodies, and the per-account PIM change log.
--
-- Scaffolder note (e0): this is the skeleton. e8 fills the typed repo methods
-- (calendars / events / event_instances / tasks / notes / contacts / groups
-- CRUD + sealed-note read/write + pim_changes state) in `crates/mw-store/src`.
--
-- `ical_raw` / `vcard_raw` are the round-trip source of truth (plan risk #13);
-- `json` is the parsed Mailwoman projection. `start_utc`/`end_utc` are the
-- expansion index bounds. Enum-like fields (status/access/op/component) are
-- opaque strings the engine owns; the store never interprets them.

-- Calendars: Mailwoman-native (caldav_url NULL) or CalDAV-backed / overlay.
-- component distinguishes an event calendar ('VEVENT') from a task list ('VTODO').
CREATE TABLE IF NOT EXISTS calendars (
    id          TEXT PRIMARY KEY NOT NULL,
    account_id  TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    color       TEXT NOT NULL DEFAULT '',
    sort_order  INTEGER NOT NULL DEFAULT 0,
    is_visible  INTEGER NOT NULL DEFAULT 1,
    role        TEXT,
    caldav_url  TEXT,
    sync_token  TEXT,
    ctag        TEXT,
    is_overlay  INTEGER NOT NULL DEFAULT 0,
    component   TEXT NOT NULL DEFAULT 'VEVENT'
);
CREATE INDEX IF NOT EXISTS idx_calendars_account ON calendars (account_id);

-- Mailwoman-native calendar ACL shares (principal → read|readWrite).
CREATE TABLE IF NOT EXISTS calendar_shares (
    calendar_id TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    principal   TEXT NOT NULL,
    access      TEXT NOT NULL DEFAULT 'read'
);
CREATE INDEX IF NOT EXISTS idx_calendar_shares ON calendar_shares (calendar_id);

-- Events (VEVENT). ical_raw is the fidelity source of truth; json the parsed
-- Mailwoman shape; start_utc/end_utc bound recurrence expansion.
CREATE TABLE IF NOT EXISTS events (
    id          TEXT PRIMARY KEY NOT NULL,
    calendar_id TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    uid         TEXT NOT NULL,
    etag        TEXT,
    ical_raw    TEXT NOT NULL,
    start_utc   TEXT,
    end_utc     TEXT,
    tzid        TEXT,
    rrule       TEXT,
    status      TEXT NOT NULL DEFAULT 'confirmed',
    json        BLOB
);
CREATE INDEX IF NOT EXISTS idx_events_calendar ON events (calendar_id);

-- Materialized recurrence instances for range queries + conflict detection
-- (regenerated on each event write, plan §2.4).
CREATE TABLE IF NOT EXISTS event_instances (
    event_id           TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    instance_start_utc TEXT NOT NULL,
    instance_end_utc   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_event_instances_range
    ON event_instances (instance_start_utc, instance_end_utc);

-- Tasks (VTODO over a task-list calendar). My Day = my_day_date pinning.
CREATE TABLE IF NOT EXISTS tasks (
    id               TEXT PRIMARY KEY NOT NULL,
    list_id          TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    uid              TEXT NOT NULL,
    etag             TEXT,
    due_utc          TEXT,
    start_utc        TEXT,
    priority         INTEGER NOT NULL DEFAULT 0,
    percent_complete INTEGER NOT NULL DEFAULT 0,
    status           TEXT NOT NULL DEFAULT 'needs-action',
    parent_id        TEXT,
    my_day_date      TEXT,
    ical_raw         TEXT NOT NULL,
    json             BLOB
);
CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks (list_id);
CREATE INDEX IF NOT EXISTS idx_tasks_due ON tasks (due_utc);
CREATE INDEX IF NOT EXISTS idx_tasks_myday ON tasks (my_day_date);

-- Notebooks group notes (Mailwoman-native).
CREATE TABLE IF NOT EXISTS notebooks (
    id         TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notebooks_account ON notebooks (account_id);

-- Notes: title/tags/color/pinned plaintext (searchable/sortable); the rich-text
-- body columns are SEALED at rest (XChaCha20-Poly1305 mw-store seal, plan §1.6)
-- — encrypted-at-rest, NOT zero-access (documented; V6 swaps the key source).
CREATE TABLE IF NOT EXISTS notes (
    id               TEXT PRIMARY KEY NOT NULL,
    account_id       TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    notebook_id      TEXT,
    title            TEXT NOT NULL DEFAULT '',
    tags_json        TEXT NOT NULL DEFAULT '[]',
    color            TEXT NOT NULL DEFAULT '',
    pinned           INTEGER NOT NULL DEFAULT 0,
    body_html_sealed BLOB,
    body_text_sealed BLOB,
    links_json       TEXT NOT NULL DEFAULT '[]',
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notes_account ON notes (account_id);
CREATE INDEX IF NOT EXISTS idx_notes_pinned ON notes (pinned);

-- Address books: Mailwoman-native (carddav_url NULL) or CardDAV-backed.
CREATE TABLE IF NOT EXISTS address_books (
    id          TEXT PRIMARY KEY NOT NULL,
    account_id  TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    is_default  INTEGER NOT NULL DEFAULT 0,
    carddav_url TEXT,
    sync_token  TEXT,
    ctag        TEXT
);
CREATE INDEX IF NOT EXISTS idx_address_books_account ON address_books (account_id);

-- Contacts (vCard). vcard_raw is the fidelity source of truth; json the parsed
-- projection. pgp_key/smime_cert are opaque V4 placeholders.
CREATE TABLE IF NOT EXISTS contacts (
    id              TEXT PRIMARY KEY NOT NULL,
    address_book_id TEXT NOT NULL REFERENCES address_books(id) ON DELETE CASCADE,
    uid             TEXT NOT NULL,
    etag            TEXT,
    vcard_raw       TEXT NOT NULL,
    json            BLOB,
    full_name       TEXT NOT NULL DEFAULT '',
    is_favorite     INTEGER NOT NULL DEFAULT 0,
    photo_blob_id   TEXT,
    pgp_key         TEXT,
    smime_cert      TEXT
);
CREATE INDEX IF NOT EXISTS idx_contacts_book ON contacts (address_book_id);
CREATE INDEX IF NOT EXISTS idx_contacts_fullname ON contacts (full_name);
CREATE INDEX IF NOT EXISTS idx_contacts_favorite ON contacts (is_favorite);

-- Contact groups / distribution lists (members as a JSON id array).
CREATE TABLE IF NOT EXISTS contact_groups (
    id              TEXT PRIMARY KEY NOT NULL,
    address_book_id TEXT NOT NULL REFERENCES address_books(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    member_ids_json TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_contact_groups_book ON contact_groups (address_book_id);

-- Per-account PIM change log (plan §2.4, mirrors the V2 `changes` table): the
-- raw material for PIM state tokens + `*/changes`. `type` is a PIM ChangeType
-- name (Calendar/CalendarEvent/Task/Note/AddressBook/ContactCard/ContactGroup).
CREATE TABLE IF NOT EXISTS pim_changes (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    type       TEXT NOT NULL,
    state      INTEGER NOT NULL,
    object_id  TEXT NOT NULL,
    op         TEXT NOT NULL,
    at         TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_pim_changes
    ON pim_changes (account_id, type, state);
