-- t34 review: retire the 0117 signin_identity triggers. They bumped the GLOBAL
-- settings revision whenever an account's sign-in address changed, and the
-- address is stored as typed, so one user alternating aliases or letter case
-- invalidated every settings memo and every published entry on each sign-in.
-- The session watch instead re-reads the address (one primary-key read) and
-- reuses cached alert preferences only for the same account AND address.
DROP TRIGGER IF EXISTS settings_revision_signin_identity_insert;
DROP TRIGGER IF EXISTS settings_revision_signin_identity_update;
DROP TRIGGER IF EXISTS settings_revision_signin_identity_delete;
