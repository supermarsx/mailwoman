-- Every foreign key is DEFERRABLE (migrate.rs: migrate-store issues
-- SET CONSTRAINTS ALL DEFERRED and copies the graph in one transaction).
-- Migrations 0102-0113 created 14 that are not. This makes them DEFERRABLE
-- INITIALLY IMMEDIATE: ordinary statements are still checked when they run;
-- only a transaction that defers constraints (migrate-store) checks at commit.
--
-- The constraints are found by table, constrained columns and referenced
-- table, not by name: PostgreSQL names them <table>_<columns>_fkey, but a
-- database whose names differ (a renamed constraint, a suffixed duplicate)
-- must not fail this migration. A constraint not found is reported as a
-- NOTICE and skipped; one already deferrable is left as it is.
DO $$
DECLARE
    wanted RECORD;
    found_name TEXT;
BEGIN
    FOR wanted IN
        SELECT * FROM (VALUES
            ('scheduled_mdn',                ARRAY['account_id'],              'accounts'),
            ('scheduled_mdn',                ARRAY['email_id'],                'messages'),
            ('mdn_sender_schedule',          ARRAY['account_id'],              'accounts'),
            ('body_gc_candidate',            ARRAY['blob_ref'],                'bodies'),
            ('message_body_cache_lease',     ARRAY['account_id'],              'accounts'),
            ('message_body_cache_lease',     ARRAY['stable_id'],               'messages'),
            ('change_log_checkpoint',        ARRAY['account_id'],              'accounts'),
            ('message_list_envelope',        ARRAY['account_id'],              'accounts'),
            ('message_list_envelope',        ARRAY['stable_id'],               'messages'),
            ('authority_publication_outbox', ARRAY['scope_kind','scope_id'],   'authority_epoch'),
            ('authority_subject_anchor',     ARRAY['account_id'],              'accounts'),
            ('authority_subject_anchor',     ARRAY['scope_kind','subject_id'], 'authority_epoch'),
            ('authority_subject_anchor',     ARRAY['session_id'],              'sessions'),
            ('mail_data_rows',               ARRAY['stable_id'],               'messages')
        ) AS w(table_name, columns, referenced)
    LOOP
        SELECT c.conname INTO found_name
          FROM pg_catalog.pg_constraint c
          JOIN pg_catalog.pg_class t ON t.oid = c.conrelid
          JOIN pg_catalog.pg_namespace n ON n.oid = t.relnamespace
          JOIN pg_catalog.pg_class r ON r.oid = c.confrelid
         WHERE c.contype = 'f'
           AND NOT c.condeferrable
           AND n.nspname = current_schema()
           AND t.relname = wanted.table_name
           AND r.relname = wanted.referenced
           AND ARRAY(SELECT a.attname::text
                       FROM unnest(c.conkey) WITH ORDINALITY k(num, ord)
                       JOIN pg_catalog.pg_attribute a
                         ON a.attrelid = c.conrelid AND a.attnum = k.num
                      ORDER BY k.ord) = wanted.columns
         LIMIT 1;
        IF found_name IS NULL THEN
            RAISE NOTICE '0124: no non-deferrable foreign key %(%) -> %; left as it is',
                wanted.table_name, array_to_string(wanted.columns, ','), wanted.referenced;
        ELSE
            EXECUTE format('ALTER TABLE %I ALTER CONSTRAINT %I DEFERRABLE INITIALLY IMMEDIATE',
                           wanted.table_name, found_name);
        END IF;
    END LOOP;
END $$;
