# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-settings. Source: ../en/admin-settings.ftl
admin-settings-key-mail-allow-send = Senden erlauben
admin-settings-help-mail-allow-send = Benutzern das Senden von Nachrichten erlauben. Wenn aus, deaktiviert der Editor Senden und das zugehörige Tastenkürzel, und der Server lehnt Übermittlungen ab; Entwürfe können weiterhin gespeichert werden.
admin-settings-key-mail-allow-reply = Antworten erlauben
admin-settings-help-mail-allow-reply = Benutzern erlauben, in der Leseansicht und per Tastenkürzel zu antworten und allen zu antworten. Wenn aus, werden diese Aktionen nicht angeboten.
admin-settings-key-mail-allow-forward = Weiterleiten erlauben
admin-settings-help-mail-allow-forward = Benutzern erlauben, Nachrichten im Text oder als Anhang weiterzuleiten. Wenn aus, werden diese Aktionen nicht angeboten.
admin-settings-key-mail-allow-download = Downloads erlauben
admin-settings-help-mail-allow-download = Benutzern das Herunterladen von Nachrichten und Anhängen erlauben. Wenn aus, bietet die Web-App keinen Download an, und der Server lehnt den Nachrichtenexport ab.
admin-settings-key-mail-allow-print = Drucken erlauben
admin-settings-help-mail-allow-print = Benutzern erlauben, eine Nachricht aus der Leseansicht zu drucken. Wenn aus, wird die Aktion Drucken nicht angeboten.
admin-settings-key-mail-show-spam = Spam-Ordner anzeigen
admin-settings-help-mail-show-spam = Zeigt den Spam- oder Junk-Ordner und als Spam markierte Nachrichten an. Wenn aus, sind sie in der Ordnerliste und in Suchergebnissen ausgeblendet; es wird nichts gelöscht.
admin-settings-key-mail-show-auth-status = Ergebnisse der Absenderauthentifizierung anzeigen
admin-settings-help-mail-show-auth-status = Zeigt die SPF-, DKIM- und DMARC-Ergebnisse aus dem Header Authentication-Results in der Leseansicht an.
admin-settings-key-mail-close-on-delete = Nachricht beim Löschen schließen
admin-settings-help-mail-close-on-delete = Schließt die geöffnete Nachricht, den Tab oder das abgelöste Fenster, wenn diese Nachricht gelöscht oder in den Papierkorb verschoben wird. Wenn aus, behält die Leseansicht ihr bisheriges Verhalten.
admin-settings-key-links-trusted-senders = Vertrauenswürdige Linkabsender
admin-settings-help-links-trusted-senders = Exakte Absenderpostfächer, deren geprüfte Nachrichtenlinks ohne Zustimmung pro Nachricht geöffnet werden dürfen. Eine Adresse pro Eintrag.
admin-settings-key-appearance-font-size = Schriftgröße der Oberfläche
admin-settings-help-appearance-font-size = Schriftgröße der Oberfläche in Pixeln, von 12 bis 24.
admin-settings-key-appearance-font-weight = Schriftstärke der Oberfläche
admin-settings-help-appearance-font-weight = Grundstärke des Oberflächentexts: 300, 400, 500 oder 600. Hervorgehobener Text behält seine Hierarchie.
admin-settings-key-appearance-font-color = Textfarbe der Oberfläche
admin-settings-help-appearance-font-color = Textfarbe der Oberfläche als #RRGGBB. Leer lassen, um dem Design zu folgen.
admin-settings-key-appearance-low-contrast = Textkontrast abschwächen
admin-settings-help-appearance-low-contrast = Schwächt den Kontrast des Oberflächentexts ab.
admin-settings-key-mail-message-list-order = Reihenfolge der Nachrichtenliste
admin-settings-help-mail-message-list-order = Reihenfolge der Nachrichtenliste: neueste, älteste, Absender oder Betreff.
admin-settings-key-mail-message-list-style = Stil der Nachrichtenliste
admin-settings-help-mail-message-list-style = Die Nachrichtenliste als Karten oder als Zeilen mit Spaltenüberschriften anzeigen.
admin-settings-key-mail-message-list-preview = Vorschau in der Nachrichtenliste
admin-settings-help-mail-message-list-preview = Textauszüge in der Nachrichtenliste anzeigen.
admin-settings-key-mail-after-delete = Nach dem Löschen einer Nachricht
admin-settings-help-mail-after-delete = Welche Nachricht die Leseansicht nach dem Löschen der geöffneten Nachricht zeigt: die nächste, die vorherige oder keine.
admin-settings-key-reader-attachments-at-end = Anhänge nach dem Nachrichtentext
admin-settings-help-reader-attachments-at-end = Anhänge nach dem Nachrichtentext statt davor anzeigen.
admin-settings-key-notifications-ask-permission = Benachrichtigungserlaubnis anbieten
admin-settings-help-notifications-ask-permission = Benutzern auf Geräten, die Benachrichtigungen weder erlaubt noch blockiert haben, die Berechtigungsabfrage des Browsers anbieten.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-title = Einstellungen
admin-settings-description = Werte für den gesamten Server, pro Domain und pro Benutzer. Ein gesperrter Wert kann weder von Benutzern noch von enger gefassten Geltungsbereichen geändert werden.
admin-settings-list-caption = Einstellungen
admin-settings-search = Einstellungen durchsuchen
admin-settings-empty = Keine Einstellung entspricht der Suche.
admin-settings-unavailable = Dieser Server bietet das Einstellungsmodell noch nicht.
admin-settings-col-setting = Einstellung
admin-settings-col-value = Wert
admin-settings-col-source = Quelle
admin-settings-col-overrides = Engere Werte
admin-settings-open = Öffnen
admin-settings-overrides-none = Keine
admin-settings-overrides = { $domains } Domains, { $users } Benutzer
admin-settings-not-found = Es gibt keine Einstellung namens { $key }.
admin-settings-value-on = An
admin-settings-value-off = Aus
admin-settings-value-empty = Nicht festgelegt
admin-settings-value-list = { $count ->
    [one] { $count } Eintrag
   *[other] { $count } Einträge
}
admin-settings-detail-key = Schlüssel: { $key }
admin-settings-global = Für den gesamten Server
admin-settings-global-help = Dieser Wert gilt für alle Benutzer, es sei denn, eine Domain oder ein Benutzer hat einen engeren Wert oder der Benutzer wählt selbst, wo dies erlaubt ist.
admin-settings-value-label = Wert
admin-settings-json-hint = JSON. Der Server prüft den Wert und weist auf den Teil hin, den er ablehnt.
admin-settings-json-invalid = Das ist kein gültiges JSON.
admin-settings-user-override = Benutzer dürfen es ändern
admin-settings-user-override-help = Wenn an, können Benutzer in ihren Einstellungen einen eigenen Wert wählen. Wenn aus, gilt für sie der hier oder für ihre Domain festgelegte Wert.
admin-settings-lock-help = Wenn gesperrt, werden Domain-Werte und Benutzerwahlen ignoriert und dieser Wert gilt für alle.
admin-settings-save = Speichern
admin-settings-saved = Gespeichert. Der Wert gilt innerhalb von 30 Sekunden in jedem Serverprozess.
admin-settings-saved-restart = Gespeichert. Der Wert gilt nach einem Neustart.
admin-settings-dedicated = Diese Einstellung wird unter { $section } geändert, wo ihre Auswirkung zuerst geprüft wird.
admin-settings-dedicated-link = { $section } öffnen
admin-settings-pinned = Durch die Umgebungsvariable { $name } festgelegt. Der hier gespeicherte Wert gilt, wenn die Variable nicht gesetzt ist.
admin-settings-lock-title = { $setting } sperren?
admin-settings-lock-body = Beim Sperren werden die unten aufgeführten engeren Werte ignoriert. Sie bleiben erhalten und gelten wieder, wenn Sie die Sperre aufheben.
admin-settings-lock-confirm = Sperren
admin-settings-impact-domain = { $count ->
    [one] { $count } Domain-Wert wird ignoriert
   *[other] { $count } Domain-Werte werden ignoriert
}
admin-settings-impact-user = { $count ->
    [one] { $count } eigene Wahl eines Benutzers wird ignoriert
   *[other] { $count } eigene Wahlen von Benutzern werden ignoriert
}
admin-settings-impact-admin-user = { $count ->
    [one] { $count } Wert pro Benutzer wird ignoriert
   *[other] { $count } Werte pro Benutzer werden ignoriert
}
admin-settings-impact-profile = { $count ->
    [one] { $count } Wert eines Funktionsprofils wird ignoriert
   *[other] { $count } Werte von Funktionsprofilen werden ignoriert
}
admin-settings-impact-group = { $count ->
    [one] { $count } Gruppenwert wird ignoriert
   *[other] { $count } Gruppenwerte werden ignoriert
}
admin-settings-domains = Domain-Werte
admin-settings-domains-help = Ein Domain-Wert gilt für die Benutzer dieser Domain anstelle des Werts für den gesamten Server.
admin-settings-domains-caption = Domain-Werte von { $setting }
admin-settings-domains-empty = Keine Domain hat einen eigenen Wert.
admin-settings-domain = Domain
admin-settings-domain-value = Wert für diese Domain
admin-settings-domain-locked = Für die Benutzer der Domain gesperrt
admin-settings-domain-add = Domain-Wert hinzufügen
admin-settings-domain-save = Domain-Wert speichern
admin-settings-domain-remove = Entfernen
admin-settings-domain-remove-title = Wert von { $domain } entfernen?
admin-settings-domain-remove-body = Die Benutzer von { $domain } erhalten wieder den Wert für den gesamten Server.
admin-settings-domain-required = Geben Sie einen Domain-Namen ein.
admin-settings-locked-yes = Gesperrt
admin-settings-locked-no = Nicht gesperrt
admin-settings-users = Werte pro Benutzer
admin-settings-users-help = Ein Wert, den ein Administrator für einen Benutzer festgelegt hat. Bei Limits wird der Benutzer über seine Adresse benannt.
admin-settings-users-caption = Werte pro Benutzer von { $setting }
admin-settings-users-empty = Kein Benutzer hat einen von einem Administrator festgelegten Wert.
admin-settings-user = Benutzer
admin-settings-user-hint = Konto-ID oder, bei Limits, die Adresse.
admin-settings-user-value = Wert für diesen Benutzer
admin-settings-user-add = Wert pro Benutzer hinzufügen
admin-settings-user-save = Benutzerwert speichern
admin-settings-user-required = Geben Sie einen Benutzer ein.
admin-settings-user-remove-title = Wert von { $user } entfernen?
admin-settings-user-remove-body = Der Benutzer erhält den Wert seiner Domain oder den Wert für den gesamten Server.
admin-settings-source-default = Eingebauter Standard
admin-settings-source-envFallback = Umgebungsvariable (kein gespeicherter Wert)
admin-settings-source-envPin = Umgebungsvariable
admin-settings-source-global = Für den gesamten Server gespeichert
admin-settings-cat-interface = Oberfläche
admin-settings-cat-appearance = Erscheinungsbild
admin-settings-cat-mail = E-Mail
admin-settings-cat-folders = Ordnerlayout
admin-settings-cat-notifications = Benachrichtigungen
admin-settings-cat-offline = Offline-Nutzung
admin-settings-cat-assistant = KI-Assistent
admin-settings-cat-upstream = Mailserver
admin-settings-cat-signin = Anmeldung
admin-settings-cat-admin = Admin-Konsole
admin-settings-cat-passwords = Passwörter
admin-settings-cat-limits = Limits und Nutzung
admin-settings-cat-plugins = Plugins
admin-settings-cat-features = Funktionsberechtigungen
admin-settings-cat-integrations = Integrationsberechtigungen
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Erlaubt mit dem Konto verknüpfte Nextcloud-Dateien, Freigaben sowie Kalender-, Kontakt- und Aufgabenoperationen.
admin-settings-key-cap-integration-google = Google-Bridge
admin-settings-help-cap-integration-google = Erlaubt Operationen der Google-Bridge, einschließlich Token-Beschaffung und -Erneuerung.
admin-settings-key-cap-integration-microsoft = Microsoft-Bridge
admin-settings-help-cap-integration-microsoft = Erlaubt Operationen der Microsoft-Bridge, einschließlich Token-Beschaffung und -Erneuerung.
admin-settings-key-cap-integration-family = Einzelne Integration
admin-settings-help-cap-integration-family = Berechtigung für die Kennung einer einzelnen Integration; die Laufzeitabdeckung steht im Funktionskatalog.
admin-settings-key-cap-ui-plugin-family = Einzelnes UI-Plugin
admin-settings-help-cap-ui-plugin-family = Berechtigung für ein UI-Plugin. Verwenden Sie den genauen Richtlinienschlüssel aus dem Integrationskatalog.
admin-settings-key-cap-ai-assistant = KI-Assistent
admin-settings-help-cap-ai-assistant = Erlaubt Konfiguration und Aufruf des Assistenten.
admin-settings-key-cap-plugins = Plugins
admin-settings-help-cap-plugins = Erlaubt Operationen von UI- und Engine-Plugins.
admin-settings-key-cap-pim-calendar = Kalender
admin-settings-help-cap-pim-calendar = Erlaubt Kalendermethoden und Abonnements.
admin-settings-key-cap-pim-contacts = Kontakte
admin-settings-help-cap-pim-contacts = Erlaubt Kontakt- und Adressbuchmethoden.
admin-settings-key-cap-pim-tasks = Aufgaben
admin-settings-help-cap-pim-tasks = Erlaubt Aufgabenmethoden.
admin-settings-key-cap-pim-notes = Notizen
admin-settings-help-cap-pim-notes = Erlaubt Notizmethoden.
admin-settings-key-cap-export = Export
admin-settings-help-cap-export = Erlaubt Exporte von E-Mails und PIM-Daten.
admin-settings-key-cap-import = Import
admin-settings-help-cap-import = Erlaubt Importe von E-Mails und PIM-Daten.
admin-settings-key-cap-api-keys = API-Schlüssel
admin-settings-help-cap-api-keys = Richtlinie für API-Schlüssel. Die Abdeckung der Durchsetzung steht im Funktionskatalog.
admin-settings-key-cap-oauth-apps = OAuth-Anwendungen
admin-settings-help-cap-oauth-apps = Richtlinie für OAuth-Anwendungen. Die Abdeckung der Durchsetzung steht im Funktionskatalog.
admin-settings-key-cap-rules = E-Mail-Regeln
admin-settings-help-cap-rules = Erlaubt verwaltete Methoden für E-Mail-Regeln und Abwesenheitsnotizen.
admin-settings-key-cap-rules-forward = Regelweiterleitung
admin-settings-help-cap-rules-forward = Weiterleitungsrichtlinie. Die Abdeckung der Durchsetzung steht im Funktionskatalog.
admin-settings-key-cap-remote-images = Externe Bilder
admin-settings-help-cap-remote-images = Berechtigung für externe Bilder. Die bestehenden Datenschutzeinstellungen regeln weiterhin die Anfragen; prüfen Sie die Funktionsabdeckung.
admin-settings-key-cap-user-background = Eigener Hintergrund
admin-settings-help-cap-user-background = Berechtigung für eigene Hintergründe, standardmäßig deaktiviert. Prüfen Sie die Funktionsabdeckung.
admin-settings-key-cap-user-css = Eigenes CSS
admin-settings-help-cap-user-css = Berechtigung für eigenes CSS, standardmäßig deaktiviert. Prüfen Sie die Funktionsabdeckung.
admin-settings-key-limit-attachment-bytes = Größenlimit für Anhänge
admin-settings-help-limit-attachment-bytes = Anhangsrichtlinie in Bytes, Standard 50000000. Die Durchsetzung zur Laufzeit steht noch aus.
admin-settings-key-limit-send-per-hour = Sendelimit pro Stunde
admin-settings-help-limit-send-per-hour = Richtlinie für Einlieferungen pro Stunde; null bedeutet unbegrenzt. Die Durchsetzung zur Laufzeit steht noch aus.
admin-settings-key-limit-recipients-per-message = Empfänger pro Nachricht
admin-settings-help-limit-recipients-per-message = Richtlinie für die Empfängerzahl; null bedeutet unbegrenzt. Die Durchsetzung zur Laufzeit steht noch aus.
admin-settings-cat-import = Import
admin-settings-cat-webhooks = Webhooks
admin-settings-cat-realtime = Live-Updates
admin-settings-cat-privacy = Datenschutz
admin-settings-cat-observability = Beobachtbarkeit
admin-settings-cat-egress = Ausgehende Verbindungen
admin-settings-cat-updates = Updates
admin-settings-cat-backups = Backups
admin-settings-cat-about = Info
admin-settings-cat-config = Konfiguration
admin-settings-cat-jobs = Jobs
admin-settings-cat-maintenance = Wartung
admin-settings-cat-health = Zustandsprüfungen
admin-settings-cat-diagnostics = Diagnose
admin-settings-cat-users = Benutzer
admin-settings-cat-server-text = Servertext
admin-settings-key-ui-language = Sprache
admin-settings-help-ui-language = Die Sprache des Mail-Clients für Benutzer, die keine gewählt haben. „Wie der Browser“ folgt den Sprachen des jeweiligen Browsers.
admin-settings-key-ui-notify-new-version = Hinweis auf neue Version
admin-settings-help-ui-notify-new-version = An (Standard) zeigt angemeldeten Benutzern einen Hinweis mit „Jetzt neu laden“ und „Später“, wenn seit dem Laden ihrer Seite eine neuere Version der Web-App bereitgestellt wurde. Aus zeigt ihn nie; Benutzer erhalten dann den neuen Code beim nächsten Neuladen.
admin-settings-key-ui-undo-history-size = Größe des Verlaufs für Rückgängig und Wiederholen
admin-settings-help-ui-undo-history-size = Wie viele der letzten E-Mail-Aktionen (Archivieren, Löschen, Verschieben, als gelesen markieren, Markieren, Beschriften und Ähnliches) die Schaltflächen Rückgängig und Wiederholen sich merken, von 0 bis 50. 0 schaltet die Schaltflächen und das Kürzel Strg+Z aus. Der Standard ist 50. Benutzer können eine eigene Größe wählen, nie über dem unten stehenden Maximum, es sei denn, ein Administrator sperrt sie.
admin-settings-key-folders-reorder-enabled = Ordner per Drag-and-drop
admin-settings-help-folders-reorder-enabled = Erlaubt es, Ordner in der Seitenleiste und in den Einstellungen per Ziehen neu anzuordnen. Aus blendet die Ziehgriffe und die Verschieben-Befehle aus. Benutzer können es für sich ausschalten, sofern dies nicht gesperrt ist.
admin-settings-key-folders-reorder-allow-nesting = Ordnerverschachtelung
admin-settings-help-folders-reorder-allow-nesting = Erlaubt es, einen Ordner auf einen anderen zu ziehen, um ihn zu verschachteln, oder zurück auf die oberste Ebene. Aus belässt jeden Ordner unter seinem aktuellen übergeordneten Ordner und erlaubt nur die Neuanordnung unter Geschwistern.
admin-settings-key-folders-reorder-allow-server-move = Ordner auf dem Server verschieben
admin-settings-help-folders-reorder-allow-server-move = Lässt das Verschachteln oder Aufheben der Verschachtelung den echten übergeordneten Ordner auf dem Mailserver ändern, sodass die Änderung überall gilt. Aus behält diese Verschiebungen nur als Anordnung in dieser App.
admin-settings-key-folders-reorder-allow-system-folders = Systemordner verschieben
admin-settings-help-folders-reorder-allow-system-folders = Erlaubt es, Posteingang, Entwürfe, Gesendet, Spam, Papierkorb und Archiv per Ziehen neu anzuordnen. Aus hält sie oben fest. Sie werden nur in dieser App neu angeordnet, nie auf dem Server umbenannt oder verschoben.
admin-settings-key-folders-reorder-max-depth = Tiefe der Ordnerverschachtelung
admin-settings-help-folders-reorder-max-depth = Die tiefste Verschachtelungsebene, die ein Ziehen erzeugen darf, von 1 bis 16. Eine Verschiebung, die einen Ordner oder einen seiner Unterordner tiefer setzen würde, wird abgelehnt.
admin-settings-key-folders-order-locked = Ordnerreihenfolge festlegen
admin-settings-help-folders-order-locked = Fixiert die Ordnerreihenfolge für alle im Geltungsbereich. Ziehen und die Verschieben-Befehle sind aus, und Benutzer sehen einen Sperrhinweis. Anheftungen und die vom Administrator festgelegte Reihenfolge bleiben bestehen.
admin-settings-key-folders-pinned-admin = Erzwungen angeheftete Ordner
admin-settings-help-folders-pinned-admin = Ordner, die in einer Gruppe Angeheftet oben in der Seitenleiste jedes Benutzers angezeigt werden. Ein Eintrag pro Zeile, als role:inbox oder als Ordnerpfad wie Shared/Support. Benutzer können sie weder lösen noch verschieben. Ein Eintrag ohne passenden Ordner wird übersprungen.
admin-settings-key-folders-order-default = Standardreihenfolge der Ordner
admin-settings-help-folders-order-default = Die Reihenfolge, in der Ordner für Benutzer beginnen, die sie nicht angeordnet haben, vom ersten bis zum letzten. Ein Eintrag pro Zeile, als role:<role> oder als Ordnerpfad. Nicht aufgeführte Ordner folgen in der Reihenfolge des Servers. Die eigene Reihenfolge eines Benutzers ersetzt diese.
admin-settings-key-folders-new-folder-placement = Platzierung neuer Ordner
admin-settings-help-folders-new-folder-placement = Wo ein auf dem Server neu erscheinender Ordner unter seinen Geschwistern eingeordnet wird: am Ende oder in alphabetischer Reihenfolge.
admin-settings-key-ui-undo-history-max = Maximum des Verlaufs für Rückgängig und Wiederholen
admin-settings-help-ui-undo-history-max = Die meisten Aktionen, die ein Benutzer im Verlauf für Rückgängig und Wiederholen behalten darf, von 0 bis 50. Die eigene Größe eines Benutzers wird auf diesen Wert gesenkt. 0 schaltet den Verlauf für alle aus. Der Verlauf liegt im Browser-Tab und wird bei der Abmeldung gelöscht.
admin-settings-key-ui-bar-mail-top-bar = Mail-Symbolleiste
admin-settings-help-ui-bar-mail-top-bar = „pinned“ zeigt die Mail-Symbolleiste mit den Layout-Steuerungen immer an. „auto“ verbirgt sie hinter einem schmalen Griff, der sie öffnet, wenn ein Benutzer darauf zeigt, tippt oder mit der Tabulatortaste dorthin wechselt.
admin-settings-key-ui-bar-workspace-tabs = Arbeitsbereich-Tabs
admin-settings-help-ui-bar-workspace-tabs = „pinned“ zeigt die Tab-Leiste des Arbeitsbereichs immer an. „auto“ zeigt nur einen schmalen Griff mit dem Namen des aktiven Tabs, bis ein Benutzer sie öffnet.
admin-settings-key-ui-bar-mail-ribbon = Mail-Menüband
admin-settings-help-ui-bar-mail-ribbon = Für das Menüband-Layout. „pinned“ hält das Menüband geöffnet. „auto“ zeigt nur die Tab-Namen; ein Tab öffnet das Menüband über der Nachrichtenliste.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-ui-bar-composer-ribbon = Editor-Menüband
admin-settings-help-ui-bar-composer-ribbon = Für jeden Editor. „pinned“ hält das Editor-Menüband geöffnet. „auto“ zeigt nur die Tab-Namen; beim Öffnen eines Tabs wird der Nachrichtentext nach unten geschoben, statt verdeckt zu werden.
admin-settings-key-appearance-theme = Design
admin-settings-help-appearance-theme = Das Farbdesign, das Benutzer sehen, bis sie ein eigenes wählen. Wird es gesperrt, bleiben alle Benutzer bei diesem Design.
admin-settings-key-appearance-accent = Akzentfarbe
admin-settings-help-appearance-accent = Die Akzentfarbe des Designs. Leer bedeutet den eigenen Akzent des Designs.
admin-settings-key-appearance-badge-color = Farbe des Ungelesen-Zählers
admin-settings-help-appearance-badge-color = Die Farbe der Zähler für ungelesene Nachrichten. Standard behält die Farbe des Designs; Akzent folgt der Akzentfarbe; accent-lighter und accent-darker verwenden den Akzent 20 % heller oder dunkler, mit schwarzer oder weißer Zahl. Eine Sperre überschreibt die Wahl des Kontos.
admin-settings-key-appearance-density = Dichte
admin-settings-help-appearance-density = Wie viel Platz Listen und Steuerelemente einnehmen: kompakt, bequem oder entspannt.
admin-settings-key-appearance-font = Oberflächenschrift
admin-settings-key-packages-allowed = Paketverfolgung erlaubt
admin-settings-help-packages-allowed = Erlaubt die Paketverfolgung für die betroffenen Konten. Der Anbieter muss außerdem unter Paketverfolgung aktiviert sein, und jeder Benutzer muss zustimmen. Wird diese Einstellung deaktiviert, verhindert das sofort neue Anbieteraufrufe.
admin-settings-help-appearance-font = Wählen Sie eine mitgelieferte Schrift für Konten ohne Schriftwahl oder sperren Sie die Wahl für alle betroffenen Konten. Schriften werden von diesem Server geladen; es erfolgt keine Anfrage an Google Fonts. Standard verwendet die Schrift des Designs.
admin-settings-key-appearance-brand-name = Produktname
admin-settings-help-appearance-brand-name = Der Name, der auf der Anmeldeseite und im Browser-Tab angezeigt wird.
admin-settings-key-login-notice = Hinweis zur Anmeldung
admin-settings-help-login-notice = Ein kurzer Text auf der Anmeldeseite, zum Beispiel ein Support-Kontakt. Leer lassen, wenn keiner angezeigt werden soll.
admin-settings-key-mail-read-receipts = Lesebestätigungen
admin-settings-key-receipts-dsn-allowed = Zustellstatus-Anfragen
admin-settings-help-receipts-dsn-allowed = Erlaubt Konten, SMTP-Zustellstatusbenachrichtigungen anzufordern. Dies fordert nicht an und belegt nicht, dass eine Nachricht gelesen wurde.
admin-settings-help-mail-read-receipts = Was geschieht, wenn eine geöffnete Nachricht eine Lesebestätigung anfordert: den Benutzer fragen, nie senden oder immer senden.
admin-settings-key-notifications-enabled = Benachrichtigungen bei neuen E-Mails
admin-settings-help-notifications-enabled = Ob Benutzer Benachrichtigungen über neue E-Mails erhalten. Ihre Benachrichtigungsregeln bleiben unverändert.
admin-settings-key-offline-retention-days = Alter des Offline-Caches
admin-settings-help-offline-retention-days = Wie viele Tage ein Gerät Nachrichten für die Offline-Nutzung behält. 0 bedeutet keine Altersgrenze.
admin-settings-key-offline-budget-mb = Größe des Offline-Caches
admin-settings-help-offline-budget-mb = Wie viele Megabyte ein Gerät für Offline-E-Mails nutzen darf. 0 bedeutet keinen Offline-Cache.
admin-settings-key-assistant-enabled = KI-Assistent
admin-settings-help-assistant-enabled = Ob Benutzer den KI-Assistenten sehen. Der Bereitstellungsschalter unter KI-Assistent gilt weiterhin übergeordnet.
admin-settings-key-assist-profile = KI-Anbieterprofil
admin-settings-help-assist-profile = Die ID des Anbieterprofils, das diese Benutzer bedient (KI-Assistent > Anbieterprofile). Leer verwendet den Endpunkt der Bereitstellung. Eine nicht vorhandene Profil-ID schaltet den KI-Assistenten für diese Benutzer aus.
admin-settings-key-upstream-default = Standard-Mailserver
admin-settings-help-upstream-default = Der Mailserver, bei dem sich eine Adresse anmeldet, wenn keine Domain einen nennt. Die Domain-Tabelle hat Vorrang.
admin-settings-key-upstream-proxy-timeout-secs = Zeitlimit des Mailservers
admin-settings-help-upstream-proxy-timeout-secs = Wie lange eine Anfrage an den Mailserver dauern darf, in Sekunden.
admin-settings-key-signin-new-users = Neue Benutzer
admin-settings-help-signin-new-users = Wer bei der ersten Anmeldung ein Konto anlegen darf: jeder (offen), nur vorab hinzugefügte Adressen (vorab angelegt), diese plus importierte Benutzer (bereitgestellt) oder niemand Neues (geschlossen). Bestehende Konten sind nie betroffen. Wird in der Anmelderichtlinie geändert.
admin-settings-key-signin-new-users-mail = Passwort-Anmeldung: neue Benutzer
admin-settings-help-signin-new-users-mail = Regel für einen neuen Postfach-Benutzer, der sich mit einem Passwort anmeldet (IMAP, POP3, Exchange). Erben verwendet Neue Benutzer.
admin-settings-key-signin-new-users-sso = Single Sign-on: neue Benutzer
admin-settings-help-signin-new-users-sso = Regel für einen neuen Benutzer, der sich per Single Sign-on oder über einen vertrauenswürdigen Header anmeldet. Erben verwendet Neue Benutzer.
admin-settings-key-signin-new-users-linked = Verknüpfte Postfächer: neue Benutzer
admin-settings-help-signin-new-users-linked = Regel für das Verknüpfen eines Postfachs, das hier noch kein Konto hat, beurteilt nach der eigenen Domain des verknüpften Postfachs. Erben verwendet Neue Benutzer.
admin-settings-key-signin-new-users-linked-imap = Verknüpfte IMAP-Postfächer: neue Benutzer
admin-settings-help-signin-new-users-linked-imap = Regel nur für verknüpfte IMAP-Postfächer. Erben verwendet Verknüpfte Postfächer.
admin-settings-key-signin-new-users-linked-pop3 = Verknüpfte POP3-Postfächer: neue Benutzer
admin-settings-help-signin-new-users-linked-pop3 = Regel nur für verknüpfte POP3-Postfächer. Erben verwendet Verknüpfte Postfächer.
admin-settings-key-signin-new-users-linked-ews = Verknüpfte Exchange-Postfächer: neue Benutzer
admin-settings-help-signin-new-users-linked-ews = Regel nur für verknüpfte Exchange-Postfächer (EWS). Erben verwendet Verknüpfte Postfächer.
admin-settings-key-signin-preprovision-ttl-days = Lebensdauer vorab hinzugefügter Benutzer (Tage)
admin-settings-help-signin-preprovision-ttl-days = Wie lange ein vorab hinzugefügter ausstehender Benutzer gültig bleibt, wenn keine andere Lebensdauer angegeben ist.
admin-settings-key-signin-preprovision-max-ttl-days = Längste Lebensdauer vorab hinzugefügter Benutzer (Tage)
admin-settings-help-signin-preprovision-max-ttl-days = Die längste Lebensdauer, die ein Administrator einem vorab hinzugefügten ausstehenden Benutzer geben darf.
admin-settings-key-signin-preprovision-domain-wide = Domainweite vorab hinzugefügte Einträge erlauben
admin-settings-help-signin-preprovision-domain-wide = Erlaubt Einträge, die jeden einer ganzen Domain zulassen, bis zu einem Nutzungslimit. Standardmäßig aus.
admin-settings-key-signin-preprovision-retention-days = Abgelaufene ausstehende Benutzer aufbewahren (Tage)
admin-settings-help-signin-preprovision-retention-days = Wie lange ein abgelaufener ausstehender Benutzer aufbewahrt wird, bevor er entfernt wird.
admin-settings-key-signin-domain-mode = Anmelde-Domains
admin-settings-help-signin-domain-mode = Ob sich jede Domain anmelden darf oder nur aufgeführte Domains. Wird in der Anmelderichtlinie geändert.
admin-settings-key-signin-refusal-detail = Ablehnungsmeldungen
admin-settings-help-signin-refusal-detail = Ob eine abgelehnte Anmeldung den Grund nennt (deaktiviertes Konto, Adresse nicht erlaubt) oder dieselbe Antwort wie bei einem falschen Passwort gibt. Wird in der Anmelderichtlinie geändert.
admin-settings-key-signin-ldap-bind-precheck = LDAP-Prüfung vor der Anmeldung
admin-settings-help-signin-ldap-bind-precheck = Prüft das Passwort vor dem Mailserver gegen das Verzeichnis. Wird nach einem Test-Bind unter Integrationen geändert.
admin-settings-key-twofa-pending-ttl-secs = Zeitlimit für den zweiten Faktor
admin-settings-help-twofa-pending-ttl-secs = Wie lange ein Benutzer nach dem Passwort Zeit hat, den zweiten Faktor einzugeben, in Sekunden.
admin-settings-key-twofa-max-attempts = Versuche für den zweiten Faktor
admin-settings-help-twofa-max-attempts = Wie viele falsche Codes für den zweiten Faktor erlaubt sind, bevor die Anmeldung von vorn beginnt.
admin-settings-key-admin-session-idle-secs = Inaktivitäts-Timeout der Konsole
admin-settings-help-admin-session-idle-secs = Wie lange eine Sitzung der Admin-Konsole ohne Anfrage besteht, in Sekunden.
admin-settings-key-admin-session-absolute-secs = Sitzungslimit der Konsole
admin-settings-help-admin-session-absolute-secs = Die längste Dauer einer Sitzung der Admin-Konsole in Sekunden, aktiv oder nicht.
admin-settings-key-admin-session-rotate-secs = Rotation der Sitzungs-ID der Konsole
admin-settings-help-admin-session-rotate-secs = Wie oft, in Sekunden, die Admin-Konsole die Sitzungs-ID während der Nutzung ersetzt (0 = nur nach einer Step-up-Bestätigung). Die Rotation verlängert nie die Inaktivitäts- oder Gesamtlimits.
admin-settings-key-passwd-policy-min-length = Mindestlänge des Passworts
admin-settings-help-passwd-policy-min-length = Die geringste Zeichenzahl eines neuen Passworts.
admin-settings-key-passwd-policy-require-upper = Großbuchstaben verlangen
admin-settings-help-passwd-policy-require-upper = Ein neues Passwort muss einen Großbuchstaben enthalten.
admin-settings-key-passwd-policy-require-lower = Kleinbuchstaben verlangen
admin-settings-help-passwd-policy-require-lower = Ein neues Passwort muss einen Kleinbuchstaben enthalten.
admin-settings-key-passwd-policy-require-digit = Ziffer verlangen
admin-settings-help-passwd-policy-require-digit = Ein neues Passwort muss eine Ziffer enthalten.
admin-settings-key-passwd-policy-require-symbol = Sonderzeichen verlangen
admin-settings-help-passwd-policy-require-symbol = Ein neues Passwort muss ein Zeichen enthalten, das weder Buchstabe noch Ziffer ist.
admin-settings-key-limit-storage-bytes = Speicherlimit
admin-settings-help-limit-storage-bytes = Die meisten Bytes, die ein Benutzer auf diesem Server speichern darf. 0 bedeutet kein Limit.
admin-settings-key-limit-send-per-day = Gesendete Nachrichten pro Tag
admin-settings-help-limit-send-per-day = Die meisten Nachrichten, die ein Benutzer pro Tag senden darf. 0 bedeutet kein Limit.
admin-settings-key-limit-ai-calls-per-day = KI-Assistent-Aufrufe pro Tag
admin-settings-help-limit-ai-calls-per-day = Die meisten KI-Assistent-Anfragen, die ein Benutzer pro Tag stellen darf. 0 bedeutet kein Limit.
admin-settings-key-quota-warn-thresholds = Warnstufen der Nutzung
admin-settings-help-quota-warn-thresholds = Die Prozentwerte eines Limits, bei denen Benutzer gewarnt werden, der niedrigste zuerst.
admin-settings-key-quota-warn-enabled = Nutzungswarnungen
admin-settings-help-quota-warn-enabled = Ob Benutzer gewarnt werden, wenn sie sich einem Limit nähern.
admin-settings-key-usage-retention-days = Nutzungsverlauf
admin-settings-help-usage-retention-days = Wie viele Tage täglicher Nutzungszahlen aufbewahrt werden.
admin-settings-key-usage-meter-api = API-Nutzung zählen
admin-settings-help-usage-meter-api = Ob API-Anfragen in den Nutzungszahlen gezählt werden.
admin-settings-key-usage-measure-interval-hours = Messintervall des Speichers
admin-settings-help-usage-measure-interval-hours = Wie oft die gespeicherten Daten jedes Benutzers gemessen werden, in Stunden.
admin-settings-key-assistant-rate-limit-per-min = Ratenlimit des KI-Assistenten
admin-settings-help-assistant-rate-limit-per-min = Die meisten KI-Assistent-Anfragen pro Minute für den gesamten Server. 0 bedeutet kein Limit.
admin-settings-key-plugins-kv-max-key-bytes = Schlüsselgröße von Plugins
admin-settings-help-plugins-kv-max-key-bytes = Der längste Schlüssel, den ein Plugin speichern darf, in Bytes.
admin-settings-key-plugins-kv-max-value-bytes = Wertgröße von Plugins
admin-settings-help-plugins-kv-max-value-bytes = Der größte Wert, den ein Plugin unter einem Schlüssel speichern darf, in Bytes.
admin-settings-key-plugins-kv-max-total-bytes = Plugin-Speicher
admin-settings-help-plugins-kv-max-total-bytes = Die meisten Bytes, die ein Plugin insgesamt speichern darf.
admin-settings-key-plugins-kv-max-keys = Plugin-Schlüssel
admin-settings-help-plugins-kv-max-keys = Die meisten Schlüssel, die ein Plugin speichern darf.
admin-settings-key-import-max-messages = Nachrichtenzahl beim Import
admin-settings-help-import-max-messages = Die meisten Nachrichten, die ein Import aufnehmen darf.
admin-settings-key-import-max-message-bytes = Nachrichtengröße beim Import
admin-settings-help-import-max-message-bytes = Die größte Nachricht, die ein Import akzeptiert, in Bytes.
admin-settings-key-webhooks-max-attempts = Webhook-Versuche
admin-settings-help-webhooks-max-attempts = Wie oft eine Webhook-Zustellung versucht wird, bevor sie aufgegeben wird.
admin-settings-key-webhooks-timeout-secs = Webhook-Zeitlimit
admin-settings-help-webhooks-timeout-secs = Wie lange eine Webhook-Zustellung dauern darf, in Sekunden.
admin-settings-key-outbox-max-attempts = Sendeversuche
admin-settings-help-outbox-max-attempts = Wie oft eine Nachricht an den Mailserver übergeben wird, bevor sie als nicht gesendet gemeldet wird.
admin-settings-key-realtime-recheck-secs = Neuprüfung von Live-Updates
admin-settings-help-realtime-recheck-secs = Wie oft eine offene Live-Update-Verbindung prüft, ob ihre Sitzung noch gültig ist, in Sekunden.
admin-settings-key-privacy-watermark-enabled = Wasserzeichen für Leser
admin-settings-help-privacy-watermark-enabled = Zeigt über geöffneten Nachrichten ein Wasserzeichen mit der Adresse des Lesers.
admin-settings-key-privacy-watermark-opacity = Stärke des Wasserzeichens
admin-settings-help-privacy-watermark-opacity = Wie sichtbar das Wasserzeichen ist, von 0,02 (schwach) bis 0,5.
admin-settings-key-privacy-open-tracking-retention-days = Verlauf der Öffnungsverfolgung
admin-settings-help-privacy-open-tracking-retention-days = Wie viele Tage Einträge der Lesenachverfolgung aufbewahrt werden.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Nicht gesendete Verfolgungs-Tokens
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = Wie viele Stunden ein Verfolgungs-Token auf das Senden seiner Nachricht wartet, bevor er gelöscht wird.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-masked-email-domain = Domain für maskierte Adressen
admin-settings-help-masked-email-domain = Die Domain, unter der maskierte Adressen angelegt werden. Leer lassen, um maskierte Adressen auszuschalten.
admin-settings-key-abuse-report-address = Adresse für Missbrauchsmeldungen
admin-settings-help-abuse-report-address = Wohin Spam- und Missbrauchsmeldungen gesendet werden.
admin-settings-key-dlp-admin-address = Hinweise zu Datenverlust
admin-settings-help-dlp-admin-address = Wohin Hinweise der Datenverlustprävention gesendet werden.
admin-settings-key-observability-error-forward-url = Adresse zur Fehlerweiterleitung
admin-settings-help-observability-error-forward-url = Eine https-Adresse, an die Fehlerberichte von Clients weitergeleitet werden. Leer lassen für keine.
admin-settings-key-egress-probe-url = Testadresse für ausgehende Verbindungen
admin-settings-help-egress-probe-url = Die https-Adresse, mit der ein ausgehender Proxy getestet wird.
admin-settings-key-push-vapid-contact = Push-Kontakt
admin-settings-help-push-vapid-contact = Der Kontakt (mailto: oder https:), der Push-Diensten bei jeder Benachrichtigung mitgegeben wird.
admin-settings-key-push-quiet-hours = Ruhezeiten für Push
admin-settings-help-push-quiet-hours = Stunden (UTC), in denen keine Push-Benachrichtigung gesendet wird. Leer lassen für keine.
admin-settings-key-updates-check = Nach Updates suchen
admin-settings-help-updates-check = Ob der Server einmal täglich nach einer neueren Version sucht und sie im Dashboard anzeigt.
admin-settings-key-updates-feed-url = Release-Feed
admin-settings-help-updates-feed-url = Die https-Adresse, die die Update-Prüfung liest.
admin-settings-key-updates-apply = Updates anwenden
admin-settings-help-updates-apply = Ob ein Update über den Host-Update-Agenten aus der Konsole gestartet werden kann.
admin-settings-key-backup-retention-count = Aufbewahrte Backups
admin-settings-help-backup-retention-count = Wie viele Datenbank-Backups aufbewahrt werden.
admin-settings-key-about-security-contact = Sicherheitskontakt
admin-settings-help-about-security-contact = Der Kontakt in security.txt (mailto: oder https:).
admin-settings-key-about-security-policy-url = Sicherheitsrichtlinie
admin-settings-help-about-security-policy-url = Die Adresse Ihrer Sicherheitsrichtlinie, die in security.txt aufgeführt wird.
admin-settings-key-about-project-links = Links unter Info
admin-settings-help-about-project-links = Links, die auf der Seite Info angezeigt werden, jeweils mit Bezeichnung und https-Adresse.
admin-settings-key-config-snapshots-kept = Konfigurations-Snapshots
admin-settings-help-config-snapshots-kept = Wie viele Konfigurations-Snapshots aufbewahrt werden.
admin-settings-key-jobs-history-days = Job-Verlauf
admin-settings-help-jobs-history-days = Wie viele Tage an Job-Läufen aufbewahrt werden.
admin-settings-key-maintenance-announce-minutes-default = Vorlaufzeit der Wartungsankündigung
admin-settings-help-maintenance-announce-minutes-default = Wie viele Minuten vor einem Wartungsfenster Benutzer darüber informiert werden.
admin-settings-key-maintenance-default-message = Wartungsmeldung
admin-settings-help-maintenance-default-message = Der Text, den Benutzer während der Wartung sehen, pro Sprache.
admin-settings-key-health-disk-free-min-percent = Untergrenze für freien Speicherplatz
admin-settings-help-health-disk-free-min-percent = Die Bereitschaftsprüfung schlägt fehl, wenn der freie Speicherplatz unter diesen Prozentsatz fällt.
admin-settings-key-observability-otlp-metrics = Metriken per OTLP senden
admin-settings-help-observability-otlp-metrics = Sendet Servermetriken an den beim Start festgelegten OTLP-Collector.
admin-settings-key-metrics-drilldown-top-n = Detailzeilen der Metriken
admin-settings-help-metrics-drilldown-top-n = Wie viele Zeilen eine Aufschlüsselung der Metriken zeigt.
admin-settings-key-metrics-rollup-days = Metrikverlauf
admin-settings-help-metrics-rollup-days = Wie viele Tage stündlicher Metrikzusammenfassungen aufbewahrt werden.
admin-settings-key-observability-client-timing-enabled = Ladezeiten im Browser
admin-settings-help-observability-client-timing-enabled = Erfasst Seitenladezeiten angemeldeter Browser als anonyme aggregierte Histogramme. Nichts identifiziert eine Person, Sitzung oder Seite. Ausschalten beendet die Erfassung.
admin-settings-key-observability-client-timing-sample-rate-percent = Stichprobenrate der Browser-Ladezeiten
admin-settings-help-observability-client-timing-sample-rate-percent = Prozentsatz der Browsersitzungen, die Ladezeiten melden (1 bis 100).
admin-settings-key-observability-client-timing-min-samples = Mindestanzahl Stichproben für Ladezeiten
admin-settings-help-observability-client-timing-min-samples = Median und 95. Perzentil der Ladezeiten bleiben verborgen, bis ein Zeitraum mindestens so viele Stichproben hat.
admin-settings-key-observability-client-timing-folder-loads = Ladezeiten von Ordnern
admin-settings-help-observability-client-timing-folder-loads = Misst, wie lange ein Ordner braucht, um seine Nachrichtenliste anzuzeigen, getrennt danach, ob sie aus dem Browser-Cache oder aus dem Netzwerk kam. Standardmäßig an. Benutzer können es für sich ausschalten, sofern diese Einstellung nicht gesperrt ist. Erfordert, dass Ladezeiten im Browser an sind.
admin-settings-key-observability-client-timing-message-loads = Ladezeiten von Nachrichten
admin-settings-help-observability-client-timing-message-loads = Misst, wie lange eine Nachricht braucht, um ihre Header und den bereinigten Text anzuzeigen, getrennt danach, ob sie im Cache lag oder vorab geladen wurde oder aus dem Netzwerk kam. Standardmäßig an. Benutzer können es für sich ausschalten, sofern diese Einstellung nicht gesperrt ist. Erfordert, dass Ladezeiten im Browser an sind.
admin-settings-key-diagnostics-retention-days = Diagnoseverlauf
admin-settings-help-diagnostics-retention-days = Wie viele Tage an Diagnoseläufen aufbewahrt werden.
admin-settings-key-data-export-link-hours = Download-Zeitfenster für Datenexporte
admin-settings-help-data-export-link-hours = Wie viele Stunden ein Datenexport heruntergeladen werden kann, bevor er gelöscht wird.
admin-settings-key-dsr-enabled = Datenanfragen
admin-settings-help-dsr-enabled = Ob Personen in den Einstellungen Datenanfragen stellen können (Auskunft, Download, Berichtigung, Löschung, Einschränkung, Widerspruch).
admin-settings-key-dsr-type-access = Auskunftsanfragen annehmen
admin-settings-help-dsr-type-access = Ob die Anfrageart Auskunft den Benutzern angeboten wird.
admin-settings-key-dsr-type-portability = Download-Anfragen (portable Kopie) annehmen
admin-settings-help-dsr-type-portability = Ob die Anfrageart Download (portable Kopie) den Benutzern angeboten wird.
admin-settings-key-dsr-type-rectification = Berichtigungsanfragen annehmen
admin-settings-help-dsr-type-rectification = Ob die Anfrageart Berichtigung den Benutzern angeboten wird.
admin-settings-key-dsr-type-erasure = Löschanfragen annehmen
admin-settings-help-dsr-type-erasure = Ob die Anfrageart Löschung den Benutzern angeboten wird.
admin-settings-key-dsr-type-restriction = Anfragen zur Einschränkung annehmen
admin-settings-help-dsr-type-restriction = Ob die Anfrageart Einschränkung den Benutzern angeboten wird.
admin-settings-key-dsr-type-objection = Widerspruchsanfragen annehmen
admin-settings-help-dsr-type-objection = Ob die Anfrageart Widerspruch den Benutzern angeboten wird.
admin-settings-key-dsr-type-other = Sonstige Anfragen annehmen
admin-settings-help-dsr-type-other = Ob die allgemeine Anfrageart den Benutzern angeboten wird.
admin-settings-key-dsr-approval-export = Genehmigung von Auskunfts- und Download-Anfragen
admin-settings-help-dsr-approval-export = auto startet den Export, sobald die Anfrage bestätigt ist. admin wartet auf die Genehmigung durch einen Administrator.
admin-settings-key-dsr-approval-erasure = Genehmigung von Löschanfragen
admin-settings-help-dsr-approval-erasure = auto plant die Löschung, sobald die Anfrage bestätigt ist. admin wartet auf die Genehmigung durch einen Administrator.
admin-settings-key-dsr-approval-other = Bearbeitung sonstiger Anfragen
admin-settings-help-dsr-approval-other = auto markiert die Anfrage nach der Bestätigung als in Bearbeitung. admin wartet auf die Genehmigung durch einen Administrator.
admin-settings-key-dsr-reauth-required = Erneute Authentifizierung für Anfragen
admin-settings-help-dsr-reauth-required = Verlangt ein frisch eingegebenes Passwort oder einen zweiten Faktor, um eine Export- oder Löschanfrage zu stellen und einen Download-Link zu erstellen.
admin-settings-key-dsr-confirmation-required = Bestätigungsnachricht
admin-settings-help-dsr-confirmation-required = Sendet eine Bestätigungsnachricht mit einem Code an das Postfach des Anfragenden; die Anfrage wird erst fortgesetzt, wenn der Code eingegeben wurde.
admin-settings-key-dsr-identity-verification-required = Schritt zur Identitätsprüfung
admin-settings-help-dsr-identity-verification-required = Ein Administrator muss festhalten, dass die Identität des Anfragenden geprüft wurde, bevor die Anfrage genehmigt wird.
admin-settings-key-dsr-erasure-grace-days = Karenzzeit für Löschungen
admin-settings-help-dsr-erasure-grace-days = Tage zwischen Genehmigung und Löschung. Der Anfragende kann in dieser Zeit abbrechen. 0 führt die Löschung beim nächsten Worker-Durchlauf aus.
admin-settings-key-dsr-sla-days = Tage zur Beantwortung einer Anfrage
admin-settings-help-dsr-sla-days = Die Anzahl der Tage von der Einreichung bis zum Fälligkeitsdatum, das Benutzern und Administratoren angezeigt wird.
admin-settings-key-dsr-sla-max-extension-days = Längste Verlängerung
admin-settings-help-dsr-sla-max-extension-days = Die meisten Tage, die ein Administrator dem Fälligkeitsdatum einer Anfrage mit protokolliertem Grund hinzufügen kann.
admin-settings-key-dsr-sla-reminder-days = Markierungen vor dem Fälligkeitsdatum
admin-settings-help-dsr-sla-reminder-days = Tage vor dem Fälligkeitsdatum, ab denen eine Anfrage in der Warteschlange als bald fällig markiert wird.
admin-settings-key-dsr-export-mail-allowed = E-Mails in Exporten
admin-settings-help-dsr-export-mail-allowed = Erlaubt dem Anfragenden, Nachrichten in einen Export aufzunehmen. Es werden nur Nachrichten aus dem eigenen Cache dieses Servers aufgenommen; E-Mails, die auf dem vorgelagerten Mailserver bleiben, nicht.
admin-settings-key-dsr-export-mail-format = E-Mail-Format in Exporten
admin-settings-help-dsr-export-mail-format = mbox schreibt eine Datei pro Ordner; eml schreibt eine Datei pro Nachricht.
admin-settings-key-dsr-export-include-settings = Export: Einstellungen
admin-settings-help-dsr-export-include-settings = Einstellungen in einen Export aufnehmen.
admin-settings-key-dsr-export-include-identities = Export: Identitäten
admin-settings-help-dsr-export-include-identities = Identitäten in einen Export aufnehmen.
admin-settings-key-dsr-export-include-contacts = Export: Kontakte
admin-settings-help-dsr-export-include-contacts = Kontakte in einen Export aufnehmen.
admin-settings-key-dsr-export-include-calendars = Export: Kalender
admin-settings-help-dsr-export-include-calendars = Kalender in einen Export aufnehmen.
admin-settings-key-dsr-export-include-notes = Export: Notizen
admin-settings-help-dsr-export-include-notes = Notizen in einen Export aufnehmen.
admin-settings-key-dsr-export-include-rules = Export: Filter und Regeln
admin-settings-help-dsr-export-include-rules = Filter und Regeln in einen Export aufnehmen.
admin-settings-key-dsr-export-include-sessions = Export: Sitzungen, Geräte und Anmeldeverlauf
admin-settings-help-dsr-export-include-sessions = Sitzungen, Geräte und Anmeldeverlauf in einen Export aufnehmen.
admin-settings-key-dsr-export-include-consent = Export: Einwilligungsnachweise
admin-settings-help-dsr-export-include-consent = Einwilligungsnachweise in einen Export aufnehmen.
admin-settings-key-dsr-export-include-audit = Export: Audit-Einträge zur Person
admin-settings-help-dsr-export-include-audit = Audit-Einträge zur Person in einen Export aufnehmen.
admin-settings-key-dsr-export-max-megabytes = Größenobergrenze des Exports
admin-settings-help-dsr-export-max-megabytes = Größter Export in Megabyte. Abschnitte, die sie überschreiten würden, werden weggelassen und im Manifest aufgeführt.
admin-settings-key-dsr-export-format = Exportformate
admin-settings-help-dsr-export-format = json schreibt JSON-Dateien. jsonCsv schreibt für Listendaten zusätzlich CSV-Dateien.
admin-settings-key-dsr-export-encrypt = Exporte im Ruhezustand versiegeln
admin-settings-help-dsr-export-encrypt = Erstellte Exporte mit dem Serverschlüssel verschlüsselt speichern, bis sie heruntergeladen werden oder ablaufen.
admin-settings-key-dsr-download-link-minutes = Gültigkeit des Download-Links
admin-settings-help-dsr-download-link-minutes = Minuten, die ein Download-Link nach seiner Erstellung funktioniert. Der Export selbst wird für das Download-Zeitfenster für Datenexporte aufbewahrt.
admin-settings-key-dsr-download-single-use = Download-Link zur einmaligen Nutzung
admin-settings-help-dsr-download-single-use = Ein Download-Link funktioniert nach einmaliger Verwendung nicht mehr.
admin-settings-key-dsr-retention-completed-days = Aufbewahrung abgeschlossener Anfragen
admin-settings-help-dsr-retention-completed-days = Tage, die eine abgeschlossene Anfrage und ihr Verlauf aufbewahrt werden, bevor sie gelöscht werden.
admin-settings-key-dsr-notify-admins = Neue Anfragen für Admins protokollieren
admin-settings-help-dsr-notify-admins = Protokolliert jede neue Anfrage als Sicherheitsereignis im Audit-Protokoll, wo Admins und Alarmziele sie sehen.
admin-settings-key-dsr-notify-requester = Fortschrittsnachrichten an den Anfragenden
admin-settings-help-dsr-notify-requester = Legt eine Nachricht im Postfach des Anfragenden ab, wenn sich der Status einer Anfrage ändert.
admin-settings-key-dsr-contact-name = Name des Datenschutzkontakts
admin-settings-help-dsr-contact-name = Name, der Benutzern auf der Seite für Datenanfragen angezeigt wird.
admin-settings-key-dsr-contact-email = Adresse des Datenschutzkontakts
admin-settings-help-dsr-contact-email = E-Mail-Adresse, die Benutzern auf der Seite für Datenanfragen angezeigt wird.
admin-settings-key-dsr-contact-url = Seite des Datenschutzkontakts
admin-settings-help-dsr-contact-url = Link, der Benutzern auf der Seite für Datenanfragen angezeigt wird.
admin-settings-key-dsr-on-behalf-allowed = Anfragen im Namen eines Benutzers
admin-settings-help-dsr-on-behalf-allowed = Erlaubt einem Administrator, eine Anfrage für einen Benutzer zu stellen, zum Beispiel eine per Post eingegangene.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-dsr-rate-limit-per-day = Anfragen pro Person und Tag
admin-settings-help-dsr-rate-limit-per-day = Die meisten Anfragen, die eine Person in 24 Stunden stellen kann.
admin-settings-key-dsr-legal-hold-subjects = Rechtssperren
admin-settings-help-dsr-legal-hold-subjects = Adressen oder @domain-Einträge, deren Löschung blockiert ist. Eine Löschanfrage für sie bleibt als blockiert in der Warteschlange, bis der Eintrag entfernt wird.
admin-settings-key-scrub-enabled = Personenbereinigung
admin-settings-help-scrub-enabled = Erlaubt die Aktion Person bereinigen und die Bereinigung, die auf eine Löschung folgen kann. Ausgeschaltet läuft keines von beiden.
admin-settings-key-scrub-pseudonym-algorithm = Länge des Pseudonyms
admin-settings-help-scrub-pseudonym-algorithm = Wie das schlüsselabhängige Pseudonym entsteht, das eine Kennung ersetzt: HMAC-SHA256 unter der aktuellen Schlüsselgeneration, gekürzt auf 16 oder 32 Hex-Zeichen.
admin-settings-key-scrub-include-logs = Logdateien bereinigen
admin-settings-help-scrub-include-logs = Eine Bereinigung schreibt die Kennung auch in den rotierten Logdateien um, komprimierte eingeschlossen.
admin-settings-key-scrub-auto-run-on-erasure = Bereinigung nach Löschung
admin-settings-help-scrub-auto-run-on-erasure = Eine abgeschlossene Löschung ersetzt zusätzlich die Kennungen der Person in Audit-Verlauf und Logs durch das Pseudonym. Standardmäßig aus.
admin-settings-key-scrub-key-retention-days = Aufbewahrung der Pseudonym-Schlüssel
admin-settings-help-scrub-key-retention-days = Tage, die ein Pseudonym-Schlüssel nach seiner Rotation aufbewahrt wird. 0 behält ihn. Nach dem Löschen lassen sich alte Pseudonyme nicht mehr neu berechnen, und mit dem Schlüssel erstellte Tombstones können nicht mehr erneut angewendet werden.
admin-settings-key-backup-propagate-erasure = Löschungen bei Wiederherstellung erneut anwenden
admin-settings-help-backup-propagate-erasure = Eine Wiederherstellung wendet die Lösch-Tombstones, die neuer als das Backup sind, erneut an, sodass seither gelöschte Personen wieder entfernt werden.
admin-settings-key-backup-tombstone-retention-days = Aufbewahrung von Lösch-Tombstones
admin-settings-help-backup-tombstone-retention-days = Tage, die ein Lösch-Tombstone aufbewahrt wird. 0 behält sie für immer.
admin-settings-key-dsr-text-intro = Einleitung der Datenanfragen
admin-settings-help-dsr-text-intro = Text über dem Anfrageformular. Ein Wert pro Sprache.
admin-settings-key-dsr-text-confirmation = Text der Bestätigungsnachricht
admin-settings-help-dsr-text-confirmation = Text, der der Bestätigungsnachricht hinzugefügt wird. Ein Wert pro Sprache.
admin-settings-key-dsr-text-completion = Text der Abschlussnachricht
admin-settings-help-dsr-text-completion = Text, der der Nachricht hinzugefügt wird, die beim Abschluss einer Anfrage gesendet wird. Ein Wert pro Sprache.
admin-settings-key-server-text-family = Servertext
admin-settings-help-server-text-family = Text, den der Server in E-Mails und Benachrichtigungen schreibt, pro Sprache und Domain.
admin-settings-cat-sieve = E-Mail-Filter
admin-settings-key-sieve-max-script-bytes = Maximale Größe des Filterskripts
admin-settings-help-sieve-max-script-bytes = Maximale Bytes in einem einzelnen Sieve-Filterskript.
admin-settings-key-sieve-max-scripts = Maximale Anzahl Filterskripte
admin-settings-help-sieve-max-scripts = Maximale Anzahl gespeicherter Skripte pro Konto.
admin-settings-key-sieve-max-redirects = Maximale Weiterleitungen
admin-settings-help-sieve-max-redirects = Maximale Anzahl von Redirect-Befehlen in einem Skript.
admin-settings-key-sieve-vacation-allowed = Abwesenheitsaktionen in Filtern
admin-settings-help-sieve-vacation-allowed = Erlaubt Skripten, Abwesenheitsantworten zu senden, wenn das Backend sie unterstützt.
admin-settings-key-sieve-allowed-extensions = Erlaubte Filtererweiterungen
admin-settings-help-sieve-allowed-extensions = Eine optionale Liste von Sieve-Erweiterungsnamen in Kleinbuchstaben. Das Backend muss jede Erweiterung ebenfalls unterstützen.
admin-settings-key-cap-sieve-raw-edit = Quelltext des Filterskripts bearbeiten
admin-settings-help-cap-sieve-raw-edit = Erlaubt Roh-Bearbeitungen von Sieve-Skripten für die ausgewählten Konten.
admin-settings-key-cap-sieve-forward-external = Außerhalb der Konto-Domain weiterleiten
admin-settings-help-cap-sieve-forward-external = Erlaubt externe Weiterleitung in Skripten. Das Speichern dieser Skripte erfordert außerdem eine aktuelle Verifizierung.
admin-settings-key-session-max-concurrent = Gleichzeitige Sitzungen
admin-settings-help-session-max-concurrent = Maximale Sitzungen pro Konto. Die ältesten Sitzungen werden widerrufen, wenn eine neue Anmeldung dieses Limit überschreitet.
admin-settings-key-session-idle-secs = Inaktivitäts-Timeout der Sitzung
admin-settings-help-session-idle-secs = Beendet inaktive Sitzungen nach dieser Anzahl Sekunden, höchstens bis zum Limit der Bereitstellung.
admin-settings-key-session-absolute-secs = Lebensdauer der Sitzung
admin-settings-help-session-absolute-secs = Beendet Sitzungen nach dieser Anzahl Sekunden, höchstens bis zum Limit der Bereitstellung.
admin-settings-key-twofa-remember-days = Lebensdauer gemerkter Geräte
admin-settings-help-twofa-remember-days = Tage, bevor ein gemerktes Gerät den zweiten Faktor erneut bestätigen muss. Null deaktiviert diese Funktion.
admin-settings-key-twofa-passkeys-enabled = Passkeys erlauben
admin-settings-help-twofa-passkeys-enabled = Erlaubt Passkeys als Authentifizierungsfaktor.
admin-settings-key-twofa-required = Zwei-Faktor-Authentifizierung verlangen
admin-settings-help-twofa-required = Verlangt einen eingerichteten zweiten Faktor, bevor eine Sitzung ausgestellt wird.
admin-settings-key-twofa-passkey-attachment = Authenticator-Typ für Passkeys
admin-settings-help-twofa-passkey-attachment = any (Standard): jeder Authenticator. platform: auf dem Gerät gespeicherte Passkeys (Windows Hello, Touch ID, Android-Bildschirmsperre). cross-platform: Sicherheitsschlüssel und Smartphones. Der Browser meldet dies; es ist nicht vom Authenticator signiert.
admin-settings-key-twofa-passkey-user-verification = Benutzerverifizierung für Passkeys
admin-settings-help-twofa-passkey-user-verification = required: Der Passkey muss beim Hinzufügen und bei jeder Anmeldung eine PIN oder Biometrie prüfen; ein Vorgang ohne Prüfung wird abgelehnt. preferred (Standard) und discouraged ändern nur, was vom Browser angefordert wird.
admin-settings-key-twofa-passkey-resident-key = Auffindbare Passkeys
admin-settings-help-twofa-passkey-resident-key = required: Die Anmeldedaten müssen auf dem Authenticator gespeichert werden. discouraged (Standard) oder preferred. Im Engine-Modus sind Passkeys immer auffindbar, weil sie auch ohne Passwort anmelden.
admin-settings-key-twofa-passkey-legacy = Vor der aktuellen Richtlinie hinzugefügte Passkeys
admin-settings-help-twofa-passkey-legacy = allow (Standard): Ein älterer Passkey, der die Richtlinie nicht erfüllt, funktioniert weiter, und der Benutzer wird gebeten, ihn zu ersetzen. refuse: Er wird bei der Anmeldung mit einer Meldung abgelehnt; der Benutzer meldet sich mit einem anderen Faktor oder einem Wiederherstellungscode an.
admin-settings-key-twofa-last-factor-removal = Entfernen des letzten zweiten Faktors
admin-settings-help-twofa-last-factor-removal = Wenn das Entfernen eines Passkeys oder einer Authenticator-App das Konto ohne zweiten Faktor ließe. allow (Standard): Er wird entfernt und die Anmeldung braucht nur das Passwort. block: Der Benutzer muss zuerst eine andere Anmeldemethode hinzufügen (Wiederherstellungscodes zählen nicht). Wo Zwei-Faktor-Authentifizierung verlangt wird, bleibt der letzte immer erhalten.
admin-settings-key-privacy-remote-content = Externe Inhalte
admin-settings-help-privacy-remote-content = Bilder blockieren, zuerst fragen, den Proxy verwenden oder direkt laden, wenn erlaubt.
admin-settings-key-privacy-trackers = Tracker blockieren
admin-settings-help-privacy-trackers = Blockiert bekannte Tracking-Bilder in E-Mails.
admin-settings-key-privacy-clean-links = Links bereinigen
admin-settings-help-privacy-clean-links = Entfernt Tracking-Parameter aus Links in Nachrichten.
admin-settings-key-privacy-max-bytes = Limit für externe Bilder
admin-settings-help-privacy-max-bytes = Maximale Bytes, die für ein externes Bild abgerufen werden.
admin-settings-key-privacy-cache-bytes = Cache für externe Bilder
admin-settings-help-privacy-cache-bytes = Maximale Bytes, die im Cache für externe Bilder gehalten werden.
admin-settings-key-privacy-rate-per-min = Anfragerate für externe Bilder
admin-settings-help-privacy-rate-per-min = Maximale Abrufe externer Bilder pro Konto und Minute.
admin-settings-privacy-fetch-title = Abruf externer Bilder
admin-settings-privacy-fetch-help = Zeitlimits, Weiterleitungen und parallele Downloads für Bilder, die der Server für den Leser abruft. Ein Bild über einem Limit zeigt einen Platzhalter.
admin-settings-key-privacy-fetch-connect-timeout-ms = Verbindungs-Timeout für externe Bilder
admin-settings-help-privacy-fetch-connect-timeout-ms = Millisekunden, die zum Öffnen der Verbindung zu einem Bild-Host erlaubt sind, einschließlich des TLS-Handshakes. Standard 5000.
admin-settings-key-privacy-fetch-headers-timeout-ms = Antwort-Timeout für externe Bilder
admin-settings-help-privacy-fetch-headers-timeout-ms = Millisekunden vom Beginn einer Anfrage, bis der Bild-Host mit Headern antwortet. Standard 10000.
admin-settings-key-privacy-fetch-idle-timeout-ms = Stillstands-Timeout für externe Bilder
admin-settings-help-privacy-fetch-idle-timeout-ms = Längste Pause in Millisekunden zwischen zwei Teilen eines Bild-Downloads, bevor er abgebrochen wird. Standard 10000.
admin-settings-key-privacy-fetch-total-timeout-ms = Gesamt-Timeout für externe Bilder
admin-settings-help-privacy-fetch-total-timeout-ms = Millisekunden, die für den Abruf eines Bildes erlaubt sind, einschließlich Weiterleitungen. Das Warten in der Warteschlange hat ein eigenes Budget, sodass ein ausgelastetes Bild seine Netzwerkzeit nicht verbraucht, bevor es beginnt. Standard 20000.
admin-settings-key-privacy-fetch-queue-timeout-ms = Warteschlangen-Timeout für externe Bilder
admin-settings-help-privacy-fetch-queue-timeout-ms = Maximale Millisekunden, die Bilder zusätzlich zu ihrem Abrufbudget auf Kapazität warten dürfen. Die begrenzte Warteschlange verkraftet große authentifizierte Lastspitzen, während die Limits für aktive Abrufe und Dekodierung weiter gelten. Standard 60000.
admin-settings-key-privacy-max-redirects = Weiterleitungen für externe Bilder
admin-settings-help-privacy-max-redirects = Weiterleitungen, denen für ein Bild gefolgt wird. Jede wird nach denselben Adressregeln geprüft wie die erste Anfrage. Standard 3.
admin-settings-key-privacy-max-concurrent-per-message = Downloads externer Bilder pro Nachricht
admin-settings-help-privacy-max-concurrent-per-message = Bilder einer Nachricht, die gleichzeitig abgerufen werden. Die übrigen warten auf einen freien Platz. Bereich 1–128; Standard 30.
admin-settings-key-privacy-max-concurrent-per-account = Downloads externer Bilder pro Konto
admin-settings-help-privacy-max-concurrent-per-account = Bilder, die für ein Konto über alle seine geöffneten Nachrichten hinweg gleichzeitig abgerufen werden. Bereich 1–256; Standard 128.
admin-settings-key-privacy-allow-direct = Direkte Bilder erlauben
admin-settings-key-cap-remote-content-direct = Berechtigung für direkte Bilder
admin-settings-help-cap-remote-content-direct = Erlaubt diesem Konto, direkte Bilder zu wählen, wenn der Server dies zulässt.
admin-settings-help-privacy-allow-direct = Erlaubt das direkte Laden von Bildern für Konten mit der erforderlichen Berechtigung.
admin-settings-key-logs-auth-retention-days = Aufbewahrung des Authentifizierungsverlaufs
admin-settings-help-logs-auth-retention-days = Tage, die Metadaten zur Authentifizierung aufbewahrt werden, unabhängig von der dauerhaften Kontoidentität.
admin-settings-key-logs-auth-max-entries = Eintragslimit des Authentifizierungsverlaufs
admin-settings-help-logs-auth-max-entries = Maximal aufbewahrte Einträge. Gleiche wiederholte Fehlschläge innerhalb einer Minute teilen sich einen Eintrag.
admin-settings-key-reader-color-mode = Nachrichtenfarben
admin-settings-cat-reader = Nachrichtenanzeige
admin-settings-key-mail-row-actions-mode = Schnellaktionen der Nachrichtenliste
admin-settings-help-mail-row-actions-mode = Wie die Schnellaktionen in jeder Nachrichtenzeile erscheinen: open (immer verfügbar, bei Hover oder Fokus eingeblendet), hover (drei Steuerelemente und eine Schaltfläche Mehr), click (eine Schaltfläche, die sie öffnet) oder off (aus den Zeilen entfernt; Kontextmenü und Symbolleiste behalten jede Aktion). Hover verhält sich auf Touchscreens wie click. Benutzer können selbst wählen, sofern ein Administrator dies nicht sperrt.
admin-settings-key-mail-quick-reply = Schnellantwort
admin-settings-help-mail-quick-reply = Bietet einen kompakten Antwort-Tab mit bereits ausgefüllten Empfängern. Standardmäßig aus; Benutzer können ihn aktivieren, sofern ein Administrator dies nicht sperrt. Die üblichen Sendeschutzmaßnahmen gelten weiterhin.
admin-settings-key-mail-request-read-receipt = Lesebestätigungen standardmäßig anfordern
admin-settings-help-mail-request-read-receipt = Beginnt unterstützte ausgehende Nachrichten mit ausgewählter Bestätigungsanfrage. Dies ist eine Voreinstellung beim Verfassen, keine Garantie, dass der Empfänger eine Bestätigung sendet.
admin-settings-help-reader-color-mode = Automatisch folgt dunklen Designs. Wählen Sie Original-, adaptive oder invertierte Darstellung im isolierten Nachrichtenrahmen.
admin-settings-key-reader-font-size = Schriftgröße der Nachricht
admin-settings-help-reader-font-size = Standard-Lesegröße in Pixeln, von 12 bis 24.
admin-settings-key-reader-line-height = Zeilenhöhe der Nachricht
admin-settings-help-reader-line-height = Zeilenabstand beim Lesen von 1,2 bis 2,2.
admin-settings-key-reader-content-width = Lesebreite der Nachricht
admin-settings-help-reader-content-width = Bequeme, breite oder die volle Bereichsbreite füllende Nachrichteninhalte.
admin-settings-key-reader-loading = Darstellung beim Laden von Nachrichten
admin-settings-help-reader-loading = Platzhalterzeilen, eine Textanzeige oder keine visuelle Anzeige. Ansagen für die Barrierefreiheit bleiben verfügbar.
admin-settings-key-reader-loading-delay-ms = Verzögerung beim Laden von Nachrichten
admin-settings-help-reader-loading-delay-ms = Verzögert nur die Ladeanzeige, zwischen 0 und 1000 Millisekunden. Die Zustellung der Nachricht wird nie verzögert.
admin-settings-key-reader-zoom = Zoom der Nachricht
admin-settings-help-reader-zoom = Standardzoom des Nachrichtentexts in Prozent, von 50 bis 200; der Leser verwendet seine nächstgelegene Zoomstufe. Nur die Nachricht wird skaliert, und beim Drucken gelten 100 %.
admin-settings-key-reader-attachments-collapsed = Anhangsleiste eingeklappt
admin-settings-help-reader-attachments-collapsed = Ob die Anhangsleiste des Lesers eingeklappt startet und nur die Anzahl und Gesamtgröße zeigt. Das letzte Öffnen oder Schließen des Benutzers wird hier gespeichert. Standardmäßig aus: Die Anhänge werden angezeigt.
admin-settings-key-login-show-domain-hint = Bevorzugte Domain bei der Anmeldung anzeigen
admin-settings-help-login-show-domain-hint = Zeigt, welche Domain einen Benutzernamen ohne E-Mail-Suffix akzeptiert. Standardmäßig verborgen. Dies ändert nur den Hinweis; die Anmeldung mit einem Benutzernamen funktioniert weiterhin.
admin-settings-key-signin-retry-cooldown-secs = Wartezeit zwischen Anmeldeversuchen
admin-settings-help-signin-retry-cooldown-secs = Mindestsekunden zwischen Passwort-Anmeldeversuchen aus einem Client-Netzwerk. Standard 3; 0 deaktiviert die Wartezeit. Bestehende Sperren nach Fehlschlägen gelten weiterhin.
admin-settings-key-mail-quote-style = Header-Stil bei Antwort und Weiterleitung
admin-settings-key-mail-external-recipient-warning = Warnung bei externen Empfängern
admin-settings-help-mail-external-recipient-warning = Aus, warnen oder im Editor eine Bestätigung verlangen. Umfasst An, Cc und Bcc. Dies ist eine Schutzmaßnahme im Editor, keine Einschränkung der Serverzustellung.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-mail-trusted-recipient-domains = Vertrauenswürdige Empfänger-Domains
admin-settings-help-mail-trusted-recipient-domains = Bis zu 100 exakte Empfänger-Domains. Auch die Domain der gewählten Absenderidentität gilt als vertrauenswürdig. Ungültige Einträge schaffen nie Vertrauen. Sperren Sie diese Einstellung, um Überschreibungen durch Benutzer zu verhindern.
admin-settings-help-mail-quote-style = Verwenden Sie klassisch zitierte Antworten oder Nachrichten-Header im Outlook-Stil. Dies legt den Standard der Benutzer fest; sperren Sie ihn, um den Stil durchzusetzen. Bestehende Entwürfe bleiben unverändert.
admin-settings-key-signin-anti-bot-mode = Bot-Schutz bei der Anmeldung
admin-settings-help-signin-anti-bot-mode = Selbst gehostete Proof-of-Work-Prüfung: off, mailbox, admin oder all. Es wird kein externer Prüfdienst kontaktiert.
admin-settings-key-signin-anti-bot-difficulty = Rechenaufwand der Anmeldeprüfung
admin-settings-help-signin-anti-bot-difficulty = Maximaler Zählerstand des Lösers, 100–20000. Höhere Werte erfordern mehr Arbeit im Browser; Standard 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Gültigkeitsdauer der Anmeldeprüfung
admin-settings-help-signin-anti-bot-expiry-secs = Sekunden, die eine einmalige Prüfung gültig bleibt, 60–600; Standard 180. Nach jedem Anmeldeversuch ist eine neue Prüfung nötig.
admin-settings-key-attachments-preview-allowed = Anhangsvorschauen erlauben
admin-settings-help-attachments-preview-allowed = Deaktivieren, um die Darstellung von Anhangsvorschauen zu verhindern; Downloads der Originale haben eine eigene Steuerung.
admin-settings-key-attachments-download-allowed = Auslieferung des Original-Anhangs erlauben
admin-settings-help-attachments-download-allowed = Das Deaktivieren verhindert auch Vorschauen, die die Original-Bytes benötigen. Browser-Vorschauen sind kein Kopierschutz.
admin-settings-key-attachments-max-bytes = Maximale Anhangsgröße in Bytes
admin-settings-help-attachments-max-bytes = Null bedeutet unbegrenzt. Gilt für ausgelieferte Original-Bytes, einschließlich Range-Anfragen.
admin-settings-key-attachments-allowed-extensions = Erlaubte Dateiendungen
admin-settings-help-attachments-allowed-extensions = JSON-Array von Endungen in Kleinbuchstaben ohne Punkt, zum Beispiel ["pdf","tar.gz"]. Leer erlaubt jede Endung; Sperrregeln haben Vorrang.
admin-settings-key-attachments-blocked-extensions = Gesperrte Dateiendungen
admin-settings-help-attachments-blocked-extensions = JSON-Array von Endungen in Kleinbuchstaben. Geprüft wird gegen gespeicherte MIME-Dateinamen, nie gegen den Namen in der Download-URL.
admin-settings-key-attachments-warn-extensions = Warnung bei Dateiendungen
admin-settings-help-attachments-warn-extensions = JSON-Array von Endungen, die vor der Auslieferung des Originals eine ausdrückliche Bestätigung erfordern.
admin-settings-key-attachments-allowed-mime-types = Erlaubte MIME-Typen
admin-settings-help-attachments-allowed-mime-types = JSON-Array von Typen in Kleinbuchstaben oder Wildcards der obersten Ebene, zum Beispiel ["image/*","application/pdf"]. Leer erlaubt jeden Typ.
admin-settings-key-attachments-blocked-mime-types = Gesperrte MIME-Typen
admin-settings-help-attachments-blocked-mime-types = JSON-Array von Typen in Kleinbuchstaben oder Wildcards der obersten Ebene. Sperrregeln haben Vorrang.
admin-settings-key-attachments-warn-mime-types = Warnung bei MIME-Typen
admin-settings-help-attachments-warn-mime-types = JSON-Array von Typen in Kleinbuchstaben oder Wildcards der obersten Ebene, die eine Bestätigung erfordern. MIME-Metadaten sind keine Malware-Erkennung.
admin-settings-cat-shortcuts = Tastenkürzel
admin-settings-cat-time = Zeitzonen
admin-settings-key-time-display-zone = Anzeige-Zeitzone
admin-settings-help-time-display-zone = IANA-Zeitzone wie Europe/Lisbon oder local, um dem aktuellen Gerät zu folgen. Zeitstempel von Nachrichten bleiben als ihre ursprünglichen Zeitpunkte gespeichert.
admin-settings-key-time-comparison-zone = Vergleichs-Zeitzone
admin-settings-help-time-comparison-zone = Optionale zweite IANA-Zeitzone für Hilfen bei Nachrichten und Kalender. Leer lassen, um den Vergleich auszublenden; aus einer E-Mail-Adresse wird kein Ort abgeleitet.
admin-settings-key-time-show-helpers = Zeitzonenhilfen anzeigen
admin-settings-help-time-show-helpers = Zeigt Vergleiche zwischen lokaler und gewählter Zeitzone für Zeitstempel von Nachrichten und Kalenderereignisse. Admin-Sperren können diese Wahl erzwingen.
admin-settings-key-shortcuts-enabled = Tastenkürzel aktivieren
admin-settings-help-shortcuts-enabled = Erlaubt Tastenkürzel der Anwendung. Sperren können sie global oder für eine Domain deaktivieren. Eingabefelder und modale Dialoge behalten ihre normale Tastaturbehandlung.
admin-settings-key-shortcuts-compose = Tastenkürzel Verfassen
admin-settings-help-shortcuts-compose = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-archive = Tastenkürzel Archivieren
admin-settings-help-shortcuts-archive = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-reply = Tastenkürzel Antworten
admin-settings-help-shortcuts-reply = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-reply-all = Tastenkürzel Allen antworten
admin-settings-help-shortcuts-reply-all = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-forward = Tastenkürzel Weiterleiten
admin-settings-help-shortcuts-forward = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-next = Tastenkürzel Nächste Nachricht
admin-settings-help-shortcuts-next = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-previous = Tastenkürzel Vorherige Nachricht
admin-settings-help-shortcuts-previous = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-search = Tastenkürzel Suche fokussieren
admin-settings-help-shortcuts-search = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-refresh = Tastenkürzel Nachrichten aktualisieren
admin-settings-help-shortcuts-refresh = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-settings = Tastenkürzel Einstellungen öffnen
admin-settings-help-shortcuts-settings = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-trash = Tastenkürzel In den Papierkorb verschieben
admin-settings-help-shortcuts-trash = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-mark-read = Tastenkürzel Als gelesen markieren
admin-settings-help-shortcuts-mark-read = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-mark-unread = Tastenkürzel Als ungelesen markieren
admin-settings-help-shortcuts-mark-unread = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-spam = Tastenkürzel Als Spam markieren
admin-settings-help-shortcuts-spam = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-zoom-in = Tastenkürzel Nachricht vergrößern
admin-settings-help-shortcuts-zoom-in = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-zoom-out = Tastenkürzel Nachricht verkleinern
admin-settings-help-shortcuts-zoom-out = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-zoom-reset = Tastenkürzel Nachrichtenzoom zurücksetzen
admin-settings-help-shortcuts-zoom-reset = Verwenden Sie inherit für die gewählte Voreinstellung, disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-shortcuts-open-in-tab = Tastenkürzel Nachricht in eigenem Tab öffnen
admin-settings-help-shortcuts-open-in-tab = Verwenden Sie inherit für die gewählte Voreinstellung (Shift+Enter), disabled, um diese Aktion zu entfernen, oder eine Tastenkombination wie Ctrl+Shift+K. Kollidierende oder ungültige Tastenkombinationen werden nicht ausgeführt.
admin-settings-key-notifications-push-allowed = Push-Benachrichtigungen im Hintergrund erlaubt
admin-settings-help-notifications-push-allowed = Erlaubt den aktivierten Benachrichtigungen dieses Kontos, sich für Hintergrund-Push zu registrieren und ihn zu empfangen. Die Browser-Berechtigung ist weiterhin erforderlich.
admin-settings-key-offline-persistent-storage-allowed = Anfragen für dauerhaften Browser-Speicher erlaubt
admin-settings-help-offline-persistent-storage-allowed = Erlaubt eine ausdrückliche Anfrage, lokal zwischengespeicherte Daten vor automatischer Verdrängung durch den Browser zu schützen. Der Browser kann ablehnen; die Berechtigung wird nicht automatisch erteilt.
admin-settings-key-security-signin-mailbox-failure-limit = Limit fehlgeschlagener Postfach-Anmeldedaten
admin-settings-help-security-signin-mailbox-failure-limit = Abgelehnte Anmeldedaten von einer Quell-IP innerhalb des Zählfensters lösen eine vorübergehende Sperre der Postfach-Anmeldung aus. Netzwerk- und Konfigurationsfehler zählen nicht.
admin-settings-key-security-signin-admin-failure-limit = Limit fehlgeschlagener Administrator-Anmeldedaten
admin-settings-help-security-signin-admin-failure-limit = Abgelehnte Administrator-Anmeldedaten von einer Quell-IP lösen eine eigene vorübergehende Sperre der Admin-Anmeldung aus. Die kurze Admin-Sitzung und die Anforderungen an frische Nachweise bleiben unverändert.
admin-settings-key-security-signin-failure-window-secs = Zählfenster für fehlgeschlagene Anmeldedaten
admin-settings-help-security-signin-failure-window-secs = Gleitendes Fenster in Sekunden für Ablehnungen von Anmeldedaten und MFA. Eine erfolgreich abgeschlossene Anmeldung setzt diese Quelle und Oberfläche zurück.
admin-settings-key-security-signin-lock-secs = Dauer der vorübergehenden Anmeldesperre
admin-settings-help-security-signin-lock-secs = Sperrdauer in Sekunden. Wiederholte Versuche während der Sperre verlängern ihren ursprünglichen Ablauf nicht. Manuelle globale IP-Sperren sind getrennt.
admin-settings-key-security-signin-geo-scope = Geltungsbereich der Länderrichtlinie bei der Anmeldung
admin-settings-help-security-signin-geo-scope = Standardmäßig aus. Gilt nur für neue Postfach-, Administrator- oder alle Anmeldungen. Bestehende Sitzungen werden nicht widerrufen. Prüfen Sie vor dem Aktivieren die Diagnose zur aktuellen Quelle; eine nicht verfügbare lokale GeoIP-Datenbank lehnt Anmeldungen ab.
admin-settings-key-security-signin-geo-mode = Länder-Erlaubnis- oder -Sperrliste
admin-settings-help-security-signin-geo-mode = allow erlaubt nur aufgeführte Länder; deny lehnt aufgeführte Länder ab. Unbekannte Adressen folgen ihrer eigenen ausdrücklichen Richtlinie.
admin-settings-key-security-signin-geo-countries = Ländercodes für die Anmeldung
admin-settings-help-security-signin-geo-countries = JSON-Array zweistelliger Ländercodes in Großbuchstaben, zum Beispiel ["PT", "GB"]. Verwendet nur die lokale Länderdatenbank (Sicherheit > Länderdatenbank oder MW_GEOIP_DB); Adressen werden nie an einen Ortungsdienst gesendet.
admin-settings-key-security-signin-geo-unknown = Richtlinie für nicht zuordenbare Quelladressen
admin-settings-help-security-signin-geo-unknown = Erlaubt oder verweigert Adressen, die in einer lesbaren lokalen Datenbank fehlen, einschließlich privater Adressen. Fehlende oder nicht lesbare Datenbanken lehnen Anmeldungen unter aktiver Richtlinie immer ab, auch wenn unbekannte Adressen erlaubt sind.
admin-settings-key-contacts-avatar-mode = Kontaktbilder und generierte Avatare
admin-settings-help-contacts-avatar-mode = Initialen lokal erzeugen, Avatare ausblenden oder eingebettete Fotos aus den eigenen Adressbüchern des Kontos verwenden. Es werden weder Foto-URLs noch ein externer Avatar-Dienst kontaktiert. Die Richtlinie für Benutzer und Domains kann von einem Administrator festgelegt werden.
admin-settings-cat-contacts = Kontaktbilder
admin-settings-key-reader-action-style = Aktionsleiste für Nachrichten
admin-settings-help-reader-action-style = Kompakte Symbole sind der Standard. Wählen Sie labels für Symbole mit sichtbarem Text; beide behalten Aktionsnamen und gestaltete Tooltips. Unterstützt globale, Domain- und Benutzersperren.
admin-settings-key-nextcloud-sync-interval-secs = Intervall der automatischen Nextcloud-Aktualisierung
admin-settings-help-nextcloud-sync-interval-secs = Sekunden zwischen Hintergrundaktualisierungen der Sammlungen für verbundene Konten mit nativer Engine. Null bedeutet nur manuell; positive Werte haben ein Minimum von sechzig Sekunden. Bei Fehlern wird zunehmend gewartet. Es wird keine Hintergrundanmeldung versucht.
admin-settings-key-mail-folder-role-inbox = Zuordnung des Posteingangsordners
admin-settings-help-mail-folder-role-inbox = Null verwendet die automatische Erkennung. Verwenden Sie ein Objekt mit exaktem Pfad für gemeinsame Standards oder ein Objekt mit einer eigenen Postfach-ID für ein Konto. Fehlende oder widersprüchliche Ziele deaktivieren diese Rolle; Administratorsperren gelten.
admin-settings-key-mail-folder-role-drafts = Zuordnung des Entwurfsordners
admin-settings-help-mail-folder-role-drafts = Null verwendet die automatische Erkennung. Verwenden Sie ein Objekt mit exaktem Pfad für gemeinsame Standards oder ein Objekt mit einer eigenen Postfach-ID für ein Konto. Fehlende oder widersprüchliche Ziele deaktivieren diese Rolle; Administratorsperren gelten.
admin-settings-key-mail-folder-role-sent = Zuordnung des Ordners Gesendet
admin-settings-help-mail-folder-role-sent = Null verwendet die automatische Erkennung. Verwenden Sie ein Objekt mit exaktem Pfad für gemeinsame Standards oder ein Objekt mit einer eigenen Postfach-ID für ein Konto. Fehlende oder widersprüchliche Ziele deaktivieren diese Rolle; Administratorsperren gelten.
admin-settings-key-mail-folder-role-trash = Zuordnung des Papierkorbs
admin-settings-help-mail-folder-role-trash = Null verwendet die automatische Erkennung. Verwenden Sie ein Objekt mit exaktem Pfad für gemeinsame Standards oder ein Objekt mit einer eigenen Postfach-ID für ein Konto. Fehlende oder widersprüchliche Ziele deaktivieren diese Rolle; Administratorsperren gelten.
admin-settings-key-mail-folder-role-junk = Zuordnung des Spam-Ordners
admin-settings-help-mail-folder-role-junk = Null verwendet die automatische Erkennung. Verwenden Sie ein Objekt mit exaktem Pfad für gemeinsame Standards oder ein Objekt mit einer eigenen Postfach-ID für ein Konto. Fehlende oder widersprüchliche Ziele deaktivieren diese Rolle; Administratorsperren gelten.
admin-settings-key-mail-folder-role-archive = Zuordnung des Archivordners
admin-settings-help-mail-folder-role-archive = Null verwendet die automatische Erkennung. Verwenden Sie ein Objekt mit exaktem Pfad für gemeinsame Standards oder ein Objekt mit einer eigenen Postfach-ID für ein Konto. Fehlende oder widersprüchliche Ziele deaktivieren diese Rolle; Administratorsperren gelten.
admin-settings-key-mail-request-delivery-receipt = Zustellbenachrichtigungen standardmäßig anfordern
admin-settings-help-mail-request-delivery-receipt = Wählt Zustellbenachrichtigungen für neue Nachrichten aus, wenn das Konto SMTP-DSN unterstützt. Die Sperre steuert diesen gespeicherten Standard; receipts.dsnAllowed regelt getrennt, ob Anfragen erlaubt sind. Eine Zustellung belegt kein Lesen.
admin-settings-key-security-signin-geo-enforce-sessions = Länderrichtlinie auf bestehende Sitzungen anwenden
admin-settings-help-security-signin-geo-enforce-sessions = Standardmäßig aus. Wendet den konfigurierten Länder-Geltungsbereich der Anmeldung auf authentifizierte Anfragen, nativen Bearer-Zugriff und signierte Leser-Inhalte an. Blockierte Clients können sich weiterhin abmelden. Bereits offene Streams werden beim erneuten Verbinden geprüft; bestehende Verbindungen werden dadurch nicht beendet.
admin-settings-key-security-protection-rate-mode = Aggregierte Anfragebudgets
admin-settings-help-security-protection-rate-mode = Standardmäßig aus. observe protokolliert erschöpfte Budgets ohne Ablehnung; enforce liefert ein vorübergehendes 429. Bestehende Parallelitätslimits der Protokolle bleiben unabhängig.
admin-settings-key-security-protection-adaptive-mode = Adaptiver Anfrageschutz
admin-settings-help-security-protection-adaptive-mode = Standardmäßig aus. observe protokolliert erklärbare typisierte Anomalien und vorgeschlagene Aktionen; enforce wendet begrenzte vorübergehende Drosselungen oder Sperren an. Dies ist eine deterministische Zählerrichtlinie, kein KI-Klassifikator.
admin-settings-key-security-protection-client-requests = Client-Anfragen pro Fenster
admin-settings-help-security-protection-client-requests = Anfragen pro Fenster von einer nicht angemeldeten Adresse sowie jede Anmeldeanfrage. Eine Anfrage mit gültiger Postfach- oder Admin-Sitzung nutzt stattdessen ihr eigenes, größeres Kontingent. Null schaltet dieses Budget aus.
admin-settings-key-security-protection-account-requests = Konto-Anfragen pro Fenster
admin-settings-help-security-protection-account-requests = Anfragen pro Fenster für ein angemeldetes Postfach über alle seine Sitzungen, Geräte und Adressen. Liegt über dem anonymen Kontingent, damit angemeldete Nutzung mehr Spielraum hat. Null schaltet dieses Budget aus.
admin-settings-key-rate-limit-diagnostics-burst = Diagnoseläufe gleichzeitig
admin-settings-help-rate-limit-diagnostics-burst = Diagnoseläufe und Backend-Prüfungen, die eine Admin-Sitzung hintereinander starten darf. Das Starten einer bereits laufenden Prüfung liefert diese Prüfung zurück, statt erneut zu zählen.
admin-settings-key-rate-limit-diagnostics-per-minute = Diagnoseläufe pro Minute
admin-settings-help-rate-limit-diagnostics-per-minute = Diagnoseläufe und Backend-Prüfungen pro Minute, die eine Admin-Sitzung zurückerhält.
admin-settings-key-security-protection-admin-requests = Admin-Anfragen pro Fenster
admin-settings-help-security-protection-admin-requests = Anfragen pro Fenster für einen angemeldeten Administrator. Liegt über dem anonymen Kontingent. Null schaltet dieses Budget aus.
admin-settings-key-security-protection-rate-window-secs = Fenster des Anfragebudgets (Sekunden)
admin-settings-help-security-protection-rate-window-secs = Festes Intervall des Anfragebudgets. Lastabwurf bei Kapazitätsengpässen und gewöhnliche Nicht-gefunden-Antworten gelten nicht als adaptiver Beleg.
admin-settings-key-security-protection-adaptive-clients = Clients für adaptiven Schutz beobachten
admin-settings-help-security-protection-adaptive-clients = Aktiviert diese Subjektdimension, wenn der adaptive Modus aktiv ist. Konten und Administratoren werden erst nach Verifizierung ihrer Identität gezählt.
admin-settings-key-security-protection-adaptive-accounts = Konten für adaptiven Schutz beobachten
admin-settings-help-security-protection-adaptive-accounts = Aktiviert diese Subjektdimension, wenn der adaptive Modus aktiv ist. Konten und Administratoren werden erst nach Verifizierung ihrer Identität gezählt.
admin-settings-key-security-protection-adaptive-admins = Admins für adaptiven Schutz beobachten
admin-settings-help-security-protection-adaptive-admins = Aktiviert diese Subjektdimension, wenn der adaptive Modus aktiv ist. Konten und Administratoren werden erst nach Verifizierung ihrer Identität gezählt.
admin-settings-key-security-protection-short-window-secs = Kurzes Anomaliefenster (Sekunden)
admin-settings-help-security-protection-short-window-secs = Gleitendes Beobachtungsfenster. Lange Fenster unterstützen Stunden bis dreißig Tage; durch die Rundung auf Intervalle kann höchstens ein zusätzliches Vierundsechzigstel eines Fensters erhalten bleiben.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protection-short-minimum-samples = Mindeststichproben für kurzes Verhältnis
admin-settings-help-security-protection-short-minimum-samples = Mindestzahl beobachteter normaler plus auffälliger Anfragen, bevor der Verhältnis-Auslöser greift. Der absolute Anomalieschwellenwert ist von diesem Minimum unabhängig.
admin-settings-key-security-protection-short-absolute-anomalies = Absolutes Anomaliebudget (kurz)
admin-settings-help-security-protection-short-absolute-anomalies = Löst aus, wenn sich diese Anzahl ausgewählter Anomalien ansammelt, auch wenn normale Anfragen das Verhältnis verwässern. Null deaktiviert diesen Auslöser.
admin-settings-key-security-protection-short-ratio-percent = Anomalieverhältnis (kurz, Prozent)
admin-settings-help-security-protection-short-ratio-percent = Löst aus, wenn ausgewählte Anomalien nach der Mindeststichprobenzahl diesen Prozentsatz erreichen. Der absolute oder der Verhältnis-Auslöser genügt. Null deaktiviert diesen Auslöser.
admin-settings-key-security-protection-long-window-secs = Langes Anomaliefenster (Sekunden)
admin-settings-help-security-protection-long-window-secs = Gleitendes Beobachtungsfenster. Lange Fenster unterstützen Stunden bis dreißig Tage; durch die Rundung auf Intervalle kann höchstens ein zusätzliches Vierundsechzigstel eines Fensters erhalten bleiben.
admin-settings-key-security-protection-long-minimum-samples = Mindeststichproben für langes Verhältnis
admin-settings-help-security-protection-long-minimum-samples = Mindestzahl beobachteter normaler plus auffälliger Anfragen, bevor der Verhältnis-Auslöser greift. Der absolute Anomalieschwellenwert ist von diesem Minimum unabhängig.
admin-settings-key-security-protection-long-absolute-anomalies = Absolutes Anomaliebudget (lang)
admin-settings-help-security-protection-long-absolute-anomalies = Löst aus, wenn sich diese Anzahl ausgewählter Anomalien ansammelt, auch wenn normale Anfragen das Verhältnis verwässern. Null deaktiviert diesen Auslöser.
admin-settings-key-security-protection-long-ratio-percent = Anomalieverhältnis (lang, Prozent)
admin-settings-help-security-protection-long-ratio-percent = Löst aus, wenn ausgewählte Anomalien nach der Mindeststichprobenzahl diesen Prozentsatz erreichen. Der absolute oder der Verhältnis-Auslöser genügt. Null deaktiviert diesen Auslöser.
admin-settings-key-security-protection-normal-cap-per-bucket = Obergrenze der Normal-Basislinie pro Intervall
admin-settings-help-security-protection-normal-cap-per-bucket = Begrenzt die in jedem Intervall des gleitenden Fensters gelernte normale Basislinie. Unabhängige absolute Anomaliebudgets verhindern, dass unbegrenztes Auffüllen mit gültigen Anfragen Missbrauch verbirgt.
admin-settings-key-security-protection-signals = Signale für adaptive Belege
admin-settings-help-security-protection-signals = Wählen Sie maßgebliche Beobachtungen: fehlerhaft, übergroß, Anmeldedaten abgelehnt, Berechtigung verweigert, Body-Frist oder optional Fehlzugriffe auf Admin-Routen. Die Erkundung von Admin-Routen schließt Antworten auf nicht gefundene Entitäten und statische Assets aus; allgemeine Statuscodes, Kapazitätsablehnungen und User-Agent-Zeichenfolgen belegen keinen Missbrauch.
admin-settings-key-security-protection-throttle-secs = Erste vorübergehende Drosselung (Sekunden)
admin-settings-help-security-protection-throttle-secs = Das erste Überschreiten des Schwellenwerts startet eine feste Drosselungsfrist. Anfragen während dieser Frist verlängern sie nicht.
admin-settings-key-security-protection-block-secs = Eskalierte vorübergehende Sperre (Sekunden)
admin-settings-help-security-protection-block-secs = Ein späteres Überschreiten des Schwellenwerts vor der Erholung kann zu dieser begrenzten Sperre eskalieren. Es wird keine dauerhafte Kontosperre erzeugt.
admin-settings-key-security-protection-decay-secs = Ruhiges Erholungsintervall (Sekunden)
admin-settings-help-security-protection-decay-secs = Nach diesem anomaliefreien Intervall und dem Ablauf der aktuellen Maßnahme kehrt die Eskalation zur Anfangsstufe zurück. Gleitende Belege verfallen gemäß ihren konfigurierten Fenstern.
admin-settings-key-security-protection-exempt-clients = Ausgenommene Client-Adressen und -Netzwerke
admin-settings-help-security-protection-exempt-clients = Exakte IP-Adressen oder CIDR-Netzwerke nehmen nur die Client-Dimension aus. Verifizierte Kontolimits gelten weiterhin. Mit Bedacht für gemeinsam genutzte NAT-Netzwerke verwenden; weitergeleitete Adressen werden nur über konfigurierte Proxys als vertrauenswürdig behandelt.
admin-settings-key-security-protection-exempt-accounts = Ausgenommene verifizierte Konto-IDs
admin-settings-help-security-protection-exempt-accounts = Exakte vom Server verifizierte Postfach-Konto-IDs nehmen nur ihre Konto-Dimension aus, nie die Dimension des Quell-Clients. E-Mail-Adressen und vom Aufrufer gelieferte Kontoparameter sind kein Identitätsnachweis.
admin-settings-key-security-protection-exempt-admins = Ausgenommene Administrator-IDs
admin-settings-help-security-protection-exempt-admins = Exakte authentifizierte Administrator-IDs nehmen nur ihre Administrator-Dimension aus. Bestehende Sperren nach fehlgeschlagenen Anmeldungen und strenge Fristen der Administratorsitzung bleiben unverändert.
admin-settings-key-security-protection-storage-failure = Richtlinie bei Speicherfehlern der Durchsetzung
admin-settings-help-security-protection-storage-failure = Closed lehnt geschützte Arbeit ab, wenn ein aktiviertes Durchsetzungsbudget nicht geprüft werden kann. Open erlaubt sie vorübergehend mit einer begrenzten betrieblichen Warnung. Fehler im reinen Beobachtungsmodus erzwingen nie eine Maßnahme.
admin-settings-key-security-admin-probe-enabled = Schutz vor Admin-Erkundung
admin-settings-help-security-admin-probe-enabled = Zählt Anfragen, die nach der Admin-Konsole suchen, pro Client-Netzwerk (IPv4-Adresse, IPv6 /64), verlangsamt eine Quelle, die weitermacht, und sperrt sie dann. Tippfehler und veraltete Lesezeichen bleiben im Toleranzrahmen. Wird unter Sicherheit > Adaptiver Schutz geändert.
admin-settings-key-security-admin-probe-grace-allowance = Toleranzrahmen (Fehlzugriffe pro Fenster)
admin-settings-help-security-admin-probe-grace-allowance = Pro Fenster ignorierte Fehlzugriffe. Der Wert sinkt kontinuierlich um so viele Punkte pro Fenster.
admin-settings-key-security-admin-probe-window-secs = Toleranzfenster (Sekunden)
admin-settings-help-security-admin-probe-window-secs = Das Fenster, auf das der Toleranzrahmen angewendet wird.
admin-settings-key-security-admin-probe-block-after = Fehlzugriffe über der Toleranz bis zur Sperre
admin-settings-help-security-admin-probe-block-after = Fehlzugriffe über dem Toleranzrahmen, die eine Sperre auslösen. Dazwischenliegende Fehlzugriffe werden nach der Tarpit-Verzögerung beantwortet.
admin-settings-key-security-admin-probe-tarpit-delay-ms = Tarpit-Verzögerung (ms)
admin-settings-help-security-admin-probe-tarpit-delay-ms = Verzögerung, die jedem gezählten Fehlzugriff über dem Toleranzrahmen hinzugefügt wird. 0 schaltet den Tarpit aus. Andere Anfragen werden nie verzögert.
admin-settings-key-security-admin-probe-block-secs = Erste Sperre (Sekunden)
admin-settings-help-security-admin-probe-block-secs = Dauer der ersten Sperre. Eine gesperrte Quelle erhält für jeden Pfad dieselbe Antwort.
admin-settings-key-security-admin-probe-max-block-secs = Längste Sperre (Sekunden)
admin-settings-help-security-admin-probe-max-block-secs = Eine wiederholte Sperre verdoppelt die vorherige Dauer, bis zu dieser Obergrenze.
admin-settings-key-security-admin-probe-repeat-window-secs = Wiederholungsfenster (Sekunden)
admin-settings-help-security-admin-probe-repeat-window-secs = Eine Sperre, die innerhalb dieser Zeitspanne nach dem Ende der vorherigen beginnt, zählt als Wiederholung.
admin-settings-key-security-admin-probe-block-status = Status der Sperrantwort
admin-settings-help-security-admin-probe-block-status = HTTP-Status, der an eine gesperrte Quelle gesendet wird, mit Retry-After: 429 oder 403.
admin-settings-key-security-admin-probe-signals = Gezählte Fehlzugriffe
admin-settings-help-security-admin-probe-signals = probe-path, slug-guess (ein 404 in der Pfadtiefe der Konsole), admin-signin-failed und admin-unauthenticated.
admin-settings-key-security-admin-probe-probe-paths = Erkundungspfade
admin-settings-help-security-admin-probe-probe-paths = Pfade, nach denen Scanner fragen, etwa /wp-admin oder /.env. Ein Pfad, der einen davon als vollständige Segmente enthält, zählt als Fehlzugriff. Der eigene Pfad der Konsole nie.
admin-settings-key-security-admin-probe-allowlist = Nie gesperrte Netzwerke
admin-settings-help-security-admin-probe-allowlist = IP-Adressen oder CIDR-Netzwerke (Büro, VPN), die nie gezählt oder gesperrt werden.
admin-settings-key-security-admin-probe-audit-ip = Quelladresse im Audit-Protokoll
admin-settings-help-security-admin-probe-audit-ip = masked zeigt IPv4 /24 und IPv6 /48; full zeigt die Adresse. Die fail2ban-Zeile enthält immer die vollständige Adresse.
admin-settings-key-security-admin-probe-fail2ban-log = fail2ban-Logzeile bei Sperre
admin-settings-help-security-admin-probe-fail2ban-log = Schreibt eine Zeile mailwoman[admin-probe] mit rhost=<address>, wenn eine Quelle gesperrt wird.
admin-settings-key-appearance-custom-css-enabled = Persönliches Aussehens-CSS erlauben
admin-settings-help-appearance-custom-css-enabled = Erlaubt eingeschränkte visuelle Stile im Arbeitsbereich des Eigentümers. Standardmäßig deaktiviert; erzwungene Aussehenssperren des Postfachs setzen diese Stile aus.
admin-settings-key-appearance-custom-js-enabled = Persönliche Aussehens-Skripte erlauben
admin-settings-help-appearance-custom-js-enabled = Erlaubt isolierte Skripte, die nur Änderungen des Erscheinungsbilds anfordern können. Standardmäßig deaktiviert; Skripte können nicht auf E-Mails, Kontodaten oder das DOM der Anwendung zugreifen.
admin-settings-key-mail-modifier-selection = Auswahl von Nachrichten mit Modifikatortasten
admin-settings-help-mail-modifier-selection = Erlaubt die Bereichsauswahl mit Strg, Befehl und Umschalt. Sperren Sie diesen Wert, um ihn durchzusetzen; sichtbare Auswahlsteuerungen bleiben verfügbar.
admin-settings-key-mail-drag-to-folder = Nachrichten in Ordner ziehen
admin-settings-help-mail-drag-to-folder = Erlaubt das kontogebundene Ziehen von Nachrichten in echte Ordner. Dies gewährt keine Schreibberechtigung für das Postfach; der Server autorisiert weiterhin jede Verschiebung.
admin-settings-key-admin-allow-last-superadmin-loss = Verlust des letzten Super-Admins erlauben
admin-settings-help-admin-allow-last-superadmin-loss = Standardmäßig aus. Wenn aktiviert, dürfen spätere Änderungen den letzten Super-Admin-Zugriff entfernen. Ein vollwertiger Super-Admin muss die Folge mit frischer Authentifizierung bestätigen.
admin-settings-last-superadmin-title = Verlust des gesamten Super-Admin-Zugriffs erlauben?
admin-settings-last-superadmin-body = Spätere Kontoänderungen können dazu führen, dass niemand mehr diese Bereitstellung verwalten kann. Eine Wiederherstellung würde lokale Verwaltung der Bereitstellung erfordern. Geben Sie ALLOW LAST SUPERADMIN LOSS ein, um diese Ausnahme zu aktivieren.
admin-settings-last-superadmin-confirm = Zugriffsverlust erlauben
admin-settings-key-mail-folder-counts = Ordnerzähler
admin-settings-help-mail-folder-counts = Wählen Sie neben Ordnern: ausgeblendet, ungelesen, ungelesen nur bei vorhandenen ungelesenen E-Mails, gesamt oder beide Zähler. Legen Sie globale, Domain- oder Benutzerstandards fest und sperren Sie die Wahl bei Bedarf. Gilt für jedes verknüpfte Postfach eines Kontos.
admin-settings-key-mail-folder-count-scope = Ordner mit Zählern
admin-settings-help-mail-folder-count-scope = Alle Ordner oder nur Hauptordner (Posteingang, Entwürfe, Gesendet, Archiv, Spam, Papierkorb, Postausgang und andere Spezialordner). Bei nur Hauptordnern zeigen eigene Ordner und ihre Unterordner keinen Zähler. Gilt für jedes verknüpfte Postfach eines Kontos.
admin-settings-key-mail-compact-folder-badges = Kompakte sekundäre Ordnerzähler
admin-settings-help-mail-compact-folder-badges = Zeigt kleinere Zähler außerhalb des Posteingangs. Benutzer können ihre Größe selbst wählen, sofern diese Einstellung nicht gesperrt ist.
admin-settings-key-mail-focused-inbox = Fokussierter Posteingang
admin-settings-help-mail-focused-inbox = Teilt den Posteingang in die Tabs Fokussiert und Sonstige; Newsletter und automatische Absender landen unter Sonstige. Standardmäßig aus. Benutzer können es unter Einstellungen > Ordner und Seitenleiste ein- oder ausschalten, sofern diese Einstellung nicht gesperrt ist.
admin-settings-key-mail-account-unread-badge = Ungelesen-Zähler neben dem Postfachnamen
admin-settings-help-mail-account-unread-badge = Zeigt den Ungelesen-Zähler des Posteingangs neben der angemeldeten Adresse oder dem Spitznamen in der Seitenleiste. Standardmäßig aus. Die Zähler anderer Postfächer im Postfachmenü werden immer angezeigt. Gilt für jedes verknüpfte Postfach eines Kontos.
admin-settings-key-privacy-remote-image-downloads = Manueller Download externer Bilder
admin-settings-key-privacy-remote-images-toast = Hinweis beim Laden von Bildern
admin-settings-help-privacy-remote-images-toast = Zeigt im Leser einen Hinweis, solange externe Bilder einer Nachricht über den Datenschutz-Proxy geladen werden, und meldet, wann sie geladen sind oder wie viele nicht geladen werden konnten. Standardmäßig an. Ändert nur, was der Leser anzeigt; welche Bilder geladen werden, entscheidet weiterhin die Richtlinie für externe Inhalte.
admin-settings-help-privacy-remote-image-downloads = Erlaubt den ausdrücklichen Download eines Bildes, auf das die eigene Nachricht verweist, über den geschützten Datenschutz-Proxy. Standardmäßig deaktiviert; die externe URL wird dabei nie direkt geöffnet.
admin-settings-key-mail-resume-tabs = Arbeitsbereich-Tabs fortsetzen
admin-settings-help-mail-resume-tabs = Stellt kontoeigene Tab-Verweise nach dem Neuladen wieder her. Das Deaktivieren löscht die lokale Arbeitsbereichsaufzeichnung, ohne Entwürfe zu löschen.
admin-settings-key-metrics-default-range-seconds = Standardzeitraum des Anfrageverlaufs
admin-settings-help-metrics-default-range-seconds = Sekunden, die angezeigt werden, wenn eine Verlaufsanfrage keinen Zeitraum angibt. Begrenzt durch Aufbewahrung und den Maximalzeitraum.
admin-settings-key-metrics-max-range-seconds = Maximaler Zeitraum des Anfrageverlaufs
admin-settings-help-metrics-max-range-seconds = Maximale Sekunden pro Verlaufsabfrage. Die Aufbewahrung bleibt eine zusätzliche Grenze.
admin-settings-key-contacts-favorites-first = Favoriten zuerst in Vorschlägen
admin-settings-help-contacts-favorites-first = Listet favorisierte Kontakte in Empfängervorschlägen vor anderen Treffern auf. Nur die Reihenfolge ändert sich; kein Vorschlag wird ausgeblendet. Überschreibungen für Benutzer und Domains können gesperrt werden.
admin-settings-key-contacts-sender-icons = Mitgelieferte Absendersymbole
admin-settings-help-contacts-sender-icons = Zeigt lokal mitgelieferte Symbole für erkannte Absender-Domains, für andere generierte Initialen. Kontaktbilder haben Vorrang. Symbole bestätigen nicht die Identität eines Absenders. Es wird kein externer Logo-Dienst kontaktiert; Überschreibungen für Benutzer und Domains können gesperrt werden.
admin-settings-key-mail-brand-avatars = Markenlogos neben verifizierten Absendern
admin-settings-help-mail-brand-avatars = Zeigt im Leser ein lokal mitgeliefertes Markenlogo neben dem Absender, nur wenn die Nachricht DKIM und eine ausgerichtete DMARC-Prüfung für die eigene Domain dieser Marke bestanden hat. Nachrichten, die diese Prüfungen nicht bestehen oder nicht haben, behalten den normalen Avatar. Es wird kein Logo vom Absender oder von einem Logo-Dienst abgerufen.
admin-settings-key-mail-draft-save-mode = Speichermodus für Konto-Entwürfe
admin-settings-help-mail-draft-save-mode = Nach einer Tipppause, periodisch oder nur auf Anforderung speichern. Das Speichern erfordert einen unterstützten Entwurfsordner beim Upstream; unsichere Schreibvorgänge werden nie automatisch wiederholt.
admin-settings-key-mail-draft-debounce-ms = Tippause für Entwürfe (Millisekunden)
admin-settings-help-mail-draft-debounce-ms = Wartet im Echtzeitmodus 500 bis 10000 Millisekunden nach dem Bearbeiten, bevor gespeichert wird.
admin-settings-key-mail-draft-interval-secs = Speicherintervall für Entwürfe (Sekunden)
admin-settings-help-mail-draft-interval-secs = Speichert bearbeitete Entwürfe im periodischen Modus alle 5 bis 3600 Sekunden. Unveränderte Entwürfe werden nicht erneut geschrieben.
admin-settings-key-login-remember-signins = Erfolgreiche Anmeldeadressen merken
admin-settings-help-login-remember-signins = Behält bis zu zehn erfolgreich verwendete Benutzernamen oder E-Mail-Adressen auf diesem Gerät für die Auswahlliste bei der Anmeldung. Speichert nie Passwörter oder Sitzungen. Das Deaktivieren entfernt die gespeicherten Adressen dieses Servers, sobald ein Gerät die Anmeldeseite das nächste Mal lädt oder eine Anmeldung abschließt; die Autofill-Funktion des Browsers wird dadurch nicht gesteuert.
admin-settings-key-login-show-remembered-hint = Erklärung zu gespeicherten Adressen anzeigen
admin-settings-help-login-show-remembered-hint = Zeigt erklärenden Text in der Auswahlliste früherer Anmeldungen. Standardmäßig verborgen. Gilt nur, wenn das Merken erfolgreicher Anmeldungen aktiviert ist.
admin-settings-key-mail-show-from = Absenderfeld im Editor anzeigen
admin-settings-help-mail-show-from = Zeigt Von standardmäßig in neuen Editoren an. Benutzer können es für eine Nachricht einblenden; nur autorisierte Adressen können senden.
admin-settings-key-accounts-linking-enabled = Verknüpfte Postfachkonten erlauben
admin-settings-help-accounts-linking-enabled = Standardmäßig deaktiviert. Eine globale Deaktivierung blockiert jeden verknüpften Zugriff; beide Konten müssen außerdem ihre Domain- und Kontorichtlinie erfüllen. Das Verknüpfen erfordert die tatsächliche Authentifizierung des untergeordneten Kontos und einen frischen Nachweis des primären Kontos.
admin-settings-key-accounts-use-linked = Verknüpfte Postfächer verwenden
admin-settings-help-accounts-use-linked = Ausstieg des Benutzers aus dem verknüpften Zugriff. Kann eine administrative Deaktivierung nicht überschreiben. Postfachdaten bleiben erhalten, wenn der Zugriff deaktiviert wird.
admin-settings-key-accounts-max-linked = Maximale Anzahl verknüpfter Postfächer
admin-settings-help-accounts-max-linked = Maximal verifizierte untergeordnete Postfächer pro primärer Identität, von 1 bis 16. Standard 5.
admin-settings-key-accounts-store-linked-credentials = Zugriff auf verknüpfte Postfächer beibehalten
admin-settings-help-accounts-store-linked-credentials = Behält das Passwort jedes verknüpften Postfachs versiegelt auf dem Server, sodass sein Zugriff sich über Sitzungen und Neustarts hinweg selbst erneuert. Wird nie für Zero-Access-Konten oder Postfächer mit Zwei-Schritt-Verifizierung gespeichert. Das Ausschalten löscht gespeicherte Passwörter; Verknüpfungen erneuern sich dann durch erneute Anmeldung. Standardmäßig an.
admin-settings-key-privacy-strip-mailer-headers = Mailer-Header ausgehender E-Mails entfernen
admin-settings-help-privacy-strip-mailer-headers = Entfernt die Header X-Mailer, User-Agent und X-MimeOLE der obersten Ebene aus nativen SMTP- und EWS-E-Mails. Angehängte Nachrichten und Berichtsinhalte bleiben unverändert. Sendungen über den vorgelagerten JMAP-Proxy sind nicht erfasst.
admin-settings-key-images-trusted-hosts = Vertrauenswürdige Bild-Hosts
admin-settings-help-images-trusted-hosts = Bis zu 100 Bild-Hosts samt Subdomains, deren externe Bilder ohne Freigabe pro Nachricht geladen werden, wenn externe Inhalte auf Nachfragen stehen. Überschreibt nie den Sperrmodus; ein gesperrter Host bleibt gesperrt.
admin-settings-key-images-blocked-hosts = Gesperrte Bild-Hosts
admin-settings-help-images-blocked-hosts = Bis zu 100 Bild-Hosts samt Subdomains, deren externe Bilder in keinem Modus geladen werden. Hat Vorrang vor vertrauenswürdigen Hosts und jeder Freigabe.
admin-settings-key-links-trusted-domains = Vertrauenswürdige Link-Domains
admin-settings-help-links-trusted-domains = Bis zu 100 exakte Domains, deren Links aus Links prüfen ohne den Zielschritt geöffnet werden. Keine Wildcards, Schemata, Ports oder IP-Adressen. Links über reines HTTP werden immer geprüft. Sperren Sie diese Einstellung, damit Benutzer die Liste nicht ändern.
admin-settings-key-links-trusted-subdomains = Subdomains vertrauenswürdiger Link-Domains vertrauen
admin-settings-help-links-trusted-subdomains = Aus: Nur die exakte Domain ist vertrauenswürdig. An: Auch Namen, die auf .<domain> enden, sind vertrauenswürdig. Verglichen wird auf Basis ganzer Labels, sodass example.com nie evilexample.com vertraut.
admin-settings-key-cap-compose-rich-text = Verfassen mit Rich-Text
admin-settings-help-cap-compose-rich-text = Ob Benutzer formatierte (HTML-)Nachrichten schreiben dürfen. Aus sendet nur Klartext.
admin-settings-key-cap-compose-attachments = Anhänge in neuen Nachrichten
admin-settings-help-cap-compose-attachments = Ob Benutzer Dateien an Nachrichten anhängen dürfen, die sie schreiben. Bilder im Text werden nicht mitgezählt.
admin-settings-key-cap-compose-share-links = Freigabelinks für große Dateien
admin-settings-help-cap-compose-share-links = Ob Benutzer eine große Datei als Nextcloud-Freigabelink statt als Anhang senden dürfen.
admin-settings-key-cap-compose-scheduled-send = Zeitgesteuertes Senden
admin-settings-help-cap-compose-scheduled-send = Ob Benutzer eine Nachricht zum späteren Senden planen dürfen.
admin-settings-key-cap-compose-read-receipts = Lesebestätigungen anfordern
admin-settings-help-cap-compose-read-receipts = Ob Benutzer Empfänger um eine Lesebestätigung bitten dürfen.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-cap-signatures = Signaturen
admin-settings-help-cap-signatures = Ob Benutzer ihre eigenen Signaturen erstellen und bearbeiten dürfen.
admin-settings-key-cap-identities = Absenderidentitäten
admin-settings-help-cap-identities = Ob Benutzer die Identitäten hinzufügen und bearbeiten dürfen, als die sie senden.
admin-settings-key-cap-keys = OpenPGP- und S/MIME-Schlüssel
admin-settings-help-cap-keys = Ob Benutzer das Schlüsselmodul haben und Nachrichten signieren und verschlüsseln dürfen.
admin-settings-key-cap-links-trusted = Vertrauenswürdige Link-Domains bearbeiten
admin-settings-help-cap-links-trusted = Ob Benutzer ihre eigene Liste vertrauenswürdiger Link-Domains ändern dürfen. Die Liste des Administrators gilt weiterhin.
admin-settings-key-cap-zoom = Zoom im Leser
admin-settings-help-cap-zoom = Ob Benutzer den Nachrichtenleser zoomen dürfen.
admin-settings-key-cap-mail-focused-inbox = Fokussierter Posteingang
admin-settings-help-cap-mail-focused-inbox = Ob Benutzer die Posteingangs-Tabs Fokussiert und Sonstige aktivieren dürfen.
admin-settings-key-cap-views-events = Ereignisansicht
admin-settings-help-cap-views-events = Ob Benutzer die Ereignisansicht für Kalender-E-Mails haben.
admin-settings-key-cap-views-attachments = Anhangsansicht
admin-settings-help-cap-views-attachments = Ob Benutzer die Anhangsansicht haben. Die Nachrichten bleiben über die Suche erreichbar.
admin-settings-key-cap-search-operators = Suchoperatoren
admin-settings-help-cap-search-operators = Ob Benutzer mit Operatoren wie from:, subject: oder has:attachment suchen dürfen.
admin-settings-key-cap-saved-searches = Gespeicherte Suchen
admin-settings-help-cap-saved-searches = Ob Benutzer Suchen speichern dürfen.
admin-settings-key-cap-vacation = Abwesenheits-Autoantwort
admin-settings-help-cap-vacation = Ob Benutzer eine automatische Abwesenheitsantwort einrichten dürfen.
admin-settings-key-cap-labels = Labels
admin-settings-help-cap-labels = Ob Benutzer Nachrichten mit Labels versehen dürfen.
admin-settings-key-cap-snooze = Zurückstellen
admin-settings-help-cap-snooze = Ob Benutzer Nachrichten zurückstellen dürfen. Das Beenden einer Zurückstellung bleibt möglich.
admin-settings-key-cap-follow-up = Wiedervorlage-Erinnerungen
admin-settings-help-cap-follow-up = Ob Benutzer Wiedervorlage-Erinnerungen an Nachrichten setzen dürfen. Das Entfernen bleibt möglich.
admin-settings-key-cap-masked = Maskierte Adressen
admin-settings-help-cap-masked = Ob Benutzer maskierte Adressen erstellen und verwalten dürfen.
admin-settings-key-cap-directory = Verzeichnissuche
admin-settings-help-cap-directory = Ob Benutzer das Verzeichnis der Organisation durchsuchen dürfen.
admin-settings-key-cap-mcp = MCP-Zugriff
admin-settings-help-cap-mcp = Ob die Schlüssel und Tokens des Benutzers den MCP-Endpunkt verwenden dürfen.
admin-settings-key-cap-appearance-custom = Anpassung des Erscheinungsbilds
admin-settings-help-cap-appearance-custom = Ob Benutzer ihr Design, ihren Akzent, eigene Farben und Zählerfarben, die Dichte und die Schrift ändern dürfen. Aus behält das Erscheinungsbild des Administrators bei.
admin-settings-key-cap-appearance-custom-code = Eigenes CSS und eigene Skripte
admin-settings-help-cap-appearance-custom-code = Ob Benutzer ihr eigenes CSS und eigene Aussehens-Skripte speichern dürfen. Die Bereitstellungsschalter unter Erscheinungsbild gelten weiterhin übergeordnet.
admin-settings-key-cap-avatars = Postfach-Avatare
admin-settings-help-cap-avatars = Ob Benutzer Postfach-Avatare ändern dürfen.
admin-settings-key-cap-shortcuts = Eigene Tastenkürzel
admin-settings-help-cap-shortcuts = Ob Benutzer Tastenkürzel ändern dürfen. Die Standardkürzel funktionieren weiterhin.
admin-settings-key-cap-analytics = Produktanalyse
admin-settings-help-cap-analytics = Ob Benutzer um Einwilligung zur Analyse gebeten werden und Nutzungsereignisse senden dürfen.
admin-settings-key-cap-apps-native = Desktop- und Mobil-Apps
admin-settings-help-cap-apps-native = Ob Benutzer die Desktop- und Mobil-Apps verwenden dürfen.
admin-settings-key-access-protocols = Protokolle für den E-Mail-Zugriff
admin-settings-help-access-protocols = Mit welchen Mailserver-Protokollen sich diese Benutzer anmelden dürfen: imap, pop3, ews. Anmeldungen über einen JMAP-Server folgen diesem Server.
admin-settings-key-limit-linked-mailboxes = Verknüpfte Postfächer pro Benutzer
admin-settings-help-limit-linked-mailboxes = Wie viele Postfächer ein Benutzer verknüpfen darf, unterhalb des Server-Maximums. 0 bedeutet, dass nur das Server-Maximum gilt.
admin-settings-key-login-show-account-help-notice = Kontohilfe-Hinweis bei der Anmeldung anzeigen
admin-settings-help-login-show-account-help-notice = Zeigt nach einer fehlgeschlagenen Anmeldung eine kurze Erklärung. Standardmäßig verborgen; der Hinweis verrät nicht, ob ein Konto existiert oder deaktiviert ist.
admin-settings-key-appearance-leetspeak = Textstil Leetspeak
admin-settings-help-appearance-leetspeak = Zeigt Beschriftungen der Oberfläche und schreibgeschützten Nachrichtentext in Leetspeak an. Originalnachricht und bearbeitbarer Text bleiben unverändert. Benutzer können normalen Text wählen, sofern diese Einstellung nicht gesperrt ist.
admin-settings-key-mail-cache-enabled = Sitzungs-Cache aktivieren
admin-settings-help-mail-cache-enabled = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-mailboxes = Nachrichtenlisten von Ordnern zwischenspeichern
admin-settings-help-mail-cache-mailboxes = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-messages = Geöffnete Nachrichten zwischenspeichern
admin-settings-help-mail-cache-messages = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-fresh-seconds = Aktualitätsintervall (Sekunden)
admin-settings-help-mail-cache-fresh-seconds = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-max-age-seconds = Maximales Cache-Alter (Sekunden)
admin-settings-help-mail-cache-max-age-seconds = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-max-entries = Maximale Anzahl zwischengespeicherter Einträge
admin-settings-help-mail-cache-max-entries = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-max-bytes = Speicherbudget (Bytes)
admin-settings-help-mail-cache-max-bytes = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-prefetch-enabled = Spekulatives Vorladen aktivieren
admin-settings-help-mail-cache-prefetch-enabled = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-hover-enabled = Vorladen beim Zeigen auf einen Ordner
admin-settings-help-mail-cache-hover-enabled = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-focus-enabled = Vorladen, wenn ein Ordner den Tastaturfokus erhält
admin-settings-help-mail-cache-focus-enabled = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-dwell-ms = Verzögerung bei Zeiger und Fokus (Millisekunden)
admin-settings-help-mail-cache-dwell-ms = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-idle-enabled = Ordner im Leerlauf vorladen
admin-settings-help-mail-cache-idle-enabled = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-idle-delay-ms = Leerlaufverzögerung (Millisekunden)
admin-settings-help-mail-cache-idle-delay-ms = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-max-concurrency = Maximale gleichzeitige Vorladevorgänge
admin-settings-help-mail-cache-max-concurrency = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-max-queued = Maximale Anzahl wartender Vorladevorgänge
admin-settings-help-mail-cache-max-queued = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-preload-messages = Spekulatives Laden von Nachrichtentexten erlauben
admin-settings-help-mail-cache-preload-messages = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-excluded-folders = Vom Vorladen ausgeschlossene Ordner
admin-settings-help-mail-cache-excluded-folders = Behält zuletzt besuchte Ordner und Nachrichten im Speicher dieses Tabs. Zwischengespeicherte Inhalte werden im Hintergrund aktualisiert. Das Schließen des Tabs oder die Abmeldung löscht sie. Das Vorladen markiert nie E-Mails als gelesen und lädt keine externen Bilder.
admin-settings-key-mail-cache-refresh-on-open = Zwischengespeicherte Inhalte beim Öffnen aktualisieren
admin-settings-help-mail-cache-refresh-on-open = Zwischengespeicherte Inhalte beim Öffnen aktualisieren
admin-settings-key-mail-templates-enabled = Antwortvorlagen
admin-settings-help-mail-templates-enabled = Aktiviert persönliche und gemeinsame Klartext-Antwortvorlagen im Editor. Benutzer können sie abwählen, sofern diese Einstellung nicht gesperrt ist.
admin-settings-key-mail-templates-allow-personal = Persönliche Vorlagen erlauben
admin-settings-help-mail-templates-allow-personal = Erlaubt Benutzern, eigene Antwortvorlagen zu erstellen. Gemeinsame Vorlagen werden getrennt von Administratoren verwaltet.
admin-settings-key-mail-templates-shared = Gemeinsame Antwortvorlagen
admin-settings-help-mail-templates-shared = Verwaltet wiederverwendbare Klartext-Antworten für diesen Geltungsbereich. Vorlagen legen nie Empfänger fest und senden keine Nachrichten.
admin-settings-key-mail-templates-personal = Persönliche Antwortvorlagen
admin-settings-help-mail-templates-personal = Persönliche wiederverwendbare Klartext-Antworten. Erstellen oder Bearbeiten erfordert, dass Antwortvorlagen und persönliche Vorlagen aktiviert sind.
admin-settings-key-mail-compose-window = Editorfenster
admin-settings-help-mail-compose-window = Öffnet neue Entwürfe in einem Arbeitsbereich-Tab oder einem separaten Fenster. Blockierte Popups weichen auf einen Tab aus.
admin-settings-key-mail-reply-copy-to-original = Antworten im ursprünglichen Ordner behalten
admin-settings-help-mail-reply-copy-to-original = Speichert eine Kopie der gesendeten Antwort im Ordner der ursprünglichen Nachricht, wo unterstützt.
admin-settings-key-mail-compose-check-invalid-recipients = Vor dem Senden auf ungültige Empfänger prüfen
admin-settings-help-mail-compose-check-invalid-recipients = Was geschieht, wenn eine Nachricht an eine ungültige Adresse gesendet wird: erlauben, warnen, warnen und die Wahl merken oder ablehnen.
admin-settings-key-mail-compose-check-missing-attachment = Vor dem Senden auf fehlenden Anhang prüfen
admin-settings-help-mail-compose-check-missing-attachment = Was geschieht, wenn der Text einen Anhang erwähnt, aber keiner angehängt ist: erlauben, warnen, warnen und die Wahl merken oder ablehnen.
admin-settings-key-mail-compose-check-empty-subject = Vor dem Senden auf leeren Betreff prüfen
admin-settings-help-mail-compose-check-empty-subject = Was geschieht, wenn eine Nachricht keinen Betreff hat: erlauben, warnen, warnen und die Wahl merken oder ablehnen.
admin-settings-key-mail-compose-auto-chip-recipients = Eingetippte Empfänger automatisch abschließen
admin-settings-help-mail-compose-auto-chip-recipients = Wandelt eine eingetippte Adresse in einen Empfänger-Chip um, wenn ein Leerzeichen, Komma oder Semikolon folgt.
admin-settings-key-mail-compose-header-collapse = Editor-Kopfzeile
admin-settings-help-mail-compose-header-collapse = Wie sich die Zeilen An, Cc, Bcc und Betreff des Editors verhalten: ausgeklappt, zu einer einzeiligen Zusammenfassung eingeklappt oder auto (eingeklappt beim Schreiben des Textes, offen, wenn ein Feld den Fokus hat). Benutzer können sie selbst ein- oder ausklappen, sofern dies nicht gesperrt ist.
admin-settings-key-mail-compose-recipient-separator = Trennzeichen für Empfänger
admin-settings-help-mail-compose-recipient-separator = Das Trennzeichen, das nach einem abgeschlossenen Empfänger eingefügt wird: ein Komma oder ein Semikolon.
admin-settings-key-mail-reply-from-mode = Für Antworten und Weiterleitungen verwendete Identität
admin-settings-help-mail-reply-from-mode = Von welcher Identität eine Antwort oder Weiterleitung gesendet wird: der, an die das Original adressiert war, der Standardidentität, einer Auswahl oder der aktuell gewählten.
admin-settings-key-mail-forward-mode = Standardformat der Weiterleitung
admin-settings-help-mail-forward-mode = Inline-Text weiterleiten oder die Originalnachricht als .eml-Datei anhängen.
admin-settings-key-mail-show-go-to-top = Schaltfläche Nach oben anzeigen
admin-settings-help-mail-show-go-to-top = Zeigt in der Nachrichtenliste eine Schaltfläche Nach oben, die zur ersten Nachricht zurückscrollt. Standardmäßig aus.
admin-settings-key-mail-show-create-folder-button = Schaltfläche Ordner erstellen anzeigen
admin-settings-help-mail-show-create-folder-button = Zeigt in der Seitenleiste neben Verfassen eine reine Symbolschaltfläche Ordner erstellen. Standardmäßig aus.
admin-settings-key-security-protection-user-agent-mode = Modus der User-Agent-Beobachtung
admin-settings-help-security-protection-user-agent-mode = Standardmäßig enforce: Ausgewählte erkannte Bot-Header werden sofort blockiert, wenn das Blockieren von Bots bei der ersten Anfrage aktiviert ist. observe protokolliert Entscheidungen ohne Ablehnung; off deaktiviert diese Dimension. Header-Angaben sind keine verifizierten Identitäten.
admin-settings-key-security-protection-user-agent-block-bots = Erkannte Bots bei der ersten Anfrage blockieren
admin-settings-help-security-protection-user-agent-block-bots = Standardmäßig true. Ausgewählte Header von Such-, KI-, Automatisierungs- und benutzerdefinierten Bots lösen im Modus enforce bei der ersten Anfrage eine vorübergehende HTTP-429-Sperre aus. observe protokolliert nur; false stellt die bisherigen Anfragebudgets wieder her. Fehlende oder ungewöhnliche Header, Geltungsbereich und Ausnahmen behalten ihr bisheriges Verhalten.
admin-settings-key-security-protection-user-agent-categories = User-Agent-Kategorien
admin-settings-help-security-protection-user-agent-categories = Konfiguriert diesen Teil der optionalen User-Agent-Beobachtung. Vom Client gelieferter User-Agent-Text ist ein schwaches Signal, kein Beleg für Identität oder Missbrauch.
admin-settings-key-security-protection-user-agent-weak-requests = Schwellenwert für Anfragen mit schwachem Signal
admin-settings-help-security-protection-user-agent-weak-requests = Konfiguriert diesen Teil der optionalen User-Agent-Beobachtung. Vom Client gelieferter User-Agent-Text ist ein schwaches Signal, kein Beleg für Identität oder Missbrauch.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-protection-user-agent-crawler-requests = Schwellenwert für Crawler-Anfragen
admin-settings-help-security-protection-user-agent-crawler-requests = Konfiguriert diesen Teil der optionalen User-Agent-Beobachtung. Vom Client gelieferter User-Agent-Text ist ein schwaches Signal, kein Beleg für Identität oder Missbrauch.
admin-settings-key-security-protection-user-agent-window-secs = Beobachtungsfenster für User-Agent (Sekunden)
admin-settings-help-security-protection-user-agent-window-secs = Konfiguriert diesen Teil der optionalen User-Agent-Beobachtung. Vom Client gelieferter User-Agent-Text ist ein schwaches Signal, kein Beleg für Identität oder Missbrauch.
admin-settings-key-security-protection-user-agent-max-action = Maximale User-Agent-Reaktion
admin-settings-help-security-protection-user-agent-max-action = Maximale Eskalation bei schwachen oder wiederholten Anfragebelegen, Standard ist Drosseln. Diese Obergrenze beschränkt nicht das Blockieren ausgewählter erkannter Bot-Header bei der ersten Anfrage.
admin-settings-key-security-protection-user-agent-scope = Geltungsbereich der User-Agent-Richtlinie
admin-settings-help-security-protection-user-agent-scope = Konfiguriert diesen Teil der optionalen User-Agent-Beobachtung. Vom Client gelieferter User-Agent-Text ist ein schwaches Signal, kein Beleg für Identität oder Missbrauch.
admin-settings-key-security-protection-user-agent-adaptive = Adaptive User-Agent-Beobachtungen
admin-settings-help-security-protection-user-agent-adaptive = Konfiguriert diesen Teil der optionalen User-Agent-Beobachtung. Vom Client gelieferter User-Agent-Text ist ein schwaches Signal, kein Beleg für Identität oder Missbrauch.
admin-settings-key-security-protection-user-agent-patterns = User-Agent-Muster
admin-settings-help-security-protection-user-agent-patterns = Konfiguriert diesen Teil der optionalen User-Agent-Beobachtung. Vom Client gelieferter User-Agent-Text ist ein schwaches Signal, kein Beleg für Identität oder Missbrauch.
admin-settings-key-security-protection-user-agent-exceptions = User-Agent-Ausnahmen
admin-settings-help-security-protection-user-agent-exceptions = Konfiguriert diesen Teil der optionalen User-Agent-Beobachtung. Vom Client gelieferter User-Agent-Text ist ein schwaches Signal, kein Beleg für Identität oder Missbrauch.
admin-settings-key-mail-reader-unsafe-html-mode = Möglicherweise unsicheres Nachrichten-HTML
admin-settings-help-mail-reader-unsafe-html-mode = ask bietet standardmäßig eine Schaltfläche für bereinigtes HTML pro Nachricht. strict zeigt nur sicheren Text; sanitized zeigt bereinigtes HTML automatisch. Aktive Inhalte und automatische externe Ressourcen bleiben blockiert. Sperren Sie den Wert, um diese Richtlinie durchzusetzen.
admin-settings-key-reader-attachments-density = Dichte der Anhänge
admin-settings-help-reader-attachments-density = Standardmäßig kompakte Datei-Schaltflächen oder bequeme Vorschauen. Steuert die Darstellung, wenn die Anhänge einer Nachricht aufgeklappt sind.
admin-settings-key-cap-compose-open-tracking = Öffnungsverfolgung von Nachrichten erlauben
admin-settings-help-cap-compose-open-tracking = Ob dieses Konto Öffnungen von Nachrichten erfassen darf. Erfordert eine konfigurierte öffentliche URL; deaktivierte Konten können keine Verfolgungs-Tokens erstellen oder binden.
admin-settings-key-mail-open-tracking = Öffnungen standardmäßig verfolgen
admin-settings-help-mail-open-tracking = Standardwahl der Verfolgung pro Nachricht für geeignete formatierte Nachrichten. Benutzer können sie überschreiben, sofern nicht gesperrt; das Deaktivieren der Verfolgungsberechtigung hat immer Vorrang.
admin-settings-key-cap-reading-original-html = Darstellung von Original-HTML erlauben
admin-settings-help-cap-reading-original-html = Ob Benutzer für einzelne Nachrichten die Original-HTML-Ansicht wählen dürfen. Der standardmäßige bereinigende Renderer und die Sicherheit des isolierten Frames bleiben bestehen.
admin-settings-key-mail-reader-allow-original-html = Original-HTML-Ansicht anbieten
admin-settings-help-mail-reader-allow-original-html = Zeigt die Option pro Nachricht, Original-HTML zu verwenden, wenn die Leseberechtigung es erlaubt. Standardmäßig aus; dies ändert nicht den Standard-Renderer für Nachrichten.
admin-settings-key-privacy-tracker-override-allowed = Tracker-Einwilligung pro Nachricht anbieten
admin-settings-help-privacy-tracker-override-allowed = Zeigt die Aktion Tracker erlauben in der Datenschutzleiste der Nachricht. Standardmäßig verfügbar, aber nie aktiv ohne ausdrückliche Freigabe. Eine gesperrte Richtlinie zum Blockieren von Trackern verbietet die Überschreibung weiterhin.
admin-settings-key-privacy-max-concurrent-global = Globale parallele Bildabrufe
admin-settings-help-privacy-max-concurrent-global = Maximale aktive Netzwerkanfragen für Bilder über diesen Serverprozess. Bereich 1–256; Standard 128. Die Limits pro Nachricht und Konto gelten weiterhin.
admin-settings-key-privacy-max-concurrent-pipeline = Gleichzeitige Bildverarbeitungs-Pipelines
admin-settings-help-privacy-max-concurrent-pipeline = Maximale Anzahl von Bildern, die gemeinsam zu Abruf, Dekodierung und Cache-Veröffentlichung zugelassen werden. Bereich 1–256; Standard 128. Netzwerk- und Decoder-Obergrenzen werden getrennt durchgesetzt.
admin-settings-key-privacy-max-concurrent-decodes = Parallele Bild-Decoder
admin-settings-help-privacy-max-concurrent-decodes = Maximale Anzahl gleichzeitig laufender Bilddekodierungen. Bereich 1–16; Standard 10. Höhere Werte verbrauchen mehr CPU und Speicher.
admin-settings-key-privacy-max-queued-requests = Wartende Bildanfragen
admin-settings-help-privacy-max-queued-requests = Maximale Anzahl von Anfragen, die auf Netzwerk- oder Verarbeitungskapazität für Bilder warten. Bereich 16–16384; Standard 2048. Überlauf oder Ablauf in der Warteschlange bleibt wiederholbar.
admin-settings-key-privacy-rate-burst = Burst-Kontingent für Bildanfragen
admin-settings-help-privacy-rate-burst = Anfängliche und maximale Abruf-Tokens pro Konto, aufgefüllt mit der konfigurierten Rate für Bildanfragen. Bereich 1–12000; Standard 1024. Zwischengespeicherte Erfolge verbrauchen keine Tokens.
admin-settings-key-privacy-image-rate-mode = Dimensionierung der Rate für Bildanfragen
admin-settings-help-privacy-image-rate-mode = Automatisch skaliert Burst und Auffüllung pro Konto anhand von Replikatkapazität, Warteschlangenlast und der Bildanzahl der Nachricht innerhalb der unten stehenden Unter- und Obergrenzen. Manuell verwendet die Einstellungen für Burst-Kontingent und Anfragerate exakt. Standard automatisch. Anfragen über dem Kontingent warten bis zur Frist der Abrufwarteschlange auf ein Token; Authentifizierung und Freigaben werden zuerst geprüft.
admin-settings-key-privacy-image-rate-burst-floor = Untergrenze des automatischen Burst-Kontingents
admin-settings-help-privacy-image-rate-burst-floor = Niedrigstes Burst-Kontingent pro Konto im automatischen Ratenmodus. Bereich 1–12000; Standard 64.
admin-settings-key-privacy-image-rate-burst-ceiling = Obergrenze des automatischen Burst-Kontingents
admin-settings-help-privacy-image-rate-burst-ceiling = Höchstes Burst-Kontingent pro Konto im automatischen Ratenmodus. Bereich 1–12000; Standard 2048.
admin-settings-key-privacy-image-rate-refill-floor = Untergrenze der automatischen Auffüllung
admin-settings-help-privacy-image-rate-refill-floor = Niedrigste Auffüllung pro Konto in Abruf-Tokens pro Minute im automatischen Ratenmodus. Bereich 10–60000; Standard 300.
admin-settings-key-privacy-image-rate-refill-ceiling = Obergrenze der automatischen Auffüllung
admin-settings-help-privacy-image-rate-refill-ceiling = Höchste Auffüllung pro Konto in Abruf-Tokens pro Minute im automatischen Ratenmodus. Bereich 10–60000; Standard 6000.
admin-settings-key-privacy-image-cache-mode = Dimensionierung des Bild-Caches
admin-settings-help-privacy-image-cache-mode = Automatisch bemisst den Cache nach dem Speicher, der nach den Reservierungen für Decoder und Pipeline übrig bleibt, und verlängert die Lebensdauer, solange die Trefferquote hoch und Speicher frei ist. Manuell verwendet die Einstellungen für Cache-Größe, Eintragsanzahl und Lebensdauer exakt. Standard automatisch.
admin-settings-key-privacy-image-cache-bytes-floor = Untergrenze der automatischen Cache-Größe
admin-settings-help-privacy-image-cache-bytes-floor = Kleinster Bild-Cache in Bytes im automatischen Modus. Bereich 0–1073741824; Standard 16777216.
admin-settings-key-privacy-image-cache-bytes-ceiling = Obergrenze der automatischen Cache-Größe
admin-settings-help-privacy-image-cache-bytes-ceiling = Größter Bild-Cache in Bytes im automatischen Modus. Bereich 0–1073741824; Standard 268435456.
admin-settings-key-privacy-cache-entries = Eintragslimit des Bild-Caches
admin-settings-help-privacy-cache-entries = Höchste Anzahl zwischengespeicherter Bilder; im manuellen Modus die feste Anzahl und im automatischen Modus die Obergrenze. Bereich 16–65536; Standard 4096.
admin-settings-key-privacy-cache-entry-bytes = Größtes zwischengespeichertes Bild
admin-settings-help-privacy-cache-entry-bytes = Größtes einzelnes neu kodiertes Bild, das im Cache gehalten wird, in Bytes. Größere Bilder werden ausgeliefert, aber nicht behalten. Bereich 65536–16777216; Standard 4194304.
admin-settings-key-privacy-image-cache-ttl-floor-secs = Untergrenze der automatischen Cache-Lebensdauer
admin-settings-help-privacy-image-cache-ttl-floor-secs = Kürzeste Cache-Lebensdauer in Sekunden im automatischen Modus, verwendet bei niedriger Trefferquote oder Speicherdruck. Bereich 0–86400; Standard 300.
admin-settings-key-privacy-image-cache-ttl-ceiling-secs = Obergrenze der automatischen Cache-Lebensdauer
admin-settings-help-privacy-image-cache-ttl-ceiling-secs = Längste Cache-Lebensdauer in Sekunden im automatischen Modus, verwendet bei hoher Trefferquote mit freiem Speicher. Bereich 0–86400; Standard 21600.
admin-settings-key-privacy-cache-ttl-secs = Cache-Lebensdauer erfolgreicher Bilder
admin-settings-help-privacy-cache-ttl-secs = Sekunden, die dekodierte Bilder in diesem Prozess wiederverwendbar bleiben. Bereich 0–86400; Standard 3600. Null deaktiviert die Wiederverwendung erfolgreicher Bilder; die Autorisierung wird immer zuerst geprüft.
admin-settings-key-privacy-image-latency-tracking = Geschwindigkeitserfassung des Bild-Proxys
admin-settings-help-privacy-image-latency-tracking = Erfasst, wie lange jedes über den Proxy geladene Bild braucht (Median, p95, p99 und die Zeit für Warten, Abrufen und Dekodieren), als Zähler in festen Zeitintervallen im Speicher. Es werden keine URLs, Hosts oder Konten gespeichert. Standardmäßig an; der Verlauf wird beim Neustart zurückgesetzt und gilt pro Serverprozess. Das Ausschalten beendet neue Aufzeichnungen; der Abschnitt Geschwindigkeit zeigt weiterhin das bereits Erfasste.
admin-settings-key-privacy-negative-cache-ttl-secs = Cache-Lebensdauer fehlgeschlagener Bilder
admin-settings-help-privacy-negative-cache-ttl-secs = Sekunden, die infrage kommende Upstream-Fehler gemerkt werden, um wiederholte Anfragen zu vermeiden. Bereich 0–300; Standard 10. Null deaktiviert die Wiederverwendung von Fehlern. Ein ausdrücklicher Wiederholungsversuch löscht die betroffenen Fehler.
admin-settings-key-privacy-image-retry-attempts = Automatische Wiederholungen für Bilder
admin-settings-help-privacy-image-retry-attempts = Maximale Wiederholungen bei vorübergehenden Bildfehlern in einem sichtbaren Nachrichtenladevorgang. Bereich 0–8; Standard 3. Null deaktiviert automatische Wiederholungen und erhält die manuelle Wiederholung.
admin-settings-key-privacy-image-retry-delay-ms = Verzögerung automatischer Bildwiederholungen
admin-settings-help-privacy-image-retry-delay-ms = Millisekunden zwischen automatischen Bildwiederholungen, nachdem der aktuelle Frame zur Ruhe gekommen ist. Bereich 500–60000; Standard 5000. Dauerhafte Ablehnungen werden nicht automatisch wiederholt.
admin-settings-key-privacy-image-decode-timeout-ms = Zeitlimit der Bilddekodierung
admin-settings-help-privacy-image-decode-timeout-ms = Millisekunden Echtzeit, die für eine Sandbox-Bilddekodierung nach dem Abruf erlaubt sind. Bereich 1000–60000; Standard 10000. Für große Bilder erhöhen; Netzwerk-Zeitlimits bleiben getrennt.
admin-settings-key-privacy-image-concurrency-mode = Profil der Bildparallelität
admin-settings-help-privacy-image-concurrency-mode = Automatisch passt sich an CPU-Kapazität, Container-Speicher und die Zahl der freigegebenen Bilder jeder Nachricht an. Geringe Ressourcen, Ausgewogen und Hoher Durchsatz passen sich ebenfalls an, mit unterschiedlichen Sicherheitsgrenzen. Manuell verwendet die numerischen Obergrenzen direkt. Das Anwenden eines Profils ändert nur diese Einstellung.
admin-settings-cat-image-proxy = Bild-Proxy
admin-settings-key-privacy-image-ram-max-mi-b = Maximum der RAM-Stufe
admin-settings-help-privacy-image-ram-max-mi-b = Obergrenze in MiB für die Bildstufe im Arbeitsspeicher, zusätzlich zur automatischen oder manuellen Cache-Größe. Null setzt keine Grenze. Halten Sie sie klein und lassen Sie die Festplattenstufe den Großteil aufnehmen. Bereich 0–1048576; Standard 0.
admin-settings-key-privacy-image-disk-enabled = Festplattenstufe für Bilder
admin-settings-help-privacy-image-disk-enabled = Behält bereits neu kodierte Bilder hinter der RAM-Stufe auf der Festplatte. Standardmäßig aus; wenn das Verzeichnis fehlt, voll oder schreibgeschützt ist, läuft der Cache stillschweigend nur im RAM weiter. Es wird nur die Ausgabe des Jails gespeichert, unter gehashten Dateinamen.
admin-settings-key-privacy-image-disk-path = Festplattenverzeichnis für Bilder
admin-settings-help-privacy-image-disk-path = Absolutes Verzeichnis für die Festplattenstufe. Leer verwendet einen Ordner image-cache neben der Datenbankdatei. Binden Sie dort ein beschreibbares Volume ein, wenn das Root-Dateisystem schreibgeschützt ist. Relative Pfade und .. werden abgelehnt und lassen die Stufe aus.
admin-settings-key-privacy-image-disk-mode = Dimensionierung der Bild-Festplatte
admin-settings-help-privacy-image-disk-mode = Automatisch verwendet ein Zehntel des Platzes, den dieser Cache nutzen könnte (freier Platz plus der bereits belegte), zwischen Unter- und Obergrenze und nie mehr als die Hälfte dieses Platzes. Manuell verwendet die unten stehende Maximalgröße exakt. Standard automatisch.
admin-settings-key-privacy-image-disk-max-mi-b = Größe der Bild-Festplatte (manuell)
admin-settings-help-privacy-image-disk-max-mi-b = Größe der Festplattenstufe in MiB im manuellen Modus; die Anzeige zeigt in beiden Fällen die wirksame Größe. Bereich 1–1048576; Standard 512.
admin-settings-key-privacy-image-disk-floor-mi-b = Untergrenze der automatischen Festplattengröße
admin-settings-help-privacy-image-disk-floor-mi-b = Kleinste Festplattenstufe in MiB im automatischen Modus; wird auch verwendet, solange der freie Platz unbekannt ist. Bereich 0–1048576; Standard 64.
admin-settings-key-privacy-image-disk-ceiling-mi-b = Obergrenze der automatischen Festplattengröße
admin-settings-help-privacy-image-disk-ceiling-mi-b = Größte Festplattenstufe in MiB im automatischen Modus. Bereich 0–1048576; Standard 4096.
admin-settings-key-privacy-image-disk-entries = Eintragslimit der Festplatte
admin-settings-help-privacy-image-disk-entries = Höchste Anzahl auf der Festplatte gehaltener Bilder. Jeder Eintrag kostet zudem eine feste Menge RAM im Index, die auf die RAM-Stufe angerechnet wird. Bereich 16–1048576; Standard 16384.
admin-settings-key-privacy-image-disk-entry-bytes = Größtes Festplattenbild
admin-settings-help-privacy-image-disk-entry-bytes = Größtes einzelnes neu kodiertes Bild, das auf die Festplatte geschrieben wird, in Bytes. Bilder über dem Eintragslimit des RAM, aber innerhalb dieses Limits, gehen direkt auf die Festplatte. Bereich 65536–67108864; Standard 8388608.
admin-settings-key-privacy-image-disk-min-hits = Treffer vor einem Festplattenschreibvorgang
admin-settings-help-privacy-image-disk-min-hits = Treffer, die ein RAM-Eintrag braucht, bevor seine Verdrängung ihn auf die Festplatte schreibt, statt ihn zu verwerfen. Null schreibt jeden verdrängten Eintrag, dessen Bewertung die Herabstufungsschwelle überschreitet. Bereich 0–100; Standard 1.
admin-settings-key-privacy-image-disk-write-queue-mi-b = Schreibwarteschlange der Festplatte
admin-settings-help-privacy-image-disk-write-queue-mi-b = Bilddaten in MiB, die auf den Hintergrund-Schreiber der Festplatte warten dürfen. Sie werden auf die RAM-Stufe angerechnet; darüber hinaus werden verdrängte Bilder verworfen, statt eingereiht zu werden. Bereich 1–256; Standard 8.
admin-settings-key-privacy-image-ttl-step-secs = Lebensdauerschritt pro Treffer
admin-settings-help-privacy-image-ttl-step-secs = Sekunden, die bei einem Treffer zur verbleibenden Lebensdauer eines Bildes addiert werden, multipliziert mit 1 plus dem Logarithmus seiner abgeklungenen Trefferzahl, sodass häufig genutzte Bilder länger leben. Null belässt jedes Bild bei der Basislebensdauer. Bereich 0–86400; Standard 600.
admin-settings-key-privacy-image-ttl-max-secs = Längste verlängerte Lebensdauer
admin-settings-help-privacy-image-ttl-max-secs = Längste verbleibende Lebensdauer in Sekunden, die Treffer aufbauen können. Bereich 60–2592000; Standard 86400.
admin-settings-key-privacy-image-max-age-secs = Höchstalter von Bildern
admin-settings-help-privacy-image-max-age-secs = Alter in Sekunden, nach dem ein Bild unabhängig von seinen Treffern erneut abgerufen wird. Der Abrufpfad behält kein Upstream-ETag oder Last-Modified, daher gibt es keine bedingte Revalidierung; das Bild wird einfach neu abgerufen. Bereich 3600–7776000; Standard 604800.
admin-settings-key-privacy-image-score-half-life-secs = Halbwertszeit der Bewertung
admin-settings-help-privacy-image-score-half-life-secs = Sekunden, in denen sich Trefferzahl und Aktualität eines Bildes in der Prioritätsbewertung halbieren, die über RAM, Festplatte und Verdrängung entscheidet. Bereich 60–604800; Standard 3600.
admin-settings-key-privacy-image-score-demote-milli = Herabstufungsbewertung
admin-settings-help-privacy-image-score-demote-milli = Bewertung in Tausendsteln, unter der ein RAM-Bild auf die Festplatte herabgestuft oder verworfen wird. Die Heraufstufungsschwelle ist diese plus die Hysterese. Bereich 0–10000; Standard 300.
admin-settings-key-privacy-image-score-hysteresis-percent = Heraufstufungs-Hysterese
admin-settings-help-privacy-image-score-hysteresis-percent = Prozentsatz, um den die Heraufstufungsschwelle über der Herabstufungsschwelle liegt. Eine Bewertung dazwischen lässt das Bild, wo es ist. Bereich 0–500; Standard 50.
admin-settings-key-privacy-image-context-open-percent = Gewicht geöffneter Nachrichten
admin-settings-help-privacy-image-context-open-percent = Bewertungsgewicht in Prozent für Bilder, die für eine Nachricht angefordert werden. Die Bilder einer Nachricht zählen nach zehn inaktiven Minuten als Kürzlich. Bereich 100–1000; Standard 200.
admin-settings-key-privacy-image-context-recent-percent = Gewicht kürzlicher Nachrichten
admin-settings-help-privacy-image-context-recent-percent = Bewertungsgewicht in Prozent für kürzlich verwendete Bilder. Anfragen, die keine Nachricht nennen, haben das Gewicht 100. Bereich 100–1000; Standard 150.
admin-settings-key-privacy-image-account-share-percent = RAM-Anteil pro Konto
admin-settings-help-privacy-image-account-share-percent = Größter Anteil der RAM-Stufe, den die Einträge eines Kontos füllen dürfen, während andere Konten den Cache nutzen; darüber verdrängt das Konto zuerst seine eigenen am niedrigsten bewerteten Bilder. Ein einzelnes Konto darf alles nutzen. Bereich 1–100; Standard 50.
admin-settings-key-privacy-image-sweep-secs = Intervall der Cache-Bereinigung
admin-settings-help-privacy-image-sweep-secs = Sekunden zwischen Hintergrund-Durchläufen, die Bilder ablaufen lassen, einen begrenzten Stapel neu bewerten, kalte RAM-Bilder unter Druck herabstufen und den freien Festplattenplatz erneut prüfen. Bereich 5–3600; Standard 30.
admin-settings-key-privacy-image-snapshot-secs = Intervall der Index-Snapshots
admin-settings-help-privacy-image-snapshot-secs = Sekunden zwischen atomaren Snapshots des Festplattenindex, die Trefferzahlen und verlängerte Lebensdauern über Neustarts hinweg ohne Schreibvorgang pro Treffer erhalten. Null schaltet Snapshots aus; ein Neustart liest dann den Header jeder Datei. Bereich 0–86400; Standard 300.
admin-settings-key-privacy-image-sweep-batch = Stapelgröße der Bereinigung
admin-settings-help-privacy-image-sweep-batch = Einträge, die ein Wartungsdurchlauf ablaufen lassen, neu bewerten oder verdrängen darf. Begrenzt die Arbeit unter der Cache-Sperre. Bereich 16–8192; Standard 256.
admin-image-proxy-title = Anzeige des Bild-Caches
admin-image-proxy-unavailable = Die Anzeige des Bild-Caches ist nicht verfügbar.
admin-image-proxy-unknown = unbekannt
admin-image-proxy-ram = RAM-Stufe: { $used } von { $budget } belegt (konfiguriert { $configured }), { $entries } Bilder, { $hot } heiß, { $cold } kalt, Trefferquote { $hit }.
admin-image-proxy-disk-off = Festplattenstufe: aus.
admin-image-proxy-disk = Festplattenstufe: { $bytes } von wirksamen { $budget } belegt ({ $mode }; Untergrenze { $floor }, Obergrenze { $ceiling }, manuell { $manual }), { $entries } Bilder, { $hot } heiß, { $cold } kalt, Trefferquote { $hit }.
admin-image-proxy-disk-inactive = Festplattenstufe: an, aber nicht aktiv. { $reason }
admin-image-proxy-moves = Heraufstufungen pro Minute: { $perMin }. Heraufstufungen gesamt { $promotions }, Herabstufungen { $demotions }, Verwerfungen { $drops }. Indexeinträge: { $index }.

# -- Machine translation (2026-11): added to match en ---------------------
admin-image-proxy-refresh = Anzeige aktualisieren
admin-settings-key-security-trust-enabled = Vertrauensstufen
admin-settings-help-security-trust-enabled = Weist jeder Anfrage eine Stufe zu: angemeldet, bekannte Quelle (eine kürzlich erfolgreiche Anmeldung von dieser Adresse) oder unbekannt. Es ändern sich nur die Kontingente anonymer Anfragen; Anmeldedaten, zweite Faktoren und Sperren werden nie gelockert.
admin-settings-key-security-trust-known-ttl-days = Fenster für bekannte Quellen (Tage)
admin-settings-help-security-trust-known-ttl-days = Wie lange eine erfolgreiche Anmeldung ihre Adresse als bekannt hält. Fehlgeschlagene Versuche zählen nie.
admin-settings-key-security-trust-known-capacity = Aufbewahrte bekannte Quellen
admin-settings-help-security-trust-known-capacity = Höchste Anzahl bekannter Quellen im Speicher (je ein gesalzener Hash, ein Land und eine Kontenanzahl). Die älteste wird zuerst verworfen.
admin-settings-key-security-trust-persist = Bekannte Quellen über Neustarts hinweg speichern
admin-settings-help-security-trust-persist = Speichert die Tabelle bekannter Quellen (schlüsselabhängige Hashes, Land, Zeitpunkt der letzten Sichtung, Kontenanzahl) im Einstellungsspeicher und lädt sie beim Start. Einträge, die älter als das Fenster sind, werden verworfen.
admin-settings-key-security-trust-persist-interval-secs = Speicherintervall für bekannte Quellen (Sekunden)
admin-settings-help-security-trust-persist-interval-secs = Sekunden zwischen Speicherungen, 30 bis 3600. Gespeichert wird nur, wenn sich die Tabelle geändert hat, und einmal mehr beim geordneten Herunterfahren.
admin-settings-key-security-trust-unknown-percent = Kontingent unbekannter Quellen (%)
admin-settings-help-security-trust-unknown-percent = Kontingent anonymer Anfragen einer unbekannten Quelle als Prozentsatz der konfigurierten Limits. 100 behält die Basislinie; niedriger ist strenger.
admin-settings-key-security-trust-known-percent = Kontingent bekannter Quellen (%)
admin-settings-help-security-trust-known-percent = Kontingent anonymer Anfragen einer bekannten Quelle. 200 verdoppelt die Anfragebudgets und halbiert die Wartezeit bei der Anmeldung und die Challenge-Schwierigkeit. Umgeht nie Anmeldedaten, zweite Faktoren oder Sperren.
admin-settings-key-security-trust-ua-missing = Bots: leerer User-Agent
admin-settings-help-security-trust-ua-missing = Aktion für anonyme Anfragen mit leerem User-Agent: allow, limit oder block. Angemeldete Sitzungen sind nie betroffen.
admin-settings-key-security-trust-ua-uncommon = Bots: nicht erkannter User-Agent
admin-settings-help-security-trust-ua-uncommon = Aktion für anonyme Anfragen mit nicht erkanntem User-Agent: allow, limit oder block.
admin-settings-key-security-trust-ua-automation = Bots: Skripte und Automatisierung
admin-settings-help-security-trust-ua-automation = Aktion für anonyme Anfragen, die sich als Skript, HTTP-Bibliothek, Headless-Browser oder generischer Bot ausgeben: allow, limit oder block.
admin-settings-key-security-trust-ua-search = Bots: Such-Crawler
admin-settings-help-security-trust-ua-search = Aktion für anonyme Anfragen, die sich als Crawler einer Suchmaschine ausgeben: allow, limit oder block.
admin-settings-key-security-trust-ua-ai-training = Bots: KI-Trainings-Crawler
admin-settings-help-security-trust-ua-ai-training = Aktion für anonyme Anfragen, die sich als GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended oder Applebot-Extended ausgeben: allow, limit oder block. Eine Angabe ist ein Header, keine verifizierte Identität.
admin-settings-key-security-trust-ua-ai-search = Bots: KI-Such-Crawler
admin-settings-help-security-trust-ua-ai-search = Aktion für anonyme Anfragen, die sich als OAI-SearchBot, PerplexityBot oder Claude-SearchBot ausgeben: allow, limit oder block.
admin-settings-key-security-trust-ua-ai-user = Bots: KI-Abrufe durch Benutzer
admin-settings-help-security-trust-ua-ai-user = Aktion für anonyme Anfragen, die sich als ChatGPT-User oder Claude-User ausgeben: allow, limit oder block.
admin-settings-key-security-trust-ua-custom = Bots: benutzerdefinierte Muster
admin-settings-help-security-trust-ua-custom = Aktion für anonyme Anfragen, die den benutzerdefinierten User-Agent-Mustern entsprechen: allow, limit oder block.
admin-settings-key-security-trust-admin-geo-enabled = Admin-Erkundung aus unbekannten Ländern blockieren
admin-settings-help-security-trust-admin-geo-enabled = Blockiert eine Quelle, die aus einem Land ohne kürzlich erfolgreiche Anmeldung nach der Admin-Konsole sucht. Erfordert eine lesbare Länderdatenbank und den Schutz vor Admin-Erkundung. MW_ADMIN_GEO_ENABLED=false schaltet dies zur Wiederherstellung aus.
admin-settings-key-security-trust-admin-geo-countries = Immer bekannte Länder (Admin-Sperre)
admin-settings-help-security-trust-admin-geo-countries = ISO-Ländercodes in Großbuchstaben, die immer als bekannt gelten, zusätzlich zu den Ländern kürzlich erfolgreicher Anmeldungen.
admin-settings-key-security-trust-admin-geo-lookup-failure = Quelle ohne Land
admin-settings-help-security-trust-admin-geo-lookup-failure = allow blockiert nicht und protokolliert den Fall; unknown behandelt eine Quelle ohne erkanntes Land als unbekanntes Land.
admin-settings-key-security-trust-admin-geo-threshold = Erkundungen bis zur Sperre
admin-settings-help-security-trust-admin-geo-threshold = Admin-Erkundungen aus einem unbekannten Land innerhalb des Fensters, die eine Sperre auslösen.
admin-settings-key-security-trust-admin-geo-window-secs = Erkundungsfenster (Sekunden)
admin-settings-help-security-trust-admin-geo-window-secs = Fenster für den Erkundungsschwellenwert.
admin-settings-key-security-trust-admin-geo-block-on-first = Bei der ersten Erkundung sperren
admin-settings-help-security-trust-admin-geo-block-on-first = Sperrt bei der ersten Erkundung aus einem unbekannten Land. Erhöht das Risiko, einen reisenden Administrator auszusperren.
admin-settings-key-security-trust-admin-geo-block-secs = Erste Sperre (Sekunden)
admin-settings-help-security-trust-admin-geo-block-secs = Dauer der ersten Sperre.
admin-settings-key-security-trust-admin-geo-second-block-secs = Zweite Sperre (Sekunden)
admin-settings-help-security-trust-admin-geo-second-block-secs = Dauer der zweiten Sperre derselben Quelle innerhalb des Wiederholungszeitraums.
admin-settings-key-security-trust-admin-geo-max-block-secs = Wiederholte Sperre (Sekunden)
admin-settings-help-security-trust-admin-geo-max-block-secs = Dauer der dritten und jeder späteren Sperre innerhalb des Wiederholungszeitraums.
admin-settings-key-security-trust-admin-geo-repeat-days = Wiederholungszeitraum (Tage)
admin-settings-help-security-trust-admin-geo-repeat-days = Eine Sperre innerhalb dieser Anzahl von Tagen nach der vorherigen zählt als Wiederholung und verwendet die nächste, längere Dauer.
admin-settings-key-security-signin-account-geo-mode = Länderrichtlinie für die Anmeldung pro Konto
admin-settings-help-security-signin-account-geo-mode = Standardmäßig aus. Blockiert Anmeldungen aus Ländern, die das Konto bisher nicht genutzt hat, oder aus jedem Land außerhalb seiner Erlaubnisliste. Erlaubte IPs und Länder werden immer durchgelassen. Gilt nur für neue Webmail-Anmeldungen, nie für bestehende Sitzungen oder Protokoll-Clients. Die Ländererkennung nutzt eine IP-Geolokalisierungsdatenbank, die falsch oder veraltet sein kann; VPNs, Mobilfunkanbieter, Proxys und Reisen können eine legitime Anmeldung blockieren oder eine ausländische durchlassen. Fügen Sie vertrauenswürdige IPs oder Länder der Erlaubnisliste hinzu.
admin-settings-key-security-signin-account-geo-allow-countries = Erlaubte Länder des Kontos
admin-settings-help-security-signin-account-geo-allow-countries = ISO-Ländercodes in Großbuchstaben, die für das Konto immer erlaubt sind. Die Ländererkennung nutzt eine IP-Geolokalisierungsdatenbank, die falsch oder veraltet sein kann; VPNs, Mobilfunkanbieter, Proxys und Reisen können eine legitime Anmeldung blockieren oder eine ausländische durchlassen. Fügen Sie vertrauenswürdige IPs oder Länder der Erlaubnisliste hinzu.
admin-settings-key-security-signin-account-geo-allow-ips = Erlaubte IPs des Kontos
admin-settings-help-security-signin-account-geo-allow-ips = IP-Adressen oder CIDR-Netzwerke, die für das Konto immer erlaubt sind. Eine einzelne IPv6-Adresse bedeutet ihr /64.
admin-settings-key-security-signin-account-geo-global-allow-countries = Erlaubte Länder für jedes Konto
admin-settings-help-security-signin-account-geo-global-allow-countries = ISO-Ländercodes in Großbuchstaben, die für jedes Konto erlaubt sind, zusätzlich zur eigenen Liste jedes Kontos.
admin-settings-key-security-signin-account-geo-global-allow-ips = Erlaubte IPs für jedes Konto
admin-settings-help-security-signin-account-geo-global-allow-ips = IP-Adressen oder CIDR-Netzwerke (ein Büro oder VPN), die für jedes Konto erlaubt sind, zusätzlich zur eigenen Liste jedes Kontos.
admin-settings-key-security-signin-account-geo-lookup-failure = Quelle ohne Land (Kontorichtlinie)
admin-settings-help-security-signin-account-geo-lookup-failure = Wenn für die Quelle kein Land erkannt wird: die Anmeldung erlauben und protokollieren oder verweigern.
admin-settings-key-security-signin-account-geo-history-days = Länderverlauf des Kontos (Tage)
admin-settings-help-security-signin-account-geo-history-days = Wie viele Tage ein Land im eigenen Anmeldeverlauf eines Kontos für die Richtlinie zu ungewöhnlichen Ländern erhalten bleibt. Er wird getrennt vom Authentifizierungsprotokoll geführt, sodass ein Kürzen des Protokolls ihn nicht verkürzt.
admin-settings-key-security-geo-no-database = Keine Länderdatenbank: Standardverhalten
admin-settings-help-security-geo-no-database = Was Länderfunktionen tun, wenn keine Datenbank geladen ist: erlauben, die Quelle als unbekanntes Land behandeln oder blockieren (block gilt nur für Anmeldungen). Die eigene Einstellung einer Funktion gilt ebenfalls, und die strengere gewinnt.
admin-settings-key-security-geo-not-found = Adresse nicht in der Datenbank: Standardverhalten
admin-settings-help-security-geo-not-found = Was Länderfunktionen tun, wenn eine Datenbank geladen ist, aber keinen Eintrag für die Adresse hat. Private, Loopback- und Link-Local-Adressen sind lokale Netzwerke und nie betroffen.
admin-settings-key-security-signin-geo-no-database = Anmelderichtlinie der Instanz ohne Datenbank
admin-settings-help-security-signin-geo-no-database = Wenn die Länderrichtlinie der Instanz für die Anmeldung an ist und keine Datenbank geladen ist: Anmeldungen blockieren (fail closed) oder ungeprüft erlauben. Lokale Netzwerke sind immer erlaubt.
admin-settings-key-security-geo-session-lookup-failure = Sitzungswarnungen ohne Land
admin-settings-help-security-geo-session-lookup-failure = Wenn das Land einer angemeldeten Anfrage nicht ermittelt werden kann: ignorieren oder eine Sitzungswarnung pro Sitzung und Tag auslösen. Eine Sitzung wird dadurch nie blockiert oder widerrufen.
admin-settings-key-security-protocol-framing-mode = Protokollwächter: Request-Framing
admin-settings-help-security-protocol-framing-mode = Klasse Request-Smuggling: Content-Length zusammen mit Transfer-Encoding, Transfer-Encoding außer genau chunked (oder bei HTTP/1.0 und HTTP/2), nicht numerische oder widersprüchliche Content-Length. enforce antwortet mit 400 und schließt die Verbindung; detect zählt nur. Eine Ausnahme über Routenpräfix gilt nie für diese Klasse.
admin-settings-key-security-protocol-host-mode = Protokollwächter: Host-Syntax
admin-settings-help-security-protocol-host-mode = Syntax des Host-Headers: bei HTTP/1.1 fehlend, leer, wiederholt oder mit Userinfo, Leerraum oder Pfadzeichen. IPv6-Literale, Ports und ein abschließender Punkt werden akzeptiert. enforce antwortet mit 400 und schließt die Verbindung.
admin-settings-key-security-protocol-host-pinning-mode = Protokollwächter: Host muss dem öffentlichen Namen entsprechen
admin-settings-help-security-protocol-host-pinning-mode = Host entspricht weder dem Host von MW_PUBLIC_URL noch einem Eintrag aus allowedHosts. Wirkungslos, solange keiner einen Host nennt. Standardmäßig detect, weil Proxys und Health-Checks einen internen Namen senden können.
admin-settings-key-security-protocol-path-control-mode = Protokollwächter: Steuerzeichen im Pfad
admin-settings-help-security-protocol-path-control-mode = NUL, Steuerzeichen oder kodiertes CR/LF (%00, %0d, %0a) im Anfragepfad. enforce antwortet mit 400. Nie durch ein Routenpräfix ausgenommen.
admin-settings-key-security-protocol-target-mode = Protokollwächter: Tricks beim Request-Target
admin-settings-help-security-protocol-target-mode = Doppelte oder überlange Prozentkodierung von Punkten und Schrägstrichen, kodierte Punkte, Punktsegmente, leere Pfadsegmente (//), Backslashes, rohe Nicht-ASCII-Bytes und ein Absolute-Form-Target mit einer anderen Autorität als Host.
admin-settings-key-security-protocol-path-length-mode = Protokollwächter: Länge von Pfad und Query
admin-settings-help-security-protocol-path-length-mode = Pfad oder Query länger als maxPathBytes oder maxQueryBytes. enforce antwortet mit 414.
admin-settings-key-security-protocol-method-mode = Protokollwächter: HTTP-Methoden
admin-settings-help-security-protocol-method-mode = Methode außerhalb von allowedMethods (standardmäßig TRACE, TRACK, CONNECT sowie unbekannte oder kleingeschriebene Methoden). enforce antwortet mit 405.
admin-settings-key-security-protocol-duplicate-headers-mode = Protokollwächter: wiederholte Header mit Einzelwert
admin-settings-help-security-protocol-duplicate-headers-mode = Ein Header, der nur einmal vorkommen sollte, kommt mehrfach vor (Content-Type, Authorization, Origin, Referer, User-Agent, Sec-Fetch-*, bedingte und Range-Header), eine wiederholte Content-Length mit gleichen Werten oder wiederholte Cookie-Header bei HTTP/1.x.
admin-settings-key-security-protocol-forwarded-mode = Protokollwächter: Weiterleitungs-Header von nicht vertrauenswürdigen Peers
admin-settings-help-security-protocol-forwarded-mode = Forwarded, X-Forwarded-* oder X-Real-IP, gesendet von einem Peer außerhalb von MW_TRUSTED_PROXIES. Die Header werden so oder so nie als vertrauenswürdig behandelt; dies protokolliert den Versuch. Er zählt nur dann für den adaptiven Schutz, wenn vertrauenswürdige Proxys konfiguriert sind.
admin-settings-key-security-protocol-header-hygiene-mode = Protokollwächter: Header-Hygiene
admin-settings-help-security-protocol-header-hygiene-mode = Header-Anzahl, ein einzelner Wert oder der Cookie-Header über den konfigurierten Grenzen; Nicht-ASCII-Header-Bytes; Upgrade oder Connection: upgrade außerhalb der WebSocket-Route; Connection, die Header wie Content-Length oder Host benennt. enforce antwortet mit 431 bei Größen- und Anzahlbefunden und mit 400 bei den übrigen.
admin-settings-key-security-protocol-body-mode = Protokollwächter: Framing des Request-Bodys
admin-settings-help-security-protocol-body-mode = GET oder HEAD mit Body, Content-Length über maxBodyBytes (413), Expect außer 100-continue (417) oder ohne Body.
admin-settings-key-security-protocol-max-path-bytes = Protokollwächter: längster Pfad (Bytes)
admin-settings-help-security-protocol-max-path-bytes = Längster Anfragepfad, bevor ihn die Klasse für die Pfadlänge meldet.
admin-settings-key-security-protocol-max-query-bytes = Protokollwächter: längste Query (Bytes)
admin-settings-help-security-protocol-max-query-bytes = Längster Query-String, bevor ihn die Klasse für die Pfadlänge meldet.
admin-settings-key-security-protocol-max-header-count = Protokollwächter: meiste Header
admin-settings-help-security-protocol-max-header-count = Meiste Header-Felder in einer Anfrage, bevor die Klasse für Header-Hygiene sie meldet. Die eigene harte Grenze des Listeners (MW_HTTP_MAX_HEADERS) gilt weiterhin.
admin-settings-key-security-protocol-max-header-value-bytes = Protokollwächter: längster Header-Wert (Bytes)
admin-settings-help-security-protocol-max-header-value-bytes = Längster einzelner Header-Wert, bevor ihn die Klasse für Header-Hygiene meldet.
admin-settings-key-security-protocol-max-cookie-bytes = Protokollwächter: größter Cookie-Header (Bytes)
admin-settings-help-security-protocol-max-cookie-bytes = Gesamtgröße der Cookie-Header, bevor sie die Klasse für Header-Hygiene meldet.
admin-settings-key-security-protocol-max-body-bytes = Protokollwächter: größter deklarierter Body (Bytes)
admin-settings-help-security-protocol-max-body-bytes = Größte Content-Length, die die Body-Klasse ohne Meldung akzeptiert. Halten Sie den Wert über dem größten Upload oder Import, den eine Route akzeptiert; eine Ausnahme über Routenpräfix kann ihn für eine Route erhöhen.
admin-settings-key-security-protocol-allowed-methods = Protokollwächter: erlaubte Methoden
admin-settings-help-security-protocol-allowed-methods = Methoden, die die Methodenklasse akzeptiert. Der Standard listet die gewöhnlichen HTTP-Verben sowie die Verben von WebDAV, CalDAV und CardDAV; der Server stellt derzeit keine eingehenden DAV-Routen bereit, daher enden diese am Router in einem 405.
admin-settings-key-security-protocol-allowed-hosts = Protokollwächter: zusätzlich akzeptierte Hosts
admin-settings-help-security-protocol-allowed-hosts = Hostnamen, die die Host-Pinning-Klasse zusätzlich zum Host von MW_PUBLIC_URL akzeptiert, zum Beispiel ein interner Name, den ein Proxy oder Health-Check verwendet. Ports und ein abschließender Punkt werden ignoriert.
admin-settings-key-security-protocol-exempt-prefixes = Protokollwächter: ausgenommene Routenpräfixe
admin-settings-help-security-protocol-exempt-prefixes = Routenpräfixe (jeweils mit / beginnend und länger als ein Zeichen), für die die heuristischen Klassen nicht laufen, für einen legitimen Client, der dies braucht, zum Beispiel eine Route für große Uploads. Verglichen werden ganze Pfadsegmente. Befunde zu Framing und Pfadsteuerung werden nie ausgenommen, und / wird ignoriert.
admin-settings-key-security-health-mode = Zugriff auf Health-Endpunkte
admin-settings-help-security-health-mode = Wer /healthz und /readyz erreichen darf. internal-only: Client-Adressen für Loopback, privat, Link-Local und Unique-Local. allowlist: diese plus die unten stehenden Netzwerke. open: jeder. Alle anderen erhalten dasselbe 404 wie bei einem unbekannten Pfad. Eine Anfrage mit dem Token MW_HEALTH_TOKEN wird immer zugelassen. Die Client-Adresse ist die über vertrauenswürdige Proxys aufgelöste: Hinter einem Reverse-Proxy ohne MW_FORWARDED_MODE und MW_TRUSTED_PROXIES wirkt jede Anfrage wie die private Adresse des Proxys, sodass das ganze Internet zugelassen wird.
admin-settings-key-security-health-networks = Netzwerke für Health-Endpunkte
admin-settings-help-security-health-networks = IP-Adressen oder CIDR-Netzwerke, die im Modus allowlist zusätzlich zu den internen Bereichen /healthz und /readyz erreichen dürfen, zum Beispiel ein externer Uptime-Monitor.
admin-settings-key-security-spray-enabled = Erkennung von Password-Spraying
admin-settings-help-security-spray-enabled = Zählt die verschiedenen Benutzernamen, mit denen eine Quelle scheitert. Von jedem Benutzernamen wird nur ein schlüsselabhängiger Hash im Speicher gehalten.
admin-settings-key-security-spray-window-secs = Spraying-Fenster (Sekunden)
admin-settings-help-security-spray-window-secs = Fenster, in dem fehlgeschlagene Anmeldungen einer Quelle nach verschiedenen Benutzernamen gezählt werden. Standard 900.
admin-settings-key-security-spray-challenge-at = Spraying: Challenge-Schwellenwert
admin-settings-help-security-spray-challenge-at = Verschiedene fehlschlagende Benutzernamen einer Quelle innerhalb des Fensters, die die Proof-of-Work-Challenge für diese Quelle erzwingen. Erfolgreiche Anmeldungen zählen nie, sodass eine gemeinsam genutzte Büro- oder Campus-Adresse nicht betroffen ist. Standard 10.
admin-settings-key-security-spray-pace-at = Spraying: Drosselungsschwellenwert
admin-settings-help-security-spray-pace-at = Verschiedene fehlschlagende Benutzernamen, die zusätzliche Wartezeit zwischen Versuchen der Quelle hinzufügen. Standard 15.
admin-settings-key-security-spray-lock-at = Spraying: Sperrschwellenwert
admin-settings-help-security-spray-lock-at = Verschiedene fehlschlagende Benutzernamen, die die Quelle über die bestehende Anmeldesperre für die Sperrdauer sperren. Standard 25.
admin-settings-key-security-spray-pace-secs = Spraying: Wartezeit der Drosselung (Sekunden)
admin-settings-help-security-spray-pace-secs = Sekunden zwischen Versuchen einer Quelle auf oder über dem Drosselungsschwellenwert. Standard 10.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-security-spray-lock-secs = Spraying: Sperrdauer (Sekunden)
admin-settings-help-security-spray-lock-secs = Dauer der vorübergehenden Sperre. Wenn ein Administrator die Quelle entsperrt, wird auch ihr Zähler zurückgesetzt. Standard 900.
admin-settings-key-security-spray-known-multiplier = Spraying: Multiplikator für bekannte Quellen
admin-settings-help-security-spray-known-multiplier = Multiplikator auf die drei Schwellenwerte für eine bekannte Quelle, also eine mit kürzlich erfolgreicher Anmeldung. Standard 3.
admin-settings-key-security-spray-capacity = Spraying: erfasste Quellen
admin-settings-help-security-spray-capacity = Höchste Anzahl im Speicher erfasster Quellen; die am längsten inaktiven werden zuerst verworfen. Standard 20000.
admin-settings-key-security-spike-enabled = Instanzweite Spitzenerkennung
admin-settings-help-security-spike-enabled = Lernt eine Basislinie fehlgeschlagener Anmeldungen und wechselt in den erhöhten Modus, wenn die gesamte Instanz weit darüber liegt. Die Instanz wird nie gesperrt.
admin-settings-key-security-spike-multiplier-percent = Spitze: Multiplikator (%)
admin-settings-help-security-spike-multiplier-percent = Ein Signal zählt, wenn es mindestens diesen Prozentsatz seiner gelernten Basislinie erreicht (400 = das Vierfache) und über seiner absoluten Untergrenze liegt.
admin-settings-key-security-spike-min-failures-per-min = Spitze: Mindestfehler pro Minute
admin-settings-help-security-spike-min-failures-per-min = Untergrenze: fehlgeschlagene Anmeldungen pro Minute, unter denen das Signal der Fehlerrate nie zählt. Standard 20.
admin-settings-key-security-spike-min-sources = Spitze: Mindestzahl fehlschlagender Quellen
admin-settings-help-security-spike-min-sources = Untergrenze: verschiedene fehlschlagende Quellen in einem Fünf-Minuten-Intervall, unter denen dieses Signal nie zählt. Standard 10.
admin-settings-key-security-spike-min-usernames = Spitze: Mindestzahl fehlschlagender Benutzernamen
admin-settings-help-security-spike-min-usernames = Untergrenze: verschiedene fehlschlagende Benutzernamen in einem Fünf-Minuten-Intervall, unter denen dieses Signal nie zählt. Standard 30.
admin-settings-key-security-spike-signals-required = Spitze: erforderliche Signale
admin-settings-help-security-spike-signals-required = Signale, die gemeinsam über ihrer Basislinie und Untergrenze liegen müssen. Die Signatur eines verteilten Sprays zählt als zwei. Standard 2.
admin-settings-key-security-spike-warmup-hours = Spitze: Aufwärmphase (Stunden)
admin-settings-help-security-spike-warmup-hours = Stunden beobachteten Datenverkehrs, bevor der erhöhte Modus beginnen kann. Bis dahin lernt die Basislinie nur. Standard 48.
admin-settings-key-security-spike-adapt-hours = Spitze: Anpassung der Basislinie (Stunden)
admin-settings-help-security-spike-adapt-hours = Zeitkonstante der Basislinie. Länger nimmt Wachstum langsamer auf. Eine Spitze aktualisiert die Basislinie nie. Standard 168.
admin-settings-key-security-spike-duration-secs = Spitze: Dauer des erhöhten Modus (Sekunden)
admin-settings-help-security-spike-duration-secs = Wie lange der erhöhte Modus nach dem letzten Beleg einer Spitze anhält; er wird verlängert, solange Signale über der Hälfte ihres Auslösepegels bleiben. Standard 1800.
admin-settings-key-security-spike-force-challenge = Spitze: Challenge erzwingen
admin-settings-help-security-spike-force-challenge = Verlangt im erhöhten Modus die Proof-of-Work-Challenge für anonyme Anmeldungen von unbekannten Quellen. Authentifizierte Benutzer und bekannte Quellen sind nicht betroffen.
admin-settings-key-security-spike-pace-factor-percent = Spitze: Wartezeit bei der Anmeldung (%)
admin-settings-help-security-spike-pace-factor-percent = Wartezeit bei der Anmeldung für unbekannte Quellen im erhöhten Modus als Prozentsatz der üblichen Wartezeit. Standard 300.
admin-settings-key-security-session-binding-enabled = Sitzungsbindung
admin-settings-help-security-session-binding-enabled = Bindet jede Browsersitzung an schlüsselabhängige Hashes ihrer Client-Attribute. Native Clients mit Bearer-Token, API-Schlüssel und Protokoll-Clients sind ausgenommen.
admin-settings-key-security-session-binding-attributes = Sitzungsbindung: Attribute
admin-settings-help-security-session-binding-attributes = Gebundene Attribute: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). network ist standardmäßig aus, weil sich Mobilfunknetze ändern. Unbekannte Namen werden ignoriert.
admin-settings-key-security-session-binding-strictness = Sitzungsbindung: Strenge
admin-settings-help-security-session-binding-strictness = tolerant akzeptiert Browser-Updates und eine einzelne Änderung eines weichen Attributs; strict markiert jede weiche Abweichung.
admin-settings-key-security-session-binding-webmail-action = Sitzungsbindung: Webmail-Aktion
admin-settings-help-security-session-binding-webmail-action = Was eine Abweichung bei einer Webmail-Sitzung bewirkt: off, detect (nur protokollieren), step-up (Passwort und zweiten Faktor erneut abfragen) oder die Sitzung widerrufen. Standard step-up.
admin-settings-key-security-session-binding-admin-action = Sitzungsbindung: Aktion der Admin-Konsole
admin-settings-help-security-session-binding-admin-action = Was eine Abweichung bei einer Sitzung der Admin-Konsole bewirkt: off, detect, step-up oder revoke. Standard revoke.
admin-settings-key-security-theft-enabled = Erkennung von Sitzungsdiebstahl
admin-settings-help-security-theft-enabled = Erkennt die Wiederverwendung einer rotierten Sitzungs-ID, unmögliche Reisen und gleichzeitige Nutzung aus verschiedenen Ländern.
admin-settings-key-security-theft-rotation-grace-secs = Diebstahl: Karenz bei Rotation (Sekunden)
admin-settings-help-security-theft-rotation-grace-secs = Nach einer Sitzungsrotation darf die alte ID so lange erneut vorgelegt werden (eine laufende Anfrage), bevor sie als Diebstahl zählt. Standard 30.
admin-settings-key-security-theft-min-travel-secs = Diebstahl: minimale Reisezeit (Sekunden)
admin-settings-help-security-theft-min-travel-secs = Ein Länderwechsel, der früher als dies nach der vorherigen Anfrage erfolgt, gilt als unmögliche Reise. Die Ländererkennung kann ungenau sein; VPNs und Mobilfunknetze können falsche Wechsel auslösen. Standard 3600.
admin-settings-key-security-theft-concurrent-use = Diebstahl: gleichzeitige Nutzung
admin-settings-help-security-theft-concurrent-use = Markiert eine Sitzung, die innerhalb von fünf Minuten abwechselnd aus verschiedenen Ländern (oder Netzwerken) genutzt wird: off, country oder network. Die Ländererkennung kann ungenau sein; VPNs und Mobilfunknetze können falsche Wechsel auslösen.
admin-settings-key-security-theft-webmail-action = Diebstahl: Reaktion bei Webmail
admin-settings-help-security-theft-webmail-action = Reaktion auf unmögliche Reisen oder gleichzeitige Nutzung bei einer Webmail-Sitzung: alert, step-up oder revoke. Die Wiederverwendung einer rotierten Sitzung widerruft immer. Standard step-up.
admin-settings-key-security-theft-admin-action = Diebstahl: Reaktion bei der Admin-Konsole
admin-settings-help-security-theft-admin-action = Reaktion auf unmögliche Reisen oder gleichzeitige Nutzung bei einer Sitzung der Admin-Konsole: alert, step-up oder revoke. Standard revoke.
admin-settings-key-security-theft-retention-days = Diebstahl: Aufbewahrung der Einträge (Tage)
admin-settings-help-security-theft-retention-days = Wie lange Marker rotierter Sitzungen und Sicherheitseinträge pro Sitzung aufbewahrt werden; stellen Sie mindestens die längste Sitzungslebensdauer ein. Standard 15.
admin-settings-key-security-alerts-country-change = Warnung bei Länderwechsel
admin-settings-help-security-alerts-country-change = Warnt, wenn eine angemeldete Sitzung aus einem Land genutzt wird, in dem sie nicht gestartet wurde. Die Ländererkennung kann ungenau sein; VPNs und Mobilfunknetze können falsche Wechsel auslösen. Benutzer können dies ändern, sofern nicht gesperrt; das Ausschalten durch einen Benutzer wird nur beachtet, wenn Stummschalten erlaubt ist.
admin-settings-key-security-alerts-country-change-enabled = Warnung bei Länderwechsel: globaler Schalter
admin-settings-help-security-alerts-country-change-enabled = Aus deaktiviert die Warnung bei Länderwechsel für alle. Die Ländererkennung kann ungenau sein; VPNs und Mobilfunknetze können falsche Wechsel auslösen.
admin-settings-key-security-alerts-country-change-cooldown-secs = Warnung bei Länderwechsel: Wartezeit (Sekunden)
admin-settings-help-security-alerts-country-change-cooldown-secs = Mindestzeit zwischen Warnungen bei Länderwechsel einer Sitzung. Jedes neue Land löst pro Sitzung einmal eine Warnung aus. Standard 300.
admin-settings-key-security-alerts-country-change-user-mute = Warnung bei Länderwechsel: Stummschalten durch Benutzer erlauben
admin-settings-help-security-alerts-country-change-user-mute = Ob ein Benutzer seine eigene Warnung bei Länderwechsel ausschalten darf. Wenn aus, warnt der Server weiter, auch wenn die eigene Einstellung des Benutzers aus ist.
admin-settings-key-security-alerts-channels = Kanäle für Sicherheitswarnungen
admin-settings-help-security-alerts-channels = Wie Sicherheitswarnungen zu Sitzungen den Benutzer erreichen: in der App, als Nachricht im eigenen Postfach oder beides. Benutzer können dies ändern, sofern nicht gesperrt.
admin-settings-key-security-alerts-email-enabled = Sicherheitswarnungen per E-Mail
admin-settings-help-security-alerts-email-enabled = Erlaubt, Sicherheitswarnungen zu Sitzungen in das eigene Postfach des Benutzers zuzustellen. Es wird keine Nachricht per SMTP gesendet; ohne Zustellsitzung bleibt die Warnung in der App und im Audit-Protokoll.
admin-settings-key-security-alerts-sensitivity = Empfindlichkeit der Sicherheitswarnungen
admin-settings-help-security-alerts-sensitivity = low warnt nur bei Standortbefunden und löst nie Step-up oder Widerruf aus; normal folgt den Aktionen des Administrators; high verdoppelt das Reiseintervall und prüft auch gleichzeitige Netzwerke. Benutzer können dies ändern, sofern nicht gesperrt.
admin-settings-key-security-alerts-notice-retention-days = Sicherheitshinweise: Aufbewahrung (Tage)
admin-settings-help-security-alerts-notice-retention-days = Wie lange Sicherheitshinweise in der App pro Konto aufbewahrt werden (höchstens 20 pro Konto). Standard 30.
admin-settings-inline-effective = Wirksam:
admin-settings-inline-reset = Änderungen verwerfen
admin-settings-inline-domains = Werte pro Domain
admin-attachments-tab-lists = Erlaubnis- und Sperrlisten
admin-attachments-tab-limits = Limits
admin-attachments-tab-delivery = Download und Vorschau
admin-attachments-tab-warnings = Warnungen
admin-attachments-tab-test = Datei testen
admin-attachments-tab-denied = Sie haben keinen Zugriff auf diesen Tab.
admin-attachments-group-lists = Erlaubnis- und Sperrlisten
admin-attachments-group-limits = Größenlimit
admin-attachments-group-delivery = Download und Vorschau
admin-attachments-group-warnings = Warnungen
admin-attachments-add-extension = Endung, zum Beispiel pdf oder .tar.gz
admin-attachments-add-mime = MIME-Typ, zum Beispiel image/png oder image/*
admin-attachments-add = Hinzufügen
admin-attachments-remove = { $rule } entfernen
admin-attachments-filter = Liste filtern
admin-attachments-list-label = Regeln in dieser Liste
admin-attachments-count = { $count } von { $max } Regeln
admin-attachments-empty-list = Keine Regeln in dieser Liste.
admin-attachments-none-match = Keine Regel entspricht dem Filter.
admin-attachments-bulk = Mehrere einfügen
admin-attachments-bulk-add = Alle hinzufügen
admin-attachments-import = Textdatei importieren
admin-attachments-export = Als Text exportieren
admin-attachments-err-extension = Keine Endung. Verwenden Sie Buchstaben, Ziffern, Bindestriche und Unterstriche, bis zu 32 Zeichen; Punkte können Teile trennen, wie in tar.gz.
admin-attachments-err-mime = Kein MIME-Typ. Verwenden Sie Typ/Untertyp oder Typ/* für jeden Untertyp.
admin-attachments-err-long = Diese Regel ist zu lang.
admin-attachments-invalid = Nicht hinzugefügt, keine gültige Regel: { $entries }
admin-attachments-duplicates = Bereits in der Liste: { $rules }
admin-attachments-overflow = Eine Liste enthält höchstens { $max } Regeln. Die übrigen wurden nicht hinzugefügt.
admin-attachments-conflict-blockedOverAllowed = { $rule } überschneidet sich mit { $other } in der anderen Liste. Sperrregeln haben Vorrang, daher wird sie gesperrt.
admin-attachments-conflict-blockedOverWarn = { $rule } überschneidet sich mit { $other } in der anderen Liste. Eine gesperrte Datei wird vor jeder Warnung abgelehnt, daher erscheint die Warnung nie.
admin-attachments-conflict-allowedAndWarn = { $rule } überschneidet sich mit { $other } in der anderen Liste. Die Datei ist erlaubt und zeigt eine Warnung.
admin-attachments-bytes-unit = Einheit
admin-attachments-unlimited = Kein Größenlimit
admin-attachments-bytes-equals = { $size } ({ $bytes } Bytes)
admin-attachments-list-allowedExtensions = erlaubte Endungen
admin-attachments-list-allowedMimeTypes = erlaubte MIME-Typen
admin-attachments-list-blockedExtensions = gesperrte Endungen
admin-attachments-list-blockedMimeTypes = gesperrte MIME-Typen
admin-attachments-list-warnExtensions = Warn-Endungen
admin-attachments-list-warnMimeTypes = Warn-MIME-Typen
admin-attachments-sim-title = Datei testen
admin-attachments-sim-help = Fragt den Server, wie die gespeicherte Richtlinie eine Datei mit diesem Namen, Typ und dieser Größe behandelt. Nichts wird geändert oder gespeichert. Nachrichten und Anhänge werden nie gelesen.
admin-attachments-sim-filename = Dateiname
admin-attachments-sim-mime = MIME-Typ
admin-attachments-sim-size = Größe
admin-attachments-sim-run = Testen
admin-attachments-sim-invalid = Geben Sie einen Dateinamen und eine Größe von null oder mehr ein.
admin-attachments-sim-allowed = Erlaubt
admin-attachments-sim-warn = Erlaubt mit Warnung
admin-attachments-sim-blocked = Nicht ausgeliefert
admin-attachments-sim-preview-yes = Die Vorschau ist erlaubt.
admin-attachments-sim-preview-no = Die Vorschau ist nicht erlaubt.
admin-attachments-sim-checked = Größe geprüft: { $size }.
admin-attachments-sim-matched = Übereinstimmende Regeln in: { $lists }.
admin-attachments-sim-matched-none = Keine Erlaubnis-, Sperr- oder Warnregel hat gegriffen.
admin-attachments-reason-downloadsDisabled = Downloads sind ausgeschaltet.
admin-attachments-reason-attachmentTooLarge = Die Datei ist größer als das Größenlimit.
admin-attachments-reason-attachmentBlocked = Eine Sperrregel greift.
admin-attachments-reason-attachmentNotAllowed = Es gibt eine Erlaubnisliste, und nichts darin passt.

# -- Machine translation (2026-11): added to match en ---------------------
admin-attachments-reason-attachmentContainerRestricted = Angehängte Nachrichten und TNEF-Dateien werden abgelehnt, solange eine Einschränkung aktiv ist, weil ihr Inhalt nicht aufgelistet ist.
admin-settings-key-mail-reply-images = Bilder in Antworten und Weiterleitungen
admin-settings-help-mail-reply-images = Ob eine Antwort oder Weiterleitung die Bilder der Originalnachricht enthält: ask (Standard), immer erlauben, nur für vertrauenswürdige Absender-Domains erlauben und sonst nachfragen oder immer ablehnen. Sperren Sie diese Einstellung, um sie durchzusetzen. Bilder werden nur gelesen, wenn sie enthalten sind.
admin-settings-key-mail-reply-image-trusted-domains = Vertrauenswürdige Absender-Domains für Bilder
admin-settings-help-mail-reply-image-trusted-domains = Bis zu 200 exakte Absender-Domains, deren Bilder ohne Nachfrage enthalten sind, wenn die Richtlinie vertrauenswürdige Domains lautet. Benutzer ergänzen ihre eigene Liste über die Zitat-Abfrage, sofern diese Einstellung nicht gesperrt ist.
admin-settings-key-mail-send-recall-delay-seconds = Zeitfenster zum Rückgängigmachen des Sendens (Standard)
admin-settings-help-mail-send-recall-delay-seconds = Wie viele Sekunden eine gesendete Nachricht im Postausgang wartet, wo der Absender sie zurückrufen kann, bevor sie zugestellt wird. 0 sendet sofort. Standard 10, bis zu 1800 (30 Minuten). Benutzer wählen ihren eigenen Wert bis zum unten stehenden Limit, sofern Sie dies nicht sperren.
admin-settings-key-mail-send-recall-delay-max-seconds = Zeitfenster zum Rückgängigmachen des Sendens (Limit)
admin-settings-help-mail-send-recall-delay-max-seconds = Das längste Zeitfenster zum Rückgängigmachen des Sendens, das ein Benutzer wählen darf, in Sekunden. Der Server kürzt eine längere Anfrage auf diesen Wert. Standard und Maximum 1800.
admin-settings-key-mail-send-allow-recall = Zurückrufen gesendeter Nachrichten erlauben
admin-settings-help-mail-send-allow-recall = Wenn aus, verlassen Nachrichten sofort das System und können nicht aus dem Postausgang zurückgerufen werden. Zeitgesteuerte Sendungen sind nicht betroffen; sie können bis zu ihrem Zeitpunkt weiterhin abgebrochen werden.
admin-settings-key-mail-send-max-delayed-seconds = Planungshorizont
admin-settings-help-mail-send-max-delayed-seconds = Wie weit im Voraus eine Nachricht geplant werden darf, in Sekunden. Standard 7776000 (90 Tage), Maximum 31536000 (1 Jahr). 0 schaltet das zeitgesteuerte Senden aus.
admin-settings-key-mail-send-max-queued = Wartende Nachrichten pro Postfach
admin-settings-help-mail-send-max-queued = Wie viele Nachrichten ein Postfach gleichzeitig im Postausgang warten haben darf (Rückgängig-Fenster oder geplant). Standard 200. Weitere Sendungen werden mit einer klaren Meldung abgelehnt und nie verworfen.
admin-attachments-sim-pick = Aus einer Datei ausfüllen (optional)
admin-attachments-mime-toggle = Häufige MIME-Typen anzeigen
admin-attachments-mime-hint = Wählen Sie einen häufigen Typ oder geben Sie einen beliebigen anderen Typ/Untertyp ein.
admin-attachments-mime-invalid = Das ist kein Typ/Untertyp wie text/plain. Er wird so gesendet, wie er eingegeben wurde.
admin-attachments-mime-group-documents = Dokumente
admin-attachments-mime-group-images = Bilder
admin-attachments-mime-group-archives = Archive
admin-attachments-mime-group-executables = Ausführbare Dateien und Skripte
admin-attachments-mime-group-media = Audio und Video
admin-attachments-mime-group-email = E-Mail und Kalender
admin-attachments-mime-group-other = Sonstige
admin-settings-key-cookies-enabled = Steuerung der Cookie-Einwilligung
admin-settings-help-cookies-enabled = Zeigt Besuchern die Steuerung der Cookie-Einwilligung. Aus lässt die Anwendung so arbeiten wie ohne die Steuerung.
admin-settings-key-cookies-mode = Einwilligungsmodell
admin-settings-help-cookies-mode = optIn: Nichts Optionales wird gespeichert, bis der Besucher zustimmt. optOut: Die Standardwerte der Kategorien gelten, bis der Besucher ablehnt. noticeOnly: Der Besucher wird informiert, ohne Wahlmöglichkeit.
admin-settings-key-cookies-region-mode = Einwilligungsmodell in strengen Regionen
admin-settings-help-cookies-region-mode = Das Modell, das für Besucher in den strengen Regionen verwendet wird. same verwendet dort ebenfalls das obige Einwilligungsmodell.
admin-settings-key-cookies-strict-regions = Strenge Regionen
admin-settings-help-cookies-strict-regions = ISO-3166-1-Ländercodes, die das Einwilligungsmodell für strenge Regionen verwenden. Standardmäßig die EU- und EWR-Staaten, das Vereinigte Königreich und die Schweiz.
admin-settings-key-cookies-region-header = Länder-Header
admin-settings-help-cookies-region-header = Name des Anfrage-Headers, der von einem Proxy oder CDN gesetzt wird und den Ländercode des Besuchers enthält.
admin-settings-key-cookies-unknown-region = Unbekanntes Land
admin-settings-help-cookies-unknown-region = Welches Modell gilt, wenn kein Land ermittelt werden kann: das Modell für strenge Regionen oder das normale.
admin-settings-key-cookies-functional-enabled = Funktionale Cookies anbieten
admin-settings-help-cookies-functional-enabled = Bietet die Kategorie Einstellungen und Funktionen (Design, Sprache, Merken des Layouts) in der Einwilligungssteuerung an.
admin-settings-key-cookies-functional-default = Standard für Funktionales
admin-settings-help-cookies-functional-default = Anfangszustand der Kategorie Funktionales, wenn das Einwilligungsmodell Standardwerte erlaubt.
admin-settings-key-cookies-analytics-enabled = Analyse-Cookies anbieten
admin-settings-help-cookies-analytics-enabled = Bietet die Kategorie Analyse in der Einwilligungssteuerung an.
admin-settings-key-cookies-analytics-default = Standard für Analyse
admin-settings-help-cookies-analytics-default = Anfangszustand der Kategorie Analyse, wenn das Einwilligungsmodell Standardwerte erlaubt.
admin-settings-key-cookies-marketing-enabled = Marketing-Cookies anbieten
admin-settings-help-cookies-marketing-enabled = Bietet die Kategorie Marketing in der Einwilligungssteuerung an.
admin-settings-key-cookies-marketing-default = Standard für Marketing
admin-settings-help-cookies-marketing-default = Anfangszustand der Kategorie Marketing, wenn das Einwilligungsmodell Standardwerte erlaubt.
admin-settings-key-cookies-layout = Layout
admin-settings-help-cookies-layout = banner ist eine Leiste über die Seite, card eine schwebende Karte, modal ein Dialog, der die Seite blockiert, bis der Besucher wählt.
admin-settings-key-cookies-position = Position
admin-settings-help-cookies-position = Wo die Einwilligungssteuerung auf der Seite erscheint.
admin-settings-key-cookies-reject-prominent = Ablehnen so auffällig wie Zustimmen
admin-settings-help-cookies-reject-prominent = Zeigt Alle ablehnen in derselben Größe und im selben Stil wie Alle akzeptieren.
admin-settings-key-cookies-policy-url = URL der Cookie-Richtlinie
admin-settings-help-cookies-policy-url = Link zu Ihrer Cookie-Richtlinie, der in der Einwilligungssteuerung angezeigt wird.
admin-settings-key-cookies-privacy-url = URL der Datenschutzerklärung
admin-settings-help-cookies-privacy-url = Link zu Ihrer Datenschutzerklärung, der in der Einwilligungssteuerung angezeigt wird.
admin-settings-key-cookies-policy-version = Version der Richtlinie
admin-settings-help-cookies-policy-version = Erhöhen Sie diese Zahl, wenn sich Ihre Cookie-Nutzung ändert, damit Besucher erneut gefragt werden.
admin-settings-key-cookies-lifetime-days = Gültigkeitsdauer der Einwilligung
admin-settings-help-cookies-lifetime-days = Tage, die eine Entscheidung gültig bleibt, bevor der Besucher erneut gefragt wird.
admin-settings-key-cookies-reprompt-on-version = Bei neuer Richtlinienversion erneut fragen
admin-settings-help-cookies-reprompt-on-version = Fragt Besucher erneut, wenn die Version der Richtlinie höher ist als die, auf die sie sich entschieden haben.
admin-settings-key-cookies-honour-gpc = Global Privacy Control beachten
admin-settings-help-cookies-honour-gpc = Behandelt ein Global-Privacy-Control-Signal des Browsers als Ablehnung von Analyse und Marketing.
admin-settings-key-cookies-honour-dnt = Do Not Track beachten
admin-settings-help-cookies-honour-dnt = Behandelt ein Do-Not-Track-Signal des Browsers als Ablehnung von Analyse und Marketing.
admin-settings-key-cookies-log-consent = Einwilligungsprotokoll führen
admin-settings-help-cookies-log-consent = Führt serverseitig einen Nachweis jeder Einwilligungsentscheidung, für Statistiken und Ihre eigenen Unterlagen.
admin-settings-key-cookies-log-retention-days = Aufbewahrung des Einwilligungsprotokolls
admin-settings-help-cookies-log-retention-days = Tage, die Zeilen des Einwilligungsprotokolls aufbewahrt werden, bevor sie gelöscht werden.
admin-settings-key-cookies-scan-interval-hours = Intervall der Speicherprüfung
admin-settings-help-cookies-scan-interval-hours = Stunden zwischen automatischen Prüfungen der Cookies und des Speichers des Browsers. 0 prüft nur auf Anforderung.
admin-settings-key-cookies-unclassified = Nicht klassifizierter Speicher
admin-settings-help-cookies-unclassified = report lässt Speicher, den nichts klassifiziert, unverändert und führt ihn zur Prüfung auf. Die anderen Werte behandeln ihn als diese Kategorie, bis Sie ihn klassifizieren.
admin-settings-key-cookies-text-title = Titel der Einwilligungssteuerung
admin-settings-help-cookies-text-title = Titel der Einwilligungssteuerung pro Sprache. Leer verwendet den eingebauten Text.
admin-settings-key-cookies-text-body = Text der Einwilligungssteuerung
admin-settings-help-cookies-text-body = Erläuternder Text der Einwilligungssteuerung pro Sprache. Leer verwendet den eingebauten Text.
admin-settings-key-cookies-label-necessary = Name der Kategorie Notwendig
admin-settings-help-cookies-label-necessary = Name der Kategorie Notwendig pro Sprache. Leer verwendet den eingebauten Namen.
admin-settings-key-cookies-description-necessary = Beschreibung der Kategorie Notwendig
admin-settings-help-cookies-description-necessary = Beschreibung der Kategorie Notwendig pro Sprache. Leer verwendet die eingebaute Beschreibung.
admin-settings-key-cookies-label-functional = Name der Kategorie Funktional
admin-settings-help-cookies-label-functional = Name der Kategorie Funktional pro Sprache. Leer verwendet den eingebauten Namen.
admin-settings-key-cookies-description-functional = Beschreibung der Kategorie Funktional
admin-settings-help-cookies-description-functional = Beschreibung der Kategorie Funktional pro Sprache. Leer verwendet die eingebaute Beschreibung.
admin-settings-key-cookies-label-analytics = Name der Kategorie Analyse
admin-settings-help-cookies-label-analytics = Name der Kategorie Analyse pro Sprache. Leer verwendet den eingebauten Namen.
admin-settings-key-cookies-description-analytics = Beschreibung der Kategorie Analyse
admin-settings-help-cookies-description-analytics = Beschreibung der Kategorie Analyse pro Sprache. Leer verwendet die eingebaute Beschreibung.
admin-settings-key-cookies-label-marketing = Name der Kategorie Marketing
admin-settings-help-cookies-label-marketing = Name der Kategorie Marketing pro Sprache. Leer verwendet den eingebauten Namen.
admin-settings-key-cookies-description-marketing = Beschreibung der Kategorie Marketing
admin-settings-help-cookies-description-marketing = Beschreibung der Kategorie Marketing pro Sprache. Leer verwendet die eingebaute Beschreibung.
admin-settings-key-encryption-rotation-days = Rotationsintervall des Datenbankschlüssels (Tage)
admin-settings-help-encryption-rotation-days = Tage nach der letzten Schlüsseländerung, nach denen der Datenbankschlüssel als rotationsfällig gemeldet wird. 0 legt keinen Fälligkeitstermin fest. Der Schlüssel selbst wird nie in den Einstellungen gespeichert.
admin-settings-key-encryption-warnings-enabled = Verschlüsselungsbefunde melden
admin-settings-help-encryption-warnings-enabled = Zeigt Klartext-Speicher und eine überfällige Schlüsselrotation als Befund in der Bewertung der Bereitstellung.
admin-settings-key-retention-sweeper-enabled = Aufbewahrungsbereinigung
admin-settings-help-retention-sweeper-enabled = Führt die tägliche Bereinigung aus, die Zeilen löscht, die älter sind als die Limits auf dieser Seite. Das Ausschalten behält jede Zeile, bis es wieder eingeschaltet wird.
admin-settings-key-retention-sweeper-batch-rows = Zeilen pro Stapel
admin-settings-help-retention-sweeper-batch-rows = Zeilen, die eine Löschanweisung entfernt. Kleinere Stapel halten Datenbanksperren kürzer.
admin-settings-key-retention-sweeper-max-batches = Stapel pro Speicher und Lauf
admin-settings-help-retention-sweeper-max-batches = Höchste Anzahl von Löschanweisungen, die ein Speicher in einem Lauf erhält. Ein Speicher mit mehr alten Zeilen wird im nächsten Lauf fortgesetzt.
admin-settings-key-retention-sweeper-jitter-minutes = Streuung der Startzeit (Minuten)
admin-settings-help-retention-sweeper-jitter-minutes = Der nächste Lauf beginnt eine zufällige Anzahl von Minuten bis zu diesem Wert nach 24 Stunden, sodass mehrere Server nicht gleichzeitig bereinigen.
admin-settings-key-retention-sweeper-vacuum-after-rows = Speicherplatz freigeben nach (Zeilen)
admin-settings-help-retention-sweeper-vacuum-after-rows = Wenn ein Lauf mindestens so viele Zeilen löscht, wird die Datenbank verdichtet (VACUUM). 0 verdichtet nach einer Bereinigung nie; der wöchentliche Vacuum-Job ist getrennt.
admin-settings-key-retention-password-change-audit-days = Einträge zu Passwortänderungen (Tage)
admin-settings-help-retention-password-change-audit-days = Tage, die ein Eintrag zu einer Passwortänderung (Konto, Methode, Ergebnis) aufbewahrt wird. 0 behält sie für immer.
admin-settings-key-retention-sso-login-audit-days = Einträge zu Single-Sign-on-Anmeldungen (Tage)
admin-settings-help-retention-sso-login-audit-days = Tage, die ein Eintrag zu einer Single-Sign-on-Anmeldung (Anbieter, gehashtes Subjekt, Ergebnis) aufbewahrt wird. 0 behält sie für immer.
admin-settings-key-retention-dlp-audit-days = Einträge zur Datenverlustprävention (Tage)
admin-settings-help-retention-dlp-audit-days = Tage, die ein Eintrag zu einem Treffer einer Regel zur Datenverlustprävention aufbewahrt wird. 0 behält sie für immer.
admin-settings-key-retention-assist-audit-days = Einträge zum Assistentenzugriff (Tage)
admin-settings-help-retention-assist-audit-days = Tage, die ein Eintrag aufbewahrt wird, wenn ein Assistent eine Fähigkeit nutzt (Akteur, Fähigkeit, Zusammenfassung des Geltungsbereichs). 0 behält sie für immer.
admin-settings-key-retention-alert-events-days = Einträge ausgelöster Alarme (Tage)
admin-settings-help-retention-alert-events-days = Tage, die ein Eintrag eines ausgelösten Administratoralarms aufbewahrt wird. 0 behält sie für immer.
admin-settings-key-retention-job-runs-days = Verlauf der Hintergrund-Jobs (Tage)
admin-settings-help-retention-job-runs-days = Tage, die ein abgeschlossener Lauf eines Hintergrund-Jobs aufbewahrt wird. Ein noch laufender Lauf wird nie entfernt. 0 behält den Verlauf für immer.
admin-settings-key-retention-outbox-days = Einträge gesendeter und abgebrochener Sendungen (Tage)
admin-settings-help-retention-outbox-days = Tage, die der Sendeeintrag einer zugestellten oder abgebrochenen Nachricht aufbewahrt wird. Eine wartende, wiederholte, fehlgeschlagene oder unsichere Nachricht wird nie entfernt. 0 behält sie für immer.
admin-settings-key-retention-expired-credentials-grace-days = Abgelaufene Anmeldedaten (Tage nach Ablauf)
admin-settings-help-retention-expired-credentials-grace-days = Tage, die ein abgelaufenes gemerktes Gerät, eine Anmelde-Challenge, ein Anmelde-Token, ein OAuth-Token, eine Webhook-Nonce, ein Fehlerzähler einer Quelle oder ein Bild-Link-Handle nach Ablauf aufbewahrt wird. 0 entfernt es beim nächsten Lauf.
admin-settings-key-retention-ip-addresses-days = Gespeicherte IP-Adressen reduzieren nach (Tage)
admin-settings-help-retention-ip-addresses-days = Nach so vielen Tagen wird die IP-Adresse im Authentifizierungsverlauf und in Audit-Einträgen gekürzt oder gehasht und die Zeile bleibt erhalten. 0 behält Adressen wie aufgezeichnet.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-retention-ip-addresses-mode = Methode der IP-Reduzierung
admin-settings-help-retention-ip-addresses-mode = Truncate behält das Präfix IPv4 /24 oder IPv6 /48. Hash ersetzt die Adresse durch einen kurzen schlüsselabhängigen Hash, der dieselbe Adresse weiterhin gruppiert.
admin-settings-key-retention-user-enabled = Benutzer dürfen Aufbewahrungsregeln festlegen
admin-settings-help-retention-user-enabled = Benutzer können eigene Aufbewahrungsregeln für Ordner und E-Mails festlegen. Wenn aus, laufen bestehende Benutzerregeln nicht mehr; die unten stehenden organisationsweiten Regeln laufen weiter.
admin-settings-key-retention-user-min-days = Mindesttage, bevor eine Benutzerregel E-Mails löscht
admin-settings-help-retention-user-min-days = Eine Benutzerregel, die E-Mails löscht, darf nicht auf E-Mails wirken, die jünger sind. Ein kleinerer Wert wird beim Speichern der Regel abgelehnt und beim Ausführen angehoben.
admin-settings-key-retention-user-max-days = Höchsttage, die eine Benutzerregel mit dem Löschen warten darf
admin-settings-help-retention-user-max-days = Eine Benutzerregel, die E-Mails löscht, darf nicht länger warten. 0 setzt keine Obergrenze. Ein größerer Wert wird beim Speichern der Regel abgelehnt und beim Ausführen gesenkt.
admin-settings-key-retention-user-max-rules = Höchstzahl Aufbewahrungsregeln pro Benutzer
admin-settings-help-retention-user-max-rules = Die meisten Aufbewahrungsregeln, die ein Benutzer behalten darf.
admin-settings-key-retention-user-allow-permanent-delete = Benutzerregeln dürfen E-Mails endgültig löschen
admin-settings-help-retention-user-allow-permanent-delete = Wenn aus, verschiebt eine Löschregel E-Mails immer in den Papierkorb, und das Rückgängig-Fenster gilt, bevor sie entfernt werden.
admin-settings-key-retention-user-undo-days = Rückgängig-Fenster für Aufbewahrungsregeln (Tage)
admin-settings-help-retention-user-undo-days = Tage, die E-Mails, die eine Aufbewahrungsregel in den Papierkorb verschoben hat, dort bleiben, bevor sie entfernt werden. Ein Benutzer darf ein kürzeres Fenster wählen. 0 entfernt sie beim nächsten Lauf.
admin-settings-key-retention-user-mandatory = Organisationsweite E-Mail-Regeln
admin-settings-help-retention-user-mandatory = Regeln, die jedes Konto erhält, als Einträge Ordnerrolle:Tage, zum Beispiel trash:30 oder junk:30. E-Mails in diesem Ordner, die älter als die Tage sind, werden entfernt. Ein Benutzer kann diese weder ändern noch anhalten noch E-Mails davon ausnehmen. E-Mails eines Kontos unter Rechtssperre werden übersprungen.
admin-settings-key-retention-user-protected-folders = Ordner, die Aufbewahrungsregeln nie berühren
admin-settings-help-retention-user-protected-folders = Ordner, auf die keine Benutzerregel wirkt, nach Rolle (drafts, sent, inbox, archive) oder nach Ordnername. Ein geschützter Ordner hat Vorrang vor einer organisationsweiten Regel für denselben Ordner.
admin-settings-key-retention-user-templates = Den Benutzern angebotene Aufbewahrungsvorlagen
admin-settings-help-retention-user-templates = Vorlagen, die Benutzer in den Einstellungen übernehmen können, nach ID: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.
admin-settings-key-retention-user-run-for-restricted = Benutzerregeln für eingeschränkte Konten ausführen
admin-settings-help-retention-user-run-for-restricted = Ob Aufbewahrungsregeln der Benutzer auch für Konten unter einer Einschränkung der Verarbeitung laufen. Aus überspringt diese Konten.
admin-settings-key-retention-user-interval-hours = Stunden zwischen Läufen der Benutzer-Aufbewahrung
admin-settings-help-retention-user-interval-hours = Stunden zwischen geplanten Läufen des Aufbewahrungsjobs der Benutzer für ein Konto.
admin-settings-key-retention-user-batch-messages = Nachrichten pro Aufbewahrungsaufruf
admin-settings-help-retention-user-batch-messages = Nachrichten, die ein Verschiebe- oder Löschaufruf bearbeitet. Es werden höchstens 200 verwendet.
admin-settings-key-retention-user-max-per-run = Höchstzahl geänderter Nachrichten pro Konto und Lauf
admin-settings-help-retention-user-max-per-run = Die meisten Nachrichten, die der Aufbewahrungsjob der Benutzer für ein Konto in einem Lauf ändert. Der Rest wartet auf den nächsten Lauf.
admin-settings-key-retention-user-pause-ms = Pause zwischen Aufbewahrungsstapeln (ms)
admin-settings-help-retention-user-pause-ms = Pause zwischen Stapeln in Millisekunden, damit eine große Bereinigung den Mailserver nicht belastet.
admin-settings-key-retention-class-short-days = Obergrenze der kurzen Klasse (Tage)
admin-settings-help-retention-class-short-days = Obergrenze für Betriebsdaten: Sitzungen, Tokens, Ratenlimits, Job-Verlauf und Metriken. Gilt, wenn das Klassenlimit an ist.
admin-settings-key-retention-class-medium-days = Obergrenze der mittleren Klasse (Tage)
admin-settings-help-retention-class-medium-days = Obergrenze für den Anmeldeverlauf, den Verlauf des Anfrageschutzes und Sicherheitshinweise. Darf nicht unter der kurzen Klasse liegen; ein niedrigerer Wert wird auf diese angehoben.
admin-settings-key-retention-class-long-days = Aufbewahrung der langen Klasse (Tage)
admin-settings-help-retention-class-long-days = Tage, die die lange Klasse (Sicherheits-Audit, Administratoraktionen, Anfragen, Einwilligung, Tombstones) aufbewahrt wird, Archiv eingeschlossen. 0 behält sie für immer, und die Konsole warnt. Darf nicht unter der mittleren Klasse liegen.
admin-settings-key-retention-class-enforce = Jeden Speicher an seine Klassenobergrenze binden
admin-settings-help-retention-class-enforce = Wenn an, wird das eigene Limit jedes Speichers auf seine Klassenobergrenze begrenzt, und 0 (behalten) zählt als die Obergrenze. Wenn aus, gelten die Limits pro Speicher.
admin-settings-key-retention-archive-enabled = Gealterte Zeilen der langen Klasse archivieren
admin-settings-help-retention-archive-enabled = Verschiebt gealterte Zeilen der langen Klasse in versiegelte, komprimierte Archivdateien, statt sie zu löschen.
admin-settings-key-retention-archive-directory = Archivverzeichnis
admin-settings-help-retention-archive-directory = Verzeichnis für Archivdateien, absolut oder relativ zum Arbeitsverzeichnis des Servers. Segmente des übergeordneten Verzeichnisses (..) werden abgelehnt.
admin-settings-key-retention-archive-batch-rows = Zeilen pro Archivdatei
admin-settings-help-retention-archive-batch-rows = Zeilen, die in eine Archivdatei geschrieben werden.
admin-settings-key-retention-archive-max-segments = Archivdateien pro Speicher und Lauf
admin-settings-help-retention-archive-max-segments = Archivdateien, die ein Lauf für jeden Speicher schreiben darf. Der Rest wartet auf den nächsten Lauf.
admin-settings-key-retention-archive-key-generation = Schlüsselgeneration der Archivversiegelung
admin-settings-help-retention-archive-key-generation = Generation des Schlüssels, der neue Archivdateien versiegelt. Erhöhen Sie sie, um den Schlüssel zu rotieren; frühere Dateien werden weiterhin mit ihrer eigenen Generation verifiziert.
admin-settings-key-retention-archive-verify-days = Tage zwischen Archivprüfungen
admin-settings-help-retention-archive-verify-days = Tage zwischen Integritätsprüfungen der Archivdateien.
admin-settings-key-retention-archive-legal-hold = Rechtssperre für das Archiv
admin-settings-help-retention-archive-legal-hold = Behält jede Archivdatei, unabhängig vom Limit der langen Klasse.
admin-settings-key-logging-file-enabled = Logdateien schreiben
admin-settings-help-logging-file-enabled = Schreibt das Serverlog zusätzlich in rotierende Dateien. Aus belässt die Protokollierung nur auf der Standardausgabe, was ein Container normalerweise nutzt.
admin-settings-key-logging-file-directory = Log-Verzeichnis
admin-settings-help-logging-file-directory = Verzeichnis für die Logdateien, absolut oder relativ zum Arbeitsverzeichnis des Servers. Segmente des übergeordneten Verzeichnisses (..) werden abgelehnt.
admin-settings-key-logging-file-max-size-mb = Rotation bei Größe (MB)
admin-settings-help-logging-file-max-size-mb = Die aktuelle Logdatei wird geschlossen und eine neue begonnen, wenn sie diese Größe erreicht.
admin-settings-key-logging-file-rotate-hours = Rotation bei Alter (Stunden)
admin-settings-help-logging-file-rotate-hours = Die aktuelle Logdatei wird rotiert, sobald sie so alt ist. 0 rotiert nur nach Größe.
admin-settings-key-logging-file-max-age-days = Rotierte Dateien aufbewahren (Tage)
admin-settings-help-logging-file-max-age-days = Rotierte Logdateien, die älter sind, werden gelöscht.
admin-settings-key-logging-file-max-files = Höchstzahl rotierter Dateien
admin-settings-help-logging-file-max-files = Die ältesten rotierten Dateien über dieser Anzahl werden gelöscht.
admin-settings-key-logging-file-max-total-mb = Höchste Gesamtgröße (MB)
admin-settings-help-logging-file-max-total-mb = Die ältesten rotierten Dateien werden gelöscht, bis das Log-Verzeichnis unter dieser Größe liegt.
admin-settings-key-logging-file-compress = Rotierte Dateien komprimieren
admin-settings-help-logging-file-compress = Rotierte Logdateien werden mit gzip komprimiert.
admin-settings-key-logging-level = Log-Level
admin-settings-help-logging-level = Das Level für Komponenten, die unten kein eigenes Level haben. Die Umgebungsvariable MW_LOG legt das Level beim Start fest, bis diese Einstellung gelesen wird.
admin-settings-key-logging-component-levels = Log-Level pro Komponente
admin-settings-help-logging-component-levels = Ein Eintrag Komponente=Level pro Zeile, zum Beispiel mw_imap=debug. Die Level sind error, warn, info, debug und trace.
admin-settings-key-logging-redaction-addresses = E-Mail-Adressen in Logs
admin-settings-help-logging-redaction-addresses = Full belässt Adressen wie protokolliert. Truncated behält den ersten Buchstaben und die Domain. Hashed ersetzt den lokalen Teil durch einen kurzen Hash. None entfernt die Adresse.
admin-settings-key-logging-redaction-ips = IP-Adressen in Logs
admin-settings-help-logging-redaction-ips = Full belässt Adressen wie protokolliert. Truncated behält das Präfix IPv4 /24 oder IPv6 /48. Hashed ersetzt die Adresse durch einen kurzen Hash. None entfernt sie.
admin-settings-key-logging-redaction-tokens = Geheimnisse in Logs
admin-settings-help-logging-redaction-tokens = Schneidet Bearer-Tokens, API-Schlüssel, Passwörter in URLs und ähnliche Geheimnisse aus Logzeilen heraus.
admin-settings-key-retention-sessions-idle-days = Ungenutzte Sitzungen (Tage)
admin-settings-help-retention-sessions-idle-days = Tage, die ein Sitzungseintrag eines Browsers oder nativen Clients samt Adresse und User-Agent ungenutzt bleiben darf, bevor er gelöscht wird. Die Sitzungs-Timeouts greifen zuerst; dies entfernt nur zurückgebliebene Zeilen. 0 behält sie.
admin-settings-key-policy-public-pages = Öffentliche Richtlinienseiten
admin-settings-help-policy-public-pages = Stellt die erzeugte Datenschutzerklärung unter /privacy und die Cookie-Richtlinie unter /cookies-policy ohne Anmeldung bereit.
admin-settings-key-policy-auto-version = Automatische Richtlinienversionen
admin-settings-help-policy-auto-version = Legt eine neue Richtlinienversion an, wenn sich eine Einstellung ändert, die die Richtlinie beschreibt. Aus: Änderungen warten auf Veröffentlichen.
admin-settings-key-policy-reconsent-on-change = Bei neuer Richtlinienversion erneut um Cookie-Einwilligung bitten
admin-settings-help-policy-reconsent-on-change = Erhöht die Version der Cookie-Richtlinie, wenn eine neue Richtlinienversion angelegt wird, sodass Besucher erneut gefragt werden (erfordert cookies.repromptOnVersion).
admin-settings-key-policy-controller-name = Name des Verantwortlichen
admin-settings-help-policy-controller-name = Name der Organisation, die über Zwecke und Mittel der Datenverarbeitung entscheidet, in der Datenschutzerklärung angezeigt.
admin-settings-key-policy-controller-address = Adresse des Verantwortlichen
admin-settings-help-policy-controller-address = Postanschrift des Verantwortlichen, in der Datenschutzerklärung angezeigt.
admin-settings-key-policy-legal-bases = Rechtsgrundlagen
admin-settings-help-policy-legal-bases = Die Rechtsgrundlagen, auf die Sie sich stützen, in Ihren eigenen Worten, in der Datenschutzerklärung angezeigt.
admin-settings-key-policy-hosting-provider = Hosting-Anbieter
admin-settings-help-policy-hosting-provider = Wer die Daten hostet und wo, in der Datenschutzerklärung angezeigt.
admin-settings-key-policy-hosting-transfers = Übermittlungen außerhalb des Hosting-Landes
admin-settings-help-policy-hosting-transfers = Alle Übermittlungen von Daten in andere Länder und die verwendeten Garantien, in der Datenschutzerklärung angezeigt.
admin-settings-key-policy-supervisory-authority = Aufsichtsbehörde
admin-settings-help-policy-supervisory-authority = Die Behörde, bei der sich Benutzer beschweren können, in der Datenschutzerklärung angezeigt.
admin-settings-key-dpia-suggest = Folgenabschätzungen vorschlagen
admin-settings-help-dpia-suggest = Listet eine vorgeschlagene Folgenabschätzung auf, wenn der Assistent, die Öffnungsverfolgung oder die Analyse aktiviert ist und keine existiert.
admin-settings-key-dpia-review-months = Überprüfungsintervall der Folgenabschätzung
admin-settings-help-dpia-review-months = Monate bis zum Überprüfungsdatum, das bei der Genehmigung einer Abschätzung gesetzt wird.
admin-settings-key-dpia-reminder-days = Erinnerungsfenster der Folgenabschätzung
admin-settings-help-dpia-reminder-days = Tage vor dem Überprüfungsdatum, ab denen eine Abschätzung als fällig aufgeführt wird.
admin-settings-key-restriction-enabled = Einschränkungen der Verarbeitung
admin-settings-help-restriction-enabled = Ob ein Administrator oder eine genehmigte Anfrage die Verarbeitung der Daten einer Person einschränken kann.
admin-settings-key-restriction-auto-apply-on-approval = Bei Genehmigung einer Anfrage einschränken
admin-settings-help-restriction-auto-apply-on-approval = Wendet die Einschränkung an, sobald eine Anfrage zur Einschränkung genehmigt wird. Aus: Ein Administrator wendet sie unter Compliance > Einschränkungen an.
admin-settings-key-restriction-mail-delivery = Neue E-Mails bei Einschränkung
admin-settings-help-restriction-mail-delivery = Was der Administrator für E-Mails gewählt hat, die eintreffen, während eine Einschränkung gilt: behalten, einreihen oder ablehnen. Dieser Server empfängt keine E-Mails; der Wert wird erfasst und angezeigt, und der Mailserver handelt danach.
admin-settings-key-restriction-default-expiry-days = Standarddauer der Einschränkung (Tage)
admin-settings-help-restriction-default-expiry-days = Tage, bis eine neue Einschränkung von selbst endet. 0 behält sie, bis jemand sie aufhebt.
admin-settings-key-restriction-notify-user = Die Person benachrichtigen
admin-settings-help-restriction-notify-user = Sendet der Person eine Mitteilung, wenn eine Einschränkung angewendet oder aufgehoben wird.
admin-settings-key-restriction-scope-indexing = Suchindizierung und Embeddings pausieren
admin-settings-help-restriction-scope-indexing = Ob Suchindizierung und Embeddings in einer neuen Einschränkung standardmäßig pausiert werden.
admin-settings-key-restriction-scope-assistant = Assistenten pausieren
admin-settings-help-restriction-scope-assistant = Ob der Assistent in einer neuen Einschränkung standardmäßig pausiert wird.
admin-settings-key-restriction-scope-analytics = Analyse und Öffnungsverfolgung pausieren
admin-settings-help-restriction-scope-analytics = Ob Analyse und Öffnungsverfolgung in einer neuen Einschränkung standardmäßig pausiert werden.
admin-settings-key-restriction-scope-classification = Regeln und automatisches Einsortieren pausieren
admin-settings-help-restriction-scope-classification = Ob Regeln und automatisches Einsortieren in einer neuen Einschränkung standardmäßig pausiert werden.
admin-settings-key-restriction-scope-push = Push-Benachrichtigungen pausieren
admin-settings-help-restriction-scope-push = Ob Push-Benachrichtigungen in einer neuen Einschränkung standardmäßig pausiert werden.
admin-settings-key-restriction-scope-third-party = Verbundene Dienste und Plugins pausieren
admin-settings-help-restriction-scope-third-party = Ob verbundene Dienste und Plugins in einer neuen Einschränkung standardmäßig pausiert werden.
admin-settings-key-consent-enabled = Einwilligung nach Zweck
admin-settings-help-consent-enabled = Zeigt Benutzern Zwecke und Einwilligungsoptionen und setzt sie zur Laufzeit durch.
admin-settings-key-consent-receipts-retention-days = Aufbewahrte Einwilligungsbelege (Tage)
admin-settings-help-consent-receipts-retention-days = Tage, die Einwilligungsbelege aufbewahrt werden. 0 behält sie, bis die Person gelöscht wird.

# -- Machine translation (2026-11): added to match en ---------------------
admin-settings-key-consent-art22-notice = Hinweis zur automatisierten Verarbeitung
admin-settings-help-consent-art22-notice = Zeigt Benutzern den Hinweis in verständlicher Sprache zu Assistenten- und Suchfunktionen, mit den Schaltern pro Funktion.
admin-settings-key-consent-analytics-basis = Rechtsgrundlage: Nutzungszähler
admin-settings-help-consent-analytics-basis = Die Rechtsgrundlage, die Benutzern für Nutzungszähler angezeigt wird. Nur die Einwilligung lässt die Funktion auf ein Ja warten; die anderen Grundlagen gelten, solange die Person nicht widerruft. Der Administrator entscheidet, welche Grundlage zutrifft.
admin-settings-key-consent-analytics-version = Textversion: Nutzungszähler
admin-settings-help-consent-analytics-version = Erhöhen Sie diese Zahl, wenn sich der Wortlaut oder die Verarbeitung ändert; Personen, die einer älteren Version zugestimmt haben, werden erneut gefragt.
admin-settings-key-consent-analytics-text = Eigener Wortlaut: Nutzungszähler
admin-settings-help-consent-analytics-text = Ihr eigener Text für diesen Zweck, pro Sprache. Leer verwendet den eingebauten Wortlaut.
admin-settings-key-consent-analytics-tag-basis = Rechtsgrundlage: Skript des Analyseanbieters
admin-settings-help-consent-analytics-tag-basis = Die Rechtsgrundlage, die Benutzern für das Skript des Analyseanbieters angezeigt wird. Nur die Einwilligung lässt die Funktion auf ein Ja warten; die anderen Grundlagen gelten, solange die Person nicht widerruft. Der Administrator entscheidet, welche Grundlage zutrifft.
admin-settings-key-consent-analytics-tag-version = Textversion: Skript des Analyseanbieters
admin-settings-help-consent-analytics-tag-version = Erhöhen Sie diese Zahl, wenn sich der Wortlaut oder die Verarbeitung ändert; Personen, die einer älteren Version zugestimmt haben, werden erneut gefragt.
admin-settings-key-consent-analytics-tag-text = Eigener Wortlaut: Skript des Analyseanbieters
admin-settings-help-consent-analytics-tag-text = Ihr eigener Text für diesen Zweck, pro Sprache. Leer verwendet den eingebauten Wortlaut.
admin-settings-key-consent-assistant-basis = Rechtsgrundlage: Assistent
admin-settings-help-consent-assistant-basis = Die Rechtsgrundlage, die Benutzern für den Assistenten angezeigt wird. Nur die Einwilligung lässt die Funktion auf ein Ja warten; die anderen Grundlagen gelten, solange die Person nicht widerruft. Der Administrator entscheidet, welche Grundlage zutrifft.
admin-settings-key-consent-assistant-version = Textversion: Assistent
admin-settings-help-consent-assistant-version = Erhöhen Sie diese Zahl, wenn sich der Wortlaut oder die Verarbeitung ändert; Personen, die einer älteren Version zugestimmt haben, werden erneut gefragt.
admin-settings-key-consent-assistant-text = Eigener Wortlaut: Assistent
admin-settings-help-consent-assistant-text = Ihr eigener Text für diesen Zweck, pro Sprache. Leer verwendet den eingebauten Wortlaut.
admin-settings-key-consent-push-basis = Rechtsgrundlage: Push-Benachrichtigungen
admin-settings-help-consent-push-basis = Die Rechtsgrundlage, die Benutzern für Push-Benachrichtigungen angezeigt wird. Nur die Einwilligung lässt die Funktion auf ein Ja warten; die anderen Grundlagen gelten, solange die Person nicht widerruft. Der Administrator entscheidet, welche Grundlage zutrifft.
admin-settings-key-consent-push-version = Textversion: Push-Benachrichtigungen
admin-settings-help-consent-push-version = Erhöhen Sie diese Zahl, wenn sich der Wortlaut oder die Verarbeitung ändert; Personen, die einer älteren Version zugestimmt haben, werden erneut gefragt.
admin-settings-key-consent-push-text = Eigener Wortlaut: Push-Benachrichtigungen
admin-settings-help-consent-push-text = Ihr eigener Text für diesen Zweck, pro Sprache. Leer verwendet den eingebauten Wortlaut.
admin-settings-key-consent-open-tracking-basis = Rechtsgrundlage: Öffnungsverfolgung
admin-settings-help-consent-open-tracking-basis = Die Rechtsgrundlage, die Benutzern für die Öffnungsverfolgung angezeigt wird. Nur die Einwilligung lässt die Funktion auf ein Ja warten; die anderen Grundlagen gelten, solange die Person nicht widerruft. Der Administrator entscheidet, welche Grundlage zutrifft.
admin-settings-key-consent-open-tracking-version = Textversion: Öffnungsverfolgung
admin-settings-help-consent-open-tracking-version = Erhöhen Sie diese Zahl, wenn sich der Wortlaut oder die Verarbeitung ändert; Personen, die einer älteren Version zugestimmt haben, werden erneut gefragt.
admin-settings-key-consent-open-tracking-text = Eigener Wortlaut: Öffnungsverfolgung
admin-settings-help-consent-open-tracking-text = Ihr eigener Text für diesen Zweck, pro Sprache. Leer verwendet den eingebauten Wortlaut.
admin-settings-key-consent-remote-content-basis = Rechtsgrundlage: externe Bilder
admin-settings-help-consent-remote-content-basis = Die Rechtsgrundlage, die Benutzern für externe Bilder angezeigt wird. Nur die Einwilligung lässt die Funktion auf ein Ja warten; die anderen Grundlagen gelten, solange die Person nicht widerruft. Der Administrator entscheidet, welche Grundlage zutrifft.
admin-settings-key-consent-remote-content-version = Textversion: externe Bilder
admin-settings-help-consent-remote-content-version = Erhöhen Sie diese Zahl, wenn sich der Wortlaut oder die Verarbeitung ändert; Personen, die einer älteren Version zugestimmt haben, werden erneut gefragt.
admin-settings-key-consent-remote-content-text = Eigener Wortlaut: externe Bilder
admin-settings-help-consent-remote-content-text = Ihr eigener Text für diesen Zweck, pro Sprache. Leer verwendet den eingebauten Wortlaut.
admin-settings-key-consent-integrations-basis = Rechtsgrundlage: verbundene Dienste
admin-settings-help-consent-integrations-basis = Die Rechtsgrundlage, die Benutzern für verbundene Dienste angezeigt wird. Nur die Einwilligung lässt die Funktion auf ein Ja warten; die anderen Grundlagen gelten, solange die Person nicht widerruft. Der Administrator entscheidet, welche Grundlage zutrifft.
admin-settings-key-consent-integrations-version = Textversion: verbundene Dienste
admin-settings-help-consent-integrations-version = Erhöhen Sie diese Zahl, wenn sich der Wortlaut oder die Verarbeitung ändert; Personen, die einer älteren Version zugestimmt haben, werden erneut gefragt.
admin-settings-key-consent-integrations-text = Eigener Wortlaut: verbundene Dienste
admin-settings-help-consent-integrations-text = Ihr eigener Text für diesen Zweck, pro Sprache. Leer verwendet den eingebauten Wortlaut.
admin-settings-key-consent-telemetry-basis = Rechtsgrundlage: Ladezeitmessungen
admin-settings-help-consent-telemetry-basis = Die Rechtsgrundlage, die Benutzern für Ladezeitmessungen angezeigt wird. Nur die Einwilligung lässt die Funktion auf ein Ja warten; die anderen Grundlagen gelten, solange die Person nicht widerruft. Der Administrator entscheidet, welche Grundlage zutrifft.
admin-settings-key-consent-telemetry-version = Textversion: Ladezeitmessungen
admin-settings-help-consent-telemetry-version = Erhöhen Sie diese Zahl, wenn sich der Wortlaut oder die Verarbeitung ändert; Personen, die einer älteren Version zugestimmt haben, werden erneut gefragt.
admin-settings-key-consent-telemetry-text = Eigener Wortlaut: Ladezeitmessungen
admin-settings-help-consent-telemetry-text = Ihr eigener Text für diesen Zweck, pro Sprache. Leer verwendet den eingebauten Wortlaut.
admin-settings-key-breach-enabled = Verzeichnis der Datenschutzverletzungen
admin-settings-help-breach-enabled = Schaltet das Verzeichnis und die Meldefrist-Uhr ein.
admin-settings-key-breach-notice-hours = Stunden bis zur Meldung an die Behörde
admin-settings-help-breach-notice-hours = Stunden vom Bekanntwerden bis zur Meldung an die Behörde. Die übliche Frist beträgt 72.
admin-settings-key-breach-reminders-hours = Erinnerungszeitpunkte
admin-settings-help-breach-reminders-hours = Stunden vor der Frist, die eine Erinnerung auslösen, zum Beispiel 6, 24 und 48.
admin-settings-key-breach-reminders-overdue-repeat-hours = Wiederholung bei Überfälligkeit
admin-settings-help-breach-reminders-overdue-repeat-hours = Stunden zwischen Erinnerungen, nachdem die Frist verstrichen ist. 0 erinnert einmal.
admin-settings-key-breach-authority-name = Aufsichtsbehörde
admin-settings-help-breach-authority-name = Name, der auf der Meldung an die Behörde gedruckt wird.
admin-settings-key-breach-authority-url = Meldeseite der Behörde
admin-settings-help-breach-authority-url = Adresse der Meldeseite der Behörde.
admin-settings-key-breach-authority-contact = Kontakt der Behörde
admin-settings-help-breach-authority-contact = Kontaktdaten der Behörde.
admin-settings-key-breach-dpo-name = Name des Datenschutzkontakts
admin-settings-help-breach-dpo-name = Name, der in Mitteilungen als Anlaufstelle angegeben wird.
admin-settings-key-breach-dpo-email = E-Mail des Datenschutzkontakts
admin-settings-help-breach-dpo-email = Adresse, die in Mitteilungen als Anlaufstelle angegeben wird.
admin-settings-key-breach-dpo-phone = Telefon des Datenschutzkontakts
admin-settings-help-breach-dpo-phone = Telefonnummer, die in Mitteilungen angegeben wird.
admin-settings-key-breach-template-subject = Betreff der Nachricht
admin-settings-help-breach-template-subject = Betreff der Nachricht an die betroffenen Personen, pro Sprache.
admin-settings-key-breach-template-body = Text der Nachricht
admin-settings-help-breach-template-body = Text der Nachricht an die betroffenen Personen, pro Sprache.
admin-settings-key-breach-notify-audit = Mitteilungen im Audit-Protokoll erfassen
admin-settings-help-breach-notify-audit = Erinnerungen und Erkennungen werden zu Sicherheitsereignissen.
admin-settings-key-breach-notify-webhook-urls = Webhook-Adressen
admin-settings-help-breach-notify-webhook-urls = Adressen, die über Erinnerungen und Nachrichten informiert werden. Behandeln Sie sie als Geheimnisse; jeder Aufruf ist signiert.
admin-settings-key-breach-notify-operator-addresses = Betreiber-Postfächer
admin-settings-help-breach-notify-operator-addresses = Lokale Postfachadressen, die Erinnerungen im eigenen Posteingang erhalten.
admin-settings-key-breach-subjects-in-app = Hinweis im Konto der Person
admin-settings-help-breach-subjects-in-app = Zeigt die Nachricht im Konto der Person an.
admin-settings-key-breach-subjects-inbox = Nachricht an das Postfach der Person
admin-settings-help-breach-subjects-inbox = Stellt die Nachricht dem eigenen Postfach der Person zu.
admin-settings-key-breach-subjects-webhook = Zusätzlich an Webhooks senden
admin-settings-help-breach-subjects-webhook = Sendet den Nachrichtentext auch an die Webhook-Adressen.
admin-settings-key-breach-retention-years = Aufbewahrung des Verzeichnisses in Jahren
admin-settings-help-breach-retention-years = Jahre, die ein abgeschlossener Vorfall im Verzeichnis bleibt.
admin-settings-key-breach-auto-enabled = Automatisierungen
admin-settings-help-breach-auto-enabled = Eröffnet Entwurfsvorfälle aus Erkennungen.
admin-settings-key-breach-auto-start-clock = Uhr für Entwürfe
admin-settings-help-breach-auto-start-clock = Startet die Uhr, wenn der Entwurf eröffnet wird, oder erst bei seiner Bestätigung.
admin-settings-key-breach-auto-assignee = Entwürfe zuweisen an
admin-settings-help-breach-auto-assignee = Wem ein Entwurf zugewiesen wird.
admin-settings-key-breach-auto-escalate-after-hours = Nach Stunden eskalieren
admin-settings-help-breach-auto-escalate-after-hours = Stunden, die ein Entwurf wartet, bevor er eskaliert wird. 0 eskaliert nie.
admin-settings-key-breach-auto-escalate-to = Eskalieren an
admin-settings-help-breach-auto-escalate-to = Wem ein eskalierter Entwurf zugewiesen wird.
admin-settings-key-breach-auto-window-minutes = Zählfenster in Minuten
admin-settings-help-breach-auto-window-minutes = Minuten, über die eine Regel Ereignisse zählt.
admin-settings-key-breach-auto-theft-threshold = Warnungen zu Sitzungsdiebstahl
admin-settings-help-breach-auto-theft-threshold = Warnungen im Fenster, die einen Entwurf eröffnen. 0 ist aus.
admin-settings-key-breach-auto-credential-stuffing-threshold = Anmelde-Spraying oder -Spitze
admin-settings-help-breach-auto-credential-stuffing-threshold = Ereignisse im Fenster, die einen Entwurf eröffnen. 0 ist aus.
admin-settings-key-breach-auto-mass-forward-threshold = Weiterleitungsregeln in großer Zahl
admin-settings-help-breach-auto-mass-forward-threshold = Ereignisse im Fenster, die einen Entwurf eröffnen. 0 ist aus.
admin-settings-key-breach-auto-role-escalation-threshold = Änderungen der Administratorrolle
admin-settings-help-breach-auto-role-escalation-threshold = Änderungen im Fenster, die einen Entwurf eröffnen. 0 ist aus.
admin-settings-key-breach-auto-export-volume-threshold = Ungewöhnliche Exporte
admin-settings-help-breach-auto-export-volume-threshold = Exporte im Fenster, die einen Entwurf eröffnen. 0 ist aus.
admin-settings-key-breach-auto-audit-chain-threshold = Fehler der Audit-Kette
admin-settings-help-breach-auto-audit-chain-threshold = Fehler, die einen Entwurf eröffnen. 0 ist aus.
admin-settings-key-breach-auto-backup-anomaly-threshold = Fehler bei Backup oder Wiederherstellung
admin-settings-help-breach-auto-backup-anomaly-threshold = Fehler im Fenster, die einen Entwurf eröffnen. 0 ist aus.
admin-settings-key-breach-auto-dsr-export-threshold = Viele Exporte von Datenanfragen
admin-settings-help-breach-auto-dsr-export-threshold = Exporte im Fenster, die einen Entwurf eröffnen. 0 ist aus.
admin-settings-key-ratelimit-queue-ms = Längste Wartezeit auf einen freien Platz
admin-settings-help-ratelimit-queue-ms = Längste Zeit, die eine Anfrage auf einen freien Platz wartet (JMAP-Parallelität, Bild-Decoder, Vorschau-Worker), bevor sie mit 429 und Retry-After abgelehnt wird. Bereich 0-30000 ms; Standard 3000. Null lehnt sofort ab.
admin-settings-key-ratelimit-jmap-per-account-concurrent = Gleichzeitige JMAP-Anfragen pro Konto
admin-settings-help-ratelimit-jmap-per-account-concurrent = JMAP-Anfragen, die ein angemeldetes Konto über seine Tabs und Geräte hinweg gleichzeitig ausführen darf. Sie werden nie mit einem anderen Konto oder einem anderen Benutzer hinter derselben Adresse geteilt. Bereich 4-64; Standard 8.
admin-settings-key-ratelimit-jmap-global-concurrent = Gleichzeitige JMAP-Anfragen, alle Konten
admin-settings-help-ratelimit-jmap-global-concurrent = JMAP-Anfragen, die der gesamte Serverprozess gleichzeitig ausführt, alle Konten zusammen. Eine Anfrage darüber wartet die Warteschlangenzeit. Bereich 4-4096; Standard 256.
admin-settings-key-ratelimit-session-burst = Zeitmessungsberichte pro Sitzung in Folge
admin-settings-help-ratelimit-session-burst = Berichte zur Client-Zeitmessung, die eine Sitzung in Folge senden darf. Der Endpunkt wird nach Sitzung geschlüsselt, nie nach Adresse. Bereich 1-1000; Standard 60.
admin-settings-key-ratelimit-session-per-minute = Pro Minute zurückgewonnene Zeitmessungsberichte
admin-settings-help-ratelimit-session-per-minute = Berichte zur Client-Zeitmessung pro Minute, die eine Sitzung nach ihrem Burst zurückgewinnt. Bereich 1-600; Standard 30.
admin-settings-key-ratelimit-anon-burst = Anonyme Einwilligungsanfragen in Folge
admin-settings-help-ratelimit-anon-burst = Anfragen, die eine Adresse in Folge an die anonymen Routen der Cookie-Einwilligung richten darf. Bemessen für ein Büro hinter einer Adresse. Bereich 1-1000; Standard 60.
admin-settings-key-ratelimit-anon-per-minute = Pro Minute zurückgewonnene anonyme Einwilligungsanfragen
admin-settings-help-ratelimit-anon-per-minute = Anfragen pro Minute, die eine Adresse auf den anonymen Routen der Cookie-Einwilligung zurückgewinnt. Bereich 1-600; Standard 30.
admin-settings-key-ratelimit-trusted-multiplier = Kontingent des Kontos bei bekannter Quelle (%)
admin-settings-help-ratelimit-trusted-multiplier = Prozentsatz des Anfragekontingents des Kontos, den ein angemeldetes Postfach erhält, wenn die Anfrage von einer Quelle mit erfasster erfolgreicher Anmeldung stammt. 100 behält das Basiskontingent. Lockert nie die Budgets für anonyme Anfragen, Administratoren oder User-Agent und nie eine aktive Sperre. Bereich 100-1000; Standard 150.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-attachments-bar-click-to-toggle = Gibt an, ob durch Klicken auf eine beliebige Stelle in der Anhangsleiste des Readers diese erweitert oder reduziert wird. Im ausgeschalteten Zustand funktioniert nur die Pfeiltaste.

admin-settings-help-attachments-bar-download-all = Ob die Anhängeleiste eine Schaltfläche „Alle herunterladen“ bietet, mit der die zulässigen Anhänge als eine ZIP-Datei gespeichert werden.

admin-settings-help-attachments-bar-show-download-buttons = Wenn jeder Anhang in der Leiste seine Download-Schaltfläche anzeigt: beim Bewegen oder Fokussieren (Standard), immer oder nie. Für Downloads gelten weiterhin die Richtlinien für Anhänge und mail.allowDownload.

admin-settings-help-attachments-bar-zip-inline-images = Ob die im Nachrichtentext angezeigten Bilder im Download all ZIP enthalten sind. Standardmäßig deaktiviert.

admin-settings-help-cap-reading-delete-sender = Ermöglichen Sie Benutzern, die Leseaktion zu aktivieren, die alle passenden Absendernachrichten nach Bestätigung in den Papierkorb verschiebt. Standardmäßig deaktiviert. Das Funktionsprofil muss dies ebenfalls zulassen.

admin-settings-help-compose-feature-family = Schalten Sie ein Composer-Steuerelement anhand der Funktions-ID ein oder aus. Standardmäßig aktiviert; Sperren Sie es, um zu verhindern, dass Benutzer es ändern.

admin-settings-help-compose-headers-allow-auto-submitted = Ermöglicht Benutzern die Einstellung „Automatisch übermittelt“ (RFC 3834) für eine Nachricht. Standardmäßig deaktiviert: Es weist andere Systeme an, nicht zu antworten.

admin-settings-help-compose-headers-allow-custom = Ermöglicht Benutzern das Hinzufügen eigener Header (bis zu 20, Namen überprüft, geschützte Header abgelehnt).

admin-settings-help-compose-headers-allow-return-receipt-to = Ermöglicht Benutzern das Anfordern von Belegen mit dem alten Return-Receipt-To-Header. Standardmäßig deaktiviert.

admin-settings-help-compose-headers-allow-threading-edit = Ermöglicht Benutzern das Bearbeiten von In-Reply-To und Referenzen oder das Starten eines neuen Threads aus einer Antwort.

admin-settings-help-compose-headers-forced = Ein Name=Wert-Paar pro Zeile, das jeder von der Domäne gesendeten Nachricht hinzugefügt wird und den Wert des Benutzers ersetzt. Geschützte Header werden abgelehnt.

admin-settings-help-compose-insert-image-resize-default = Die Größe, mit der das Dialogfeld „Bild aus Datei“ beginnt: Original, groß (1600 Pixel) oder mittel (1024 Pixel).

admin-settings-help-compose-insert-max-inline-image-mb = Bilder, die größer sind als dieser Wert, werden nicht im Nachrichtentext platziert; Sie können weiterhin angebracht werden. 1 bis 50.

admin-settings-help-compose-layout-default-font = Schriftfamilie, -größe und -farbe, mit der eine neue Nachricht beginnt, als Familie|Größe|#Farbe. Empty verwendet die Designschriftart.

admin-settings-help-compose-reply-to-allow-override = Wenn diese Option deaktiviert ist, wird die Antwortadresse der Identität unverändert verwendet und Benutzer können sie nicht bearbeiten oder entfernen.

admin-settings-help-compose-reply-to-show = Ob der Verfasser die Zeile „Antwort an“ anzeigt: nur, wenn eine Identität oder die ursprüngliche Nachricht eine hat, immer oder nie.

admin-settings-help-compose-sent-folder-fixed-id = Ordnerrolle oder -ID, die verwendet wird, wenn die Regel „fester Ordner“ ist.

admin-settings-help-compose-sent-folder-mode = Wohin die Kopie einer gesendeten Nachricht geht: der normale Ordner „Gesendet“, ein fester Ordner, nach Empfängerdomäne, nach Identität oder nach Betreffregeln.

admin-settings-help-compose-sent-folder-per-identity = Ein Identität=Ordner-Paar pro Zeile. Wird verwendet, wenn die Regel „nach Identität“ lautet.

admin-settings-help-compose-sent-folder-rules = Eine Regel pro Zeile, domain:example.com=Ordner oder subject:text=Ordner. Das erste Spiel gewinnt.

admin-settings-help-compose-sent-folder-skip-copy = Bewahren Sie niemals eine Kopie einer gesendeten Nachricht auf. Ein Administrator kann dies aktivieren oder deaktivieren.

admin-settings-help-compose-styles-custom = Die eigenen Composer-Stile eines Benutzers als JSON, geschrieben von der Stilgalerie. Benutzer legen es fest; Hier gibt es nichts zu konfigurieren.

admin-settings-help-compose-styles-default-set = Das Aussehen der Styles-Galerie des Komponisten: Standard, modern, klassisch oder kompakt.

admin-settings-help-compose-styles-max-custom = Die meisten benutzerdefinierten Stile kann ein Benutzer im Composer behalten. 0 schaltet „Stil erstellen“ aus.

admin-settings-help-contacts-save-from-message-add-note = Fügen Sie den aus einer Nachricht gespeicherten Kontakten die Notiz „Gespeichert aus einer Nachricht am <Datum>“ hinzu. Benutzer- und Domänenüberschreibungen können gesperrt werden.

admin-settings-help-contacts-save-from-message-default-book = Die Adressbuch-ID, die aus einer Nachricht gespeicherte Kontakte empfängt; „default“ verwendet das Standardadressbuch. Benutzer- und Domänenüberschreibungen können gesperrt werden.

admin-settings-help-contacts-save-from-message-default-scope = Welche Teilnehmer im Dialog „Kontakte speichern“ zuerst angekreuzt werden: nur der Absender, der Absender und Antwortempfänger oder alle. Benutzer- und Domänenüberschreibungen können gesperrt werden.

admin-settings-help-contacts-save-from-message-enabled = Angebot Speichern Sie Kontakte aus dieser Nachricht in Nachrichtenmenüs. Wenn diese Option deaktiviert ist, ist das Element ausgeblendet. Benutzer- und Domänenüberschreibungen können gesperrt werden.

admin-settings-help-contacts-save-from-message-skip-automated = Lassen Sie Nicht-Antwort-, Benachrichtigungs- und Bounce-Adressen aus dem Dialogfeld „Kontakte speichern“ weg, es sei denn, Sie aktivieren das Häkchen manuell. Benutzer- und Domänenüberschreibungen können gesperrt werden.

admin-settings-help-contacts-save-sender-enabled = Bieten Sie in Nachrichtenmenüs die Ein-Klick-Option „Absender als Kontakt speichern“ an. Wenn diese Option deaktiviert ist, ist das Element ausgeblendet. Benutzer- und Domänenüberschreibungen können gesperrt werden.

admin-settings-help-contacts-save-sender-use-reply-to = Speichern Sie die Antwortadresse anstelle von „Von“, wenn es sich beim Absender um eine Mailingliste oder eine Benachrichtigungsadresse handelt. Benutzer- und Domänenüberschreibungen können gesperrt werden.

admin-settings-help-dsr-admin-step-up = Ein (Standard): Ein Administrator, der eine Datenanfrage genehmigt, eine Löschung durchführt oder eine Massengenehmigung erteilt, muss zunächst noch einmal bestätigen, dass er es ist (Passwort und zweiter Faktor oder ein Passkey). Die Bestätigung dauert fünf Minuten. Aus entfernt die zusätzliche Prüfung; Die Entscheidung wird weiterhin im Anforderungspfad aufgezeichnet.

admin-settings-help-export-default-format = Das Format, mit dem der Exportdialog beginnt.

admin-settings-help-export-default-scope = Der Bereich, mit dem der Exportdialog beginnt: Nachricht, Konversation, Auswahl oder Ordner.

admin-settings-help-export-dialog-enabled = Beim Exportieren in der Nachrichtenansicht und den Nachrichtenmenüs wird direkt ein Dialog geöffnet. Wenn diese Option deaktiviert ist, laden diese Menüs die eine Nachricht als .eml herunter.

admin-settings-help-export-folder-enabled = Erlauben Sie Benutzern, einen ganzen Ordner als ZIP Nachrichten herunterzuladen. Jeder Export wird geprüft.

admin-settings-help-export-folder-format = Die Formate, die ein Ordnerexport bietet: ein ZIP mit .eml-Dateien, ein ZIP mit einer Mbox-Datei oder beides.

admin-settings-help-export-folder-include-subfolders = Ob ein Ordnerexport standardmäßig seine Unterordner enthält, wird als Ordnerstruktur im ZIP beibehalten.

admin-settings-help-export-folder-max-bytes = Die meisten Bytes an Nachrichten, die ein Ordnerexport enthalten kann. Ein größerer Ordner wird mit der angezeigten Größe abgelehnt.

admin-settings-help-export-folder-max-messages = Die meisten Nachrichten, die ein Ordner exportieren kann.

admin-settings-help-export-formats = Die Formate, die der Exportdialog anbietet: eml, mbox, txt, md, msg, docx, csv, json, pdf. Der Server lehnt ein aufgelistetes, vom Server erstelltes Format ab.

admin-settings-help-export-message-download-enabled = Ermöglichen Sie Benutzern, eine Nachricht als .eml, nur deren Header oder als Mbox-Datei herunterzuladen.

admin-settings-help-folders-hide-empty = Leere Ordner in der Seitenleiste ausblenden. Benutzer können die Standardeinstellung ändern oder für einzelne Ordner überschreiben, sofern diese Einstellung nicht gesperrt ist. Es wird keine E-Mail gelöscht.

admin-settings-help-folders-pinned-hide-from-tree = Angeheftete Ordner werden in einer angehefteten Gruppe über dem Ordnerbaum angezeigt und behalten auch ihren Platz im Baum. Ein blendet einen angehefteten Ordner aus, der keine Unterordner in der Baumstruktur hat, sodass er nur in der Gruppe „Angeheftet“ angezeigt wird. Ein angehefteter Ordner mit Unterordnern bleibt in der Baumstruktur.

admin-settings-help-mail-badge-auth = Wenn im Header des Lesegeräts das Authentifizierungskennzeichen DMARC angezeigt wird: immer, nur wenn es auffällt oder nie.

admin-settings-help-mail-badge-density = Größe der Ausweise im Lesekopf: kompakt oder komfortabel.

admin-settings-help-mail-badge-folder = Wenn in der Kopfzeile des Lesers das Ordnerabzeichen angezeigt wird: immer, nur wenn es auffällt oder nie.

admin-settings-help-mail-badge-headers = Zeigen Sie im Reader ein Abzeichen für die Header „Abgelaufen“, „Antworten bis“, „Vertraulichkeit“ und „Ersetzt“ einer Nachricht an.

admin-settings-help-mail-badge-labels = Ob in der Kopfzeile des Lesers ein Symbol mit Text oder nur das Symbol angezeigt wird.

admin-settings-help-mail-badge-list = Wenn in der Kopfzeile des Lesers das Mailinglisten-Badge angezeigt wird: immer, nur wenn es auffällt oder nie. Es benötigt einen List-Id- oder List-Unsubscribe-Header.

admin-settings-help-mail-badge-sanitized = Wenn in der Kopfzeile des Lesegeräts das bereinigte HTML-Abzeichen angezeigt wird: immer, nur wenn es auffällt oder nie.

admin-settings-help-mail-badge-spam = Wenn im Header des Lesers das Spam-Score-Abzeichen angezeigt wird: immer, nur wenn es auffällt oder nie.

admin-settings-help-mail-brand-avatars-require-auth = niemals: Zeigt ein Logo an, wenn die Domain übereinstimmt. Known-Failure (Standard): Zeigt es an, es sei denn, das zwischengespeicherte DKIM- oder DMARC-Ergebnis der Nachricht ist ein Fehler. Ungeprüfte Nachrichten behalten das Logo. immer: erst nach einem abgeglichenen DMARC-Durchlauf mit DKIM für die markeneigene Domain anzeigen. Benutzer- und Domänenüberschreibungen können gesperrt werden.

admin-settings-help-mail-compose-header-auto-collapse-seconds = Sekunden muss der Text den Fokus halten, bevor ein vollständiger Empfänger- und Betreff-Header in die Zusammenfassung eingeblendet wird. Wählen Sie 0, um das zeitgesteuerte Falten zu deaktivieren, oder 1–60 Sekunden. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-mail-compose-priority-default = Die Priorität, mit der eine neue Nachricht im Composer beginnt.

admin-settings-help-mail-compose-priority-enabled = Zeigt die Prioritätsauswahl „Normal“, „Hoch“ und „Niedrig“ in der Gruppe „Composer-Optionen“ an. Wenn diese Option deaktiviert ist, werden neue E-Mails ohne Prioritätsheader gesendet.

admin-settings-help-mail-expiry-hide-expired = Was tun mit einer Nachricht, deren Ablaufdatum abgelaufen ist? Nichts tun, sie abgeblendet anzeigen oder vorschlagen, sie zu archivieren.

admin-settings-help-mail-list-keep = Absender und Mailinglisten werden immer gespeichert, eine pro Zeile als sender:address oder list:List-Id. Bereinigen Sie diesen Absender, Abmeldevorschläge und Aufbewahrungsregeln überspringen sie.

admin-settings-help-mail-list-page-size = Wie viele Nachrichten ruft die Liste jedes Mal ab, wenn ein Benutzer bis zum Ende scrollt? Standardwert 50, von 20 bis 200. Benutzer wählen ihren eigenen Wert, sofern Sie diesen nicht sperren.

admin-settings-help-mail-list-unsubscribe-mode = So funktioniert „Abbestellen“, wenn eine Liste eine Ein-Klick-Anfrage anbietet: Fragen Sie zuerst, senden Sie sie sofort ab oder öffnen Sie nur den Link oder Entwurf der Liste.

admin-settings-help-mail-message-list-group = Gruppieren Sie Konversationen nach Empfangs- oder Sendedatum, Absender, Betreff, Wichtigkeit oder Größe. Benutzer können dies außer Kraft setzen, sofern sie nicht gesperrt sind.

admin-settings-help-mail-priority-marks = Welche Priorität markiert die Nachrichtenliste, Konversationen und die Leseranzeige: sowohl hoch als auch niedrig, nur hoch oder keine. Benutzer wählen unter „Einstellungen“ > „Ordner und Seitenleiste“, sofern diese nicht gesperrt sind.

admin-settings-help-mail-priority-marks-style = So sieht eine Prioritätsmarkierung aus: ein Symbol, ein Symbol mit den Worten „Hoch“ oder „Niedrig“ oder ein dünner Balken. Symbol und Text behalten ihre Bedeutung ohne Farbe.

admin-settings-help-mail-priority-send-headers = Welche Header eine Nachricht mit hoher oder niedriger Priorität beim Senden erhält: Wichtigkeit, X-Priorität und X-MSMail-Priorität.

admin-settings-help-mail-priority-trust-sender-headers = Ob eine vom Absender festgelegte Priorität angezeigt wird: immer, nur wenn die Nachricht eine ausgerichtete DMARC-Prüfung bestanden hat, oder nie. Verwenden Sie „Vertrauenswürdig“ oder „Nie“ gegen Absender, die jeder Nachricht eine hohe Priorität zuweisen.

admin-settings-help-mail-reply-from-header-priority = Die Nachrichtenkopfzeilen werden in dieser Reihenfolge gelesen, um bei der Auswahl der Identität für eine Antwort die Adresse zu ermitteln, an die eine Nachricht gesendet wurde. Namen: Geliefert an, X-Original-An, Umschlag-An, X-Weitergeleitet an, Erneut gesendet an, An, Cc. Ein weggelassener Header wird nicht gelesen.

admin-settings-help-mail-search-default-scope = Starten Sie eine neue normale E-Mail-Suche im aktuellen Ordner oder in allen Ordnern. Dies steuert die anfängliche Standardeinstellung, auch wenn sie gesperrt ist; Benutzer können weiterhin einen anderen Bereich für eine bestimmte Suche auswählen. Gespeicherte und explizit bereichsbezogene Abfragen bleiben erhalten.

admin-settings-help-metrics-dimensions-enabled = Speichern Sie die gespeicherten Statistiken aufgeteilt nach Clienttyp (Web) und Geräteklasse (Desktop, Mobilgerät). Schalten Sie diese Option aus, um eine kombinierte Zeile pro Metrik und Minute zu speichern.

admin-settings-help-metrics-persist-enabled = Speichern Sie die Leistungsstatistiken (Bild-Proxy-Geschwindigkeit, Browser-Ladezeiten, Ratenbegrenzung und Anforderungsschutzanzahl) bei jedem Spülintervall und beim Stoppen des Servers in der Datenbank, damit die Diagramme ihren Verlauf nach einem Neustart behalten. Nichts identifiziert eine Person, Adresse oder Nachricht. Schalten Sie diese Option aus, um sie nur im Speicher zu behalten.

admin-settings-help-metrics-persist-flush-seconds = Wie oft werden die Leistungsstatistiken in die Datenbank geschrieben (5 bis 3600 Sekunden). Ein Server, der ohne Vorwarnung getötet wird, verliert höchstens so viel.

admin-settings-help-observability-client-capture-console = Melden Sie auch console.error-Aufrufe, höchstens fünf pro Minute und Seite. Standardmäßig deaktiviert, da Bibliotheken erwartete Bedingungen protokollieren.

admin-settings-help-observability-client-capture-csp = Melden Sie Verstöße gegen die Content-Security-Policy, die der Browser erkennt, nur mit der blockierten Direktive und dem Ursprung. Standardmäßig aktiviert.

admin-settings-help-observability-client-enabled = Sammeln Sie Fehlerberichte aus der Web-App (Fehlergrenzen, nicht erfasste Fehler, fehlgeschlagene Blöcke, Worker-Fehler, CSP-Verstöße) mit bereinigten Stack-Traces, angezeigt unter Observability > Client-Fehler. Standardmäßig aktiviert. Berichte enthalten keine Nachrichteninhalte, Adressen, Token, Anhangsnamen oder URL-Abfragen. Ausschalten, um die Sammlung zu stoppen; bestehende Berichte erlöschen mit deren Aufbewahrung.

admin-settings-help-observability-client-forward = Senden Sie außerdem jeden bereinigten Bericht an die mit MW_ERROR_FORWARD_URL konfigurierte Fehlersenke. Standardmäßig deaktiviert. Ohne konfigurierte Senke wird nichts gesendet.

admin-settings-help-observability-client-include-account-hash = Fügen Sie Berichten, die von einer angemeldeten Sitzung gesendet werden, einen verschlüsselten Einweg-Hash des Kontos hinzu, damit Berichte von einem Konto unterschieden werden können, ohne es zu benennen. Standardmäßig deaktiviert. Der Hash ändert sich, wenn der Datenschutzschlüssel rotiert.

admin-settings-help-observability-client-max-per-session = Die meisten Fehlerberichte können beim Laden einer Seite gesendet werden (1 bis 1000). Wiederholungen desselben Fehlers innerhalb von 60 Sekunden zählen einmal. Standard 20.

admin-settings-help-observability-client-sample-rate = Anteil der Fehlerberichte, die ein Browser sendet, von 0 bis 1. Standard 1 (alle). Das Laden jeder Seite stoppt immer noch beim Limit pro Sitzung.

admin-settings-help-observability-client-source-maps = Lösen Sie Bundle-Positionen in Stack-Traces zu Quelldateien auf, wenn der Build versteckte Quellzuordnungen lieferte (erstellt mit MW_SOURCEMAP=hidden). Karten werden nur vom Server gelesen und niemals an Browser bereitgestellt. Standardmäßig aktiviert; Ohne Karten werden die Rohbilder angezeigt.

admin-settings-help-privacy-error-reports = Ob ein Browser einen Fehlerbericht sendet, wenn ein Teil der App ausfällt. Standardmäßig aktiviert. Globaler, Domänen- und Benutzerbereich: Legen Sie hier die Standardeinstellung fest oder sperren Sie sie. Personen können es unter „Einstellungen“ > „Datenschutz“ selbst ausschalten, sofern es nicht gesperrt ist.

admin-settings-help-privacy-image-cache-max-stale-secs = Sekunden nach Ablauf seiner Aktualität wird ein zwischengespeichertes Bild aufbewahrt, um zu antworten, wenn der Ursprung nicht erreicht werden kann. Niemals unterhalb des Verlängerungsfensters. Bereich 0–604800; Standard 86400.

admin-settings-help-privacy-image-cache-stale-secs = Sekunden nach Ablauf seiner Aktualität wird ein zwischengespeichertes Bild immer noch sofort bereitgestellt, während es im Hintergrund erneut validiert wird. Zero schaltet es aus. Bereich 0–86400; Standard 3600.

admin-settings-help-privacy-image-resize-enabled = Skalieren Sie übergroße Remote-Bilder auf die vom Leser gewünschte Breite (320, 640, 960, 1280 oder 1600 Pixel). Animierte Bilder werden unskaliert bereitgestellt. Standardmäßig deaktiviert.

admin-settings-help-privacy-image-resize-format = Wählt automatisch den besten Encoder im Build aus, nämlich PNG. Standardmäßig automatisch.

admin-settings-help-privacy-image-resize-max-pixels = Bilder mit mehr dekodierten Pixeln werden unskaliert bereitgestellt; Die Größe wird vor der Dekodierung aus dem Header gelesen. Bereich 100000–64000000; Standard 4000000.

admin-settings-help-privacy-image-resize-max-width = Das breiteste Bild, das durch die Größenänderung erzeugt wird. Bereich 320–1600; Standard 1600.

admin-settings-help-privacy-sessions-show-linked-origin = Wie Einstellungen > Sicherheit > Sitzungen das Konto benennen, von dem eine Zugriffssitzung für verknüpfte Postfächer stammt: die vollständige Adresse, die Adresse mit maskiertem ersten Teil des Namens oder nichts. Der Wert wird nur dem Eigentümer der Sitzungen angezeigt, niemals anderen Benutzern. Standard: vollständige Adresse.

admin-settings-help-properties-folder-enabled = Bieten Sie Eigenschaften in den Ordnermenüs und in der Verknüpfung an. Wenn diese Option deaktiviert ist, ist der Dialog nicht verfügbar.

admin-settings-help-properties-folder-notes-enabled = Erlauben Sie für jeden Ordner eine Beschreibung oder Notiz.

admin-settings-help-properties-folder-notes-max-length = Die meisten Zeichen, die eine Ordnernotiz enthalten darf.

admin-settings-help-properties-folder-overrides-allowed = Welche Einstellungen ein Benutzer für einen einzelnen Ordner überschreiben kann. Eine nicht aufgeführte Überschreibung wird ausgeblendet und gespeicherte Werte werden ignoriert.

admin-settings-help-properties-folder-sections = Die Abschnitte des Ordnereigenschaftendialogs: Allgemein, Remote, Größe, Offline, Überschreibungen, Notizen, Aktionen. Ein nicht aufgeführter Abschnitt ist ausgeblendet.

admin-settings-help-properties-folder-size-compute = Wenn die Größe eines Ordners auf dem Server ermittelt wird: niemals, beim Öffnen des Dialogs oder im Hintergrund. Das Zählen eines großen Postfachs kostet den Server einen Scan.

admin-settings-help-properties-headers-raw = Erlauben Sie dem Dialogfeld „Nachrichteneigenschaften“, die Rohkopfzeilen aufzulisten. Wenn diese Option deaktiviert ist, wird nur die analysierte Zusammenfassung angezeigt.

admin-settings-help-properties-message-enabled = Bieten Sie Eigenschaften in den Nachrichtenmenüs und auf der Verknüpfung an. Wenn diese Option deaktiviert ist, ist der Dialog nicht verfügbar.

admin-settings-help-properties-message-permanent-delete = Ob der Nachrichtendialog eine Nachricht endgültig löschen darf: zulassen, zweimal fragen oder ablehnen. Eine Nachricht mit gesetzlicher Aufbewahrungspflicht wird niemals gelöscht. Das Löschen auf diese Weise kann nicht rückgängig gemacht werden.

admin-settings-help-properties-message-sections = Die Abschnitte des Dialogfelds „Nachrichteneigenschaften“: Zusammenfassung, Bewertung, Kopfzeilen, Anhänge, Aktionen. Ein nicht aufgeführter Abschnitt ist ausgeblendet.

admin-settings-help-properties-offline-clear-enabled = Ermöglichen Sie Benutzern, den Offline-Cache eines einzelnen Ordners über dessen Eigenschaftendialog zu löschen.

admin-settings-help-properties-remote-location-visibility = Wie viel vom serverseitigen Standort Benutzern angezeigt wird: vollständig (Host und Pfad), maskierter Host (Host teilweise ausgeblendet), nur Pfad (Postfachpfad ohne Host) oder ausgeblendet.

admin-settings-help-properties-scoring-show = Zeigen Sie Spam-Score, Authentifizierungsergebnisse und Sicherheitssignale im Dialogfeld „Nachrichteneigenschaften“ an.

admin-settings-help-properties-shortcut-enabled = Öffnen Sie das Dialogfeld „Eigenschaften“ über die Tastatur für einen fokussierten Ordner oder eine ausgewählte Nachricht.

admin-settings-help-properties-shortcut-key = Die Tastenkombination, die Eigenschaften öffnet; Standardmäßig Alt+Eingabetaste.

admin-settings-help-reader-actions-archive = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-block-sender = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-colors = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-conversation = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-create-filter = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-delete = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-delete-sender = Standardmäßig ausgeblendet. Wählen Sie die Symbolleiste oder das Menü „Mehr“, um es zu aktivieren, wenn die Funktion zum Löschen des Absenders und das Funktionsprofil dies zulassen. Benutzer können die Platzierung überschreiben, sofern sie nicht gesperrt ist.

admin-settings-help-reader-actions-edit-draft = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-export = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-finish-followup = Wählen Sie „Symbolleiste“, „Mehr“ oder „Ausblenden“, um markierte Nachrichten und Folgeerinnerungen abzuschließen.

admin-settings-help-reader-actions-flag = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-focus-focused = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-focus-other = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-forward = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-headers = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-links = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-move = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-not-spam = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-open-draft = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-pin = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-print = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-properties = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-read = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-reply = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-reply-all = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-save-contacts = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-save-sender = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-search = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-security = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-spam = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-sweep = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-view-source = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-reader-actions-zoom = Wählen Sie Symbolleiste, Menü „Mehr“ oder „Ausgeblendet“. Funktionseinschränkungen, Berechtigungen und Nachrichtenkontext gelten weiterhin. Benutzer können wählen, es sei denn, dies ist gesperrt.

admin-settings-help-retention-client-errors-event-days = Tage, an denen ein einzelner Browserfehler auftritt (Stack-Trace und Browserklasse) werden gespeichert. Standard 30. Eine Gruppe bleibt länger erhalten, siehe nächste Einstellung.

admin-settings-help-retention-client-errors-group-days = Tage, die eine Browser-Fehlergruppe (Name, Nachricht, Anzahl) aufbewahrt wird, nachdem sie das letzte Mal gesehen wurde. Standardwert 180. Niemals kürzer als die Vorkommensaufbewahrung.

admin-settings-help-retention-metrics-day-buckets-days = Tagelang werden tägliche Leistungsstatistiken aufbewahrt, bevor sie gelöscht werden. 0 behält sie unbegrenzt bei, wodurch die Tabelle langsam wächst, solange der Server läuft.

admin-settings-help-retention-metrics-hour-buckets-days = Tage der stündlichen Leistungsstatistiken, die aufbewahrt werden, bevor sie in Tageszeilen zusammengeführt werden (mindestens 1).

admin-settings-help-retention-metrics-minute-buckets-hours = Stunden von einminütigen Leistungsstatistiken, die aufbewahrt werden, bevor sie in stündlichen Zeilen zusammengeführt werden (mindestens 1).

admin-settings-help-retention-user-allow-strip-attachments = Ob Benutzer Regeln erstellen dürfen, die große Anhänge aus alten E-Mails entfernen. Standardmäßig deaktiviert. Wenn diese Option deaktiviert ist, wird die Aktion im Regeleditor ausgeblendet, eine Regel, die sie verwendet, wird beim Speichern abgelehnt und zuvor gespeicherte Regeln werden bei der Ausführung des Jobs übersprungen.

admin-settings-help-retention-user-calendar-enabled = Ob Benutzer Kalender auswählen dürfen, deren Ereignisse eine festgelegte Anzahl von Tagen nach ihrem Ende gelöscht werden. Standardmäßig deaktiviert. Wenn diese Option deaktiviert ist, wird sie in den Einstellungen ausgeblendet, eine Auswahl wird beim Speichern abgelehnt und zuvor gespeicherte Auswahlmöglichkeiten werden nicht ausgeführt.

admin-settings-help-retention-user-calendar-max-days = An den meisten Tagen kann ein Benutzer warten, bis beendete Ereignisse gelöscht werden. Ein größerer Wert wird beim Speichern abgelehnt und bei der Ausführung verringert.

admin-settings-help-retention-user-calendar-min-days = Wenige Tage nach dem Ende eines Ereignisses, bevor es durch eine Benutzerregel gelöscht werden kann. Ein kleinerer Wert wird beim Speichern abgelehnt und bei der Ausführung erhöht.

admin-settings-help-retention-user-notes-enabled = Gibt an, ob Benutzer Notizen entfernen können, die sich für eine festgelegte Anzahl von Tagen nicht geändert haben. Standardmäßig deaktiviert. Entfernte Notizen können wiederhergestellt werden, bis das Rückgängig-Fenster für Aufbewahrungsregeln endet.

admin-settings-help-retention-user-notes-max-days = An den meisten Tagen kann es sein, dass ein Benutzer wartet, bevor unberührte Notizen entfernt werden. Ein größerer Wert wird beim Speichern abgelehnt und bei der Ausführung verringert.

admin-settings-help-retention-user-notes-min-days = Wenige Tage muss eine Notiz unberührt bleiben, bevor eine Benutzerregel sie entfernen kann. Ein kleinerer Wert wird beim Speichern abgelehnt und bei der Ausführung erhöht.

admin-settings-help-retention-user-strip-keep-inline-images = Ob die im Nachrichtentext angezeigten Bilder erhalten bleiben, wenn Anhänge entfernt werden.

admin-settings-help-retention-user-strip-min-age-days = Eine Nachricht muss mindestens wenige Tage alt sein, bevor eine Regel ihre Anhänge entfernen darf. Ein kleinerer Wert wird beim Speichern der Regel abgelehnt und bei der Ausführung erhöht.

admin-settings-help-retention-user-strip-min-attachment-kb = Es werden nur Anhänge mit einer Mindestgröße dieser Größe entfernt. Kleinere Anhänge bleiben bei der Nachricht.

admin-settings-help-retention-user-strip-undo-days = Tage, die die ursprüngliche Nachricht im Papierkorb verbleibt, nachdem ihre Anhänge entfernt wurden. -1 folgt dem Rückgängig-Fenster für Aufbewahrungsregeln. 0 löscht das Original sofort.

admin-settings-help-server-shutdown-grace-seconds = Gibt an, wie lange ein stoppender Server laufende Anfragen zu Ende bringt, bevor er Verbindungen schließt (in Sekunden). Die Variable MW_HTTP_SHUTDOWN_TIMEOUT_MS überschreibt es. Gilt beim nächsten Start.

admin-settings-help-session-last-activity-flush-seconds = Die zuletzt gesehene Zeit einer Sitzung wird höchstens so oft in Sekunden geschrieben. Zero schreibt es auf jede Anfrage. Es ändert nichts an den Leerlauf- oder absoluten Grenzwerten. Gilt beim nächsten Start.

admin-settings-help-session-persist-across-restarts = Wenn diese Option aktiviert ist, bleiben Postfach-, App- und Konsolensitzungen nach dem Neustart des Servers gültig. Wenn diese Option deaktiviert ist, endet jede Sitzung, wenn der Server startet. Gilt beim nächsten Start.

admin-settings-key-attachments-bar-click-to-toggle = Klicken Sie zum Umschalten auf die Anhängeleiste

admin-settings-key-attachments-bar-download-all = Klicken Sie auf die Schaltfläche „Alle herunterladen“.

admin-settings-key-attachments-bar-show-download-buttons = Schaltflächen zum Herunterladen von Anhängen

admin-settings-key-attachments-bar-zip-inline-images = Inline-Bilder im Download all ZIP

admin-settings-key-cap-reading-delete-sender = Erlauben Sie das Löschen aller E-Mails von einem Absender

admin-settings-key-compose-feature-family = Komponistenfunktionen

admin-settings-key-compose-headers-allow-auto-submitted = Automatische Übermittlung zulassen

admin-settings-key-compose-headers-allow-custom = Benutzerdefinierte Header zulassen

admin-settings-key-compose-headers-allow-return-receipt-to = Erlauben Sie „Rücksendebestätigung an“.

admin-settings-key-compose-headers-allow-threading-edit = Bearbeiten von Threading-Headern zulassen

admin-settings-key-compose-headers-forced = Erzwungene Header

admin-settings-key-compose-insert-image-resize-default = Voreingestellte Bildgröße

admin-settings-key-compose-insert-max-inline-image-mb = Größtes Inline-Bild (MB)

admin-settings-key-compose-layout-default-font = Standardschriftart für Nachrichten

admin-settings-key-compose-reply-to-allow-override = Erlauben Sie das Ändern der Antwortadresse

admin-settings-key-compose-reply-to-show = Zeile „Antwort an“.

admin-settings-key-compose-sent-folder-fixed-id = Gesendete Kopie: fester Ordner

admin-settings-key-compose-sent-folder-mode = Gesendete Kopie: Ordnerregel

admin-settings-key-compose-sent-folder-per-identity = Gesendete Kopie: Ordner pro Identität

admin-settings-key-compose-sent-folder-rules = Gesendete Kopie: Regeln

admin-settings-key-compose-sent-folder-skip-copy = Gesendete Kopie: nicht speichern

admin-settings-key-compose-styles-custom = Benutzerdefinierte Stile

admin-settings-key-compose-styles-default-set = Stil-Set

admin-settings-key-compose-styles-max-custom = Benutzerdefinierte Stile pro Benutzer

admin-settings-key-contacts-save-from-message-add-note = Hinweis zu aus einer Nachricht gespeicherten Kontakten

admin-settings-key-contacts-save-from-message-default-book = Adressbuch für aus einer Nachricht gespeicherte Kontakte

admin-settings-key-contacts-save-from-message-default-scope = Beim Speichern aus einer Nachricht werden Kontakte zuerst angekreuzt

admin-settings-key-contacts-save-from-message-enabled = Kontakte aus einer Nachricht speichern

admin-settings-key-contacts-save-from-message-skip-automated = Überspringen Sie automatische Absender beim Speichern von Kontakten

admin-settings-key-contacts-save-sender-enabled = Absender als Kontakt speichern

admin-settings-key-contacts-save-sender-use-reply-to = Speichern Sie „Antwort an“ für Listenabsender

admin-settings-key-dsr-admin-step-up = Bestätigen Sie vor der Genehmigung noch einmal

admin-settings-key-export-default-format = Standardformat des Exportdialogs

admin-settings-key-export-default-scope = Standardumfang des Exportdialogs

admin-settings-key-export-dialog-enabled = Export-Dialog

admin-settings-key-export-folder-enabled = Ordnerexport

admin-settings-key-export-folder-format = Ordnerexportformat

admin-settings-key-export-folder-include-subfolders = Unterordner in den Export einbeziehen

admin-settings-key-export-folder-max-bytes = Größenbeschränkung für den Ordnerexport

admin-settings-key-export-folder-max-messages = Nachrichtenlimit für den Ordnerexport

admin-settings-key-export-formats = Dialogformate exportieren

admin-settings-key-export-message-download-enabled = Nachrichten-Download

admin-settings-key-folders-hide-empty = Leere Ordner ausblenden

admin-settings-key-folders-pinned-hide-from-tree = Verstecken Sie angeheftete Ordner in der Baumstruktur

admin-settings-key-mail-badge-auth = Authentifizierungsausweis

admin-settings-key-mail-badge-density = Größe des Leserausweises

admin-settings-key-mail-badge-folder = Ordnerabzeichen

admin-settings-key-mail-badge-headers = Header-Abzeichen

admin-settings-key-mail-badge-labels = Etiketten für Leserausweise

admin-settings-key-mail-badge-list = Mailinglisten-Abzeichen

admin-settings-key-mail-badge-sanitized = Desinfiziertes HTML-Abzeichen

admin-settings-key-mail-badge-spam = Spam-Score-Abzeichen

admin-settings-key-mail-brand-avatars-require-auth = Authentifizierung für Markenlogos erforderlich

admin-settings-key-mail-compose-header-auto-collapse-seconds = Verzögerung beim automatischen Reduzieren des Composer-Headers

admin-settings-key-mail-compose-priority-default = Standard-Nachrichtenpriorität

admin-settings-key-mail-compose-priority-enabled = Prioritätsauswahl im Composer

admin-settings-key-mail-expiry-hide-expired = Abgelaufene Nachrichten

admin-settings-key-mail-list-keep = Immer behalten

admin-settings-key-mail-list-page-size = Seitengröße der Nachrichtenliste

admin-settings-key-mail-list-unsubscribe-mode = Mailingliste Abmelden

admin-settings-key-mail-message-list-group = Nachrichtengruppierung

admin-settings-key-mail-priority-marks = Prioritätszeichen

admin-settings-key-mail-priority-marks-style = Stil der Prioritätsmarke

admin-settings-key-mail-priority-send-headers = Beim Senden geschriebene Prioritätsheader

admin-settings-key-mail-priority-trust-sender-headers = Vertrauen Sie der vom Absender festgelegten Priorität

admin-settings-key-mail-reply-from-header-priority = Header, die die Antwortidentität auswählen

admin-settings-key-mail-search-default-scope = Standardmäßiger E-Mail-Suchbereich

admin-settings-key-metrics-dimensions-enabled = Teilen Sie die Statistiken nach Clienttyp und Geräteklasse auf

admin-settings-key-metrics-persist-enabled = Behalten Sie Statistiken über Neustarts hinweg bei

admin-settings-key-metrics-persist-flush-seconds = Statistikspeicherintervall (Sekunden)

admin-settings-key-observability-client-capture-console = Konsolenfehler melden

admin-settings-key-observability-client-capture-csp = Melden Sie CSP-Verstöße

admin-settings-key-observability-client-enabled = Browser-Fehlerberichte

admin-settings-key-observability-client-forward = Fehlerberichte weiterleiten

admin-settings-key-observability-client-include-account-hash = Konto-Hash zu Fehlerberichten hinzufügen

admin-settings-key-observability-client-max-per-session = Fehlerberichte pro Seitenladevorgang

admin-settings-key-observability-client-sample-rate = Abtastrate des Fehlerberichts

admin-settings-key-observability-client-source-maps = Lösen Sie Stapelspuren mit Quellkarten auf

admin-settings-key-privacy-error-reports = Fehlerberichte senden

admin-settings-key-privacy-image-cache-max-stale-secs = Veraltet halten, wenn der Ursprung ausfällt (Sekunden)

admin-settings-key-privacy-image-cache-stale-secs = Während der erneuten Validierung veraltet servieren (Sekunden)

admin-settings-key-privacy-image-resize-enabled = Größe großer Bilder ändern

admin-settings-key-privacy-image-resize-format = Größe des Bildformats geändert

admin-settings-key-privacy-image-resize-max-pixels = Größtes Bild, dessen Größe geändert werden soll (Pixel)

admin-settings-key-privacy-image-resize-max-width = Größte Bildgröße (Pixel)

admin-settings-key-privacy-sessions-show-linked-origin = Ursprung des verknüpften Postfachs in Sitzungen

admin-settings-key-properties-folder-enabled = Dialogfeld „Ordnereigenschaften“.

admin-settings-key-properties-folder-notes-enabled = Ordnernotizen

admin-settings-key-properties-folder-notes-max-length = Länge der Ordnernotiz

admin-settings-key-properties-folder-overrides-allowed = Überschreibungen pro Ordner sind zulässig

admin-settings-key-properties-folder-sections = Abschnitte des Ordnerdialogs

admin-settings-key-properties-folder-size-compute = Berechnung der Ordnergröße

admin-settings-key-properties-headers-raw = Rohe Header

admin-settings-key-properties-message-enabled = Dialogfeld „Nachrichteneigenschaften“.

admin-settings-key-properties-message-permanent-delete = Dauerhaftes Löschen aus Eigenschaften

admin-settings-key-properties-message-sections = Abschnitte des Nachrichtendialogs

admin-settings-key-properties-offline-clear-enabled = Offline-Cache pro Ordner löschen

admin-settings-key-properties-remote-location-visibility = Details zum entfernten Standort

admin-settings-key-properties-scoring-show = Bewertungs- und Authentifizierungsdetails

admin-settings-key-properties-shortcut-enabled = Eigenschaftenverknüpfung

admin-settings-key-properties-shortcut-key = Tastenkombination „Eigenschaften“.

admin-settings-key-reader-actions-archive = Platzierung der Archivaktion

admin-settings-key-reader-actions-block-sender = Blockieren Sie die Platzierung der Absenderaktion

admin-settings-key-reader-actions-colors = Aktionsplatzierung der Nachrichtenfarben

admin-settings-key-reader-actions-conversation = Platzierung der Konversationsaktion anzeigen

admin-settings-key-reader-actions-create-filter = Erstellen Sie eine Filteraktionsplatzierung

admin-settings-key-reader-actions-delete = Aktionsplatzierung löschen

admin-settings-key-reader-actions-delete-sender = Alle Nachrichten aus der Absenderaktion löschen

admin-settings-key-reader-actions-edit-draft = Bearbeiten Sie die Platzierung des Aktionsentwurfs

admin-settings-key-reader-actions-export = Aktionsplatzierung exportieren

admin-settings-key-reader-actions-finish-followup = Markieren Sie die Nachverfolgung als abgeschlossene Aktion

admin-settings-key-reader-actions-flag = Aktionsplatzierung kennzeichnen/deaktivieren

admin-settings-key-reader-actions-focus-focused = Gehen Sie zur fokussierten Aktionsplatzierung über

admin-settings-key-reader-actions-focus-other = Wechseln Sie zu „Andere Aktionsplatzierung“.

admin-settings-key-reader-actions-forward = Platzierung der Vorwärtsaktion

admin-settings-key-reader-actions-headers = Aktionsplatzierung der rohen Header

admin-settings-key-reader-actions-links = Überprüfen Sie die Platzierung der Link-Aktion

admin-settings-key-reader-actions-move = Aktionsplatzierung verschieben

admin-settings-key-reader-actions-not-spam = Keine Spam-Aktionsplatzierung

admin-settings-key-reader-actions-open-draft = Aktionsplatzierung für offenen Antwortentwurf

admin-settings-key-reader-actions-pin = Aktionsplatzierung anheften/entfernen

admin-settings-key-reader-actions-print = Aktionsplatzierung drucken

admin-settings-key-reader-actions-properties = Aktionsplatzierung für Nachrichteneigenschaften

admin-settings-key-reader-actions-read = Gelesene/ungelesene Aktionsplatzierung

admin-settings-key-reader-actions-reply = Platzierung der Antwortaktion

admin-settings-key-reader-actions-reply-all = Auf alle Aktionsplatzierungen antworten

admin-settings-key-reader-actions-save-contacts = Aktionsplatzierung für Nachrichtenkontakte speichern

admin-settings-key-reader-actions-save-sender = Speichern Sie die Platzierung der Absenderaktion

admin-settings-key-reader-actions-search = Platzierung verwandter Suchaktionen

admin-settings-key-reader-actions-security = Platzierung der Aktion im Sicherheitsmodus

admin-settings-key-reader-actions-spam = Aktionsplatzierung als Spam markieren

admin-settings-key-reader-actions-sweep = Platzierung der Absenderaktion durchsuchen

admin-settings-key-reader-actions-view-source = Anzeigen der Aktionsplatzierung der Nachrichtenquelle

admin-settings-key-reader-actions-zoom = Platzierung der Zoom-Aktion

admin-settings-key-retention-client-errors-event-days = Aufbewahrte Fehlerereignisse (Tage)

admin-settings-key-retention-client-errors-group-days = Aufbewahrte Fehlergruppen (Tage)

admin-settings-key-retention-metrics-day-buckets-days = Täglich geführte Statistiken (Tage)

admin-settings-key-retention-metrics-hour-buckets-days = Stündliche Statistiken (Tage)

admin-settings-key-retention-metrics-minute-buckets-hours = Geführte Minutenstatistik (Stunden)

admin-settings-key-retention-user-allow-strip-attachments = Benutzer können Anhänge aus alten E-Mails entfernen

admin-settings-key-retention-user-calendar-enabled = Benutzer können beendete Kalenderereignisse löschen

admin-settings-key-retention-user-calendar-max-days = Die meisten Tage vor beendeten Ereignissen werden gelöscht

admin-settings-key-retention-user-calendar-min-days = Wenige Tage bevor beendete Ereignisse gelöscht werden

admin-settings-key-retention-user-notes-enabled = Benutzer können unberührte Notizen entfernen

admin-settings-key-retention-user-notes-max-days = In den meisten Tagen werden unberührte Notizen entfernt

admin-settings-key-retention-user-notes-min-days = Wenige Tage, bevor unberührte Notizen entfernt werden

admin-settings-key-retention-user-strip-keep-inline-images = Behalten Sie Inline-Bilder bei

admin-settings-key-retention-user-strip-min-age-days = Wenige Tage, bevor Anhänge entfernt werden

admin-settings-key-retention-user-strip-min-attachment-kb = Kleinster entfernter Anhang (KB)

admin-settings-key-retention-user-strip-undo-days = Rückgängig-Fenster nach dem Entfernen von Anhängen (Tage)

admin-settings-key-server-shutdown-grace-seconds = Kulanzfrist für das Herunterfahren

admin-settings-key-session-last-activity-flush-seconds = Zuletzt gesehenes Schreibintervall

admin-settings-key-session-persist-across-restarts = Behalten Sie Sitzungen über Neustarts hinweg bei

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-mail-cache-keep-indefinitely = Standardmäßig deaktiviert. Speichern Sie verschlüsselte, kontobezogene E-Mail-Snapshots über Browser-Neuladevorgänge hinweg ohne Ablaufdatum. Bestehende Eintrags- und Bytegrenzen gelten weiterhin. Benutzer können gespeicherte Kopien löschen; Durch Deaktivieren dieser Einstellung werden sie im aktiven Browser entfernt. Dadurch wird keine Berechtigung zum dauerhaften Speichern des Browsers oder zum Herunterladen von Anhängen gewährt.

admin-settings-help-mail-conversations-allowed = Ermöglichen Sie Benutzern, verwandte Antworten in Konversationen zu gruppieren. Standardmäßig verfügbar, aber die Benutzerpräferenz beginnt. Durch das Deaktivieren der Verfügbarkeit wird die Auswahl des Benutzers außer Kraft gesetzt, ohne dass die Kategoriegruppierung geändert wird.

admin-settings-help-mail-conversations-enabled = Standardmäßig deaktiviert. Gruppieren Sie geladene Antworten nach Thread, unabhängig von „Gruppieren nach“. Legen Sie die Standardeinstellung fest oder sperren Sie diese Einstellung. Die Konversationsverfügbarkeit muss ebenfalls aktiviert sein.

admin-settings-key-mail-cache-keep-indefinitely = Behalten Sie den E-Mail-Cache des Browsers auf unbestimmte Zeit bei

admin-settings-key-mail-conversations-allowed = Konversations-Threading verfügbar machen

admin-settings-key-mail-conversations-enabled = Gruppieren Sie Nachrichten in Konversationen

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-links-trusted-sender-domains = Bis zu 100 genaue Absenderdomänen in Kleinbuchstaben, deren überprüfte Nachrichtenlinks automatisch aktiviert werden. Subdomains sind getrennt. Dadurch werden Zielwebsites nicht als vertrauenswürdig eingestuft und Remote-Bilder werden nicht zugelassen. Standardmäßig leer; Benutzer können es ändern, sofern es nicht gesperrt ist.

admin-settings-key-links-trusted-sender-domains = Vertrauenswürdige Absenderdomänen für Nachrichtenlinks

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-settings-help-reader-attachment-action-style = Download- und Dateidetails werden neben dem Vorschautitel als Symbole (Standard), Beschriftungen oder Symbole und Beschriftungen angezeigt. Kann global, pro Domäne oder pro Benutzer festgelegt und gesperrt werden.

admin-settings-help-reader-attachment-thumbnails = Standardmäßig deaktiviert. Erlauben Sie sichtbaren Bildanhängen das Laden von Miniaturvorschauen. Es gelten weiterhin die Sicherheitsregeln für Anhänge. Kann global, pro Domäne oder pro Benutzer festgelegt und gesperrt werden.

admin-settings-key-reader-attachment-action-style = Vorschauaktionen für Anhänge

admin-settings-key-reader-attachment-thumbnails = Miniaturansichten der Bildanhänge

admin-settings-key-mail-compose-auto-add-mention-recipients = Erwähnte Personen automatisch als Empfänger hinzufügen
admin-settings-help-mail-compose-auto-add-mention-recipients = Wenn Sie eine Person aus den @Erwähnungsvorschlägen auswählen, wird sie zu An hinzugefügt, sofern sie nicht bereits in An, Cc oder Bcc steht. Standardmäßig deaktiviert.

admin-settings-linked-disable-title = Gespeicherten Zugriff auf verknüpfte Konten deaktivieren?
admin-settings-linked-disable-body = Beim Speichern werden die Zugangsdaten zur Erneuerung des Zugriffs auf verknüpfte Konten entfernt. Benutzer müssen sich zur Erneuerung des Zugriffs erneut anmelden. Erneutes Aktivieren stellt diese Zugangsdaten nicht wieder her.
admin-settings-linked-disable-confirm = Deaktivieren und gespeicherten Zugriff löschen

admin-settings-key-reader-auto-hide-header = Nachrichtenköpfe automatisch ausblenden
admin-settings-help-reader-auto-hide-header = Jede Nachricht mit eingeklappten Absender-, Empfänger- und Metadaten öffnen. Betreff und Aktionen bleiben sichtbar. Mit Kopf anzeigen lassen sich die Details einblenden. Standardmäßig deaktiviert.

admin-settings-key-time-region = Region
admin-settings-help-time-region = auto oder ein Regionscode mit zwei Buchstaben, z. B. GB, US oder PT.

admin-settings-key-time-date-format = Datumsformat
admin-settings-help-time-date-format = auto, short, medium, long, full oder ein Muster wie yyyy-MM-dd oder dd/MM/yyyy. Literalen Text in 'Anführungszeichen' setzen.

admin-settings-key-time-time-format = Zeitformat
admin-settings-help-time-time-format = auto, short, medium, long, full oder HH:mm:ss / h:mm a. H = 24 Stunden, h = 12 Stunden, a = AM/PM.

admin-settings-key-time-hour-cycle = Uhrformat
admin-settings-help-time-hour-cycle = Benutzerdefinierte H- oder h-Muster haben Vorrang.

admin-settings-key-time-first-day-of-week = Erster Wochentag
admin-settings-help-time-first-day-of-week = Automatisch folgt den regionalen Kalenderregeln.

admin-settings-key-compliance-mail-enabled = E-Mail aktivieren
admin-settings-help-compliance-mail-enabled = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-compliance-mail-host = SMTP-Relay-Host
admin-settings-help-compliance-mail-host = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-compliance-mail-port = SMTP-Port
admin-settings-help-compliance-mail-port = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-compliance-mail-security = TLS-Modus
admin-settings-help-compliance-mail-security = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-compliance-mail-from = Absenderadresse
admin-settings-help-compliance-mail-from = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-compliance-mail-username = SMTP-Benutzername
admin-settings-help-compliance-mail-username = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-compliance-mail-allowPrivate = Privates Relay zulassen
admin-settings-help-compliance-mail-allowPrivate = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-dsr-notify-emailAdmins = Betreiber über Datenanfragen benachrichtigen
admin-settings-help-dsr-notify-emailAdmins = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-dsr-notify-operatorAddresses = Betreiberempfänger
admin-settings-help-dsr-notify-operatorAddresses = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-dsr-notify-emailRequester = Anfragende per E-Mail informieren
admin-settings-help-dsr-notify-emailRequester = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-breach-notify-email = Betreiber bei Datenschutzverletzungen benachrichtigen
admin-settings-help-breach-notify-email = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-breach-subjects-email = Betroffene per E-Mail informieren
admin-settings-help-breach-subjects-email = Konfigurieren Sie ein SMTP-Relay mit TLS. E-Mail nutzt die Proxyroute system_mail. Prüfen Sie eine ungewisse Annahme vor erneutem Versand.

admin-settings-key-compliance-mail-requireProxy = Proxyroute verlangen
admin-settings-help-compliance-mail-requireProxy = Proxyroute verlangen (system_mail).
admin-settings-key-retention-complianceMail-days = Aufbewahrung abgeschlossener E-Mails (Tage)
admin-settings-help-retention-complianceMail-days = Abgeschlossene Zustellungen werden so viele Tage aufbewahrt. Offene und ungewisse bleiben bis zur Klärung.
