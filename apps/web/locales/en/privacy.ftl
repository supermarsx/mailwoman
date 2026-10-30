privacy-title = Privacy
privacy-help = Remote images can reveal when you open a message. Choose when they load and whether the server fetches them for you.
privacy-remote-mode = Remote content
privacy-mode-block = Block remote content
privacy-mode-ask = Ask before loading through the server
privacy-mode-proxy = Load through the server
privacy-mode-direct = Load directly from the sender's servers
privacy-trackers = Block tracking images and beacons
privacy-clean-links = Remove tracking parameters from links
privacy-direct-unavailable = Direct loading requires permission from your administrator. It exposes your network address to the image host.
privacy-direct-native = Direct loading is not available in this native app. Images can load through the server.
privacy-inherit = Use inherited setting
privacy-content-failed = Some images could not be authorized. The message remains readable with those images blocked.
privacy-frame-failed = The isolated image viewer could not load. Reload the message to try again.
privacy-download-file = Download through the server
privacy-file-failed = This file could not be downloaded through the server.
privacy-admin-title = Privacy and remote content
privacy-admin-help = Set defaults, locks and limits through the settings policy. Diagnostic counts cover this server process only.
privacy-cache-entries = Cached images
privacy-cache-bytes = Cache size (bytes)
privacy-replaced-trackers = Tracking pixels replaced
privacy-edit-setting = Edit policy
privacy-outbound-title = Outbound connections
privacy-fail-closed = A failed configured route blocks the request. It never falls back to a direct connection.
privacy-linked-files = Files linked in this message
privacy-allow-direct = Allow direct image loading
privacy-max-bytes = Maximum image size (bytes)
privacy-cache-limit = Maximum cache size (bytes)
privacy-rate = Requests per minute
privacy-on = On
privacy-off = Off
privacy-locked = Locked by administrator policy
privacy-process-stats = Since this server process started
privacy-refresh = Refresh
privacy-traffic-class = Traffic class
privacy-allowed = Allowed
privacy-refused = Refused by policy
privacy-failed = Failed

privacy-policy-values = Remote-content policy
privacy-cache-summary = Cache and tracking protection
privacy-traffic-summary = Outbound traffic processing
privacy-metric = Metric
privacy-value = Value
privacy-total-duration = Total time
privacy-no-traffic = No outbound traffic has been recorded since this process started.
privacy-analytics-title = Optional usage counts
privacy-analytics-help = Off by default. If your administrator enables a collector, you may share fixed screen names such as Mail or Calendar. Messages, email addresses, searches, route details and browser identifiers are never sent. These are event counts, not unique visitor statistics.
privacy-analytics-consent = Allow optional screen usage counts for this account
privacy-analytics-signal = Your browser privacy signal prevents analytics, even when consent is saved.
privacy-analytics-save-error = Consent could not be synchronized. This browser pauses new analytics requests until the change is saved.
privacy-analytics-saving = Saving consent…

privacy-back-policies = Back to privacy policies

privacy-image-toast = Show a notification while remote images load
privacy-image-downloads = Manual remote-image downloads
privacy-image-download-help = If an image does not display, download a checked copy through this server. The server verifies that it belongs to this message and still applies its privacy and security rules. Client-decrypted images are not available here.
privacy-image-download-ready = The image is ready and its download has started.
privacy-image-download-failed = The image could not be downloaded. It may be unavailable or blocked by your privacy policy.

privacy-strip-mailer-headers = Remove outgoing mailer headers
privacy-strip-mailer-help = Native SMTP and EWS sends remove top-level client fingerprint headers. Message bodies, attached messages and report payloads remain intact. This does not cover upstream JMAP proxy sends or headers added later by a relay. Pre-signed messages covering a removed header require this setting to be disabled before sending.
privacy-frame-compatible = Compatible viewer: images loaded directly from other sites stay blocked for privacy. Embedded images and images loaded through the server are supported.
privacy-analytics-tag-custom = a custom analytics script
privacy-analytics-tag-title = Browser analytics: { $provider }
privacy-analytics-tag-help = Your administrator also offers { $provider }, which runs in this page in your browser. Unlike the counts above, its script can see the pages you open in this app, including messages and addresses shown on screen, your browser and IP address, and it may set its own identifiers. The provider can update its script without asking you again. Off unless you turn it on here; turning it off reloads the page without it, in every tab.
privacy-analytics-tag-consent = Allow { $provider } to run in this app for this account
privacy-analytics-tag-changed = The analytics provider, or how this server loads it, changed since you last agreed. It stays off until you agree again.
privacy-trusted-links-title = Trusted link domains
privacy-trusted-links-help = Links to these domains open straight from Review links, without the destination step. Other links, including lookalike domains, are still reviewed. Links that use plain HTTP or an IP address are always reviewed.
privacy-trusted-links-domains = Domains
privacy-trusted-links-domains-help = One domain per line, for example example.com. Matching is exact: example.com does not trust examples.com or example.com.other.net. Use the ASCII (xn--) form for international names.
privacy-trusted-links-invalid = Enter up to 100 domain names with at least two labels and no scheme, path, port or wildcard. Not accepted: { $entries }
privacy-trusted-links-save = Save trusted domains
privacy-trusted-links-subdomains = Also trust subdomains (mail.example.com for example.com)
privacy-trusted-links-subdomains-help = Off: only the exact domain is trusted. On: any name ending in .example.com is trusted as well.

privacy-linked-files-count = { $count ->
    [one] One file linked in this message
   *[other] { $count } files linked in this message
}
privacy-linked-links = Links in this message
privacy-linked-files-show = Show files
privacy-linked-links-show = Download a link
privacy-linked-files-help = Downloads go through this server, so the file host does not see your address. A file's size is known only once it is downloaded.
privacy-linked-links-help = A link in this message can be downloaded as a file through this server, so its host does not see your address.
privacy-linked-file-item = Download { $name } from { $host }
privacy-linked-other-item = Download { $link }

privacy-links-enable = Enable links
privacy-links-disable = Disable links
privacy-links-enable-help = Allow links in this message to open in a new tab. You can disable them again.

privacy-links-loading = Preparing links…

privacy-body-blocked = Message HTML blocked for safety
privacy-body-blocked-detail = This body contains potentially unsafe content or could not be checked. Only safe text is shown. The original message is unchanged.
