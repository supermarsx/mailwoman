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

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [one] { $count } remote image blocked
   *[other] { $count } remote images blocked
}
remote-blocked-trackers = { $count ->
    [one] { $count } tracker blocked
   *[other] { $count } trackers blocked
} of { $blocked ->
    [one] { $blocked } remote image
   *[other] { $blocked } remote images
}
remote-blocked-hosts = Blocked hosts

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = Load images
remote-grant-sender = Always load from { $sender }
remote-grant-domain = Always load from { $domain }
remote-grant-all = Always load all remote images

remote-grant-done-single = Remote images loaded for this message.
remote-grant-done-per-sender = Remote images will load from this sender.
remote-grant-done-per-domain = Remote images will load from this domain.
remote-grant-done-all = Remote images will load for all mail.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = Remote images are loading for this message.
remote-revoke = Turn off
remote-revoke-done = Remote images turned off. They block again on next open.
