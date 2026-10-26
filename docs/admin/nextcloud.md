# Nextcloud account links

Administrators approve Nextcloud instances in **Administration → Nextcloud**. Each mailbox user then verifies their own link under **User settings → Nextcloud**. Instance settings never contain a shared mailbox password. Existing `MW_NEXTCLOUD_*` environment credentials no longer grant every mailbox access to one shared cloud account.

The installation URL must use HTTPS and may include an installation prefix, for example `https://cloud.example/nextcloud/`. User information, query strings, fragments, and encoded path prefixes are refused. The optional PEM field adds certificate authorities for this instance's origin connection. Normal public trust and hostname verification remain enabled. It does not change certificate trust for an HTTPS forward proxy.

Changing an instance creates a new authority revision. Existing links become unavailable until their owners verify them again. Disabling an instance stops new outbound operations. Users can still remove their disabled or stale links. Editing requires integration administration permission and fresh authentication proof; saving a stale form is refused rather than overwriting a newer revision.

Users enter their Nextcloud username and an application password. Verification makes one bounded authenticated DAV request; rejected credentials are not retried automatically. Secrets are sealed against the mailbox account, instance, authority revision, and remote username. A request that observes rejected stored credentials removes only the rejected link revision, preserving a concurrently verified replacement. Opening the composer reads local link metadata and does not replay credentials upstream.

File operations use the selected outbound route and the dedicated Nextcloud traffic class. They do not fall back to direct networking when a selected proxy fails. Redirects on credential-bearing requests are refused. Requests and responses are bounded to 8 MiB, with four concurrent operations and a 15-second transport deadline. Multipart attachment selection has a combined 8 MiB limit.

Exporting a message attachment requires its owned message and blob identifiers. The server derives the filename and MIME type and applies the current attachment download policy before contacting Nextcloud. A blocked file is refused; a warning requires explicit acknowledgement. Export does not bypass download restrictions.

## Current limits

This implementation supplies account-owned file listing, attachment import/export APIs, and public share-link creation. Calendar and contact synchronization are separate pending integrations; the presence of `mw-dav` does not make these Nextcloud flows available. Multi-instance selection in file pickers and the message reader's save action still require their UI integration before acceptance.

Private-network instance permission management is not yet exposed on this page. Metadata destinations remain blocked. A private destination that a configured proxy cannot safely authorize is refused; it is never reached through a direct fallback. Do not use a certificate verification bypass to work around routing or trust failures.
