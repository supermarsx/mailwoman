# Admin console: Server metadata

The **Server metadata** page (`#/server-metadata`) shows and edits the RFC 5464 (IMAP METADATA)
server-level annotations of one provisioned account. With no account provisioned the page offers
**Add a user**, which opens the Users section.

<a id="account"></a>
## Account

The account whose annotations are shown. The editor appears once one is chosen. Reads and writes go to
that account's mail server (`ServerMetadata/get` and `ServerMetadata/set` over `/jmap/api`), which decides
what the account may set; an upstream that does not offer METADATA is shown as not available inside the
editor.
