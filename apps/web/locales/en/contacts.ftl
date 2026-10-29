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
contacts-favorites = Favorites
contacts-groups = Groups
contacts-new-group = New group
contacts-new-group-name = New group name
contacts-group-name-placeholder = Group name
contacts-create = Create
contacts-data = Data
contacts-import = Import…

# -- Business card -----------------------------------------------------------
contacts-card = Contact { $name }
contacts-favorited = Favorited
contacts-favorite-action = Favorite
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

contacts-editor-person = Name and work
contacts-editor-reach = Email and phone
contacts-editor-other = Notes and preferences
contacts-editor-save-error = The contact could not be saved. Your changes are still here. Try again.

contacts-group-editor-help = Create a group in an address book, then add members from their contact cards.
contacts-group-details = Group details
contacts-group-save-error = The group could not be saved. Your changes are still here. Try again.

contacts-edit-group = Edit contact group
contacts-edit-named-group = Edit group { $name }
contacts-group-name = Group name
contacts-group-edit-help = Update this group's name. Its address book and membership remain unchanged.

# -- Management pages (t39 R6) -------------------------------------------------
contacts-back-to-list = Back to contacts
contacts-membership-title = Group membership
contacts-membership-help = Pick a group, then add or remove its members. Changes are saved as you make them.
contacts-membership-no-groups = No groups yet. Create one to start adding members.
contacts-membership-pick = Select a group to see its members.
contacts-membership-of = Members of { $name }
contacts-membership-members = Members
contacts-membership-add = Type a name or address to add a member
contacts-membership-add-group = Type a group name to add this contact
contacts-membership-count = { $count ->
    [one] 1 member
   *[other] { $count } members
}
contacts-membership-failed = Some membership changes could not be saved. The list shows what is stored.
contacts-membership-manage = Manage group membership
contacts-import-source = Source
contacts-import-book = Address book
contacts-import-review = Review
contacts-import-empty = Choose a file or paste vCard or CSV text, then select Preview. The contacts found are listed here, and you can leave any of them out.
contacts-merge-sets = Possible duplicates
contacts-merge-set = Duplicates of { $name }
contacts-merge-keep = Card to keep
contacts-merge-done = Merged into { $name }.
