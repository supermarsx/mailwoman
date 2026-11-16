# Sync, offline access and multiple windows

Offline access is off by default. Enable **Offline access** in the device
preferences section of Settings. Administrators can hold the setting and the
sync policy. Use this on a device whose browser profile you trust.

## Choose what to keep

| Setting | Effect |
| --- | --- |
| Header window | Download 30, 90 or 365 days of headers, or all available headers. Flagged and pinned messages are included beyond the window. |
| Message bodies | Download on opening, or prefetch for offline reading. Protected messages are prefetched. |
| Attachments | Download on demand, prefetch inline parts, or prefetch attachments separately from bodies. Automatic downloads still respect attachment policies. |
| Cache budget | Bound the encrypted mail/PIM cache across accounts on this device. Pending work and draft recovery have separate storage. |
| Retention | Expire ordinary cached records after this many days; zero means no age expiry. Pins and flags are protected. |
| Eviction | Remove least recently used records, remove oldest records, or refuse additional cache writes until you clear space. |

The usage display reports encrypted record bytes and protected bytes. Clearing
the read cache does not discard drafts or unsent queued changes. Browser storage
permissions and the browser's own quota can prevent persistence. Clearing site
data removes local offline work too.

Only downloaded content is available offline. Ordinary attachment caching is
limited to 10 MiB per file; a newly attached offline file may be up to 50 MiB.
The header window is a prefetch policy, not a restriction on browsing older mail
online. A full cache can stop read-ahead before the entire requested window fits.

## Working without a connection

An opted-in account can reopen from an encrypted local session snapshot. Mail,
body search, calendar/task/note/contact reads use cached objects. Calendar
recurrence can expand cached masters in a local worker, including time zones,
exclusions and overrides. Uncached external directory or provider data still
requires the server.

Composition, file attachment, filing, flagging and supported PIM edits enter an
encrypted durable queue. New PIM items have temporary local identifiers and can
be edited again before reconnecting. Local acknowledgement is explicitly marked
as queued and is never treated as a successful server replay.

After reconnecting, Mailwoman verifies the live account and its server-issued
storage authority. A changed account or an explicit session refusal prevents
replay and invalidates the offline bootstrap. Flags and moves retain the server
version if the captured baseline changed; the failed change remains reviewable.
Tracked sends keep the same prepared request and retry identity.

The profile encryption key is non-extractable and stored in IndexedDB. This
protects persisted bytes; it is not zero-access protection against code already
running with permission inside that browser profile.

## Background replay and windows

Where the browser supports one-shot Background Sync, queued work can replay
with all Mailwoman windows closed, while the browser is running and its scheduler
permits execution. Unsupported or refused registration is reported honestly:
keep an online Mailwoman window open. With an active service worker, startup
and reconnect request a foreground replay across the verified queued mailboxes;
an open online window retries once per minute when Background Sync is absent
or refused. An expired link keeps its own work pending without blocking healthy
accounts. Without a service worker, the selected mailbox still resumes its queue
after online startup. Background execution is not guaranteed
at a particular time. Desktop shells register an operating-system wake job when
work is queued: Windows Task Scheduler, a Linux user systemd timer, or a macOS
LaunchAgent. The job opens a hidden WebView using the same encrypted queue and
fresh account checks. It stores no credentials in the job definition. OS policy,
a signed-in desktop session, an accessible saved native session and Web Locks
support for cross-window job coordination are required. Older WebViews report
background scheduling as unsupported;
execution at a particular time is not guaranteed. Android/iOS background replay
remains unavailable; keep an online shell open on those platforms.

For the same deployed app build, browser windows share one JMAP/push coordinator.
SharedWorker is preferred; BroadcastChannel plus Web Locks provides leader
failover. Without either mechanism, windows operate independently. Selection is
scoped to the same account and list query; opening a different folder does not
change that folder's selection. Toast actions execute in their originating
window, which must remain open. Encrypted draft recovery updates across windows;
an active editor retains its working copy until explicitly reopened.

## Verification

`apps/web/e2e/sync-offline.spec.ts` builds the production transport, workers,
cache, sanitizer and replay code against a loopback fixture. It exercises two
windows sharing one socket, selection and toast actions, coordinator failover,
offline cold reload, encrypted mail/body/PIM reads, local sanitization, PDF viewer
assets, attachment ranges, future recurring events, provisional note edits,
principal-change refusal and a closed-tab send with a staged attachment. It also
reopens a note created by background replay while offline.

This is browser integration evidence, not certification against every upstream
JMAP, IMAP, POP3, Graph or EWS service. Server push/session scoping is tested
separately in `crates/mw-server/tests/t32_h13_realtime_scope.rs`. Native replay
contracts and desktop scheduler definitions have focused tests. Actual OS wake
behavior still requires platform/device qualification; mobile scheduling is not
implemented.
