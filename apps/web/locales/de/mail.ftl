# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Einstellungen
mail-compose = Verfassen
mail-nav-mailboxes = Postfächer
mail-nav-attachments = Anhänge
mail-nav-outbox = Postausgang
mail-nav-sharing = Ordner teilen
mail-nav-apps = Apps
mail-logout = Abmelden
mail-offline = Offline
mail-module-loading = { $module } wird geladen…

# -- Search ------------------------------------------------------------------
mail-search = Suchen
mail-search-label = E-Mails durchsuchen
mail-search-placeholder = E-Mails durchsuchen — from:alice subject:invoice larger:1mb
mail-search-clear = Leeren

# -- Message list ------------------------------------------------------------
mail-list-label = Nachrichten
mail-loading = Nachrichten werden geladen…
mail-empty = Keine Nachrichten
mail-unknown-sender = (unbekannter Absender)
mail-no-subject = (kein Betreff)
mail-pinned = Angeheftet
mail-has-attachment = Mit Anhang
mail-answered = Beantwortet
mail-forwarded = Weitergeleitet
mail-unread = Ungelesen
mail-flagged = Markiert
mail-selected = Ausgewählt
mail-row-position = Nachricht { $pos } von { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } Nachricht
   *[other] { $count } Nachrichten
}
mail-thread-expand = Unterhaltung aufklappen, { $count ->
    [one] { $count } Nachricht
   *[other] { $count } Nachrichten
}
mail-thread-collapse = Unterhaltung zuklappen, { $count ->
    [one] { $count } Nachricht
   *[other] { $count } Nachrichten
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = Ansichtsoptionen
mail-reading-pane = Lesebereich
mail-reading-pane-right = Rechts
mail-reading-pane-bottom = Unten
mail-reading-pane-off = Aus

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Ausgewählte Nachrichten
mail-bulk-count = { $count } ausgewählt
mail-mark-read = Als gelesen markieren
mail-mark-unread = Als ungelesen markieren
mail-flag-message = Markieren
mail-unflag-message = Markierung entfernen
mail-move-to = In Ordner verschieben
mail-move-to-placeholder = Verschieben nach…
mail-select-all = Alle auswählen
mail-clear-selection = Auswahl aufheben

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-filter = Posteingangsfilter
mail-inbox-focused = Relevant
mail-inbox-other = Sonstige

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Geöffnete Tabs
mail-subtab-pin = { $title } anheften
mail-subtab-unpin = { $title } lösen
mail-subtab-tearoff = { $title } in neuem Fenster öffnen
mail-subtab-close = { $title } schließen

# -- Per-row actions ---------------------------------------------------------
mail-pin = Anheften
mail-unpin = Lösen
mail-snooze = Zurückstellen
mail-snooze-menu = Zurückstellen bis
mail-snooze-later = Später heute
mail-snooze-tomorrow = Morgen
mail-snooze-next-week = Nächste Woche
mail-unsnooze = Nicht mehr zurückstellen
mail-label = Label
mail-labels-menu = Labels
mail-flag = Zur Nachverfolgung markieren
mail-clear-flag = Nachverfolgung entfernen
mail-archive = Archivieren
mail-delete = Löschen
mail-more-actions = Weitere Aktionen

# -- Reader ------------------------------------------------------------------
mail-reader-label = Nachricht
mail-reader-empty = Wählen Sie eine Nachricht zum Lesen aus
mail-back = Zurück
mail-reader-from = Von: { $addr }
mail-reader-to = An: { $addr }
mail-reader-cc = Cc: { $addr }
mail-reader-actions = Nachrichtenaktionen
mail-reply = Antworten
mail-reply-all = Allen antworten
mail-forward = Weiterleiten
mail-reply-failed = Die Nachricht konnte nicht vorbereitet werden. Versuchen Sie es erneut.
mail-reply-unthreaded = Die ID der ursprünglichen Nachricht konnte nicht gelesen werden. Diese Antwort wird nicht mit ihr in einer Unterhaltung verknüpft.
mail-spam = Spam
mail-export = Exportieren
mail-sweep-sender = Absender aufräumen
mail-message-body = Nachrichtentext
mail-no-content = Kein Inhalt
mail-sanitizing = Wird bereinigt…
mail-attachments = Anhänge
mail-attachment-unnamed = (unbenannt)
mail-attachment-open = Anhang { $name }
mail-attachment-close = Anhang schließen
mail-attachment-loading = Anhang wird geladen…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Verschlüsselte Nachricht
mail-encrypted-note = Diese Nachricht ist Ende-zu-Ende-verschlüsselt. Entsperren Sie sie auf diesem Gerät, um sie zu lesen.
mail-key-passphrase = Schlüssel-Passphrase
mail-decrypt = Entschlüsseln
mail-decrypting = Wird entschlüsselt…
mail-decrypt-no-key = Es ist kein privater Schlüssel zum Entschlüsseln dieser Nachricht verfügbar.
mail-decrypt-failed = Entschlüsselung fehlgeschlagen

# -- Reply and forward: text written into the outgoing message --------------
mail-quote-attribution = Am { $date } schrieb { $name }:
mail-quote-attribution-undated = { $name } schrieb:
mail-forward-heading = ---------- Weitergeleitete Nachricht ----------
mail-forward-from = Von: { $addr }
mail-forward-date = Datum: { $date }
mail-forward-subject = Betreff: { $subject }
mail-forward-to = An: { $addr }
mail-forward-cc = Cc: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Nachricht verfassen
mail-compose-title = Neue Nachricht
mail-compose-title-reply = Antworten
mail-compose-title-reply-all = Allen antworten
mail-compose-title-forward = Weiterleiten
mail-compose-close = Schließen
mail-compose-from = Von
mail-compose-from-default = Standard
mail-compose-to = An
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Kontaktvorschläge
mail-compose-cc = Cc
mail-compose-bcc = Bcc
mail-compose-show-cc-bcc = Cc oder Bcc hinzufügen
mail-compose-bcc-encrypted = Eine verschlüsselte Nachricht enthält den Schlüssel jedes Empfängers, daher wäre ein Bcc-Empfänger für die anderen sichtbar. Entfernen Sie die Bcc-Empfänger oder schalten Sie die Verschlüsselung aus.
mail-compose-plaintext-confirm-title = Entschlüsselten Text unverschlüsselt senden?
mail-compose-plaintext-confirm-body = Diese Nachricht zitiert Text aus einer verschlüsselten Nachricht. Sie ist nicht zur Verschlüsselung eingestellt, daher würde der zitierte Text ungeschützt gesendet.
mail-compose-plaintext-confirm-send = Unverschlüsselt senden
mail-compose-plaintext-confirm-cancel = Zurück
mail-compose-subject = Betreff
mail-compose-body = Text
mail-compose-attachments = Anhänge
mail-compose-remove-attachment = { $name } entfernen
mail-compose-attach-nextcloud = Aus Nextcloud anhängen
mail-compose-close-nextcloud = Nextcloud schließen
mail-compose-attach-file = Datei anhängen
mail-compose-uploading = Wird hochgeladen…
mail-compose-upload-unavailable = Das Hochladen von Dateien ist derzeit nicht verfügbar.
mail-compose-upload-failed = { $name } konnte nicht hochgeladen werden.
mail-compose-upload-too-large = { $name } ist { $size } MB groß. Die maximale Upload-Größe beträgt { $max } MB.
mail-compose-send-later = Später senden
mail-compose-cancel = Abbrechen
mail-compose-send = Senden
mail-compose-schedule = Planen
mail-compose-sending = Wird gesendet…
mail-compose-encrypted-subject = Verschlüsselte Nachricht
mail-compose-dlp-blocked = Das Senden wird durch eine Regel zur Verhinderung von Datenverlust blockiert (siehe Warnung oben).
mail-compose-send-failed = Senden fehlgeschlagen
mail-compose-send-later-past = Wählen Sie einen Sendezeitpunkt in der Zukunft oder leeren Sie das Feld, um sofort zu senden.
mail-compose-options = Sendeoptionen
mail-compose-request-receipt = Lesebestätigung anfordern
mail-compose-request-receipt-hint = Das Mailprogramm des Empfängers fragt möglicherweise nach, sendet ohne Nachfrage eine Bestätigung oder ignoriert die Anforderung.
mail-compose-track-opens = Erfassen, wann diese Nachricht geöffnet wird
mail-compose-track-opens-hint = Der Nachricht wird ein 1×1-Bild hinzugefügt, das von diesem Server geladen wird. Viele Mailprogramme blockieren Bilder oder laden sie vorab, daher beweist eine erfasste Öffnung nicht, dass die Nachricht gelesen wurde, und das Fehlen einer Erfassung beweist nicht, dass sie ungelesen ist. Der Empfänger wird nicht darüber informiert.
mail-compose-image-type = { $name } kann nicht in den Text eingefügt werden. Verwenden Sie ein PNG-, JPEG-, GIF- oder WebP-Bild.
mail-send-not-signed-in = Sie sind nicht angemeldet.
mail-send-no-draft-folder = Dieses Konto hat keinen Ordner für den Entwurf.
mail-send-no-recipient = Fügen Sie mindestens einen Empfänger hinzu.
mail-send-bad-recipient = { $count ->
    [one] Dies ist keine E-Mail-Adresse: { $addresses }
   *[other] Dies sind keine E-Mail-Adressen: { $addresses }
}
mail-send-later-past = Der Sendezeitpunkt liegt nicht in der Zukunft. Es wurde nichts gesendet.
mail-send-tracking-failed = Öffnungen können derzeit nicht erfasst werden, daher wurde nichts gesendet. Deaktivieren Sie „Erfassen, wann diese Nachricht geöffnet wird“, um ohne diese Funktion zu senden.
mail-send-draft-refused = Der Server hat diese Nachricht abgelehnt: { $reason }. Es wurde nichts gesendet.
mail-send-submission-refused = Der Server hat das Senden dieser Nachricht abgelehnt: { $reason }. Es wurde nichts gesendet.
mail-compose-sign-unlock-title = Signaturschlüssel entsperren
mail-compose-sign-unlock-note = Geben Sie Ihre Schlüssel-Passphrase ein, um diese Nachricht zu signieren. Der Schlüssel bleibt nur für dieses Verfassen-Fenster entsperrt.
mail-compose-sign-unlock = Entsperren
mail-compose-sign-unlocking = Wird entsperrt…
mail-compose-sign-no-key = Auf diesem Gerät ist kein Signaturschlüssel verfügbar.
mail-compose-sign-unlock-failed = Der Signaturschlüssel konnte nicht entsperrt werden.
mail-compose-sign-unlock-required = Entsperren Sie Ihren Signaturschlüssel, um diese signierte Nachricht zu senden.

mail-compose-format-plain = Nur Text
mail-compose-format-rich = Formatierter Text
mail-compose-rt-toolbar = Formatierung
mail-compose-rt-bold = Fett
mail-compose-rt-italic = Kursiv
mail-compose-rt-underline = Unterstrichen
mail-compose-rt-strike = Durchgestrichen
mail-compose-rt-bullet = Aufzählung
mail-compose-rt-ordered = Nummerierte Liste
mail-compose-rt-quote = Zitat
mail-compose-rt-link = Link
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Übernehmen
mail-compose-rt-image = Bild einfügen

mail-compose-signature = Signatur
mail-compose-signature-none = Signatur hinzufügen…

mail-compose-recall = Zurückrufen
mail-compose-recall-action = Zurückrufen
mail-compose-recall-holding = Zurückgehalten, wartet auf Versand
mail-compose-recall-scheduled = Geplant für { $when }

mail-compose-drafts = Entwürfe
mail-compose-drafts-empty = Keine gespeicherten Entwürfe
mail-compose-drafts-delete = Entwurf löschen
mail-compose-drafts-no-recipient = (kein Empfänger)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Postausgang
mail-outbox-refresh = Aktualisieren
mail-outbox-empty = Nichts wartet auf Versand
mail-outbox-send-now = Jetzt senden
mail-outbox-cancel = Abbrechen
mail-outbox-scheduled = Geplant
mail-outbox-holding = Wird bald gesendet
mail-outbox-sent = Gesendet
mail-outbox-canceled = Abgebrochen
mail-outbox-held = Zurückgehalten
mail-outbox-failed = Nicht gesendet
mail-outbox-release = Freigeben
mail-outbox-discard = Verwerfen
mail-outbox-held-note = Zurückgehalten. Wird erst nach Freigabe gesendet.
mail-outbox-held-note-api-key = Zurückgehalten — erstellt durch API-Schlüssel { $name }. Wird erst nach Freigabe gesendet.
mail-outbox-held-note-oauth = Zurückgehalten — erstellt durch die verbundene App { $name }. Wird erst nach Freigabe gesendet.
mail-outbox-origin-api-key = Erstellt durch API-Schlüssel { $name }
mail-outbox-origin-oauth = Erstellt durch die verbundene App { $name }
mail-outbox-to = An { $recipients }
mail-outbox-no-subject = (kein Betreff)
mail-outbox-attempts = { $count }-mal versucht. Letzter Fehler: { $error }
mail-outbox-error = Fehler: { $error }
mail-outbox-toast-canceled = Senden abgebrochen
mail-outbox-toast-discarded = Verworfen. Die Nachricht wurde nicht gesendet.
mail-outbox-toast-sent = Gesendet
mail-outbox-toast-retrying = Noch nicht gesendet: { $error }. Es wird erneut versucht.
mail-outbox-toast-failed = Nicht gesendet: { $error }
mail-outbox-toast-not-released = Senden nicht möglich: { $error }
mail-outbox-toast-not-canceled = Abbrechen nicht möglich: { $error }

# -- Network status and the offline queue ------------------------------------
mail-net-lost = Verbindung unterbrochen — neuer Versuch…
mail-net-back = Wieder online
mail-queue-sent = { $count ->
    [one] { $count } ausstehende Änderung gesendet
   *[other] { $count } ausstehende Änderungen gesendet
}
mail-queue-failed = { $count ->
    [one] { $count } ausstehende Änderung fehlgeschlagen
   *[other] { $count } ausstehende Änderungen fehlgeschlagen
}
mail-queue-notice-label = Änderungen, die nicht übernommen werden konnten
mail-queue-notice-title = { $count ->
    [one] { $count } offline vorgenommene Änderung konnte nicht übernommen werden
   *[other] { $count } offline vorgenommene Änderungen konnten nicht übernommen werden
}
mail-queue-kind-send = „{ $subject }“ senden
mail-queue-kind-send-untitled = Nachricht ohne Betreff senden
mail-queue-kind-move = Nachricht verschieben
mail-queue-kind-flag = Label oder Markierung ändern
mail-queue-kind-draft = Entwurf speichern
mail-queue-kind-pim = Änderung an Kalender, Aufgabe, Notiz oder Kontakt
mail-queue-reason = Grund: { $reason }
mail-queue-reason-unknown = Der Server hat keinen Grund genannt.
mail-queue-retry = Erneut versuchen
mail-queue-discard = Verwerfen
mail-queue-retry-item = Erneut versuchen: { $what }
mail-queue-discard-item = Verwerfen: { $what }
mail-queue-discarded = Verworfen. Die Änderung wurde nicht übernommen.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Nachrichten aufräumen
mail-sweep-title = { $sender } aufräumen
mail-sweep-all = Alle von diesem Absender löschen
mail-sweep-keep-latest = Neueste behalten, Rest löschen
mail-sweep-older-than = Älter als N Tage löschen
mail-sweep-block = Alle löschen und Absender blockieren
mail-sweep-days = Tage
mail-sweep-count = { $count ->
    [one] { $count } Nachricht wird in den Papierkorb verschoben
   *[other] { $count } Nachrichten werden in den Papierkorb verschoben
}
mail-sweep-cancel = Abbrechen
mail-sweep-run = { $count } aufräumen
mail-sweeping = Wird aufgeräumt…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Schließen
mail-undo-action = Rückgängig
mail-undo-send-cancel = Abbrechen
mail-undo-failed = Rückgängigmachen nicht möglich

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Label hinzugefügt
mail-toast-label-removed = Label entfernt
mail-toast-pinned = Angeheftet
mail-toast-unpinned = Gelöst
mail-toast-snoozed = Zurückgestellt
mail-toast-follow-up-set = Nachverfolgung gesetzt
mail-toast-follow-up-cleared = Nachverfolgung entfernt
mail-toast-archived = Archiviert
mail-toast-trashed = In den Papierkorb verschoben
mail-toast-marked-spam = Als Spam markiert
mail-toast-moved-to = Nach { $folder } verschoben
mail-toast-moved = In Ordner verschoben
mail-toast-archived-count = { $count } Nachrichten archiviert
mail-toast-trashed-count = { $count } Nachrichten in den Papierkorb verschoben
mail-toast-marked-spam-count = { $count } Nachrichten als Spam markiert
mail-toast-moved-count = { $count } Nachrichten verschoben
mail-toast-moved-to-count = { $count } Nachrichten nach { $folder } verschoben
mail-toast-marked-read = { $count ->
    [one] Als gelesen markiert
   *[other] { $count } Nachrichten als gelesen markiert
}
mail-toast-marked-unread = { $count ->
    [one] Als ungelesen markiert
   *[other] { $count } Nachrichten als ungelesen markiert
}
mail-toast-flagged = { $count ->
    [one] Markiert
   *[other] { $count } Nachrichten markiert
}
mail-toast-unflagged = { $count ->
    [one] Markierung entfernt
   *[other] Markierung von { $count } Nachrichten entfernt
}
mail-toast-move-failed = Die Nachrichten konnten nicht verschoben werden. Es wurde nichts verschoben.
mail-toast-update-failed = Die Nachrichten konnten nicht geändert werden. Es wurde nichts geändert.
mail-toast-update-refused = { $count ->
    [one] Der Server hat die Änderung für eine Nachricht abgelehnt
   *[other] Der Server hat die Änderung für { $count } Nachrichten abgelehnt
}
mail-toast-no-archive-folder = Kein Archivordner
mail-toast-no-trash-folder = Kein Papierkorb
mail-toast-no-spam-folder = Kein Spam-Ordner
mail-toast-sweep-nothing = Nichts aufzuräumen
mail-toast-swept = { $count ->
    [one] { $count } Nachricht aufgeräumt
   *[other] { $count } Nachrichten aufgeräumt
}
mail-toast-load-more-failed = Weitere Nachrichten konnten nicht geladen werden
mail-toast-export-nothing = Nichts zu exportieren
mail-toast-exported = .eml exportiert
mail-toast-export-failed = Export fehlgeschlagen
mail-toast-send-queued = In Warteschlange — wird gesendet, sobald Sie wieder online sind
mail-toast-tracking-not-bound = Die Nachricht wurde gesendet, aber ihre Öffnungen werden nicht erfasst.
mail-toast-send-scheduled = Versand geplant
mail-toast-sent = Nachricht gesendet
mail-toast-send-canceled = Senden abgebrochen

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Menüband
mail-ribbon-tab-home = Start
mail-ribbon-tab-view = Ansicht
mail-ribbon-tab-folder = Ordner
mail-ribbon-expand = Aufklappen
mail-ribbon-collapse = Zuklappen
mail-ribbon-group-new = Neu
mail-ribbon-compose = Verfassen
mail-ribbon-group-session = Sitzung
mail-ribbon-logout = Abmelden
mail-ribbon-group-theme = Design
mail-ribbon-group-density = Dichte
mail-ribbon-group-settings = Einstellungen
mail-ribbon-more = Mehr…
mail-ribbon-group-folders = Ordner
mail-density-compact = kompakt
mail-density-cozy = normal
mail-density-relaxed = großzügig

# -- Message security level: the three-position switch in the reader toolbar
mail-maxsec-label = Sicherheitsstufe der Nachricht
mail-maxsec-full = Vollständig
mail-maxsec-full-hint = Bereinigtes HTML mit Bildern und Formatierung.
mail-maxsec-no-media = Ohne Medien
mail-maxsec-no-media-hint = Bereinigtes HTML, alle Bilder und Medien entfernt.
mail-maxsec-plain-text = Nur Text
mail-maxsec-plain-text-hint = Kein HTML dargestellt — Nachricht wird nur als Text angezeigt.
