-- t34 review: retire the 0117 signin_identity triggers (see the SQLite
-- migration). On PostgreSQL they also put a row lock on the settings_revision
-- row into every first sign-in and address change, in every mode.
DROP TRIGGER IF EXISTS settings_revision_signin_identity_insert ON signin_identity;
DROP TRIGGER IF EXISTS settings_revision_signin_identity_update ON signin_identity;
DROP TRIGGER IF EXISTS settings_revision_signin_identity_delete ON signin_identity;
DROP TRIGGER IF EXISTS settings_revision_signin_identity_truncate ON signin_identity;
