-- Constant default avoids rewriting existing cached message data.
ALTER TABLE messages ADD COLUMN cache_revision TEXT NOT NULL DEFAULT '';
CREATE FUNCTION mw_message_cache_revision() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.cache_revision := gen_random_uuid()::text;
    RETURN NEW;
END;
$$;
CREATE TRIGGER message_cache_revision BEFORE INSERT OR UPDATE ON messages
FOR EACH ROW EXECUTE FUNCTION mw_message_cache_revision();
