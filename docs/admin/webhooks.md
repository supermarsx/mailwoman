---
title: Webhooks
screen: true
---

# Webhooks

A webhook connects one account to another system. An **outbound** webhook sends a signed notice to a URL when something happens in the account. An **inbound** webhook lets another system ask the server to run a mail rule action in the account. Webhooks are managed in the administration console under **Integrations**, on the **Webhooks** tab.

![The Webhooks tab of Integrations with one outbound and one inbound webhook listed above the Add a webhook form.](/screenshots/admin-ops/opsc-webhooks-list.webp){.mw-shot}

## Before you start

- Listing webhooks needs the console permission `integrations.read`. Adding and deleting need `integrations.write`.
- An outbound URL must reach a public internet address. The server resolves the host when you add the webhook and again on every delivery, and refuses private, loopback and link-local addresses. Outbound webhooks do not use the routes under **Outbound connections**.
- The **Account** field takes the account identifier as the server uses it. The server does not check that the account exists, so a mistyped value is saved and never matches an event.

## Add an outbound webhook

1. Open **Integrations** and choose the **Webhooks** tab.
2. Under **Add a webhook**, leave **Direction** on **Outbound — Mailwoman calls your URL.**
3. Enter the **Account**, an optional **Name**, and the **URL**. Deliveries use `http://` or `https://` addresses.
4. Tick the **Events** the URL should receive. **New message** is ticked at first.
5. Choose **Create webhook**.

![The Add a webhook form for an outbound webhook, with markers on the direction, the account, the URL and the Create webhook button.](/screenshots/admin-ops/opsc-webhooks-add.webp){.mw-crop}

1. **Direction** chooses between **Outbound** and **Inbound**.
2. **Account** is the account whose events are sent.
3. **URL** is where deliveries go.
4. **Create webhook** saves it and shows the secret.
{.mw-callouts}

The events and the names they have on the wire:

| Event in the console | Sent as |
|---|---|
| **New message** | `message.received` |
| **Message read, flagged or labelled** | `message.flags` |
| **Message moved** | `message.moved` |
| **Message deleted** | `message.deleted` |
| **Message sent** | `submission.sent` |
| **Message could not be sent** | `submission.failed` |
| **Calendar, contacts, tasks or notes changed** | `pim.changed` |

An endpoint receives only the events you tick. Choose at least one event; the form refuses an empty choice.

### The secret

After **Create webhook** the page shows **Secret for** the webhook name. **Copy this secret now. It is not shown again.** Choose **I have copied it** when you have.

![The secret of a new webhook, hidden here, with the line saying it is not shown again, how deliveries are signed, and the button I have copied it.](/screenshots/admin-ops/opsc-webhooks-secret.webp){.mw-crop}

The list never shows a secret. To get a new one, delete the webhook and add it again.

### What a delivery looks like

A delivery is an HTTP `POST` with a JSON body, the header `X-Mailwoman-Event` naming the event, and the header `X-Mailwoman-Signature: sha256=<hex>`. The signature is the HMAC-SHA256 of the exact body bytes under the secret; compare it in constant time. The body holds the event name as `type`, the `accountId`, the opaque `ids` the event touched and a `timestamp`. It never holds a subject, an address or message text.

The server tries a delivery up to three times, with a short pause that doubles between tries, and retries only after a `5xx` answer or a connection failure. An answer in the `2xx` or `4xx` range ends the attempts. A redirect answer ends the delivery without following it, so the signed body never reaches the redirect target.

## Add an inbound webhook

1. Under **Add a webhook**, choose **Inbound — Your system calls Mailwoman.**
2. Enter the **Account** and an optional **Name**.
3. Tick the **Allowed actions**: **Run the account's mail rules on a message description** (`run_rules`) or **Apply one rule action to a message** (`sieve`).
4. Choose **Create webhook** and copy the secret.

The caller sends `POST /api/webhooks/inbound` with these headers:

- `X-Mailwoman-Webhook`: the webhook id, shown in the page that holds the secret.
- `X-Mailwoman-Timestamp`: the current time in Unix seconds.
- `X-Mailwoman-Nonce`: 16 to 128 letters, digits, hyphens or underscores, never used before for this webhook.
- `X-Mailwoman-Signature`: `sha256=` and the hexadecimal HMAC-SHA256 of `timestamp.nonce.body` under the secret.

The body is JSON with an `action` object whose `kind` is `run_rules` or `sieve`, and optionally an `account` that must name the webhook's own account.

| Answer | Meaning |
|---|---|
| `202` with `dispatched: true` | The action ran; `fired` counts the actions it fired. |
| `202` with `dispatched: false` | The call was accepted but this server has no mail engine to run it (proxy mode). |
| `400` | The body is not a recognised action. |
| `401` | Unknown webhook, wrong signature, or a timestamp more than five minutes from the server clock. |
| `403` | The body names another account, or an action this webhook was not allowed to run. |
| `409` | The nonce was used before. |
| `502` | The mail engine failed to run the action. |

The list shows **Not used yet** under an inbound webhook until its first accepted call, then **Last used** with the time.

## Delete a webhook

Choose **Delete** on the row and confirm in the dialog that asks **Delete the webhook** and the webhook's name. Deliveries to it stop, or calls signed with its secret are refused, from then on. This cannot be undone; a new webhook gets a new secret.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Webhook attempts** ([`webhooks.maxAttempts`](/admin/settings#webhooks.maxattempts)) | Appears in Settings but has no effect: the server always makes up to 3 delivery attempts. | 3 | Administrator |
| **Webhook timeout** ([`webhooks.timeoutSecs`](/admin/settings#webhooks.timeoutsecs)) | Appears in Settings but has no effect: each delivery attempt always times out after 10 seconds. | 10 | Administrator |

## Troubleshooting

- **The URL's host does not resolve to a public internet address, so deliveries to it would be refused. Private, loopback and link-local addresses are not allowed.** The URL points at an internal host or a name that does not resolve. Use a public address.
- **Someone changed the webhooks since you opened this page. The list was reloaded; try again.** Another administrator added or deleted a webhook. Repeat your change on the reloaded list.
- **Enter a URL starting with https:// or http://.** The URL is not a full address.
- Inbound calls answered `401` with a correct secret: check the clock of the calling system against the server's, and that the signature covers `timestamp.nonce.body` exactly as sent.
- The server refuses to start when `MW_WEBHOOK_INBOUND_SECRET` is set, even empty. Remove it from the environment and create one inbound webhook for each caller.

## Related

- [Settings reference](/admin/settings)
- [Outbound connections](/security/egress)
- [Mail rules for users](/user/rules-filters)
