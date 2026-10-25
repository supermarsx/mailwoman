-- Persist the sign-in origin; an upstream service credential is not user proof.
ALTER TABLE sessions ADD COLUMN authentication_method TEXT NOT NULL DEFAULT 'legacy';
-- Old rows cannot distinguish a user's password from a header-auth service password.
DELETE FROM native_sessions WHERE account_id IN (SELECT account_id FROM sessions WHERE authentication_method='legacy');
DELETE FROM sessions WHERE authentication_method='legacy';
CREATE INDEX IF NOT EXISTS idx_signin_username_result ON signin_event(lower(username_attempted), result, ts);
