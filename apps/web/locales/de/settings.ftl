# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: settings. Source: ../en/settings.ftl

settings-title = Einstellungen
settings-appearance = Darstellung
settings-close = Einstellungen schließen

settings-theme = Design
settings-density = Dichte
settings-accent = Akzentfarbe
settings-font = Schriftart der Oberfläche
settings-layout = Layout

settings-density-compact = Kompakt
settings-density-cozy = Normal
settings-density-relaxed = Großzügig

settings-font-default = Standard
settings-font-system = System
settings-font-serif = Serifen
settings-font-mono = Monospace

settings-layout-default = Standard
settings-layout-ribbon = Menüband

settings-cancel = Abbrechen
settings-edit = Bearbeiten
settings-delete = Löschen
settings-save = Speichern
settings-saved = Gespeichert

settings-2fa-title = Zwei-Faktor-Authentifizierung
settings-2fa-intro = Fügen Sie einen zweiten Faktor hinzu, damit bei der Anmeldung mehr als ein Passwort nötig ist.
settings-2fa-policy-required = Ihre Organisation verlangt für dieses Konto einen zweiten Faktor.
settings-2fa-enabled = Aktiviert
settings-2fa-recovery-title = Wiederherstellungscodes
settings-2fa-recovery-once = Speichern Sie diese jetzt. Sie werden nur einmal angezeigt und können nicht erneut abgerufen werden. Jeder Code funktioniert nur einmal.
settings-2fa-recovery-ack = Ich habe diese Codes gespeichert
settings-2fa-totp-title = Authenticator-App
settings-2fa-totp-enrol = Authenticator-App einrichten
settings-2fa-totp-scan = Fügen Sie dieses Geheimnis Ihrer Authenticator-App hinzu und geben Sie dann den angezeigten Code ein.
settings-2fa-totp-uri-link = In einer Authenticator-App öffnen
settings-2fa-totp-code-label = Authenticator-Code
settings-2fa-totp-confirm = Bestätigen
settings-2fa-totp-disable = Authenticator entfernen
settings-2fa-passkey-title = Passkeys
settings-2fa-passkey-remove = Entfernen
settings-2fa-passkey-unsupported = Dieser Browser unterstützt keine Passkeys.
settings-2fa-passkey-label-placeholder = Name des Passkeys (optional)
settings-2fa-passkey-add = Passkey hinzufügen
settings-2fa-recovery-remaining = Noch { $count } { $count ->
        [one] Wiederherstellungscode
       *[other] Wiederherstellungscodes
    } übrig
settings-2fa-recovery-regenerate = Neue Wiederherstellungscodes erzeugen
settings-2fa-recovery-code-label = Wiederherstellungscode
settings-2fa-error-generic = Das hat nicht funktioniert. Versuchen Sie es erneut.
settings-2fa-error-code = Dieser Code konnte nicht bestätigt werden.
settings-2fa-error-passkey = Der Passkey konnte nicht registriert werden.

settings-2fa-challenge-title = Zwei-Faktor-Authentifizierung
settings-2fa-challenge-intro = Bestätigen Sie Ihren zweiten Faktor, um die Anmeldung abzuschließen.
settings-2fa-challenge-failed = Das konnte nicht bestätigt werden. Versuchen Sie es erneut.
settings-2fa-challenge-use-passkey = Passkey verwenden
settings-2fa-challenge-method = Methode wählen
settings-2fa-challenge-totp-tab = Authenticator-Code
settings-2fa-challenge-recovery-tab = Wiederherstellungscode
settings-2fa-challenge-verify = Bestätigen

settings-sessions-title = Aktive Sitzungen
settings-sessions-intro = Sitzungen, die derzeit bei diesem Konto angemeldet sind.
settings-sessions-last-seen = Zuletzt aktiv { $when }
settings-sessions-revoke = Abmelden
settings-sessions-current = Diese Sitzung
settings-sessions-revoke-others = { $count } { $count ->
        [one] andere Sitzung
       *[other] andere Sitzungen
    } abmelden
settings-sessions-error = Die Sitzung konnte nicht geändert werden.

settings-sig-title = Signaturen
settings-sig-intro = Text, der an gesendete Nachrichten angehängt wird. Eine Signatur kann der Standard sein.
settings-sig-default = Standard
settings-sig-new = Neue Signatur
settings-sig-name-label = Name
settings-sig-body-label = Signatur
settings-sig-default-label = Als Standard verwenden
settings-sig-error-name = Geben Sie der Signatur einen Namen.
settings-sig-error-generic = Die Signatur konnte nicht gespeichert werden.

settings-ident-title = Identitäten
settings-ident-intro = Der Name und die Adresse, mit denen Sie senden. Jede Identität kann eine eigene Signatur verwenden.
settings-ident-new = Neue Identität
settings-ident-name-label = Anzeigename
settings-ident-email-label = E-Mail-Adresse
settings-ident-replyto-label = Antwortadresse (optional)
settings-ident-signature-label = Standardsignatur
settings-ident-signature-none = Keine
settings-ident-error-fields = Geben Sie einen Namen und eine gültige E-Mail-Adresse ein.
settings-ident-error-generic = Die Identität konnte nicht gespeichert werden.

settings-notif-title = Benachrichtigungen
settings-notif-intro = Wählen Sie, bei welchen neuen Nachrichten Sie benachrichtigt werden und wann Ruhe herrschen soll.
settings-notif-enabled-label = Benachrichtigungen für neue E-Mails anzeigen
settings-notif-quiet-title = Ruhezeiten
settings-notif-quiet-enabled-label = Benachrichtigungen zu einer festgelegten Zeit stummschalten
settings-notif-quiet-start = Von
settings-notif-quiet-end = Bis
settings-notif-rules-title = Regeln
settings-notif-rule-match-placeholder = Absender, Postfach oder Betreff enthält…
settings-notif-rule-action-label = Aktion
settings-notif-rule-notify = Benachrichtigen
settings-notif-rule-mute = Stummschalten
settings-notif-rule-add = Regel hinzufügen
settings-notif-error = Die Benachrichtigungseinstellungen konnten nicht gespeichert werden.

settings-search-title = Gespeicherte Suchen
settings-search-intro = Eine gespeicherte Suche als Ordner in Ihrer Postfachliste anzeigen.
settings-search-empty = Sie haben noch keine gespeicherten Suchen.
settings-search-as-folder = { $name } als Ordner anzeigen
settings-search-as-folder-label = Als Ordner anzeigen
settings-search-error = Die gespeicherte Suche konnte nicht geändert werden.

settings-kbd-title = Tastenkürzel
settings-kbd-intro = Wählen Sie einen Satz Tastenkürzel. Änderungen gelten auf diesem Gerät.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = Aus (keine Tastenkürzel)
settings-kbd-action-compose = Verfassen
settings-kbd-action-archive = Archivieren
settings-kbd-action-reply = Antworten
settings-kbd-action-next = Nächste Nachricht
settings-kbd-action-previous = Vorherige Nachricht
settings-kbd-action-search = Suchen

settings-offline-title = Offline-Cache
settings-offline-intro = Wie viele E-Mails für die Offline-Nutzung auf diesem Gerät bleiben und wie Speicherplatz freigegeben wird.
settings-offline-budget-label = Cache-Größe (MB)
settings-offline-retention-label = Aufbewahren für (Tage)
settings-offline-strategy-label = Wenn die Größe erreicht ist
settings-offline-lru = Am längsten nicht verwendete entfernen
settings-offline-oldest = Älteste zuerst entfernen
settings-offline-manual = Nur wenn ich ihn leere
settings-offline-purge = Offline-Cache jetzt leeren

settings-dir-title = Schreibrichtung der Oberfläche
settings-dir-intro = Der Sprache folgen oder eine Richtung erzwingen. Rechts-nach-links spiegelt das Layout.
settings-dir-auto = Automatisch
settings-dir-ltr = Links nach rechts
settings-dir-rtl = Rechts nach links
settings-dir-preview-title = Vorschau
settings-dir-preview-body = Dieser Text wird gespiegelt, wenn die Richtung rechts nach links ist.
settings-mail-features = E-Mail-Funktionen
settings-mail-feature-allowSend = Senden von Nachrichten erlauben
settings-mail-feature-allowSend-help = Wenn aus, sind Senden und das zugehörige Tastenkürzel deaktiviert. Entwürfe können weiterhin gespeichert werden.
settings-mail-feature-allowReply = Antworten und Allen antworten erlauben
settings-mail-feature-allowReply-help = Wenn aus, werden die Aktionen Antworten und Allen antworten sowie ihre Tastenkürzel nicht angeboten.
settings-mail-feature-allowForward = Weiterleiten erlauben
settings-mail-feature-allowForward-help = Wenn aus, werden die Aktionen zum Weiterleiten und das zugehörige Tastenkürzel nicht angeboten.
settings-mail-feature-allowDownload = Herunterladen von Nachrichten und Anhängen erlauben
settings-mail-feature-allowDownload-help = Wenn aus, werden keine Downloads von Nachrichten und Anhängen angeboten.
settings-mail-feature-allowPrint = Drucken erlauben
settings-mail-feature-allowPrint-help = Wenn aus, wird die Aktion Drucken nicht angeboten.
settings-mail-feature-showSpam = Spam-Ordner anzeigen
settings-mail-feature-showSpam-help = Wenn aus, werden der Spam-Ordner und als Spam markierte Nachrichten in der Ordnerliste und in Suchergebnissen ausgeblendet. Es wird nichts gelöscht.
settings-mail-feature-showAuthStatus = Ergebnisse der Absenderauthentifizierung anzeigen
settings-mail-feature-showAuthStatus-help = Zeigt die SPF-, DKIM- und DMARC-Ergebnisse einer Nachricht in der Leseansicht an.
settings-mail-feature-closeOnDelete = Nachricht beim Löschen schließen
settings-mail-feature-closeOnDelete-help = Schließt die geöffnete Nachricht, den Tab oder das Fenster, wenn diese Nachricht gelöscht oder in den Papierkorb verschoben wird.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = Einstellungspfad
settings-crumb-loading = Name wird geladen
settings-crumb-new = Neu
settings-crumb-edit = Bearbeiten
settings-badge-color = Farbe des Ungelesen-Zeichens
settings-badge-color-accent = Wie Akzentfarbe
settings-badge-color-accent-lighter = Hellere Akzentfarbe
settings-badge-color-accent-darker = Dunklere Akzentfarbe
settings-badge-color-option = { $color }-Zeichen
settings-appearance-theme-card = Design und Umschaltung
settings-appearance-theme-card-help = Wählen Sie ein Design und ob es dem System oder der Tageszeit folgt.
settings-appearance-colours-card = Farben
settings-appearance-colours-card-help = Die Akzentfarbe für Aktionen und Auswahl sowie die Farbe der Ungelesen-Zähler.
settings-appearance-text-card = Text und Layout
settings-appearance-text-card-help = Wie dicht die Oberfläche ist, welche Schrift sie verwendet und wie E-Mails angeordnet werden.
settings-sig-create-variant = Variante erstellen
settings-sig-error-existing = Eine Signatur mit diesem Namen existiert bereits. Wählen Sie einen anderen Namen.
settings-sig-save-failed = Die Signatur konnte nicht gespeichert werden. { $reason }
settings-ident-signature-new = Neue Nachrichten
settings-ident-signature-account = Kontosignatur verwenden
settings-ident-signature-replies = Antworten
settings-ident-signature-inherit = Signatur für neue Nachrichten verwenden
settings-ident-signature-unavailable = Nicht verfügbare Signatur: { $name }
settings-ident-signature-help = Erstellen Sie Signaturvarianten unter Signaturen und weisen Sie hier eine zu. Dieselbe Variante kann für beides verwendet werden.
settings-ident-save-failed = Die Identität konnte nicht gespeichert werden. { $reason }
settings-search-save-failed = Die gespeicherte Suche konnte nicht gespeichert werden. { $reason }
settings-kbd-preset-title = Tastenkürzel-Vorlage
settings-editor-discard-title = Nicht gespeicherte Änderungen verwerfen?
settings-editor-discard-detail = Ihre Änderungen wurden nicht gespeichert. Bearbeiten Sie weiter oder verwerfen Sie diesen Entwurf, um den Editor zu verlassen.
settings-editor-keep = Weiter bearbeiten
settings-editor-discard = Änderungen verwerfen
settings-editor-missing = Dieses Element existiert nicht mehr. Kehren Sie zur Liste zurück und wählen Sie ein anderes Element.
settings-editor-back = Zurück zur Liste
settings-sig-edit-title = Signatur bearbeiten
settings-ident-edit-title = Identität bearbeiten
settings-sections = Einstellungsbereiche
settings-category-account = Konto
settings-category-mail = E-Mail
settings-category-interface = Oberfläche
settings-category-integrations = Integrationen
settings-category-about = Info
settings-nav-preferences = Tastatur und Anzeige
settings-nav-rules = E-Mail-Regeln
settings-nav-access = API-Zugriff
settings-nav-metadata = Serverinformationen
settings-folders-title = Ordner und Seitenleiste
settings-folders-help = Heften Sie Ordner über die anderen, wählen Sie ein Symbol und ordnen Sie Ordner innerhalb jeder Gruppe per Ziehen oder mit den Verschieben-Schaltflächen an. Diese Auswahl wird mit diesem Konto synchronisiert und verschiebt keine Nachrichten und benennt keine Ordner um.
settings-folders-pin = { $name } anheften
settings-folders-icon = Symbol für { $name }
settings-folders-automatic = Automatische Ordnerrolle
settings-folders-drag = { $name } zum Neuanordnen ziehen
settings-folders-up = { $name } nach oben verschieben
settings-folders-down = { $name } nach unten verschieben
settings-folders-sync-idle = Melden Sie sich an, um Ordnereinstellungen zu synchronisieren.
settings-folders-sync-loading = Kontoeinstellungen werden geladen …
settings-folders-sync-saving = Kontoeinstellungen werden gespeichert …
settings-folders-sync-synced = Einstellungen mit diesem Konto synchronisiert.
settings-folders-sync-error = Die Einstellungen konnten nicht synchronisiert werden. Ihre Änderungen bleiben in dieser Sitzung erhalten und werden erneut versucht, sobald die Verbindung wieder besteht.
settings-folder-icon-folder = Ordner
settings-folder-icon-star = Stern
settings-folder-icon-flag = Flagge
settings-folder-icon-tag = Label
settings-folder-icon-clock = Uhr
settings-folder-icon-calendar = Kalender
settings-folder-icon-file = Dokument
settings-folder-icon-image = Bild
settings-folder-icon-lock = Schloss
settings-folder-icon-users = Personen
settings-folder-icon-archive = Archiv
settings-folder-icon-mail = Umschlag
settings-folder-icon-attachment = Anhang
settings-folder-icon-check = Häkchen
settings-folder-icon-alert = Warnung
settings-search-new = Neuer Suchordner
settings-search-editor = Suchordner
settings-search-editor-help = Nachrichten müssen allen Bedingungen entsprechen. Lassen Sie eine Bedingung leer, um sie zu ignorieren. Suchordner zeigen passende Nachrichten an, ohne sie zu verschieben oder zu kopieren.
settings-search-name = Ordnername
settings-search-name-required = Geben Sie einen Namen für diesen Suchordner ein.
settings-search-invalid = Diese gespeicherte Suche hat einen ungültigen Filter. Sie kann nicht sicher bearbeitet werden.
settings-search-saved = Suchordner „{ $name }“ gespeichert.
settings-search-field-text = Gesamten Nachrichtentext durchsuchen
settings-search-field-from = Absender enthält
settings-search-field-to = Empfänger enthält
settings-search-field-subject = Betreff enthält
settings-search-field-body = Nachrichtentext enthält
settings-search-field-filename = Anhangsdateiname enthält
settings-search-field-inMailbox = Auf Postfachkennung beschränken (optional)
settings-search-field-after = Empfangen nach (UTC-Datum und -Uhrzeit)
settings-search-field-before = Empfangen vor (UTC-Datum und -Uhrzeit)
settings-search-attachment = Anhänge
settings-search-any = Beliebige Nachricht
settings-search-with-attachments = Mit Anhängen
settings-search-without-attachments = Ohne Anhänge
settings-search-unread = Nur ungelesene Nachrichten
settings-search-flagged = Nur markierte Nachrichten
settings-sessions-search = Konto, Gerät oder IP suchen
settings-sessions-account = Konto
settings-sessions-device = Gerät und Client
settings-sessions-location = IP und zuletzt bekanntes Land
settings-sessions-location-unavailable = Land nicht verfügbar
settings-sessions-activity = Aktivität
settings-sessions-actions = Aktionen
settings-sessions-revoke-device = { $device } abmelden
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = Verfassen und Antworten
settings-quote-style = Kopfzeilenstil für Antworten und Weiterleitungen
settings-quote-classic = Klassische zitierte Antwort
settings-quote-outlook = Kopfzeile im Outlook-Stil
settings-quote-help = Der Outlook-Stil fügt Von, Gesendet, An, Cc und Betreff über der ursprünglichen Nachricht ein, sowohl in Rich-Text als auch in Nur-Text. Bestehende Entwürfe bleiben unverändert.
settings-external-warning = Warnungen bei externen Empfängern
settings-external-off = Aus
settings-external-warn = Warnung anzeigen
settings-external-confirm = Vor dem Senden Bestätigung verlangen
settings-external-help = An, Cc und Bcc werden mit der Domain der gewählten Identität und der Liste vertrauenswürdiger Domains abgeglichen. Dies schützt Sendungen aus diesem Editor; es ist keine serverseitige Zustellungsregel.
settings-trusted-domains = Vertrauenswürdige Empfängerdomains
settings-trusted-domains-help = Geben Sie bis zu 100 exakte Domains ein, eine pro Zeile. Subdomains müssen separat aufgeführt werden. Die eigene Domain der Absenderidentität ist immer vertrauenswürdig. URLs, E-Mail-Adressen und Platzhalter werden nicht akzeptiert.
settings-trusted-domains-invalid = Geben Sie bis zu 100 gültige Domainnamen ein, ohne URLs, E-Mail-Adressen oder Platzhalter.
settings-trusted-domains-save = Vertrauenswürdige Domains speichern
settings-shortcuts-enabled = Anwendungstastenkürzel aktivieren
settings-shortcuts-action = Aktion
settings-shortcuts-binding = Belegung
settings-shortcuts-edit = Tastenkürzel bearbeiten
settings-shortcuts-edit-action = Tastenkürzel für { $action } bearbeiten
settings-shortcuts-help = Verwenden Sie „inherit“ für Ihre gewählte Vorlage, „disabled“ für kein Tastenkürzel oder eine Tastenkombination wie Ctrl+Shift+K. Ctrl bedeutet unter macOS Cmd. Kollidierende Belegungen sind inaktiv. Eingabefelder und Dialoge behalten ihre üblichen Tasten; vom Browser reservierte Kombinationen sind möglicherweise nicht verfügbar.
settings-shortcuts-invalid = Geben Sie „inherit“, „disabled“ oder eine gültige einzelne Taste mit optionalen Modifikatoren Ctrl, Alt und Shift ein.
settings-shortcuts-conflict = Zwei Aktionen verwenden dasselbe Tastenkürzel. Ändern Sie eine Belegung; kollidierende Tastenkürzel werden nicht ausgeführt.

# -- Machine translation (2026-11): added to match en ---------------------
settings-shortcuts-discard = Nicht gespeicherte Änderung des Tastenkürzels verwerfen?
settings-shortcuts-save = Tastenkürzel speichern
settings-shortcuts-back = Zurück zu den Tastenkürzeln
settings-shortcuts-off = Ihre Vorlage „Aus“ deaktiviert alle Tastenkürzel, auch eigene Belegungen. Wählen Sie unter Einstellungen eine andere Vorlage, um diese Belegungen zu verwenden.
settings-kbd-action-replyAll = Allen antworten
settings-kbd-action-forward = Weiterleiten
settings-kbd-action-refresh = Nachrichten aktualisieren
settings-kbd-action-settings = Einstellungen öffnen
settings-kbd-action-trash = In den Papierkorb verschieben
settings-kbd-action-markRead = Als gelesen markieren
settings-kbd-action-markUnread = Als ungelesen markieren
settings-kbd-action-spam = Als Spam markieren
settings-kbd-action-zoomIn = Nachricht vergrößern
settings-kbd-action-zoomOut = Nachricht verkleinern
settings-kbd-action-zoomReset = Nachrichtenzoom zurücksetzen
settings-kbd-action-renameFolder = Ordner umbenennen
settings-kbd-action-openInTab = Nachricht in eigenem Tab öffnen
settings-notif-device-title = Berechtigung auf diesem Gerät
settings-notif-device-help = Ihre Benachrichtigungseinstellungen folgen Ihrem Konto. Die Browserberechtigung gehört zu diesem Gerät; Mailwoman fragt nur, wenn Sie sie erlauben möchten.
settings-notif-ask-permission = Benachrichtigungsberechtigung auf neuen Geräten anbieten
settings-notif-ask-permission-help = Eine Berechtigungsabfrage anzeigen, wenn dieses Gerät Benachrichtigungen weder erlaubt noch blockiert hat. Standardmäßig aktiviert; diese Einstellung wird sofort gespeichert.
settings-notif-permission-default = Die Browserberechtigung wurde noch nicht angefragt.
settings-notif-permission-granted = Browser-Benachrichtigungen sind auf diesem Gerät erlaubt.
settings-notif-permission-denied = Benachrichtigungen sind blockiert. Ändern Sie die Berechtigung dieser Website in den Browsereinstellungen, um sie zu erlauben.
settings-notif-permission-unsupported = Dieser Browser unterstützt hier keine Systembenachrichtigungen.
settings-notif-permission-request = Benachrichtigungen auf diesem Gerät erlauben
settings-notif-permission-error = Der Browser konnte die Benachrichtigungsberechtigung nicht anfragen.
settings-notif-timezone = Zeitzone der Ruhezeiten (IANA)
settings-notif-timezone-device = Zeitzone dieses Geräts verwenden
settings-permissions-background-title = Hintergrundbenachrichtigungen in diesem Browser
settings-permissions-background-help = Speichern Sie zuerst die Benachrichtigungseinstellungen und erlauben Sie Browser-Benachrichtigungen. Registrieren Sie diesen Browser, um Kontoaktivität auch bei geschlossener App zu erhalten.
settings-permissions-push-unsupported = Web-Push im Hintergrund ist in diesem Browser nicht verfügbar.
settings-permissions-push-registered = Dieser Browser hat ein Push-Abonnement. Registrieren Sie es, um es mit dem aktuellen Konto zu verknüpfen.
settings-permissions-push-unregistered = Dieser Browser hat kein Push-Abonnement.
settings-permissions-push-register = Hintergrundbenachrichtigungen registrieren
settings-permissions-push-remove = Browser-Abonnement entfernen
settings-permissions-push-policy = Ihre Administration erlaubt keine Push-Benachrichtigungen im Hintergrund.
settings-permissions-storage-title = Browserspeicher
settings-permissions-storage-help = Persistenter Speicher schützt die lokalen Daten dieser Website vor automatischer Verdrängung, sofern der Browser ihn gewährt. Er gilt nur für dieses Gerät und ändert die Aufbewahrung auf dem Server nicht. Sie können Websitedaten in den Browsereinstellungen löschen; nicht gesendete Offline-Änderungen können dabei verloren gehen.
settings-permissions-storage-unsupported = Anfragen für persistenten Speicher sind in diesem Browser nicht verfügbar.
settings-permissions-storage-persistent = Der Browser hat persistenten Speicher für diese Website gewährt.
settings-permissions-storage-evictable = Der Browser kann die lokalen Daten dieser Website automatisch verdrängen.
settings-permissions-storage-usage = Aktueller Speicher dieser Website: { $mib } MiB.
settings-permissions-storage-request = Persistenten Speicher anfragen
settings-permissions-storage-policy = Ihre Administration erlaubt keine Anfragen für persistenten Speicher.
settings-permissions-error = Die Browseraktion konnte nicht abgeschlossen werden. Prüfen Sie die gespeicherten Kontoeinstellungen und die Browserberechtigungen dieser Website.
settings-notif-background-rules = Inhaltsregeln werden ausgewertet, solange Mailwoman geöffnet ist. Systembenachrichtigungen im Hintergrund werden unterdrückt, solange Inhaltsregeln konfiguriert sind, da ein undurchsichtiges Hintergrund-Wecken keine Nachrichtendetails enthält.
settings-nav-regional = Regionale Einstellungen
settings-time-title = Zeitzonen
settings-time-help = Wählen Sie, wie Nachrichtenzeiten angezeigt werden, und optional eine Vergleichszeitzone für die Terminplanung. Standorte werden nie aus E-Mail-Adressen abgeleitet.
settings-time-displayZone = Anzeigezeitzone
settings-time-displayZone-help = Verwenden Sie „local“ für dieses Gerät oder einen IANA-Namen wie Europe/Lisbon.
settings-time-comparisonZone = Vergleichszeitzone
settings-time-comparisonZone-help = Geben Sie einen IANA-Namen wie America/New_York ein oder lassen Sie das Feld leer, um Vergleiche auszublenden.
settings-time-helpers = Zeitzonenhilfen anzeigen
settings-time-save = Zeitzone speichern
settings-time-invalid = Geben Sie eine gültige IANA-Zeitzone ein. Verwenden Sie „local“ nur für die Anzeigezeitzone oder lassen Sie die Vergleichszeitzone leer.
settings-time-discard = Nicht gespeicherte Zeitzonenänderungen verwerfen?
settings-folders-name = Ordner
settings-folders-actions = Aktionen
settings-folders-edit = Darstellung von { $name } bearbeiten
settings-folders-icon-help = Wählen Sie eines von fünfzig thematischen Symbolen oder behalten Sie die automatische Auswahl nach dem Zweck des Ordners bei. Dies folgt Ihrem Konto und benennt das Postfach nicht um.
settings-folders-save = Ordnerdarstellung speichern
settings-folders-back = Zurück zu den Ordnern
settings-folders-discard = Nicht gespeicherte Änderungen der Ordnerdarstellung verwerfen?
settings-folder-labels = Ordner-Labels
settings-folder-labels-help = Ordnen Sie diesen Ordner mit bis zu 32 Konto-Labels ein. Nachrichten darin werden dadurch weder mit Labels versehen noch verschoben. Verwalten Sie Labelnamen und Farben unter Labels.
settings-folders-leave-pending = Diese Änderungen warten noch auf die Synchronisierung. Diesen Editor verlassen?
settings-folders-retry = Speichern erneut versuchen
settings-folder-role-title = Spezialordner
settings-folder-role-help = Wählen Sie den tatsächlichen Ordner für jede E-Mail-Rolle. „Automatisch“ nutzt die Servererkennung. „Administrationsstandard verwenden“ entfernt Ihre Überschreibung. Kollidierende oder fehlende Ziele deaktivieren die Rolle, statt einen anderen Ordner zu wählen.
settings-folder-role-auto = Automatische Erkennung
settings-folder-role-inherit = Administrationsstandard verwenden
settings-folder-role-path = Konfigurierter Pfad: { $path }
settings-folder-role-missing = Der konfigurierte Ordner ist nicht verfügbar
settings-folder-role-conflict = Dieses Ziel fehlt oder kollidiert mit einer anderen Rolle. Diese Rolle ist nicht verfügbar, bis die Zuordnung korrigiert wird.
settings-folder-role-refresh-error = Gespeichert, aber die Ordnerliste konnte nicht aktualisiert werden. Aktualisieren Sie die Ordner, bevor Sie die neue Zuordnung verwenden.
settings-folder-role-unsupported = Diese Überschreibungen erfordern native Mailordner. Upstream-JMAP-Konten behalten die Ordnerrollen ihres Servers.
settings-folder-role-inbox = Posteingang
settings-folder-role-drafts = Entwürfe
settings-folder-role-sent = Gesendete E-Mails
settings-folder-role-trash = Papierkorb
settings-folder-role-junk = Junk und Spam
settings-folder-role-archive = Archiv
settings-sessions-select = Auswählen
settings-sessions-select-device = Sitzung für { $device } auswählen
settings-sessions-select-visible = Alle sichtbaren anderen Sitzungen auswählen
settings-sessions-current-protected = Diese Sitzung bleibt angemeldet und kann hier nicht ausgewählt werden. Verwenden Sie Abmelden, um sie zu beenden.
settings-sessions-selected-title = Ausgewählte Sitzungen abmelden ({ $count })
settings-sessions-selected-confirm = Nur die ausgewählten anderen Sitzungen beenden. Diese Sitzung bleibt angemeldet. Fehlgeschlagene oder übersprungene Anfragen werden ohne automatische Wiederholung gemeldet.
settings-sessions-selected-action = Ausgewählte Sitzungen abmelden
settings-sessions-cancel = Abbrechen
settings-sessions-selected-result = Abgemeldet: { $revoked }. Bereits beendet: { $unchanged }. Nicht bestätigt: { $failed }. Nicht versucht: { $skipped }.
settings-sessions-refresh-failed = Die Sitzungsliste konnte nicht aktualisiert werden. Laden Sie sie neu, bevor Sie weitere Sitzungen auswählen.
settings-sig-format = Signaturformat
settings-sig-format-plain = Nur Text
settings-sig-format-html = Rich-Text (HTML)
settings-sig-html-help = HTML ist auf 2 MiB begrenzt und wird vor dem Speichern und Senden bereinigt. Externe Bilder und aktive Inhalte werden entfernt.
settings-sig-html-source = HTML-Quelltext bearbeiten
settings-sig-editor-loading = Signatureditor wird geladen …
settings-sig-rich-body = Rich-Text-Signatur
settings-sig-plain-alternative = Nur-Text-Alternative
settings-sig-plain-help = Nur-Text-Nachrichten verwenden diesen Text. Rich-Text-Änderungen aktualisieren ihn, bis Sie ihn anpassen; Änderungen am HTML-Quelltext behalten Ihre Alternative bei.
settings-sig-account-changed = Ihr angemeldetes Konto hat sich geändert. Öffnen Sie diese Signatur erneut im richtigen Konto.
settings-sig-image-type = { $name } ist kein PNG-, JPEG-, GIF- oder WebP-Bild.
settings-sig-image-size = { $name } passt nicht: Das HTML einer Signatur einschließlich Bildern ist auf 2 MiB begrenzt.
settings-sig-too-large = Diese Signatur hat mehr als 2 MiB HTML. Entfernen Sie ein Bild oder etwas Formatierung.
settings-sig-images-help = Bilder werden in der Signatur gespeichert und als Inline-Bilder jeder Nachricht gesendet.
settings-folder-icon-inbox = Posteingang
settings-folder-icon-sent = Gesendet
settings-folder-icon-edit = Bearbeiten
settings-folder-icon-delete = Löschen
settings-folder-icon-spam = Spam
settings-folder-icon-search = Suche
settings-folder-icon-pin = Stecknadel
settings-folder-icon-link = Link
settings-folder-icon-shield = Schild
settings-folder-icon-unlock = Entsperren
settings-folder-icon-fileAudio = Audio
settings-folder-icon-fileVideo = Video
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = Code
settings-folder-icon-print = Drucken
settings-folder-icon-download = Download
settings-folder-icon-preferences = Einstellungen
settings-folder-icon-dashboard = Dashboard
settings-folder-icon-analytics = Analytik
settings-folder-icon-nextcloud = Cloud-Dateien
settings-folder-icon-domains = Domains
settings-folder-icon-twofactor = Authentifizierung
settings-folder-icon-assistant = Assistent
settings-folder-icon-integrations = Integrationen
settings-folder-icon-logs = Protokolle
settings-folder-icon-home = Startseite
settings-folder-icon-briefcase = Aktentasche
settings-folder-icon-heart = Herz
settings-folder-icon-shopping = Einkaufen
settings-folder-icon-education = Bildung
settings-folder-icon-travel = Reisen
settings-folder-icon-coffee = Kaffee
settings-folder-icon-gift = Geschenk

# -- Machine translation (2026-11): added to match en ---------------------
settings-folder-icon-bank = Bank
settings-folder-icon-book = Buch
settings-nav-keyboard-display = Tastatur & Anzeige
settings-2fa-passkey-policy-disabled = Ihre Administration hat die Anmeldung und Registrierung mit Passkeys für dieses Konto deaktiviert. Vorhandene Anmeldedaten bleiben aufgelistet, damit Sie sie verwalten oder entfernen können.
settings-mail-interactions = Nachrichtenauswahl und Verschieben
settings-mail-modifier-selection = Nachrichten mit Modifikatortasten auswählen
settings-mail-modifier-selection-help = Strg oder Befehl schaltet einzelne Nachrichten um. Umschalt wählt einen Bereich aus, Umschalt mit Pfeiltasten erweitert ihn. Die sichtbare Auswahlschaltfläche bleibt verfügbar, wenn dies aus ist.
settings-mail-drag-folder = Nachrichten in Ordner ziehen
settings-mail-drag-folder-help = Ziehen Sie eine ausgewählte Nachricht, um die Auswahl zu verschieben, oder eine nicht ausgewählte Nachricht, um nur diese zu verschieben. Es werden nur Ordner dieses Kontos akzeptiert. Touch- und Tastaturnutzer können stattdessen die Auswahlleiste verwenden.
settings-search-sections = Einstellungen durchsuchen
settings-search-no-sections = Keine passenden Einstellungsbereiche.
settings-time-local = Lokale Zeitzone des Geräts
settings-time-none = Keine Vergleichszeitzone
settings-folder-counts = Ordneranzahlen
settings-folder-counts-none = Anzahlen ausblenden
settings-folder-counts-unread = Ungelesene Nachrichten
settings-folder-counts-total = Alle Nachrichten
settings-folder-counts-both = Ungelesene und alle Nachrichten
settings-folder-counts-unreadIfAny = Ungelesene Nachrichten, nur wenn vorhanden
settings-folder-count-scope = Ordner mit Anzahlen
settings-folder-count-scope-all = Alle Ordner
settings-folder-count-scope-main = Nur Hauptordner
settings-folder-count-scope-help = Hauptordner sind Posteingang, Entwürfe, Gesendet, Archiv, Junk, Papierkorb, Postausgang und andere Spezialordner. Eigene Ordner und ihre Unterordner zeigen keine Anzahl.
settings-account-unread-badge = Ungelesen-Anzahl neben dem Postfachnamen anzeigen
settings-folder-counts-compact = Kleinere Zeichen außerhalb des Posteingangs
settings-focused-inbox = Fokussierter Posteingang
settings-focused-inbox-help = Den Posteingang über der Liste in die Tabs „Fokussiert“ und „Andere“ aufteilen. Nachrichten von Newslettern und automatischen Absendern landen in „Andere“. Andere Ordner und Suchergebnisse zeigen jede Nachricht.
settings-focused-senders-title = Absender für „Fokussiert“ verwalten
settings-focused-senders-help = Absender, die Sie immer in „Fokussiert“ oder „Andere“ verschieben lassen. Entfernen Sie einen, damit wieder die automatische Sortierung entscheidet.
settings-focused-senders-empty = Noch keine Absender. Verwenden Sie bei einer Nachricht „Immer in Fokussiert verschieben“ oder „Immer in Andere verschieben“.
settings-focused-senders-to-focused = Immer Fokussiert
settings-focused-senders-to-other = Immer Andere
settings-focused-senders-remove = Entfernen
settings-focused-senders-remove-label = { $sender } entfernen
settings-focused-senders-remove-failed = Der Absender konnte nicht entfernt werden. Versuchen Sie es erneut.
settings-folder-counts-reset = Ordneranzahlen übernehmen
settings-folder-counts-reset-size = Zeichengröße übernehmen
settings-mail-resume-tabs = Meine geöffneten Tabs fortsetzen
settings-mail-resume-tabs-help = Kontobezogene Tab-Verweise nach dem erneuten Öffnen von Mailwoman wiederherstellen. Nachrichtentexte und Anhangs-URLs werden nicht im Arbeitsbereichseintrag gespeichert.
settings-row-actions-mode = Schnellaktionen der Nachrichtenliste
settings-row-actions-mode-open = Immer geöffnet
settings-row-actions-mode-hover = Beim Überfahren anzeigen
settings-row-actions-mode-click = Beim Klicken anzeigen
settings-row-actions-mode-off = Aus
settings-row-actions-mode-help = Wie die Schnellaktionen in jeder Nachrichtenzeile erscheinen. „Beim Überfahren“ zeigt drei Steuerelemente und eine Schaltfläche „Mehr“; „Beim Klicken“ zeigt eine Schaltfläche, die sie öffnet; auf Touchscreens verhält sich das Überfahren wie ein Klick. „Aus“ entfernt sie aus den Zeilen (Kontextmenü und Symbolleiste bieten weiterhin jede Aktion).
settings-mail-feature-showGoToTop = Schaltfläche „Nach oben“ in der Nachrichtenliste anzeigen
settings-mail-feature-showGoToTop-help = Fügt eine Schaltfläche hinzu, die die Nachrichtenliste zur ersten Nachricht zurückscrollt. Standardmäßig aus.
settings-mail-feature-showCreateFolderButton = Schaltfläche „Ordner erstellen“ neben „Verfassen“ anzeigen
settings-mail-feature-showCreateFolderButton-help = Fügt in der Seitenleiste eine reine Symbolschaltfläche zum Erstellen eines Ordners hinzu. Standardmäßig aus.
settings-draft-save = Kontoentwürfe
settings-draft-save-help = In den Entwurfsordner Ihres Kontos speichern, um auf einem anderen Gerät darauf zuzugreifen. Das Speichern erfolgt unauffällig; nur Fehler lösen eine Benachrichtigung aus. Nicht unterstützte Konten und geschützte entschlüsselte Inhalte werden nie stillschweigend hochgeladen.
settings-draft-save-mode = Automatisch speichern
settings-draft-realtime = Nach einer Schreibpause
settings-draft-periodic = In regelmäßigen Abständen
settings-draft-manual = Nur wenn ich Speichern wähle
settings-draft-debounce = Schreibpause (Millisekunden)
settings-draft-interval = Speicherintervall (Sekunden)
settings-permissions-push-other-registration = Dieser Browser hat ein Abonnement, das nicht für dieses Konto registriert ist. Entfernen Sie es, bevor Sie sich erneut registrieren.
settings-show-from = „Von“ beim Verfassen anzeigen
settings-show-from-help = Mit sichtbarem Feld „Von“ beginnen. Sie können es auch für eine einzelne Nachricht über das Menüband einblenden. Es werden nur autorisierte Absenderadressen akzeptiert.
settings-sessions-table = Tabelle der aktiven Sitzungen
settings-folders-table = Ordnerliste
settings-kbd-reference = Belegungen der Vorlage anzeigen
settings-kbd-reference-help = Nur Vorlagenreferenz. Eigene Belegungen und Richtlinien der Administration können die aktiven Tastenkürzel ändern.
settings-sessions-client-details = Client-Details
settings-2fa-passkey-rules = Ihre Organisation akzeptiert nur Passkeys, die:
settings-2fa-passkey-rule-platform = auf diesem Gerät gespeichert sind (Windows Hello, Touch ID, Android-Bildschirmsperre)
settings-2fa-passkey-rule-cross-platform = auf einem Sicherheitsschlüssel oder einem Smartphone liegen
settings-2fa-passkey-rule-user-verification = Ihre PIN, Ihren Fingerabdruck oder Ihr Gesicht prüfen
settings-2fa-passkey-rule-resident-key = auf dem Authenticator gespeichert sind (auffindbar)
settings-2fa-passkey-gap-allowed = Vor der aktuellen Richtlinie hinzugefügt und erfüllt sie nicht. Er funktioniert weiterhin; fügen Sie einen neuen Passkey hinzu und entfernen Sie diesen.
settings-2fa-passkey-gap-refused = Erfüllt die aktuelle Richtlinie nicht und wird für die Anmeldung nicht mehr akzeptiert. Fügen Sie einen neuen Passkey hinzu und entfernen Sie diesen.
settings-2fa-passkey-browser-refused-policy = Ihr Browser hat keinen Authenticator gefunden, der die Passkey-Richtlinie Ihrer Organisation erfüllt, oder die Anfrage wurde abgebrochen.
settings-2fa-policy-user-verification = Ihre Organisation verlangt einen Passkey, der Ihre PIN, Ihren Fingerabdruck oder Ihr Gesicht prüft. Dieser Authenticator hat Sie nicht verifiziert.
settings-2fa-policy-platform = Ihre Organisation akzeptiert nur auf diesem Gerät gespeicherte Passkeys (etwa Windows Hello, Touch ID oder Android-Bildschirmsperre). Sicherheitsschlüssel und Smartphones werden nicht akzeptiert.
settings-2fa-policy-cross-platform = Ihre Organisation akzeptiert nur Sicherheitsschlüssel oder Smartphone-Passkeys. Auf diesem Gerät gespeicherte Passkeys werden nicht akzeptiert.
settings-2fa-policy-attachment-unknown = Ihr Browser hat nicht angegeben, welche Art von Authenticator dies ist, daher kann er nicht anhand der Passkey-Richtlinie Ihrer Organisation geprüft werden. Versuchen Sie einen anderen Browser.
settings-2fa-policy-resident-key = Ihre Organisation verlangt einen Passkey, der auf dem Authenticator gespeichert ist. Dieser Authenticator hat keinen gespeichert.
settings-2fa-policy-legacy = Dieser Passkey wurde vor der aktuellen Passkey-Richtlinie Ihrer Organisation hinzugefügt und erfüllt sie nicht mehr. Melden Sie sich mit einem anderen Faktor oder einem Wiederherstellungscode an und fügen Sie dann einen neuen Passkey hinzu.
settings-2fa-policy-last-factor-blocked = Ihre Organisation erlaubt nicht, Ihren letzten zweiten Faktor zu entfernen. Fügen Sie zuerst eine andere Anmeldemethode hinzu.
settings-2fa-policy-last-factor-required = Ihre Organisation verlangt die Zwei-Faktor-Authentifizierung. Fügen Sie eine andere Anmeldemethode hinzu, bevor Sie diese entfernen.
settings-2fa-last-factor-hint = Fügen Sie zuerst eine andere Anmeldemethode hinzu.
settings-2fa-last-factor-tooltip = Fügen Sie zuerst eine andere Anmeldemethode hinzu. Ihre Organisation verlangt mindestens einen Passkey oder eine Authenticator-App für Ihr Konto.
settings-2fa-challenge-passkey-not-used = Der Passkey wurde nicht verwendet: Die Anfrage wurde abgebrochen oder ist abgelaufen, oder dieser Authenticator kann nicht erfüllen, was Ihre Organisation verlangt (etwa PIN-, Fingerabdruck- oder Gesichtsprüfung). Versuchen Sie es erneut oder verwenden Sie eine andere Methode.
settings-folders-hidden-help = Ausgeblendete Ordner und ihre Unterordner bleiben aus der Seitenleiste heraus. Sie werden weder gelöscht noch im Zugriff beschränkt. Wenn Sie hier einen Unterordner einblenden, werden auch seine ausgeblendeten übergeordneten Ordner eingeblendet. Der Posteingang bleibt immer verfügbar.
settings-folders-collapse-user = Benutzerordner in der Seitenleiste einklappen
settings-folders-show = { $name } in der Seitenleiste anzeigen
settings-folders-hide = { $name } in der Seitenleiste ausblenden
settings-folders-show-short = Anzeigen
settings-folders-hide-short = Ausblenden
settings-folders-search = Ordner durchsuchen
settings-folders-select = Ordner auswählen: { $name }
settings-folders-selected = Ausgewählte Ordner: { $count }
settings-folders-select-visible = Sichtbare auswählen
settings-folders-select-all = Alle Ordner auswählen
settings-folders-clear = Auswahl aufheben
settings-folders-pin-selected = Ausgewählte anheften
settings-folders-unpin-selected = Anheftung der ausgewählten aufheben
settings-folders-show-selected = Ausgewählte anzeigen
settings-folders-hide-selected = Ausgewählte ausblenden
settings-folders-visible = Angezeigt
settings-folders-hidden = Ausgeblendet
settings-folders-pinned = Angeheftet
settings-folders-location = Ort
settings-folders-counts = { $unread } ungelesen / { $total } gesamt
settings-folders-more = Weitere Aktionen für { $name }
settings-folders-empty = Keine Ordner entsprechen Ihrer Suche.
settings-folders-root = Oberste Ebene
settings-folders-order-help = Ziehen Sie Ordner, um sie in der Seitenleiste neu anzuordnen. Angeheftete Ordner bleiben zusammen.
cache-title = E-Mail-Cache und Vorladen
cache-help = Zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs behalten. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Beim Schließen des Tabs oder Abmelden wird er geleert. Das Vorladen markiert E-Mails nie als gelesen und lädt keine externen Bilder.
cache-enabled = Sitzungs-Cache aktivieren
cache-mailboxes = Nachrichtenlisten von Ordnern zwischenspeichern
cache-messages = Geöffnete Nachrichten zwischenspeichern
cache-freshSeconds = Aktualitätsintervall (Sekunden)
cache-maxAgeSeconds = Maximales Cache-Alter (Sekunden)
cache-maxEntries = Maximale Anzahl an Cache-Einträgen
cache-maxBytes = Speicherbudget (Bytes)
cache-prefetchEnabled = Spekulatives Vorladen aktivieren
cache-hoverEnabled = Vorladen beim Zeigen auf einen Ordner
cache-focusEnabled = Vorladen, wenn ein Ordner den Tastaturfokus erhält
cache-dwellMs = Verzögerung bei Zeiger und Fokus (Millisekunden)
cache-idleEnabled = Ordner im Leerlauf vorladen
cache-idleDelayMs = Leerlaufverzögerung (Millisekunden)
cache-maxConcurrency = Maximale gleichzeitige Vorladevorgänge
cache-maxQueued = Maximale Anzahl wartender Vorladevorgänge
cache-preloadMessages = Spekulative Nachrichtentexte zulassen
cache-excludedFolders = Vom Vorladen ausgeschlossene Ordner
cache-exclusions-help = Eine Regel pro Zeile: role:trash, role:junk, role:inbox, role:drafts, role:sent, role:archive oder id: gefolgt von einer exakten Ordner-ID.
cache-refreshOnOpen = Zwischengespeicherte Inhalte beim Öffnen aktualisieren
cache-clear = Zwischengespeicherte E-Mails jetzt leeren
settings-compose-window = Neue Nachrichten öffnen in
settings-compose-window-tab = Einem Arbeitsbereich-Tab
settings-compose-window-new = Einem neuen Fenster
settings-compose-window-help = Wenn Ihr Browser ein neues Fenster blockiert, wird der Editor in einem Arbeitsbereich-Tab geöffnet.

# -- Machine translation (2026-11): added to match en ---------------------
settings-compose-read-receipt = Immer Lesebestätigungen anfordern
settings-compose-delivery-receipt = Immer Zustellbestätigungen anfordern
settings-compose-reply-copy = Kopie von Antworten im ursprünglichen Ordner speichern
settings-compose-receipts-help = Bestätigungen sind Anfragen. Empfänger oder deren Mailserver können sie ablehnen oder ignorieren.
settings-compose-forward = Standardformat für Weiterleitungen
settings-compose-forward-inline = Nachrichtentext im Text
settings-compose-forward-attachment = Originalnachricht als .eml-Anhang
settings-compose-saving = Verfassen-Einstellung wird gespeichert …
settings-compose-saved = Verfassen-Einstellung gespeichert
settings-ident-organization = Organisation
settings-ident-auto-bcc = Automatisch in Bcc
settings-ident-auto-bcc-help = Fügen Sie beim Senden mit dieser Identität bis zu 20 Adressen in Bcc hinzu. Trennen Sie die Adressen mit Kommas.
settings-ident-pgp-key = OpenPGP-Schlüssel
settings-ident-pgp-none = Standardschlüssel verwenden
settings-ident-pgp-unavailable = Gespeicherter Schlüssel nicht verfügbar – wählen Sie einen anderen Schlüssel
settings-ident-pgp-help = Wählen Sie einen Ihrer vorhandenen OpenPGP-Signaturschlüssel. Das private Schlüsselmaterial bleibt in Ihrem Schlüsseltresor.
settings-ident-error-optional = Prüfen Sie die Antwortadresse, die automatischen Bcc-Adressen, die Organisation und den ausgewählten Schlüssel.
settings-compose-open-tracking = Nachrichtenöffnungen standardmäßig erfassen
settings-compose-open-tracking-help = Fügt geeigneten formatierten Nachrichten ein Tracking-Bild hinzu. Öffnungen hängen davon ab, ob der Empfänger Bilder lädt, und belegen nicht, dass die Nachricht gelesen wurde. Ihre Administration kann das Tracking deaktivieren oder diese Voreinstellung sperren.
settings-compose-open-tracking-inherit = Geerbte Tracking-Voreinstellung verwenden
settings-reply-images = Bilder in Antworten und Weiterleitungen
settings-reply-images-ask = Jedes Mal fragen
settings-reply-images-allow = Immer einschließen
settings-reply-images-trusted = Bei vertrauenswürdigen Absenderdomains einschließen, sonst fragen
settings-reply-images-deny = Nie einschließen
settings-reply-images-help = Legen Sie fest, ob eine Antwort oder Weiterleitung die Bilder der Originalnachricht enthält. Eingeschlossene Bilder werden als Anhänge in den Entwurf kopiert.
settings-reply-image-domains = Vertrauenswürdige Absenderdomains für Bilder
settings-reply-image-domains-help = Geben Sie bis zu 200 exakte Absenderdomains ein, eine pro Zeile. Wird verwendet, wenn oben die Option für vertrauenswürdige Domains gewählt ist.
settings-reply-image-domains-save = Vertrauenswürdige Bilddomains speichern
settings-ident-group-basics = Grundlagen
settings-ident-group-replies = Umgang mit Antworten
settings-ident-group-signature = Signatur
settings-ident-group-sending = Sendeoptionen
settings-ident-name-required = Geben Sie einen Anzeigenamen ein.
settings-ident-email-invalid = Geben Sie eine gültige E-Mail-Adresse ein, zum Beispiel name@example.org.
settings-ident-email-duplicate = Sie haben bereits eine Identität mit dieser Adresse.
settings-ident-replyto-invalid = Geben Sie eine gültige E-Mail-Adresse ein oder lassen Sie das Feld leer.
settings-ident-replyto-help = Antworten auf Nachrichten, die mit dieser Identität gesendet wurden, gehen an diese Adresse statt an die Absenderadresse.
settings-ident-bcc-invalid = Prüfen Sie die Bcc-Liste: Verwenden Sie gültige Adressen, durch Kommas getrennt, höchstens 20.
settings-ident-delete-title = Identität { $email } löschen?
settings-ident-delete-body = Sie können mit dieser Identität nicht mehr senden. Entwürfe und gesendete Nachrichten bleiben unverändert.
settings-ident-empty = Sie haben noch keine Identitäten. Fügen Sie eine hinzu, um beim Schreiben Name, Adresse und Signatur zu wählen.
settings-sig-delete-title = Signatur { $name } löschen?
settings-sig-delete-body = Identitäten, die sie verwenden, greifen auf die Kontosignatur zurück. Bereits gesendete Nachrichten bleiben unverändert.
settings-sig-deleted = Signatur gelöscht.
settings-reply-from = Antworten von
settings-reply-from-match = Die Adresse, an die die Nachricht gesendet wurde
settings-reply-from-default = Immer meine Standardidentität
settings-reply-from-ask = Fragen, wenn mehrere Identitäten passen
settings-reply-from-keep = Bereits ausgewählte Identität beibehalten
settings-reply-from-help = Legen Sie fest, welche Identität beim Antworten oder Weiterleiten verwendet wird. Passt keine Identität zur Adresse, an die die Nachricht gesendet wurde, wird Ihre Standardidentität verwendet. Plus-Adressen wie name+tag@example.org passen zu name@example.org.
settings-ident-default-badge = Standard
settings-ident-make-default = Als Standard festlegen
settings-ident-default-set = { $email } ist jetzt Ihre Standardidentität.
settings-ident-create-variant = Variante erstellen
settings-ident-variant-count = { $count } { $count ->
        [one] Variante
       *[other] Varianten
    }
settings-ident-inherits = Verwendet { $base } für: { $fields }
settings-ident-field-replyTo = Antwortadresse
settings-ident-field-autoBcc = automatisches Bcc
settings-ident-field-organization = Organisation
settings-ident-field-pgpKeyId = OpenPGP-Schlüssel
settings-ident-field-signatureName = Signatur
settings-ident-delete-default-body = Dies ist Ihre Standardidentität. Ihre älteste verbleibende Identität wird zur Standardidentität. Sie können mit dieser Identität nicht mehr senden. Entwürfe und gesendete Nachrichten bleiben unverändert.
settings-ident-variants-choice-title = { $email } löschen?
settings-ident-variants-choice-body = Diese Identität hat { $count } { $count ->
        [one] Variante
       *[other] Varianten
    }. Löschen Sie { $count ->
        [one] sie
       *[other] sie
    } ebenfalls oder behalten Sie { $count ->
        [one] sie
       *[other] sie
    } als eigenständige Identitäten. Eine behaltene Variante übernimmt als Kopie die Einstellungen, die sie von dieser Identität geerbt hat.
settings-ident-delete-with-variants = Identität und Varianten löschen
settings-ident-delete-keep-variants = Identität löschen, Varianten behalten
settings-ident-new-variant = Neue Variante
settings-ident-edit-variant-title = Variante bearbeiten
settings-ident-variant-intro = Eine Variante von { $base }. Lassen Sie ein Feld leer, um den Wert dieser Identität zu verwenden.
settings-ident-label-label = Variantenbezeichnung
settings-ident-label-required = Geben Sie eine Bezeichnung für diese Variante ein, zum Beispiel Newsletter.
settings-ident-inherit-hint = Leer: verwendet { $value } der Basisidentität.
settings-ident-inherit-none = nichts
settings-check-policy-allow = Ohne Nachfrage senden
settings-check-policy-warn = Jedes Mal fragen
settings-check-policy-warn-remember = Fragen, mit Option zum Abschalten
settings-check-policy-deny = Senden blockieren
settings-check-invalid = Ungültige Empfängeradressen
settings-check-attachment = Anhang erwähnt, aber nicht vorhanden
settings-check-subject = Leerer Betreff
settings-check-help = Was passiert, wenn die Prüfung ein Problem findet. Bei „Fragen“ steht immer „Trotzdem senden“ zur Wahl; nur „Senden blockieren“ hält eine Nachricht zurück.
settings-external-warn-remember = Fragen, mit Option zum Abschalten
settings-external-deny = Senden blockieren
settings-auto-chip = Empfänger abschließen, wenn ich ein Leerzeichen, Komma oder Semikolon tippe
settings-auto-chip-help = Eine gültige Adresse wird zu einem fertigen Empfänger, und das Trennzeichen wird für Sie hinzugefügt.
settings-recipient-separator = Empfängertrennzeichen
settings-recipient-separator-comma = Komma
settings-recipient-separator-semicolon = Semikolon
settings-header-collapse = Nachrichtenkopf (An, Cc, Betreff)
settings-header-collapse-expanded = Immer anzeigen
settings-header-collapse-collapsed = Auf eine Übersichtszeile einklappen
settings-header-collapse-auto = Einklappen, während ich den Text schreibe
settings-header-collapse-help = Eingeklappt zeigt der Kopf eine Zeile wie „An: Ana +2“ und öffnet sich per Klick oder wenn ein Feld Aufmerksamkeit braucht.
settings-retention-title = Aufbewahrung
settings-folder-icon-camera = Kamera
settings-folder-icon-music = Musik
settings-folder-icon-film = Film
settings-folder-icon-lightbulb = Idee
settings-folder-icon-rocket = Rakete
settings-folder-icon-trophy = Pokal
settings-folder-icon-leaf = Blatt
settings-folder-icon-globe = Globus
settings-folder-icon-cloud = Wolke
settings-folder-icon-bookmark = Lesezeichen
settings-folder-icon-wallet = Geldbörse
settings-folder-icon-medical = Medizin
settings-folder-icon-bell = Glocke
settings-folder-icon-key = Schlüssel
settings-folder-icon-receipt = Beleg
settings-folder-icon-cart = Warenkorb
settings-folder-icon-package = Paket
settings-folder-icon-wrench = Werkzeuge
settings-folder-icon-chart = Diagramm
settings-folder-icon-team = Team
settings-folder-icon-store = Geschäft
settings-folder-icon-office = Büro
settings-folder-icon-laptop = Laptop
settings-folder-icon-crown = Krone
settings-folder-icon-family = Familie
settings-folder-icon-group-general = Allgemein
settings-folder-icon-group-mail = E-Mail
settings-folder-icon-group-work = Arbeit
settings-folder-icon-group-finance = Finanzen und Einkauf
settings-folder-icon-group-travel = Reisen und Zuhause
settings-folder-icon-group-personal = Persönliches und Freizeit
settings-folder-icon-group-tech = Technik und Werkzeuge
settings-folder-icon-search-label = Symbole suchen
settings-folder-icon-no-match = Keine passenden Symbole

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = Authentifizierung (DMARC)

settings-badge-auth-help = Das DMARC-Ergebnis; Es werden die vollständigen Sicherheitsdetails geöffnet.

settings-badge-density = Abzeichengröße

settings-badge-density-comfortable = Bequem

settings-badge-density-compact = Kompakt

settings-badge-density-help = Kompakte oder komfortable Höhe und Abstände für jedes Abzeichen.

settings-badge-folder = Ordner

settings-badge-folder-help = Wo die Nachricht gespeichert ist. Nur wenn es auffällt, zeigt es sich für Junk and Trash.

settings-badge-labels = Abzeichenetiketten

settings-badge-labels-help = Symbol mit Text oder nur das Symbol (der vollständige Text bleibt im Tooltip und für Screenreader).

settings-badge-labels-icon = Nur Symbol

settings-badge-labels-text = Symbol und Text

settings-badge-list = Mailingliste

settings-badge-list-help = Die Mailingliste, von der die Nachricht stammt; Es öffnet die Abonnementdetails und -aktionen. Nur wenn bemerkenswert, wird es angezeigt, wenn weiterhin E-Mails eingehen, nachdem Sie sich abgemeldet haben.

settings-badge-mode-always = Immer

settings-badge-mode-never = Niemals

settings-badge-mode-notable = Nur wenn es bemerkenswert ist

settings-badge-sanitized = Desinfiziert HTML

settings-badge-sanitized-help = Was das Desinfektionsmittel HTML mit der Nachricht gemacht hat. Erst wenn bemerkenswert ist, wird angezeigt, wenn etwas blockiert wurde.

settings-badge-spam = Spam-Score

settings-badge-spam-help = Die Punktzahl, die der empfangende Server in die Nachrichtenheader geschrieben hat.

settings-folders-hide-empty = Leere Ordner in der Seitenleiste ausblenden

settings-folders-hide-empty-help = Ordner werden wieder angezeigt, wenn sie Nachrichten enthalten. Posteingang, Suchordner, Ordner mit unbekannter Anzahl und übergeordnete Ordner sichtbarer Ordner bleiben angezeigt. Überschreiben Sie einzelne Ordner unter Eigenschaften, Anzeige. Jeder Ordner bleibt in dieser Liste.

settings-folders-list-card = Ordnerliste und Reihenfolge

settings-header-auto-collapse = Falten Sie den Header nach dem Schreiben (Sekunden)

settings-header-auto-collapse-help = Wenn der Text so viele Sekunden lang den Fokus hatte und die Kopfzeile einen Empfänger und einen Betreff enthält, wird sie in die Zusammenfassungszeile eingeblendet. 0 schaltet dies aus.

settings-message-badges = Nachrichtenabzeichen

settings-message-badges-help = Wählen Sie aus, welche Abzeichen in der Kopfzeile einer geöffneten Nachricht angezeigt werden. Prioritätsmarkierungen werden unter Ordnern und Seitenleiste festgelegt. Sie können auch mit der rechten Maustaste auf ein Abzeichen klicken, um es auszublenden.

settings-reply-from-headers = Header, die die Identität auswählen

settings-reply-from-headers-help = Ein Header-Name pro Zeile, versucht in dieser Reihenfolge: Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc. Ein weggelassener Header wird nicht gelesen.

settings-reply-from-headers-invalid = Verwenden Sie nur diese Namen, einen pro Zeile: Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To, Cc.

settings-reply-from-headers-save = Kopfreihenfolge speichern

settings-sessions-linked-created = Link erstellt

settings-sessions-linked-details = Linkdetails

settings-sessions-linked-kind = Linktyp

settings-sessions-linked-kind-linked = Verknüpftes Postfach

settings-sessions-linked-last-used = Zuletzt verwendet

settings-sessions-linked-line = Zugriff auf verknüpfte Postfächer · ab { $origin }

settings-sessions-linked-line-hidden = Zugriff auf verknüpfte Postfächer

settings-sessions-linked-origin = Kommt von

settings-sessions-linked-removed = Unbekannt (Konto entfernt)

settings-sessions-linked-revoke = Linkzugriff widerrufen

settings-sessions-linked-revoke-label = Linkzugriff für { $mailbox } widerrufen

settings-sessions-linked-target = Postfach geöffnet

settings-sessions-origin-full = Vollständige Adresse

settings-sessions-origin-help = Steuert, wie diese Tabelle das Konto benennt, von dem eine verknüpfte Postfachsitzung stammt. Nur Sie sehen es in Ihren eigenen Sitzungen.

settings-sessions-origin-hidden = Versteckt

settings-sessions-origin-masked-local-part = Adresse mit maskiertem ersten Teil

settings-sessions-origin-setting = Zeigen Sie an, woher der Zugriff auf das verknüpfte Postfach kommt

settings-shortcuts-search = Suchverknüpfungen

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
cache-help-durable = Halten Sie kürzlich besuchte Ordner und Nachrichten zum Öffnen bereit. Standardmäßig bleiben Kopien im Speicher dieser Registerkarte. Aktivieren Sie die unbegrenzte Speicherung, um verschlüsselte Kopien auch beim erneuten Laden aufzubewahren. Durch das Vorladen werden E-Mails niemals als gelesen markiert oder entfernte Bilder geladen.

cache-keepIndefinitely = Bewahren Sie zwischengespeicherte E-Mails auf unbestimmte Zeit auf

cache-keepIndefinitely-help = Standardmäßig deaktiviert. Behalten Sie verschlüsselte Kopien in diesem Browser über Neuladevorgänge hinweg bei, ohne dass das Alter abläuft. Online-Nachrichten werden weiterhin aktualisiert. Die Eintrags- und Bytegrenzen gelten weiterhin. Durch Deaktivieren oder Löschen zwischengespeicherter E-Mails werden gespeicherte Kopien entfernt. Durch das Löschen von Browserdaten können sie ebenfalls entfernt werden. Anhänge werden nicht zur Offline-Verwendung heruntergeladen.

settings-notif-view-enabled = Benachrichtigungsansicht aktivieren
settings-notif-view-sidebar = Benachrichtigungen in der Seitenleiste anzeigen
settings-notif-view-help = Steuert die Benachrichtigungszentrale. Sicherheitshinweise, E-Mail-Hinweise und Browserbenachrichtigungen behalten ihre eigenen Einstellungen.
