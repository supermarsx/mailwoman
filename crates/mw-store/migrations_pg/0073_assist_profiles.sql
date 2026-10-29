-- t39-e11 (N8): AI provider profiles. A profile is a named endpoint, model and
-- options in one of three request dialects; which profile applies to whom is the
-- settings registry key `assist.profile` (deployment, domain and per-user values
-- with locks). The API key is sealed with the server key (XChaCha20-Poly1305,
-- purpose "assist-profile-key-v1") and bound by AAD to id + dialect + endpoint,
-- so a blob copied onto another row, or kept across an endpoint change, does not
-- open. It is never returned by the admin API.
CREATE TABLE assist_profile (
 id TEXT PRIMARY KEY NOT NULL,
 name TEXT NOT NULL,
 preset TEXT NOT NULL,
 dialect TEXT NOT NULL CHECK(dialect IN ('openai','azure-openai','anthropic')),
 endpoint TEXT NOT NULL,
 model TEXT NOT NULL,
 params TEXT NOT NULL DEFAULT '{}',
 api_key_sealed BYTEA,
 created_by TEXT NOT NULL,
 created_at BIGINT NOT NULL,
 updated_by TEXT NOT NULL,
 updated_at BIGINT NOT NULL
);
-- t39-e11 (N8): the host:port destinations a profile may reach although they
-- resolve to a loopback or private address (Ollama on localhost or the LAN).
-- Without a row here a profile is limited to public addresses. Link-local and
-- metadata addresses are refused even with a row (checked in mw-assist).
CREATE TABLE assist_egress_grant (
 id TEXT PRIMARY KEY NOT NULL,
 destination TEXT NOT NULL UNIQUE,
 note TEXT NOT NULL DEFAULT '',
 created_by TEXT NOT NULL,
 created_at BIGINT NOT NULL
);
