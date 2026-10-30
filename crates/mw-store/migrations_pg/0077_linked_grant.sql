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
-- D3: the endpoint is unique per owner instead of across the deployment (see
-- the SQLite variant).
ALTER TABLE personal_push_subscription DROP CONSTRAINT personal_push_subscription_endpoint_key;
CREATE UNIQUE INDEX personal_push_owner_endpoint ON personal_push_subscription(owner_subject, endpoint);
CREATE INDEX personal_push_endpoint_idx ON personal_push_subscription(endpoint);
