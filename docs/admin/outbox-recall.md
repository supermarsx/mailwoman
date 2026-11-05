# Outbox, recall and scheduled sending

Every message sent from the web client becomes an `EmailSubmission` row held by the server. The Outbox is that queue. Because the row is stored in the database (SQLite or Postgres), a queued message survives closing the browser, a server restart and a rolling deploy.

## States

A submission moves through one state machine. Each step is a compare-and-set on the row, so two actors (the user's Undo, the dispatcher, a second server instance) cannot both win.

| State | Meaning |
|---|---|
| `pending` | Queued. Either inside the undo window (`createdAt + holdSeconds`) or waiting for its scheduled `sendAt`. Can be recalled, sent now or cancelled. |
| `sending:<token>` | The dispatcher claimed the row before dialling SMTP. It can no longer be recalled; the client says so. |
| `final` | Accepted by the outgoing server. The Sent copy is filed now, not before. |
| `canceled` | Recalled or cancelled. The message stays a draft. |
| `failed` / `uncertain` | The engine stopped retrying, or an attempt may have delivered. Never retried automatically when uncertain. |

A recall of a row that is not `pending` is refused (`notUpdated`) and the interface shows "Too late to recall: the message is already being sent". Recalling twice is idempotent: the second request is refused and creates no duplicate.

## Recall

* Post-send toast: "Message queued — sends in 0:25" with a live countdown, **Undo** and **Send now**. Undo reopens the message in the composer as a draft with its recipients (including Bcc), body, attachments and sending identity.
* Outbox view: each waiting row shows its countdown or scheduled time and offers **Recall and edit**, **Send now** and **Cancel**.
* The undo window is the server's: it ends on time even if the browser is closed or the toast is hovered.

Settings (Settings, Compose, "Undo send"; Admin, Settings):

| Key | Default | Notes |
|---|---|---|
| `mail.send.recallDelaySeconds` | 10 | 0 to 1800. 0 sends at once. User choice within the cap; an administrator can set a domain default or lock it. |
| `mail.send.recallDelayMaxSeconds` | 1800 | Cap on the user's choice. Enforced by the engine at create time, not only by the client. |
| `mail.send.allowRecall` | true | Off sends immediately and removes the recall window. |

## Scheduled sending

**Send later** next to Send offers "In 1 hour", "Later today, 17:00", "Tomorrow morning, 09:00", "Monday morning, 09:00" and a date-time picker that shows the viewer's time zone. The instant is stored as UTC (`sendAt`); the time zone is only used to display it. Pre-send checks run before the message is queued, not at send time.

* The dispatcher scans for due rows on a timer and at startup. A message whose time passed while the server was down is sent promptly after start. Decisions use the server's clock only, so a client with a wrong clock cannot send early (a `sendAt` in the past is refused).
* Scheduled messages appear in the Outbox with a "Scheduled" badge and their send time.
* A send that fails is retried with backoff. When the engine gives up, the row is marked failed with the last error and the message stays in Drafts; nothing is dropped silently.

| Key | Default | Notes |
|---|---|---|
| `mail.send.maxDelayedSeconds` | 7776000 (90 days) | 0 to 31536000. A `sendAt` further ahead is refused with `invalidProperties`. 0 turns scheduling off. |
| `mail.send.maxQueued` | 200 | Waiting messages per mailbox. Further sends are refused with `overQuota`. |

## Operational notes

* Queue and limits are per mailbox. Limits are read from the settings model at create time (global, then domain; a user cannot store them).
* In a multi-instance deployment all instances scan the same table; the claim token makes exactly one of them send a given row.
* To stop all delayed sending for a domain: set `mail.send.allowRecall` to false and `mail.send.maxDelayedSeconds` to 0.
* The retained draft of a recalled message exists twice, in the browser (the reopened composer) and in the account's Drafts folder; discard the Drafts copy if it is not wanted.
