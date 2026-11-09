# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: contacts. Source: ../en/contacts.ftl

contacts-title = Kontakte
contacts-search = Kontakte durchsuchen
contacts-new = Neuer Kontakt
contacts-find-duplicates = Duplikate suchen
contacts-loading = Wird geladen…
contacts-none = Keine Kontakte.
contacts-list = Kontaktliste
contacts-favorite = { $name } als Favorit
contacts-export-vcard = vCard exportieren
contacts-export-csv = CSV exportieren
contacts-select-hint = Wählen Sie einen Kontakt aus, um seine Karte zu sehen.

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = Adressbücher und Gruppen
contacts-address-books = Adressbücher
contacts-all = Alle Kontakte
contacts-favorites = Favoriten
contacts-groups = Gruppen
contacts-new-group = Neue Gruppe
contacts-new-group-name = Name der neuen Gruppe
contacts-group-name-placeholder = Gruppenname
contacts-create = Erstellen
contacts-data = Daten
contacts-import = Importieren…

# -- Business card -----------------------------------------------------------
contacts-card = Kontakt { $name }
contacts-favorited = Favorit
contacts-favorite-action = Favorit
contacts-email = E-Mail
contacts-phone = Telefon
contacts-dates = Daten
contacts-groups-label = Gruppen
contacts-notes = Notizen
contacts-security-key = Sicherheitsschlüssel
contacts-key-on-file = Ein Schlüssel/Zertifikat ist hinterlegt (nur Anzeige, bis PGP/S/MIME verfügbar ist).
contacts-no-key = Kein Schlüssel hinterlegt.
contacts-directory-security = Verzeichnissicherheit
contacts-member-of = Mitglied von
contacts-group-membership = Mitgliedschaft in { $name }

# -- Editor ------------------------------------------------------------------
contacts-new-contact = Neuer Kontakt
contacts-edit-contact = Kontakt bearbeiten
contacts-full-name = Vollständiger Name
contacts-given = Vorname
contacts-given-placeholder = Vorname
contacts-surname = Nachname
contacts-surname-placeholder = Nachname
contacts-organization = Organisation
contacts-job-title = Position
contacts-job-title-placeholder = Position
contacts-email-n = E-Mail { $n }
contacts-email-label = Bezeichnung für E-Mail { $n }
contacts-email-label-placeholder = Arbeit
contacts-remove-email = E-Mail { $n } entfernen
contacts-add-email = E-Mail hinzufügen
contacts-phone-n = Telefon { $n }
contacts-remove-phone = Telefon { $n } entfernen
contacts-add-phone = Telefon hinzufügen
contacts-security-key-label = Sicherheitsschlüssel (undurchsichtiger Platzhalter)
contacts-security-key-placeholder = PGP-Schlüssel / S/MIME-Zertifikat (nur Anzeige)
contacts-favorite-word = Favorit

# -- Import dialog -----------------------------------------------------------
contacts-import-title = Kontakte importieren
contacts-imported = { $count ->
    [one] { $count } Kontakt importiert
   *[other] { $count } Kontakte importiert
}.
contacts-import-hint = Fügen Sie vCard (.vcf) oder CSV ein oder wählen Sie eine Datei. Das Format wird automatisch erkannt.
contacts-import-file = Datei importieren
contacts-paste = vCard oder CSV einfügen
contacts-preview = Vorschau
contacts-map-columns = Spalten zuordnen
contacts-col = Spalte
contacts-maps-to = Zugeordnet zu
contacts-sample = Beispiel
contacts-map-column = Spalte { $header } zuordnen
contacts-preview-count = Vorschau ({ $count })
contacts-import-preview = Importvorschau
contacts-no-name = (kein Name)
contacts-no-email = keine E-Mail
contacts-import-n = { $count ->
    [one] { $count } Kontakt importieren
   *[other] { $count } Kontakte importieren
}

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = Duplikate zusammenführen
contacts-merge-review-hint = Prüfen Sie die zusammengeführte Karte. Die anderen Karten werden als gelöscht markiert (umkehrbar).
contacts-merged-preview = Vorschau der Zusammenführung
contacts-emails = E-Mails
contacts-phones = Telefonnummern
contacts-merge-confirm = Kontakte zusammenführen
contacts-no-duplicates = Keine Duplikate gefunden.
contacts-possible-duplicates = { $count } mögliche Duplikate
contacts-review-merge = Zusammenführung prüfen

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = Kontaktdaten
contacts-address-recipients = Empfänger anzeigen
contacts-address-save = Kontakt speichern
contacts-address-compose = E-Mail senden
contacts-address-unsaved = Diese Adresse ist nicht in Ihren Kontakten gespeichert.
contacts-address-book = Adressbuch
contacts-address-lookup-failed = Kontaktinformationen konnten nicht geladen werden. Versuchen Sie es erneut.
contacts-address-save-failed = Der Kontakt konnte nicht gespeichert werden. Versuchen Sie es erneut.
contacts-address-saved = Kontakt gespeichert.
contacts-address-loading = Kontakt wird geladen …
contacts-editor-person = Name und Beruf
contacts-editor-reach = E-Mail und Telefon
contacts-editor-other = Notizen und Einstellungen
contacts-editor-save-error = Der Kontakt konnte nicht gespeichert werden. Ihre Änderungen sind noch vorhanden. Versuchen Sie es erneut.
contacts-group-editor-help = Erstellen Sie eine Gruppe in einem Adressbuch und fügen Sie dann über die Kontaktkarten Mitglieder hinzu.
contacts-group-details = Gruppendetails
contacts-group-save-error = Die Gruppe konnte nicht gespeichert werden. Ihre Änderungen sind noch vorhanden. Versuchen Sie es erneut.
contacts-edit-group = Kontaktgruppe bearbeiten
contacts-edit-named-group = Gruppe { $name } bearbeiten
contacts-group-name = Gruppenname
contacts-group-edit-help = Den Namen dieser Gruppe ändern. Adressbuch und Mitglieder bleiben unverändert.
contacts-back-to-list = Zurück zu den Kontakten
contacts-membership-title = Gruppenmitgliedschaft
contacts-membership-help = Wählen Sie eine Gruppe und fügen Sie Mitglieder hinzu oder entfernen Sie sie. Änderungen werden sofort gespeichert.
contacts-membership-no-groups = Noch keine Gruppen. Erstellen Sie eine, um Mitglieder hinzuzufügen.
contacts-membership-pick = Wählen Sie eine Gruppe aus, um ihre Mitglieder zu sehen.
contacts-membership-of = Mitglieder von { $name }
contacts-membership-members = Mitglieder
contacts-membership-add = Namen oder Adresse eingeben, um ein Mitglied hinzuzufügen
contacts-membership-add-group = Gruppennamen eingeben, um diesen Kontakt hinzuzufügen
contacts-membership-count = { $count ->
    [one] 1 Mitglied
   *[other] { $count } Mitglieder
}
contacts-membership-failed = Einige Änderungen an der Mitgliedschaft konnten nicht gespeichert werden. Die Liste zeigt den gespeicherten Stand.
contacts-membership-manage = Gruppenmitgliedschaft verwalten
contacts-import-source = Quelle
contacts-import-book = Adressbuch
contacts-import-review = Prüfen
contacts-import-empty = Wählen Sie eine Datei oder fügen Sie vCard- oder CSV-Text ein und wählen Sie dann „Vorschau“. Die gefundenen Kontakte werden hier aufgelistet; einzelne können Sie weglassen.
contacts-merge-sets = Mögliche Duplikate
contacts-merge-set = Duplikate von { $name }
contacts-merge-keep = Beizubehaltende Karte
contacts-merge-done = Mit { $name } zusammengeführt.
contacts-address-actions = Kontaktaktionen
contacts-address-save-to = In Kontakten speichern
contacts-address-goto = Zum Kontakt
contacts-address-copy = Kontaktinformationen kopieren
contacts-address-copied = Kontaktinformationen kopiert.
contacts-address-copy-failed = Die Kontaktinformationen konnten nicht kopiert werden.
contacts-share = Kontakt teilen
contacts-share-email = Kontakt per E-Mail senden
contacts-share-copy = Als vCard kopieren
contacts-share-download = .vcf herunterladen
contacts-share-native = Teilen …
contacts-share-copied = vCard kopiert.
contacts-share-copy-failed = Die vCard konnte nicht kopiert werden.
contacts-share-downloaded = vCard heruntergeladen.
contacts-share-failed = Der Kontakt konnte nicht geteilt werden.
contacts-share-attach-failed = Die .vcf-Datei konnte nicht angehängt werden, daher steht die vCard im Nachrichtentext.
contacts-address-saved-favorite = Kontakt gespeichert und zu den Favoriten hinzugefügt.
contacts-favorite-add = Zu Favoriten hinzufügen
contacts-favorite-remove = Aus Favoriten entfernen
contacts-favorite-save = Speichern und zu Favoriten hinzufügen
contacts-favorite-added = { $name } zu den Favoriten hinzugefügt.
contacts-favorite-removed = { $name } aus den Favoriten entfernt.
contacts-favorite-failed = Die Favoriten konnten nicht aktualisiert werden.
contacts-select-row = { $name } auswählen
contacts-selected-count = { $count } ausgewählt
contacts-select-clear = Auswahl aufheben
contacts-bulk-actions = Aktionen für die ausgewählten Kontakte
contacts-bulk-favorites-added = { $count } zu den Favoriten hinzugefügt.
contacts-bulk-favorites-removed = { $count } aus den Favoriten entfernt.
contacts-row-menu = Aktionen für { $name }
contacts-favorites-empty = Noch keine Favoriten. Mit dem Stern bei einem Kontakt fügen Sie einen hinzu.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
contacts-save-already = { $name } befindet sich bereits in Ihren Kontakten.

contacts-save-already-badge = Bereits gespeichert

contacts-save-already-many = Alle { $count } Kontakte sind bereits gespeichert.

contacts-save-automated = Dies sieht aus wie ein automatisierter Absender (keine Antwort, Benachrichtigungen).

contacts-save-book = Adressbuch

contacts-save-book-readonly = { $name } (schreibgeschützt)

contacts-save-cancel = Abbrechen

contacts-save-choose = Kontakte auswählen…

contacts-save-contacts = Kontakte aus dieser Nachricht speichern

contacts-save-dialog-title = Kontakte aus dieser Nachricht speichern

contacts-save-email = E-Mail

contacts-save-failed = Der Kontakt konnte nicht gespeichert werden. Versuchen Sie es erneut.

contacts-save-group = Gruppe

contacts-save-group-none = Keine Gruppe

contacts-save-loading = Adressen werden geladen…

contacts-save-name = Name

contacts-save-new = Neue Kontakte speichern

contacts-save-no-book = Kein Adressbuch kann neue Kontakte aufnehmen.

contacts-save-no-sender = Diese Nachricht hat keine Absenderadresse.

contacts-save-note = Gespeichert aus einer Nachricht am { $date }

contacts-save-nothing = Niemand zum Speichern: Jeder ist bereits in Ihren Kontakten, Sie selbst, oder sieht automatisiert aus.

contacts-save-open = Offen

contacts-save-open-contact = Kontakt öffnen

contacts-save-open-link = Kontakt öffnen

contacts-save-own = Dies ist eine Ihrer eigenen Adressen.

contacts-save-partial = { $saved } von { $total } Kontakten gespeichert. Die anderen konnten nicht gerettet werden.

contacts-save-pick = Sparen Sie { $email }

contacts-save-recipients = Speichern Sie alle Empfänger

contacts-save-removed = Gespeicherte Kontakte entfernt.

contacts-save-role-bcc = Bcc

contacts-save-role-cc = CC

contacts-save-role-from = Von

contacts-save-role-replyTo = Antwort an

contacts-save-role-sender = Absender

contacts-save-role-to = Zu

contacts-save-saved-heading = Bereits in Ihren Kontakten

contacts-save-saved-many = Gespeicherte { $count } Kontakte zu { $book }

contacts-save-saved-one = { $name } bis { $book } gespeichert

contacts-save-sender = Absender als Kontakt speichern

contacts-save-settings-book = Adressbuch

contacts-save-settings-book-default = Standardadressbuch

contacts-save-settings-enabled = Angebot Kontakte aus dieser Nachricht speichern

contacts-save-settings-enabled-help = Fügt das Element den Nachrichtenmenüs und dem Menü „Mehr“ des Readers hinzu.

contacts-save-settings-note = Fügen Sie eine Notiz mit dem Datum hinzu

contacts-save-settings-note-help = Die Notiz lautet „Gespeichert ab einer Nachricht am“, gefolgt vom Datum.

contacts-save-settings-reply-to = Verwenden Sie „Antwort an“ für Listen- und Benachrichtigungsabsender

contacts-save-settings-reply-to-help = Speichert für diese Absender die Antwortadresse anstelle der Absenderadresse.

contacts-save-settings-scope = Im Dialog zuerst angekreuzt

contacts-save-settings-scope-all = Jeder auf der Nachricht

contacts-save-settings-scope-sender = Nur Absender

contacts-save-settings-sender = Angebot Absender als Kontakt speichern

contacts-save-settings-sender-help = Ein Ein-Klick-Element in Nachrichtenmenüs.

contacts-save-settings-skip = Überspringen Sie automatisierte Absender

contacts-save-settings-skip-help = Lässt No-Reply-, Benachrichtigungs- und Bounce-Adressen aus, sofern Sie diese nicht ankreuzen.

contacts-save-settings-title = Kontakte aus Nachrichten speichern

contacts-save-show-automated = Automatische und No-Reply-Adressen anzeigen

contacts-save-submit = { $count ->
        [one] { $count } Kontakt speichern
       *[other] Speichern Sie { $count } Kontakte
    }

contacts-save-undo = Rückgängig machen
