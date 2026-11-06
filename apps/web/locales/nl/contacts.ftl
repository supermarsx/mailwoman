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
