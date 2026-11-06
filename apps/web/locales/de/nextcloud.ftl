# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Nach oben
nextcloud-loading = Wird geladen…
nextcloud-list-error = Dieser Ordner konnte nicht aufgelistet werden.
nextcloud-empty = Dieser Ordner ist leer.
nextcloud-file-list = Nextcloud-Dateien
nextcloud-open-folder = Ordner { $name } öffnen
nextcloud-select-file = { $name } auswählen

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Aus Nextcloud anhängen
nextcloud-attach-action = { $count ->
    [0] Anhängen
    [one] { $count } Datei anhängen
   *[other] { $count } Dateien anhängen
  }
nextcloud-error-select-file = Wählen Sie mindestens eine Datei zum Anhängen aus
nextcloud-error-attach-failed = Die ausgewählten Dateien konnten nicht angehängt werden

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = In Nextcloud speichern
nextcloud-save-title = „{ $name }“ in Nextcloud speichern
nextcloud-save-action = Hier speichern ({ $dir })
nextcloud-saved = Gespeichert unter { $path }
nextcloud-error-save-failed = Speichern in Nextcloud fehlgeschlagen

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Freigabelink erstellen
nextcloud-share-title = Freigabelink
nextcloud-share-intro = Erstellen Sie einen öffentlichen Link zu { $path }, anstatt die Datei anzuhängen.
nextcloud-protect-password = Mit Passwort schützen
nextcloud-password-label = Passwort
nextcloud-share-password = Freigabepasswort
nextcloud-set-expiry = Ablaufdatum festlegen
nextcloud-expires-on = Läuft ab am
nextcloud-expiry-date = Ablaufdatum
nextcloud-create-link = Link erstellen
nextcloud-password-protected = Passwortgeschützt
nextcloud-no-password = Kein Passwort
nextcloud-expires = läuft ab am { $date }
nextcloud-no-expiry = kein Ablaufdatum
nextcloud-error-need-password = Geben Sie ein Passwort ein oder deaktivieren Sie den Passwortschutz
nextcloud-error-need-expiry = Wählen Sie ein Ablaufdatum oder deaktivieren Sie den Ablauf
nextcloud-error-share-failed = Der Freigabelink konnte nicht erstellt werden

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } überschreitet die Anhangsgrenze. Laden Sie die Datei in den Ordner „Mail attachments“ Ihrer Nextcloud hoch und fügen Sie stattdessen einen Freigabelink in die Nachricht ein.
nextcloud-upload-and-link = Hochladen und Link einfügen
nextcloud-share-not-now = Jetzt nicht
nextcloud-upload-progress = Wird hochgeladen: { $percent } %
nextcloud-upload-progress-label = Upload-Fortschritt
nextcloud-share-inserted = Freigabelink für { $name } wurde in die Nachricht eingefügt.
nextcloud-share-body = { $name } ({ $size }): { $url }
nextcloud-share-body-expires = Der Link ist bis { $date } gültig.
nextcloud-share-body-password = Der Link ist passwortgeschützt.
nextcloud-account-title = Nextcloud-Konten
nextcloud-account-help = Verknüpfen Sie Ihr eigenes Konto mit einer von Ihrer Administration freigegebenen Instanz. Die Zugangsdaten werden nur für dieses Postfachkonto verschlüsselt gespeichert.
nextcloud-account-load-error = Die Nextcloud-Verknüpfungen konnten nicht geladen werden. Laden Sie die Seite neu und versuchen Sie es erneut.
nextcloud-account-unlink-error = Die Verknüpfung konnte nicht entfernt werden. Laden Sie die Seite neu, bevor Sie es erneut versuchen.
nextcloud-account-reload = Neu laden
nextcloud-account-no-instances = Es sind keine aktivierten Nextcloud-Instanzen verfügbar. Bitten Sie Ihre Administration, eine einzurichten.
nextcloud-account-ready = Verknüpft und bereit
nextcloud-account-stale = Die Instanzkonfiguration hat sich geändert. Prüfen Sie die Verknüpfung erneut.
nextcloud-account-unlinked = Nicht verknüpft
nextcloud-account-relink = Verknüpfung erneut prüfen
nextcloud-account-link = Konto verknüpfen
nextcloud-account-unlink = Verknüpfung aufheben
nextcloud-account-password-help = Erstellen Sie in den persönlichen Sicherheitseinstellungen von Nextcloud ein App-Passwort und kopieren Sie Benutzernamen und Passwort hierher. Die Prüfung sendet genau eine begrenzte Anfrage; abgelehnte Zugangsdaten werden nicht erneut versucht.
nextcloud-account-username = Benutzername des Nextcloud-App-Passworts
nextcloud-account-password = Nextcloud-App-Passwort
nextcloud-account-verify = Prüfen und verknüpfen
nextcloud-account-verify-error = Die Prüfung oder das Speichern ist fehlgeschlagen. Kontrollieren Sie Instanz, App-Passwort-Benutzernamen und Passwort. Das Passwortfeld wurde geleert; eine bestehende funktionierende Verknüpfung wurde nicht ersetzt.
nextcloud-admin-title = Nextcloud-Instanzen
nextcloud-admin-help = Geben Sie HTTPS-Instanzen frei, die Postfachnutzer einzeln verknüpfen dürfen. Postfachkonten erhalten kein gemeinsames Dienstpasswort. Dateivorgänge sind auf 8 MiB begrenzt. Nutzer der nativen Engine suchen und verbinden Kalender-, Aufgaben- und Adressbuchsammlungen nach der Verknüpfung ausdrücklich selbst.
nextcloud-admin-add = Nextcloud-Instanz hinzufügen
nextcloud-admin-edit = Nextcloud-Instanz bearbeiten
nextcloud-admin-enabled = Aktiviert
nextcloud-admin-disabled = Deaktiviert
nextcloud-admin-missing = Diese Instanz existiert nicht mehr. Kehren Sie zur Instanzliste zurück.
nextcloud-admin-private-help = Private Adressen erfordern eine ausdrückliche Nextcloud-Egress-Berechtigung. Konfigurierte Proxy-Routen bleiben maßgeblich; eine abgelehnte Proxy-Anfrage fällt nie auf eine direkte Verbindung zurück.
nextcloud-admin-revalidation = Jede Änderung an einer Instanz erfordert, dass Nutzer ihre Verknüpfungen erneut prüfen. Das Deaktivieren einer Instanz stoppt neue Vorgänge sofort.
nextcloud-admin-authority = Instanzberechtigung
nextcloud-admin-name = Anzeigename
nextcloud-admin-name-help = Ein kurzer Name, der Nutzern bei der Auswahl einer Instanz angezeigt wird.
nextcloud-admin-url = HTTPS-Installations-URL
nextcloud-admin-url-help = Verwenden Sie das Installationsstammverzeichnis, etwa https://cloud.example.org/ oder https://example.org/nextcloud/. Das Zertifikat muss vertrauenswürdig sein und zum Hostnamen passen. Zugangsdaten, Query-Parameter und Fragmente werden abgelehnt.
nextcloud-admin-enable = Nutzern erlauben, diese Instanz zu verknüpfen
nextcloud-admin-ca = Zusätzliche Zertifizierungsstellen (PEM)
nextcloud-admin-ca-help = Optionale private CA-Zertifikate für diesen Nextcloud-Server. Das öffentliche Vertrauen bleibt bestehen, und die Hostnamenprüfung bleibt erforderlich. Änderungen erfordern, dass Nutzer ihre Verknüpfungen erneut prüfen; das Zertifikatsvertrauen des Proxys ist davon getrennt.
nextcloud-instance-select = Nextcloud-Instanz
nextcloud-instance-placeholder = Verknüpfte Instanz wählen
nextcloud-instances-empty = Keine geprüften Nextcloud-Verknüpfungen. Fügen Sie in den Benutzereinstellungen → Nextcloud eine Verknüpfung hinzu oder prüfen Sie sie.
nextcloud-instances-error = Die verknüpften Instanzen konnten nicht geladen werden. Öffnen Sie die Auswahl erneut, um es noch einmal zu versuchen.
nextcloud-save-warning = Ihre Administration verlangt eine Bestätigung, bevor dieser Anhang das Postfach verlassen darf.
nextcloud-save-acknowledge = Ich habe die Anhangswarnung verstanden und möchte diese Datei in der ausgewählten Nextcloud-Instanz speichern.
nextcloud-save-attachment = { $name } in Nextcloud speichern
nextcloud-save-close = Nextcloud-Ziel schließen
nextcloud-save-options = Anhänge in Nextcloud speichern
nextcloud-pim-title = Kontakte und Kalender
nextcloud-pim-help = Suchen Sie Sammlungen in diesem geprüften Nextcloud-Konto und verbinden Sie ausdrücklich die gewünschten. Synchronisieren Sie, um aktuelle Daten abzurufen; Änderungen an verbundenen Sammlungen erfolgen in der entfernten Sammlung und schlagen fehl, wenn die Verknüpfung oder die Berechtigungen widerrufen wurden.
nextcloud-pim-category = Sammlungskategorie
nextcloud-pim-calendar = Kalender
nextcloud-pim-contacts = Adressbücher
nextcloud-pim-tasks = Aufgabenlisten
nextcloud-pim-discover = Sammlungen suchen
nextcloud-pim-discovered = Verfügbare Sammlungen
nextcloud-pim-empty = Es wurden keine passenden Sammlungen zurückgegeben.
nextcloud-pim-connect = Verbinden
nextcloud-pim-connected = Verbundene Sammlungen
nextcloud-pim-sync = Jetzt synchronisieren
nextcloud-pim-sync-needed = Sammlung verbunden. Synchronisieren Sie jetzt, um ihren Inhalt zu laden.
nextcloud-pim-synced = Synchronisierung abgeschlossen.
nextcloud-pim-error = Der Vorgang ist fehlgeschlagen. Prüfen Sie Kontoverknüpfung und Berechtigungen und wiederholen Sie ihn dann ausdrücklich. Bereits zwischengespeicherte Daten wurden nicht verworfen.
nextcloud-pim-native-required = Die Synchronisierung von Kontakten und Kalendern erfordert die native Mail-Engine. Der Dateizugriff bleibt über Ihre Kontoverknüpfung verfügbar.
nextcloud-pim-disconnect = Trennen
nextcloud-pim-disconnect-help = Beim Trennen bleiben zwischengespeicherte Daten erhalten, und Schreibvorgänge zum Server werden gestoppt. Bei neu verknüpften Konten müssen Sammlungen erneut gesucht und ausdrücklich verbunden werden.
nextcloud-pim-ready = Bereit
nextcloud-pim-disconnected = Getrennt oder Verknüpfung geändert
nextcloud-pim-unnamed = Unbenannte Sammlung
nextcloud-pim-back = Zurück zu den Nextcloud-Konten
nextcloud-pim-create-title = Neue Sammlung in Nextcloud
nextcloud-pim-create-help = Erstellt einen leeren Kalender oder eine leere Aufgabenliste in diesem Nextcloud-Konto und verbindet sie hier.
nextcloud-pim-create-name = Name der neuen Sammlung
nextcloud-pim-create = In Nextcloud erstellen
nextcloud-pim-created = { $name } wurde in Nextcloud erstellt und verbunden. Synchronisieren Sie jetzt, um sie zu laden.
nextcloud-pim-interval = Intervall der automatischen Aktualisierung (Sekunden)
nextcloud-pim-interval-help = Null bedeutet nur manuelle Aktualisierung. Positive Intervalle haben ein Minimum von 60 Sekunden. Verbundene Konten werden im Hintergrund aktualisiert; bei Fehlern wird die Wartezeit verlängert, und bei abgelehnten Zugangsdaten werden weitere Anfragen bis zur erneuten Verknüpfung eingestellt.
nextcloud-pim-interval-reset = Standard der Administration verwenden
nextcloud-admin-empty = Noch keine freigegebenen Instanzen. Fügen Sie eine Instanz hinzu, damit sie zur Kontoverknüpfung verfügbar ist.
nextcloud-admin-status = Status
nextcloud-admin-open = { $name } öffnen
