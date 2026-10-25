# Webhooks

Open **Integrations → Webhooks** in the admin console to create, list or delete
webhooks. Reading requires `integrations.read`; creating and deleting require
`integrations.write`. API paths below use the default admin path; replace `/admin`
with your configured `MW_ADMIN_PATH` when it differs.

Each webhook belongs to one account. Its secret is shown only in the creation
response. Store it securely: listing webhooks does not reveal it. Deleting a
webhook revokes its use.

## Outbound events

Create an outbound webhook with an account, an HTTP or HTTPS endpoint and the event kinds
it should receive. Registration and delivery both check the destination through
the egress policy. Redirects are refused. Payloads contain identifiers and state
tokens, not message subjects or bodies.

Engine events are `message.received`, `message.flags`, `message.moved`,
`message.deleted`, `submission.sent`, `submission.failed` and `pim.changed`.
Older registrations with an empty filter or `StateChange` retain the coalesced
state-change feed.

Verify `X-Mailwoman-Signature: sha256=<hex>` using HMAC-SHA256 over the exact
request body with the webhook secret. Compare signatures in constant time.

## Inbound actions

Create an inbound webhook with an account and its allowed actions (`run_rules`
and/or `sieve`). Send requests to `POST /api/webhooks/inbound` with:

- `X-Mailwoman-Webhook`: the webhook ID.
- `X-Mailwoman-Timestamp`: the current Unix timestamp in seconds.
- `X-Mailwoman-Nonce`: a unique 16–128 character value using letters, digits,
  underscores or hyphens.
- `X-Mailwoman-Signature`: `sha256=` followed by the hexadecimal HMAC-SHA256 of
  `timestamp.nonce.body`, using the exact timestamp, nonce and body bytes sent.

Timestamps outside a five-minute window are refused with `401`. Reused nonces
are refused with `409`, including across processes sharing the database. A
webhook cannot act on another account or use an action absent from its grant.
An accepted call returns `202`; check `dispatched` in the response. Proxy mode
has no engine dispatcher and returns `dispatched: false`.

## API and upgrade

`GET /admin/webhooks` returns `webhooks`, `eventKinds`, `actionKinds` and `etag`.
`POST /admin/webhooks` creates a webhook; `DELETE /admin/webhooks/{id}` removes
one. Send the listing's ETag as `If-Match` on writes. A stale tag returns `412`;
reload before retrying. Creation accepts `dryRun: true` to validate without saving.

The former process-wide `MW_WEBHOOK_INBOUND_SECRET` is no longer supported.
Remove it from the environment before starting the server: even an empty value
is refused. Create individual inbound webhooks and update each caller to use its
ID, secret, timestamp and nonce. Existing callers using only the old global
signature must be updated before they can resume sending actions.
