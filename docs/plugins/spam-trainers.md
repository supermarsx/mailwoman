---
title: Spam classifier plugins
---

# Spam classifier plugins

Two first-party plugins let Mailwoman ask an existing spam filter whether a new message is spam: `spam-rspamd` talks to Rspamd and `spam-spamassassin` talks to SpamAssassin. They classify only. They do not train the filter.

![The Rspamd plugin page with Status, Granted capabilities (spam-action, net, store-kv-scoped), Network and limits, and the Spam classifier panel with the classifier address field and the Save address and Test classifier buttons, the latter noting that the plugin is not loaded so it cannot be tested.](/screenshots/integrations/admin-plugin-classifier.webp){.mw-shot}

## What happens to a message

When a new message arrives in an Inbox, the server sends its raw bytes to the classifier plugin. The plugin returns one of three verdicts:

- **spam**: the message is tagged with the `$Junk` keyword and moved to the account's Junk folder. If the account has no Junk folder, the message stays in the Inbox, tagged.
- **ham**: nothing changes.
- **unknown**: nothing changes. This is the result when the filter is unreachable, answers with an error status, returns more than 1 MiB or returns malformed data. A failure never blocks or drops a message.

Messages delivered to other folders are not classified. The server records a content-free audit event, `spam.classified`, with the verdict only. If no classifier is loaded, delivery is unchanged.

Only one classifier runs at a time: the first approved, enabled plugin in id order that has the `spam-action` grant and a component that passes the digest check. The page of any other classifier shows **Not loaded: another spam classifier is loaded. Only one runs at a time.**

## Set up a classifier (administrator)

1. Open **Plugins** in the admin console and approve, grant and enable the plugin as described in [Plugins](./index.md#approve-and-enable). Grant `spam-action` and `net`. Also grant `store-kv-scoped`, which the plugin needs to read the address you set below.
2. In the **Spam classifier** panel, enter the **Classifier address** and choose **Save address**. Leave it empty to use the plugin's default. The host must be in the plugin's host list.
3. Choose **Test classifier**. The page shows **Test verdict:** with the verdict. The button does not work while the plugin is not loaded.

The host lists and defaults are:

| Plugin | Default address | Host list |
|---|---|---|
| `spam-rspamd` | `http://rspamd:11333` | `rspamd` |
| `spam-spamassassin` | `spamassassin:783` | `spamassassin` |

To use another host name, register the plugin again with a different host list. See [Plugins](./index.md#sandbox-and-limits) for how requests are routed and limited. A classifier on a private network needs an internal-destination permission in **Egress** for the **Approved plugin and provider requests** class.

## Rspamd

The plugin posts the raw message to the scan worker's `/checkv2` endpoint (port 11333 by default) and reads the JSON answer. The message is spam when the action is `reject`, `add header`, `rewrite subject` or `soft reject`, and ham for `no action` or `greylist`. When the action is missing or unknown, the score is compared with `required_score`. A well-formed answer with no decidable signal counts as ham.

## SpamAssassin

SpamAssassin's `spamd` speaks its own protocol over raw TCP, not HTTP. The plugin builds a SPAMC/1.5 request and sends it as the body of an HTTP POST, so you need a small relay between the server and `spamd` that forwards the body to `spamd` and returns the reply. The repository includes one, `scripts/spamc-relay/relay.py`, used by the project's tests, which forwards to `spamd` at `SPAMD_ADDR` (default `spamd:783`) and listens on `RELAY_PORT` (default 783). The plugin reads the `Spam: True/False ; score / threshold` line of the reply.

## Limits

- **No training.** The plugins implement classification only. The Rspamd learn endpoints (`/learnham`, `/learnspam`) are not called, and moving a message to Junk or out of Junk does not teach the filter.
- Responses larger than 1 MiB are treated as unknown.
- The plugin's time limit is 10 seconds per call and its memory limit is 32 MiB.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| **Not loaded: no capability it needs has been granted.** | `spam-action` is not granted. | Grant it and save. |
| Test verdict is `unknown`. | The filter is unreachable, the host is not in the host list, or the address is wrong. | Check the address, the host list and the **Egress** route. |
| Messages are never moved. | The verdict is not spam, or the message did not arrive in the Inbox. | Use **Test classifier** with a known test message. |
| SpamAssassin never answers. | No relay in front of `spamd`. | Run a SPAMC-over-HTTP relay as described above. |

## Related

- [Plugins](./index.md)
- [Outbound connections](../security/egress.md)
