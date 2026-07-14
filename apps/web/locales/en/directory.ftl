# Mailwoman — directory / GAL module strings (source locale: en, SPEC §13).
#
# Lazily loaded by the directory module components (DirectorySearch,
# GroupExpand, ContactSecurity). Ids are kebab-case, prefixed `directory-`.
# Untrusted values (display names, emails, group names) are bidi-isolated at
# the call site via i18n `isolate()` before interpolation (SPEC §24).

# -- Directory / GAL search --------------------------------------------------
directory-matches-label = Directory matches
directory-search-error = directory lookup failed
directory-group-badge = Group
directory-load-more = Load more

# -- Distribution-group expand-before-send -----------------------------------
directory-members-of = Members of { $name }
directory-is-distribution-group = is a distribution group.
directory-who-in-this = Who is actually in this?
directory-recipient-count = { $count ->
    [one] { $count } recipient
   *[other] { $count } recipients
}
directory-replace-with = Replace group with { $count ->
    [one] { $count } recipient
   *[other] { $count } recipients
}
directory-expand-error = could not expand the group

# -- Per-contact security tab ------------------------------------------------
directory-contact-security-label = Contact security
directory-photo-alt = { $email } directory photo
directory-published-material = Directory-published security material
directory-smime-certificates = S/MIME certificates
directory-no-cert = No certificate published in the directory for this contact.
directory-valid-until = Valid until { $date }
directory-no-expiry = No expiry advertised
directory-cert-expired = Expired
directory-cert-current = Current
directory-looking-up = Looking up directory…
directory-lookup-failed = Directory lookup failed.
