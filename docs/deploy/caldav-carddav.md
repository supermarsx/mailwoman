---
title: Calendars, contacts and CalDAV
---

# Calendars, contacts and CalDAV

Mailwoman has calendars, tasks, notes and contacts. This page is for the person running the server. It says where that data lives, which remote servers the server can synchronise with, and which read-only endpoints it exposes. Mailwoman is not a CalDAV or CardDAV server: other clients cannot sync with it over those protocols. The user's side is in [Calendar](/user/calendar), [Contacts](/user/contacts), [Tasks](/user/tasks) and [Notes](/user/notes).

## Where the data lives

In engine mode the server keeps calendars, tasks, notes and contacts in its own database and the web app reaches them through JMAP methods under Mailwoman's own capabilities (`urn:mailwoman:calendars`, `tasks`, `notes` and `contacts`; see [Connecting clients through Mailwoman](/admin/jmap)). These screens need engine mode; in proxy mode the server has no store for them.

Notes are always encrypted at rest with the server key (`MW_SERVER_KEY`). That is not zero-access encryption: the server can read them. A note's title, tags, colour and pinned state are stored in clear so that search and sorting work without a plaintext index of the body.

## Synchronising with a CalDAV or CardDAV server

The server synchronises calendars, task lists and address books with a remote CalDAV or CardDAV server only through the [Nextcloud integration](/integrations/nextcloud). A user links their Nextcloud account, discovers the collections on it, connects the ones they want, and the server pulls changes and writes edits back with conditional requests. There is no setting that attaches another CalDAV or CardDAV server (Radicale, Baïkal, a hosted service) to an account. Where the server supports it, the pull is incremental with a `sync-token`; otherwise the server compares the collection's `ctag` and entity tags.

The **CalDAV URL** and **CardDAV URL** fields in a domain's connection settings (**Domains** in the admin console) are stored with the domain and are checked by the connection test and by the diagnostics. They do not make the server synchronise a user's calendar or contacts with that address.

Calendars a user subscribes to by URL (an `.ics` address) are read-only copies. The server fetches them through the same checked fetcher as remote images, so the outbound connection rules apply; see [Outbound connections](/security/egress).

## Read-only collection endpoints

Two `GET` routes return one collection to a signed-in person in engine mode (other modes answer that the feature needs engine mode):

| Address | Answer |
|---|---|
| `GET /dav/calendars/{accountId}/{calendarId}` | A calendar as iCalendar. The owner can read it, a person it is shared with at read or read and write level can read it, everyone else gets 403. |
| `GET /dav/addressbooks/{accountId}/{addressBookId}` | An address book, for its owner only; address books have no sharing. |

There are no other `/dav` routes: no `PROPFIND`, no writes.

## Holiday feeds

The server includes a few fixed-date holiday lists and serves them as iCalendar to signed-in users:

| Address | Answer |
|---|---|
| `GET /api/holidays` | The list of regions, each with an `id`, a `name` and a `url` |
| `GET /api/holidays/{region}` | One region as a `text/calendar` feed, for example `/api/holidays/us` |

The regions are `us`, `uk`, `ca` and `international`. Each holiday is an all-day yearly event on a fixed date; holidays whose date moves from year to year (such as Easter) are not included.

## Contacts: merging

Merging two contacts is a change made in Mailwoman's own store only. It is never written back to a remote address book: the merged-away cards stay on the remote server and the surviving card there keeps its earlier content. Synchronising again pulls from the remote server and does not carry the merge outward, so another program reading the same address book can still show the duplicates.

## Troubleshooting

- A user's calendar or contacts screen shows errors: the server is in proxy mode. Calendars, tasks, notes and contacts need engine mode.
- A calendar of a remote server never appears: it has to be connected through the Nextcloud integration. A CalDAV or CardDAV URL in the domain settings does not attach it.
- Discovery on a linked Nextcloud account finds nothing new, or sync stops: the app password may have been rejected. Link the account again. See the troubleshooting section of [Nextcloud](/integrations/nextcloud).

## Related

- [Nextcloud](/integrations/nextcloud)
- [Calendar](/user/calendar)
- [Contacts](/user/contacts)
- [IMAP and POP3 mail servers](/deploy/imap-pop3)
