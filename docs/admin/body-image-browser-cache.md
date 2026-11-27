# Browser cache for message body images

Settings → Privacy includes **Cache message body images in this browser**, enabled
by default, and **Body image cache lifetime**, initially 10 minutes. The lifetime
can be set from one minute to three years, using minutes, hours, days, or years.
A year is 365 days. Remote and inline images loaded through Mailwoman
use the browser's private HTTP cache. Direct sender loads, when explicitly
permitted, retain the sender's cache policy. This does not change the server image cache.

Admins can set defaults and lock either preference in Privacy → Remote content
or in the settings registry. Global, domain, and user policy inheritance applies.

| Setting | Default | Values |
| --- | --- | --- |
| `privacy.bodyImageCacheEnabled` | `true` | Boolean |
| `privacy.bodyImageCacheTtlSecs` | `600` | 60–94608000 seconds (three years) |

Fresh images are reused without another image request. Expired images revalidate
with an ETag; an unchanged image returns `304` without downloading its bytes.
Reopening a message reauthorizes its images and reuses stable browser-cache URLs;
explicit image retry uses a new URL. Browser freshness does not extend server
authorization: the signed image capability expires within one hour and is renewed
only after a successful authorized signing request. This allows long-lived private
browser copies without granting three-year network access to an image link.

Turning caching off gives newly loaded images `Cache-Control: private, no-store`.
Changing either preference changes newly signed image URLs, so later message loads
do not reuse copies under the previous preference. Images already displayed stay
visible. The browser manages cache storage and eviction; this setting does not
reserve storage or alter encrypted offline mail storage.
