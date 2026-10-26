-- Ephemeral, deployment-local single-use proof-of-work challenges.
CREATE TABLE signin_challenge (
 nonce TEXT PRIMARY KEY NOT NULL,
 surface TEXT NOT NULL CHECK(surface IN ('mailbox','admin')),
 peer_key TEXT NOT NULL,
 expires_at BIGINT NOT NULL
);
CREATE INDEX signin_challenge_expiry ON signin_challenge(expires_at);
CREATE INDEX signin_challenge_peer ON signin_challenge(peer_key);
