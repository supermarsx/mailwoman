# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Contacten
contacts-search = Contacten zoeken
contacts-new = Nieuw contact
contacts-find-duplicates = Dubbele zoeken
contacts-loading = Laden…
contacts-none = Geen contacten.
contacts-list = Contactenlijst
contacts-favorite = { $name } als favoriet markeren
contacts-export-vcard = vCard exporteren
contacts-export-csv = CSV exporteren
contacts-select-hint = Selecteer een contact om de kaart te bekijken.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Adresboeken en groepen
contacts-address-books = Adresboeken
contacts-all = Alle contacten
contacts-favorites = Favorieten
contacts-groups = Groepen
contacts-new-group = Nieuwe groep
contacts-new-group-name = Naam nieuwe groep
contacts-group-name-placeholder = Groepsnaam
contacts-create = Aanmaken
contacts-data = Gegevens
contacts-import = Importeren…

# -- Business card -----------------------------------------------------------
contacts-card = Contact { $name }
contacts-favorited = Favoriet
contacts-favorite-action = Favoriet
contacts-email = E-mail
contacts-phone = Telefoon
contacts-dates = Datums
contacts-groups-label = Groepen
contacts-notes = Notities
contacts-security-key = Beveiligingssleutel
contacts-key-on-file = Er is een sleutel/certificaat opgeslagen (alleen weergave totdat PGP/S-MIME beschikbaar is).
contacts-no-key = Geen sleutel opgeslagen.
contacts-directory-security = Beveiliging uit de directory
contacts-member-of = Lid van
contacts-group-membership = Lidmaatschap van { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Nieuw contact
contacts-edit-contact = Contact bewerken
contacts-full-name = Volledige naam
contacts-given = Voornaam
contacts-given-placeholder = Voornaam
contacts-surname = Achternaam
contacts-surname-placeholder = Achternaam
contacts-organization = Organisatie
contacts-job-title = Functie
contacts-job-title-placeholder = Functie
contacts-email-n = E-mail { $n }
contacts-email-label = Label voor e-mail { $n }
contacts-email-label-placeholder = werk
contacts-remove-email = E-mail { $n } verwijderen
contacts-add-email = E-mail toevoegen
contacts-phone-n = Telefoon { $n }
contacts-remove-phone = Telefoon { $n } verwijderen
contacts-add-phone = Telefoon toevoegen
contacts-security-key-label = Beveiligingssleutel (ondoorzichtige placeholder)
contacts-security-key-placeholder = PGP-sleutel / S-MIME-certificaat (alleen weergave)
contacts-favorite-word = Favoriet

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Contacten importeren
contacts-imported = { $count ->
    [one] { $count } contact geïmporteerd
   *[other] { $count } contacten geïmporteerd
}.
contacts-import-hint = Plak een vCard (.vcf) of CSV, of kies een bestand. Het formaat wordt automatisch herkend.
contacts-import-file = Bestand importeren
contacts-paste = vCard of CSV plakken
contacts-preview = Voorbeeld
contacts-map-columns = Kolommen koppelen
contacts-col = Kolom
contacts-maps-to = Koppelen aan
contacts-sample = Voorbeeldwaarde
contacts-map-column = Kolom { $header } koppelen
contacts-preview-count = Voorbeeld ({ $count })
contacts-import-preview = Importvoorbeeld
contacts-no-name = (geen naam)
contacts-no-email = geen e-mail
contacts-import-n = { $count ->
    [one] { $count } contact importeren
   *[other] { $count } contacten importeren
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Dubbele samenvoegen
contacts-merge-review-hint = Controleer de samengevoegde kaart. De andere kaarten worden als verwijderd gemarkeerd (omkeerbaar).
contacts-merged-preview = Voorbeeld na samenvoegen
contacts-emails = E-mailadressen
contacts-phones = Telefoonnummers
contacts-merge-confirm = Contacten samenvoegen
contacts-no-duplicates = Geen dubbele gevonden.
contacts-possible-duplicates = { $count } mogelijke dubbele
contacts-review-merge = Samenvoegen controleren

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Contactgegevens
contacts-address-recipients = Ontvangers tonen
contacts-address-save = Contact opslaan
contacts-address-compose = E-mail sturen
contacts-address-unsaved = Dit adres is niet opgeslagen in uw contacten.
contacts-address-book = Adresboek
contacts-address-lookup-failed = De contactgegevens konden niet worden geladen. Probeer het opnieuw.
contacts-address-save-failed = Het contact kon niet worden opgeslagen. Probeer het opnieuw.
contacts-address-saved = Contact opgeslagen.
contacts-address-loading = Contact laden…
contacts-editor-person = Naam en werk
contacts-editor-reach = E-mail en telefoon
contacts-editor-other = Notities en voorkeuren
contacts-editor-save-error = Het contact kon niet worden opgeslagen. Uw wijzigingen zijn nog aanwezig. Probeer het opnieuw.
contacts-group-editor-help = Maak een groep in een adresboek en voeg daarna leden toe vanaf hun contactkaarten.
contacts-group-details = Groepsgegevens
contacts-group-save-error = De groep kon niet worden opgeslagen. Uw wijzigingen zijn nog aanwezig. Probeer het opnieuw.
contacts-edit-group = Contactgroep bewerken
contacts-edit-named-group = Groep { $name } bewerken
contacts-group-name = Groepsnaam
contacts-group-edit-help = Werk de naam van deze groep bij. Het adresboek en de leden blijven ongewijzigd.
contacts-back-to-list = Terug naar contacten
contacts-membership-title = Groepslidmaatschap
contacts-membership-help = Kies een groep en voeg leden toe of verwijder ze. Wijzigingen worden direct opgeslagen.
contacts-membership-no-groups = Nog geen groepen. Maak er een aan om leden toe te voegen.
contacts-membership-pick = Selecteer een groep om de leden te zien.
contacts-membership-of = Leden van { $name }
contacts-membership-members = Leden
contacts-membership-add = Typ een naam of adres om een lid toe te voegen
contacts-membership-add-group = Typ een groepsnaam om dit contact toe te voegen
contacts-membership-count = { $count ->
    [one] 1 lid
   *[other] { $count } leden
}
contacts-membership-failed = Sommige lidmaatschapswijzigingen konden niet worden opgeslagen. De lijst toont wat is opgeslagen.
contacts-membership-manage = Groepslidmaatschap beheren
contacts-import-source = Bron
contacts-import-book = Adresboek
contacts-import-review = Controleren
contacts-import-empty = Kies een bestand of plak vCard- of CSV-tekst en selecteer daarna Voorbeeld. De gevonden contacten worden hier getoond en u kunt er elk van weglaten.
contacts-merge-sets = Mogelijke dubbelen
contacts-merge-set = Dubbelen van { $name }
contacts-merge-keep = Te behouden kaart
contacts-merge-done = Samengevoegd met { $name }.
contacts-address-actions = Contactacties
contacts-address-save-to = Opslaan in contacten
contacts-address-goto = Naar contact gaan
contacts-address-copy = Contactgegevens kopiëren
contacts-address-copied = Contactgegevens gekopieerd.
contacts-address-copy-failed = De contactgegevens konden niet worden gekopieerd.
contacts-share = Contact delen
contacts-share-email = Contact per e-mail versturen
contacts-share-copy = Kopiëren als vCard
contacts-share-download = .vcf downloaden
contacts-share-native = Delen…
contacts-share-copied = vCard gekopieerd.
contacts-share-copy-failed = De vCard kon niet worden gekopieerd.
contacts-share-downloaded = vCard gedownload.
contacts-share-failed = Het contact kon niet worden gedeeld.
contacts-share-attach-failed = Het .vcf-bestand kon niet worden bijgevoegd, dus de vCard staat in de berichttekst.
contacts-address-saved-favorite = Contact opgeslagen en aan favorieten toegevoegd.
contacts-favorite-add = Aan favorieten toevoegen
contacts-favorite-remove = Uit favorieten verwijderen
contacts-favorite-save = Opslaan en aan favorieten toevoegen
contacts-favorite-added = { $name } aan favorieten toegevoegd.
contacts-favorite-removed = { $name } uit favorieten verwijderd.
contacts-favorite-failed = Favorieten konden niet worden bijgewerkt.
contacts-select-row = { $name } selecteren
contacts-selected-count = { $count } geselecteerd
contacts-select-clear = Selectie wissen
contacts-bulk-actions = Acties voor de geselecteerde contacten
contacts-bulk-favorites-added = { $count } aan favorieten toegevoegd.
contacts-bulk-favorites-removed = { $count } uit favorieten verwijderd.
contacts-row-menu = Acties voor { $name }
contacts-favorites-empty = Nog geen favorieten. Gebruik de ster bij een contact om er een toe te voegen.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
contacts-save-already = { $name } staat al in je contacten.

contacts-save-already-badge = Al opgeslagen

contacts-save-already-many = Alle { $count } contacten zijn al opgeslagen.

contacts-save-automated = Dit lijkt op een geautomatiseerde afzender (geen antwoord, meldingen).

contacts-save-book = Adresboek

contacts-save-book-readonly = { $name } (alleen-lezen)

contacts-save-cancel = Annuleer

contacts-save-choose = Contacten kiezen…

contacts-save-contacts = Bewaar contacten uit dit bericht

contacts-save-dialog-title = Bewaar contacten uit dit bericht

contacts-save-email = E-mail

contacts-save-failed = Het contact kon niet worden opgeslagen. Probeer het opnieuw.

contacts-save-group = Groep

contacts-save-group-none = Geen groep

contacts-save-loading = Adressen laden…

contacts-save-name = Naam

contacts-save-new = Bewaar nieuwe contacten

contacts-save-no-book = Geen enkel adresboek kan nieuwe contacten opnemen.

contacts-save-no-sender = Dit bericht heeft geen afzenderadres.

contacts-save-note = Opgeslagen uit een bericht op { $date }

contacts-save-nothing = Niemand om op te slaan: iedereen staat al in je contacten, jij ook, of ziet er geautomatiseerd uit.

contacts-save-open = Openen

contacts-save-open-contact = Contactpersoon openen

contacts-save-open-link = Contactpersoon openen

contacts-save-own = Dit is een van uw eigen adressen.

contacts-save-partial = { $saved } van { $total } contacten opgeslagen. De anderen konden niet worden gered.

contacts-save-pick = Bespaar { $email }

contacts-save-recipients = Bewaar alle ontvangers

contacts-save-removed = Opgeslagen contacten verwijderd.

contacts-save-role-bcc = Bcc

contacts-save-role-cc = CC

contacts-save-role-from = Van

contacts-save-role-replyTo = Antwoord-aan

contacts-save-role-sender = Afzender

contacts-save-role-to = Aan

contacts-save-saved-heading = Staat al in uw contacten

contacts-save-saved-many = { $count } contacten opgeslagen in { $book }

contacts-save-saved-one = Opgeslagen { $name } tot { $book }

contacts-save-sender = Afzender opslaan als contactpersoon

contacts-save-settings-book = Adresboek

contacts-save-settings-book-default = Standaard adresboek

contacts-save-settings-enabled = Aanbieding Bewaar contacten uit dit bericht

contacts-save-settings-enabled-help = Voegt het item toe aan berichtmenu's en het menu Meer van de lezer.

contacts-save-settings-note = Voeg een notitie toe met de datum

contacts-save-settings-note-help = Op het briefje staat 'Opgeslagen uit een bericht op', gevolgd door de datum.

contacts-save-settings-reply-to = Gebruik Reply-To voor afzenders van lijsten en meldingen

contacts-save-settings-reply-to-help = Slaat het antwoordadres op in plaats van Van voor die afzenders.

contacts-save-settings-scope = Als eerste aangevinkt in het dialoogvenster

contacts-save-settings-scope-all = Iedereen op het bericht

contacts-save-settings-scope-sender = Alleen afzender

contacts-save-settings-sender = Aanbieding Bewaar afzender als contactpersoon

contacts-save-settings-sender-help = Een item met één klik in berichtmenu's.

contacts-save-settings-skip = Sla geautomatiseerde afzenders over

contacts-save-settings-skip-help = Laat no-reply-, notificatie- en bounce-adressen achterwege, tenzij u ze aanvinkt.

contacts-save-settings-title = Bewaar contacten uit berichten

contacts-save-show-automated = Toon geautomatiseerde adressen en adressen zonder antwoord

contacts-save-submit = { $count ->
        [one] Bewaar { $count } contactpersoon
       *[other] Bewaar { $count } contacten
    }

contacts-save-undo = Ongedaan maken
