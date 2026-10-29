# Mailwoman — remote-content (image-grant) bar strings (source locale: en).
#
# Owned by t16-e14b (reader image-grant bar + blocked/tracker count). Lazily
# loaded via `loadCatalog('remote-images')`. Ids are kebab-case, `remote-`
# prefixed. Sender addresses / domains interpolated below are untrusted and are
# rendered with `dir="auto"` at the call site.
#
# HONESTY: this copy states exactly what the sanitizer blocked and what each
# action loads — no hype. Do NOT soften "blocked"/"remote images" into vague
# reassurance when translating.

remote-bar-label = Remote content

remote-change-failed = Could not update remote image permissions. { $reason }

# -- Single-line banner (t55) ---------------------------------------------------
remote-banner-blocked = Remote images blocked
remote-banner-blocked-policy = Remote images blocked by your privacy settings
remote-banner-shown = Images shown
remote-banner-proxied = Images loaded through the server
remote-banner-direct = Images loaded from the sender's servers
remote-banner-trackers = { $count ->
    [one] { $count } tracker removed
   *[other] { $count } trackers removed
}
remote-banner-trackers-limited = At least { $count } trackers removed
remote-banner-show = Show images
remote-banner-show-help = Load this message's images through the server, so the sender does not see your IP address
remote-banner-more = More image options
remote-banner-always-sender = Always show images from { $sender }
remote-banner-always-domain = Always show images from { $domain }
remote-banner-always-all = Always show images in all mail
remote-banner-settings = Privacy settings
remote-banner-undo = Undo
remote-banner-undo-help-single = Block this message's images again
remote-banner-undo-help-per-sender = Stop showing images from { $sender } automatically
remote-banner-undo-help-per-domain = Stop showing images from { $domain } automatically
remote-banner-undo-help-all = Stop showing images in all mail automatically

# Tooltip text: the explanation that used to sit in the banner.
remote-banner-blocked-help = Images stored on other servers can tell the sender when and where you opened this message. They stay blocked until you choose to show them.
remote-banner-help-block = Your privacy settings block all remote images. Change this in Privacy settings.
remote-banner-help-ask = Images load through this server, which hides your IP address from the sender.
remote-banner-help-proxy = Images load through this server, which hides your IP address from the sender.
remote-banner-help-direct = Images load straight from the sender's servers, which can see your IP address.
remote-banner-hosts = Blocked hosts: { $hosts }
remote-banner-trackers-help = Tracking images are removed before the message is shown. Images the server recognises as trackers only after downloading them are counted for a limited time.

# -- Loading-images toast (t55) ---------------------------------------------------
remote-images-toast-loading = Loading images…
remote-images-toast-done = Images loaded
remote-images-toast-failed = { $count ->
    [one] { $count } image couldn't load
   *[other] { $count } images couldn't load
}
remote-images-toast-retry = Retry
