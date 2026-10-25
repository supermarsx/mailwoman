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
