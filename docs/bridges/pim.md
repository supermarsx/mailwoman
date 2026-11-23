---
title: Calendars and tasks through a bridge
---

# Calendars and tasks through a bridge

The Microsoft Graph and Exchange (EWS) plugins can read calendars and tasks from the provider, and the server has code that would copy them into the calendar and task screens. In the running server that copy is not started. A mailbox connected through a bridge therefore has its mail from the provider, and its calendar and task screens work on local data and on any CalDAV or Nextcloud collections you add.

This page states what is and is not connected, so you can plan around it.

## What reaches users

| Feature | Graph | EWS | Gmail |
|---|---|---|---|
| Free/busy for another person, asked from the calendar | Yes | Yes | No |
| Provider calendars shown on the calendar screen | No | No | No |
| Provider tasks shown on the task screen | No | No | No |
| Completing a task marks it complete at the provider | No | No | No |

Free/busy lookups try three sources in order and use the first that answers: a calendar shared with you by another account on the same server, then the bridge, then your CalDAV server. An address no source answers for is shown as unknown.

## Why calendars and tasks are not shown

The engine copies bridge calendars and tasks only in a full calendar sync, and the server's calendar syncs (the Nextcloud collection refresh, started on demand or on a timer) always pass a list of selected collections, which skips the bridge. No other server code starts the copy. The plugins' `calendar` and `tasks` interfaces (list collections, read changes, find rooms, get a schedule, complete a task) exist and are tested through the sandbox, but only the schedule call is reachable.

Even if the copy were started, the interfaces have no call to create or change an event or task, so edits made in Mailwoman would stay local.

## Calendars and tasks for a bridge mailbox

The calendar and task screens work on collections stored in Mailwoman. See [CalDAV and CardDAV](../deploy/caldav-carddav.md) for connecting external collections and [Nextcloud](../integrations/nextcloud.md) for the Nextcloud integration. Neither reads Microsoft 365, Exchange or Google calendars through the bridge.

## Other provider features in the plugins

The Graph and EWS plugins also contain code for voting, message recall, Focused Inbox, contact and directory search, room lists and, for EWS, Out-of-Office. Neither provider offers mailbox reactions. No server or web client code calls these, so none of them are available.

## Related

- [Mail provider bridges](./index.md)
- [Plugins](../plugins/index.md)
- [CalDAV and CardDAV](../deploy/caldav-carddav.md)
