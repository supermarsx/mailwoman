-- t87 (D1, D3): durable credentials a linked mailbox created while it was open
-- in its owner's browser: API/MCP keys (`apiKey`, ref = api_keys.key_prefix),
-- OAuth grants (`oauthToken`, ref = oauth_tokens.token_hash, the auth code and
-- every token minted from it) and browser push subscriptions (`push`, ref =
-- personal_push_subscription.id). Unlinking revokes or deletes each one in the
-- same transaction that removes the link, then drops these rows. There is
-- deliberately no foreign key to linked_account: a cascade would forget a grant
-- without revoking it.
CREATE TABLE linked_grant (
 link_id TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN ('apiKey','oauthToken','push')),
 ref TEXT NOT NULL,
 created_at BIGINT NOT NULL,
 PRIMARY KEY(kind,ref)
);
CREATE INDEX linked_grant_link ON linked_grant(link_id);
-- D3: a browser has one push endpoint, and each linked mailbox open in it now
-- keeps its own subscription row for that endpoint. The endpoint is unique per
-- owner instead of across the deployment; the server still refuses to share an
-- endpoint with an owner outside the signed-in person's own mailboxes.
CREATE TABLE personal_push_subscription_t87 (
    id TEXT PRIMARY KEY NOT NULL,
    owner_subject TEXT NOT NULL,
    transport TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    p256dh TEXT,
    auth TEXT,
    app_id TEXT,
    expires_at TEXT,
    created_at TEXT NOT NULL,
    last_wake_at TEXT
);
INSERT INTO personal_push_subscription_t87 (id, owner_subject, transport, endpoint, p256dh, auth, app_id, expires_at, created_at, last_wake_at)
 SELECT id, owner_subject, transport, endpoint, p256dh, auth, app_id, expires_at, created_at, last_wake_at FROM personal_push_subscription;
DROP TABLE personal_push_subscription;
ALTER TABLE personal_push_subscription_t87 RENAME TO personal_push_subscription;
CREATE INDEX personal_push_owner_idx ON personal_push_subscription(owner_subject);
CREATE UNIQUE INDEX personal_push_owner_endpoint ON personal_push_subscription(owner_subject, endpoint);
CREATE INDEX personal_push_endpoint_idx ON personal_push_subscription(endpoint);
