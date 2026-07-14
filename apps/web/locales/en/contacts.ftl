# Mailwoman — contacts module strings (source locale: en).
# Lazily loaded by the contacts area (loadCatalog('contacts')). Untrusted contact
# names / emails are <bdi>-wrapped for display; raw name interpolation into
# accessible names is intentional (isolation marks would break exact SR labels).

contacts-title = Contacts
contacts-search = Search contacts
contacts-new = New contact
contacts-find-duplicates = Find duplicates
contacts-loading = Loading…
contacts-none = No contacts.
contacts-list = Contact list
contacts-favorite = Favorite { $name }
contacts-export-vcard = Export vCard
contacts-export-csv = Export CSV
contacts-select-hint = Select a contact to see their card.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Address books and groups
contacts-address-books = Address books
contacts-all = All contacts
contacts-favorites = ★ Favorites
contacts-groups = Groups
contacts-new-group = New group
contacts-new-group-name = New group name
contacts-group-name-placeholder = Group name
contacts-create = Create
contacts-data = Data
contacts-import = Import…

# -- Business card -----------------------------------------------------------
contacts-card = Contact { $name }
contacts-favorited = ★ Favorited
contacts-favorite-action = ☆ Favorite
contacts-email = Email
contacts-phone = Phone
contacts-dates = Dates
contacts-groups-label = Groups
contacts-notes = Notes
contacts-security-key = Security key
contacts-key-on-file = A key/certificate is on file (display-only until PGP/S-MIME lands).
contacts-no-key = No key on file.
contacts-directory-security = Directory security
contacts-member-of = Member of
contacts-group-membership = { $name } membership

# -- Editor ------------------------------------------------------------------
contacts-new-contact = New contact
contacts-edit-contact = Edit contact
contacts-full-name = Full name
contacts-given = Given name
contacts-given-placeholder = Given
contacts-surname = Surname
contacts-surname-placeholder = Surname
contacts-organization = Organization
contacts-job-title = Job title
contacts-job-title-placeholder = Title
contacts-email-n = Email { $n }
contacts-email-label = Email { $n } label
contacts-email-label-placeholder = work
contacts-remove-email = Remove email { $n }
contacts-add-email = Add email
contacts-phone-n = Phone { $n }
contacts-remove-phone = Remove phone { $n }
contacts-add-phone = Add phone
contacts-security-key-label = Security key (opaque placeholder)
contacts-security-key-placeholder = PGP key / S-MIME cert (display-only)
contacts-favorite-word = Favorite

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Import contacts
contacts-imported = Imported { $count ->
    [one] { $count } contact
   *[other] { $count } contacts
}.
contacts-import-hint = Paste vCard (.vcf) or CSV, or choose a file. The format is detected automatically.
contacts-import-file = Import file
contacts-paste = Paste vCard or CSV
contacts-preview = Preview
contacts-map-columns = Map columns
contacts-col = Column
contacts-maps-to = Maps to
contacts-sample = Sample
contacts-map-column = Map column { $header }
contacts-preview-count = Preview ({ $count })
contacts-import-preview = Import preview
contacts-no-name = (no name)
contacts-no-email = no email
contacts-import-n = Import { $count ->
    [one] { $count } contact
   *[other] { $count } contacts
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Merge duplicates
contacts-merge-review-hint = Review the merged card. The other cards become tombstones (reversible).
contacts-merged-preview = Merged preview
contacts-emails = Emails
contacts-phones = Phones
contacts-merge-confirm = Merge contacts
contacts-no-duplicates = No duplicates found.
contacts-possible-duplicates = { $count } possible duplicates
contacts-review-merge = Review merge
