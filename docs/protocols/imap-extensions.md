# IMAP extensions

<!-- Generated from crates/mw-imap/src/extensions.rs by scripts/imap/gen-matrix.sh. Do not edit by hand. -->

Which IMAP extensions Mailwoman's IMAP client supports when it talks to an upstream server. Every extension is used only when the server advertises it; without it the client falls back to plain IMAP4rev1 behaviour.

51 extensions: 14 used, 12 parsed but not used, 25 not implemented.

| Extension | Defined in | Support | Notes |
|---|---|---|---|
| `ACL` | RFC 4314 | used | Folder sharing rights (GETACL, SETACL, DELETEACL, LISTRIGHTS, MYRIGHTS). |
| `APPENDLIMIT` | RFC 7889 | not implemented | Planned: refuse an upload that is too large before sending it. |
| `AUTH=OAUTHBEARER` | RFC 7628 | parsed, not used | The SASL frame exists; the server never offers OAuth sign-in to an IMAP upstream. |
| `AUTH=PLAIN` | RFC 4616 | used | Password sign-in when SCRAM is not offered. |
| `AUTH=SCRAM-SHA-1` | RFC 5802 | not implemented | Planned only behind the weak-mechanism setting. |
| `AUTH=SCRAM-SHA-256` | RFC 7677 | used | Preferred password sign-in. |
| `AUTH=SCRAM-SHA-256-PLUS` | RFC 7677, RFC 5929 | used | Preferred over TLS: binds the sign-in to the server certificate (tls-server-end-point). |
| `AUTH=SCRAM-SHA-512` | draft-melnikov-scram-sha-512 | not implemented | Planned. |
| `AUTH=XOAUTH2` | Google, Microsoft | parsed, not used | The SASL frame exists; the server never offers OAuth sign-in to an IMAP upstream. |
| `BINARY` | RFC 3516 | not implemented | Planned: fetch decoded parts. |
| `CATENATE` | RFC 4469 | not implemented | Planned: forward as attachment without downloading. |
| `COMPRESS=DEFLATE` | RFC 4978 | parsed, not used | Capability read; the DEFLATE stream exists but is not switched on. |
| `CONDSTORE` | RFC 7162 | parsed, not used | Capability read; the sync path does not reach it yet. |
| `CREATE-SPECIAL-USE` | RFC 6154 | not implemented | Planned: create Drafts, Sent, Trash with their role. |
| `ENABLE` | RFC 5161 | used | Turns on the extensions below that need it. |
| `ESEARCH` | RFC 4731 | parsed, not used | Replies parse; searches do not ask for them yet. |
| `ID` | RFC 2971 | used | Sends the client name and version. |
| `IDLE` | RFC 2177 | used | Push for INBOX only. |
| `IMAP4rev2` | RFC 9051 | parsed, not used | Capability read; not enabled until ESEARCH replies are used. |
| `INPROGRESS` | RFC 9585 | not implemented | Planned: show progress of long commands in diagnostics. |
| `LIST-EXTENDED` | RFC 5258 | parsed, not used | Extended LIST replies parse; no selection options are sent. |
| `LIST-STATUS` | RFC 5819 | used | Folder list and counts in one command. |
| `LITERAL+` | RFC 7888 | not implemented | Planned: uploads without waiting for the server's go-ahead. |
| `LITERAL-` | RFC 7888 | not implemented | Planned: as LITERAL+, up to 4096 bytes. |
| `LOGINDISABLED` | RFC 9051 | not implemented | Planned: never send LOGIN when the server forbids it. |
| `MESSAGELIMIT` | RFC 9738 | not implemented | Planned: keep batches within the server's limit. |
| `METADATA` | RFC 5464 | used | Server and folder annotations. |
| `MOVE` | RFC 6851 | used | Moves messages; COPY plus delete otherwise. |
| `MULTIAPPEND` | RFC 3502 | not implemented | Planned: faster imports. |
| `NAMESPACE` | RFC 2342 | not implemented | Planned: shared and other users' folders. |
| `NOTIFY` | RFC 5465 | not implemented | Planned: one connection watches all subscribed folders. |
| `OBJECTID` | RFC 8474 | parsed, not used | Capability read; EMAILID and THREADID are not fetched. |
| `PARTIAL` | RFC 9394 | not implemented | Planned: paged search and fetch for large folders. |
| `PREVIEW` | RFC 8970 | not implemented | Planned: list previews without fetching bodies. |
| `QRESYNC` | RFC 7162 | parsed, not used | Capability read; the sync path does not reach it yet. |
| `QUOTA` | RFC 9208 | not implemented | Planned: show the upstream mailbox quota. |
| `REPLACE` | RFC 8508 | not implemented | Planned: draft saves in one command. |
| `SASL-IR` | RFC 4959 | not implemented | Planned: one round trip less at sign-in. |
| `SAVEDATE` | RFC 8514 | not implemented | Planned. |
| `SEARCHRES` | RFC 5182 | not implemented | Planned: reuse a search result in the next command. |
| `SORT` | RFC 5256 | parsed, not used | Implemented; only tests call it. |
| `SPECIAL-USE` | RFC 6154 | used | Folder roles from LIST attributes. |
| `STARTTLS` | RFC 9051 | used | Upgrade to TLS on port 143. |
| `STATUS=SIZE` | RFC 8438 | not implemented | Planned: folder sizes for diagnostics and quota display. |
| `THREAD=ORDEREDSUBJECT` | RFC 5256 | parsed, not used | Implemented; only tests call it. |
| `THREAD=REFERENCES` | RFC 5256 | parsed, not used | Implemented; only tests call it. |
| `UIDONLY` | RFC 9586 | not implemented | Planned. |
| `UIDPLUS` | RFC 4315 | used | New UIDs after upload, copy and move. |
| `UNSELECT` | RFC 3691 | used | Leaves a folder before renaming or deleting it, without expunging. |
| `UTF8=ACCEPT` | RFC 6855 | not implemented | Planned: UTF-8 folder names; modified UTF-7 otherwise. |
| `WITHIN` | RFC 5032 | not implemented | Planned: date-relative search. |
