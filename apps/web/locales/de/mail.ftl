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
mail-feature-off = Durch Ihre Einstellungen oder Ihren Administrator ausgeschaltet
mail-send-disabled = Das Senden ist durch Ihre Einstellungen oder Ihren Administrator ausgeschaltet. Sie können weiterhin einen Entwurf speichern.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = Termine
mail-events-upcoming = Nur anstehende
mail-events-filter = Terminfilter
mail-loading-more = Ältere Nachrichten werden geladen …
mail-event-state-invitation = Einladung
mail-event-state-updated = Aktualisiert
mail-event-state-cancelled = Abgesagt
mail-event-state-reply = Antwort
mail-event-state-counter = Neue Zeit vorgeschlagen
mail-event-state-event = Termin
mail-event-hint = { $state }, { $when }
mail-row-actions-expand = Weitere Aktionen anzeigen
mail-remote-moved = Diese Nachricht wurde von einem anderen Gerät oder einer anderen App nach { $folder } verschoben. Sie lesen weiterhin die Kopie, die Sie geöffnet haben.
mail-remote-moved-unknown = Diese Nachricht wurde von einem anderen Gerät oder einer anderen App verschoben. Sie lesen weiterhin die Kopie, die Sie geöffnet haben.
mail-remote-deleted = Diese Nachricht wurde von einem anderen Gerät oder einer anderen App gelöscht. Sie können diese Kopie weiterlesen, bis Sie sie schließen.
mail-remote-dismiss = Schließen
mail-reply-draft = Antwort in Bearbeitung
mail-reply-draft-open = Entwurf öffnen
mail-draft-load-failed = Der Entwurf konnte nicht geöffnet werden. Versuchen Sie es erneut.
mail-reply-images-omitted =
    { $count ->
        [one] Ein Bild konnte nicht in die Antwort aufgenommen werden.
       *[other] { $count } Bilder konnten nicht in die Antwort aufgenommen werden.
    }
mail-export-forbidden = Ihre Administration hat den Nachrichtenexport für dieses Konto deaktiviert.
mail-export-open-failed = Die Exportoptionen konnten nicht geöffnet werden. Schließen Sie einen Arbeitsbereich-Tab und versuchen Sie es erneut.
mail-headers-title = Rohe Nachrichtenkopfzeilen
mail-headers-action-help = Die ursprünglichen Kopfzeilen der Nachricht anzeigen, einschließlich Zustellungs- und Authentifizierungsdetails.
mail-headers-help = Ursprüngliche Kopfzeilen der Nachricht. Umbrüche bleiben erhalten; hier werden nur Kopfzeilen angezeigt.
mail-headers-loading = Nachrichtenkopfzeilen werden geladen …
mail-headers-failed = Die Nachrichtenkopfzeilen konnten nicht geladen werden. Prüfen Sie Ihre Verbindung und versuchen Sie es erneut.
mail-headers-copy = Kopfzeilen kopieren
mail-headers-copied = Kopfzeilen kopiert.
mail-headers-copy-failed = Die Kopfzeilen konnten nicht kopiert werden. Markieren Sie den Text, um ihn manuell zu kopieren.
mail-attachment-failed = Dieser Anhang konnte nicht heruntergeladen werden. Versuchen Sie es erneut, sobald eine Verbindung besteht.
mail-compose-rt-rule = Horizontale Linie
mail-compose-signature-new = Neue Nachrichten
mail-compose-signature-replies = Antworten
mail-editor-body-size = { $size } KiB
mail-editor-body-size-label = Textgröße
mail-editor-body-size-help = Größe des aktuellen Nachrichtentexts als UTF-8, einschließlich HTML-Markup und eingebetteter Bilder. Kopfzeilen und separate Anhänge sind nicht enthalten.
mail-outbox-load-failed = Der Postausgang konnte nicht geprüft werden. Prüfen Sie Ihre Verbindung und wählen Sie dann Aktualisieren.
mail-outbox-unavailable = Postausgang nicht verfügbar. Wählen Sie Aktualisieren, um es erneut zu versuchen.
mail-maxsec-view = Ansicht: { $mode }
mail-maxsec-menu = Ansichtsmodus
mail-toast-tab-pinned = Tab „{ $target }“ angeheftet
mail-toast-tab-unpinned = Tab „{ $target }“ gelöst
mail-workspace-message = Nachricht
mail-opened-announcement = Geöffnet: { $subject }
mail-workspace-layout = Arbeitsbereich-Layout
mail-workspace-tabs = Tabs
mail-workspace-side-by-side = Nebeneinander
mail-workspace-stack = Untereinander
mail-workspace-refresh = Ordner aktualisieren
mail-workspace-load-error = Diese Nachricht konnte nicht geladen werden. Schließen Sie den Tab und versuchen Sie es erneut.
mail-resize-folders = Größe der Ordnerleiste ändern
mail-resize-list = Größe der Nachrichtenliste ändern
mail-resize-tiles = Größe der Arbeitsbereichsbereiche ändern
mail-show-folders = Ordnerleiste anzeigen
mail-hide-folders = Ordnerleiste ausblenden
mail-compact-folders = Kompakte Ordnersymbole
mail-toggle-list = Nachrichtenliste ein- oder ausblenden
mail-toggle-reader = Lesebereich ein- oder ausblenden
mail-folder-counts = { $unread } ungelesen, { $total } Nachrichten
mail-print = Drucken / Als PDF speichern
mail-go-to-top = Nach oben
mail-print-help = Drucken Sie diese Nachricht oder wählen Sie im Druckdialog „Als PDF speichern“. Externe Bilder bleiben blockiert.
mail-file-details = Dateidetails und Fingerabdruck
mail-file-type = Dateityp
mail-file-extension = Erweiterung
mail-file-detected = Erkannte Signatur
mail-file-unknown = Keine erkannte Signatur
mail-file-size = Größe
mail-file-size-bytes = { $count } Bytes
mail-file-hash-unavailable = Fingerabdruck nicht verfügbar (erfordert einen sicheren Browser und eine Datei bis 64 MiB).
mail-file-fingerprint-help = Ein Fingerabdruck identifiziert genau diese Bytes. Er beweist weder, dass die Datei sicher ist, noch wer sie erstellt hat.
mail-file-warning-executable = Diese Datei kann Programme ausführen oder Systemeinstellungen ändern. Laden Sie sie nur herunter, wenn Sie der Quelle vertrauen.
mail-file-warning-macro = Dieses Dokument kann Makros enthalten. Aktivieren Sie Makros nur, wenn Sie Absender und Zweck geprüft haben.
mail-file-warning-disguisedName = Der Dateiname könnte die tatsächliche Erweiterung verschleiern. Prüfen Sie die Dateidetails, bevor Sie sie herunterladen.
mail-file-warning-typeMismatch = Die erkannte Dateisignatur stimmt nicht mit dem angegebenen Typ überein. Die automatische Vorschau wurde blockiert.
mail-file-warning-activeDocument = Dieses Format kann aktive Inhalte enthalten. Die Vorschau ist isoliert; eine heruntergeladene Kopie kann sich beim Öffnen anders verhalten.
mail-file-preview-blocked = Die automatische Vorschau ist für diesen Dateityp oder bei abweichendem Typ deaktiviert.
mail-quote-header-from = Von:
mail-quote-header-sent = Gesendet:
mail-quote-header-to = An:
mail-quote-header-cc = Cc:
mail-quote-header-subject = Betreff:
mail-spam-score = Spam-Analyse
mail-spam-score-loading = Spam-Kopfzeilen werden geprüft …
mail-spam-score-unavailable = Spam-Bewertung nicht verfügbar
mail-spam-score-unverified = Laut Nachrichtenkopfzeilen gemeldet, nicht unabhängig überprüft. Vom Absender gelieferte Kopfzeilen können gefälscht sein.
mail-spam-score-positive = Die Kopfzeile meldet Spam.
mail-spam-score-negative = Die Kopfzeile meldet kein Spam.
mail-spam-score-unknown = Zu dieser Bewertung wurde kein Urteil mitgeliefert.
mail-external-recipients = Externe Empfänger
mail-external-count = { $count } Empfänger außerhalb vertrauenswürdiger Domains
mail-external-help = Prüfen Sie die Empfänger in An, Cc und Bcc, bevor Sie vertrauliche Informationen senden. Das Vertrauen richtet sich nach der gewählten Absenderidentität und Ihrer wirksamen Domainrichtlinie.
mail-external-reviewed = Ich habe diese externen Empfänger geprüft.
mail-external-confirm-required = Prüfen und bestätigen Sie die externen Empfänger vor dem Senden.
mail-quick-reply = Schnellantwort
mail-quick-reply-help = Einen kompakten Antwort-Tab mit bereits ausgefüllten Empfängern anbieten. Standardmäßig aus. Die üblichen Sicherheitsprüfungen und Sendeoptionen gelten weiterhin.
mail-quick-reply-expand = Empfänger und Betreff bearbeiten
mail-attachment-policy-loading = Anhangsrichtlinie wird geprüft …
mail-attachment-policy-unavailable = Die Anhangsrichtlinie konnte nicht geprüft werden. Versuchen Sie es erneut, bevor Sie diese Datei öffnen.
mail-attachment-policy-blocked = Ihre Administration hat diesen Anhang oder seinen Originaldownload blockiert.
mail-attachment-policy-warning = Ihre Administration empfiehlt Vorsicht bei diesem Anhang. Fahren Sie nur fort, wenn Sie der Quelle vertrauen.
mail-attachment-policy-acknowledge = Verstanden – fortfahren
mail-attachment-policy-no-preview = Anhangsvorschauen sind von Ihrer Administration deaktiviert. Originaldownloads sind möglicherweise weiterhin verfügbar.
mail-time-sent = Gesendet
mail-time-received = Empfangen
mail-time-unknown = Nicht angegeben
mail-time-comparison = Zeit in { $zone }
mail-time-privacy = Die Vergleichszeitzone ist Ihre Auswahl. Zeitstempel von Nachrichten geben keinen Aufschluss über den Standort von Personen.
mail-time-event-comparison = Terminzeit im Vergleich
mail-time-ambiguous = Diese Ortszeit tritt bei der Zeitumstellung zweimal auf. Beide möglichen Zeitpunkte werden angezeigt; prüfen Sie den vorgesehenen Offset des Termins, bevor Sie speichern.
mail-bulk-add-label = Label hinzufügen
mail-select-messages = Nachrichten auswählen
mail-bulk-remove-label = Label entfernen
mail-mark-spam = Als Spam markieren
mail-bulk-forward = Ausgewählte Nachrichten als Anhänge weiterleiten
mail-bulk-download = Ausgewählte Nachrichten als mbox herunterladen
mail-bulk-forward-subject = Weitergeleitete Nachrichten ({ $count })
mail-bulk-transfer-failed = Die ausgewählten Nachrichten konnten nicht vorbereitet werden. Wählen Sie bis zu 100 Nachrichten mit insgesamt höchstens 32 MiB und prüfen Sie die Exportberechtigungen Ihres Kontos.
mail-toast-labels-added = Label { $label } zu { $count } Nachrichten hinzugefügt
mail-toast-labels-removed = Label { $label } von { $count } Nachrichten entfernt
mail-action-help-reply = Dem Absender in einem neuen Verfassen-Tab antworten.
mail-action-help-reply-all = Dem Absender und den anderen Empfängern in einem neuen Verfassen-Tab antworten.
mail-action-help-forward = Diese Nachricht prüfen und an Empfänger Ihrer Wahl weiterleiten.
mail-action-help-pin = Diese Nachricht für schnellen Zugriff anheften oder die Anheftung aufheben.
mail-action-help-read = Ändern, ob diese Nachricht als gelesen markiert ist. Sie können die Änderung rückgängig machen.
mail-action-help-flag = Die Markierung zur Nachverfolgung setzen oder entfernen.
mail-action-help-archive = Diese Nachricht in den konfigurierten Archivordner verschieben.
mail-action-help-delete = Diese Nachricht in den Papierkorb verschieben. Sie können das Verschieben rückgängig machen.
mail-action-help-spam = Diese Nachricht als Junk markieren und in den konfigurierten Spam-Ordner verschieben.
mail-action-help-export = Die Exportseite öffnen, um diese Nachricht in einem Format Ihrer Wahl herunterzuladen.
mail-action-help-sweep = Aktionen für weitere Nachrichten dieses Absenders prüfen, bevor sie angewendet werden.
mail-empty-trash = Papierkorb leeren
mail-trash-help = Nachrichten im Papierkorb prüfen und endgültig löschen. Dies kann nicht rückgängig gemacht werden.
mail-trash-preparing = Eine Momentaufnahme der Nachrichten im Papierkorb wird vorbereitet …
mail-trash-prepare-failed = Die Papierkorb-Vorschau konnte nicht vorbereitet werden. Prüfen Sie Ihre Verbindung und öffnen Sie sie erneut. Es wurden keine Nachrichten gelöscht.
mail-trash-confirm = { $count } Nachrichten endgültig aus dem Papierkorb löschen? Dies kann nicht rückgängig gemacht werden. Nach dieser Vorschau eingehende Nachrichten sind ausgeschlossen; aus dem Papierkorb verschobene Nachrichten werden übersprungen.
mail-trash-limit = Dieser Vorgang umfasst bis zu { $count } Nachrichten. Weitere verbleiben im Papierkorb; prüfen Sie anschließend einen weiteren Vorgang, um sie zu entfernen.
mail-trash-permanent = Endgültig löschen
mail-trash-stop = Nach der aktuellen Anfrage anhalten
mail-trash-progress = { $deleted } gelöscht; { $skipped } Nachrichten übersprungen, die nicht mehr im Papierkorb sind; { $remaining } noch in dieser Vorschau.
mail-trash-stopped = Löschen angehalten. Bestätigt gelöscht: { $deleted }; übersprungen: { $skipped }; verbleibend: { $remaining }; Ergebnis unbekannt: { $uncertain }. Prüfen Sie den Papierkorb, bevor Sie einen weiteren Vorgang starten.
mail-links-title = Nachrichtenlinks
mail-links-review = Links prüfen ({ $count })
mail-links-help = Nachrichtenlinks werden erst geöffnet, nachdem Sie ihr Ziel hier geprüft haben. Es wird kein Ziel automatisch abgerufen.
mail-links-inert = Verwenden Sie „Links prüfen“ oberhalb der Nachricht, um dieses Ziel zu untersuchen und zu öffnen.
mail-links-limit = Die ersten 100 verschiedenen unterstützten Ziele werden aufgelistet. Weitere Links bleiben in der Nachricht inaktiv.
mail-links-destination = Zu öffnendes Ziel
mail-links-external = Dieses Ziel stammt vom Absender der Nachricht. Beim Öffnen verlassen Sie Mailwoman; prüfen Sie die Adresse, bevor Sie fortfahren.
mail-links-open = In neuem Tab öffnen
mail-links-compose = Neue Nachricht prüfen
mail-links-warning-http = Diese Adresse verwendet unverschlüsseltes HTTP.
mail-links-warning-address = Dieses Ziel verwendet eine IP-Adresse statt eines Domainnamens.
mail-links-warning-international = Diese Domain enthält einen internationalisierten Namen. Prüfen Sie die ASCII-Schreibweise sorgfältig.
mail-links-warning-mismatch = Die sichtbare Webadresse nennt einen anderen Host als das tatsächliche Ziel.
mail-links-warning-direction = Der Linktext enthält Zeichen zur Richtungsänderung, die seine Adresse verschleiern können.
mail-outbox-uncertain = Zustellstatus unbekannt
mail-outbox-uncertain-note = Der Versand wurde begonnen, die Zustellung konnte aber nicht bestätigt werden. Diese Nachricht wird nicht automatisch erneut gesendet. Prüfen Sie „Gesendet“ oder fragen Sie beim Empfänger nach, bevor Sie erneut senden.
mail-conversation-title = Unterhaltung anzeigen
mail-conversation-help = Verfügbare Nachrichten dieser Unterhaltung in diesem Konto über alle Ordner hinweg durchsuchen. Das Öffnen einer Nachricht ändert Ihre Ordnerauswahl nicht.
mail-conversation-count = { $loaded } von { $total } Nachrichten der Unterhaltung werden angezeigt.
mail-conversation-other-folder = Anderer Ordner
mail-conversation-loading = Unterhaltung wird geladen …
mail-conversation-missing = { $count } Nachricht(en) nicht mehr verfügbar.
mail-conversation-error = Die Unterhaltung konnte nicht geladen werden. Versuchen Sie es erneut.
mail-conversation-more = Weitere Nachrichten laden

# -- Machine translation (2026-11): added to match en ---------------------
mail-conversation-limit = Diese Ansicht ist auf { $count } Nachrichten begrenzt.
mail-compose-request-delivery = Zustellbenachrichtigungen anfordern
mail-compose-request-delivery-hint = Empfangende Mailserver bitten, Zustellung, Fehler oder Verzögerung zu melden. Dies fordert nicht an und belegt nicht, dass jemand die Nachricht gelesen hat. Zurückgesendete Berichte enthalten Kopfzeilen, nicht den vollständigen Text.
mail-compose-delivery-unavailable = Zustellbenachrichtigungen sind für dieses Konto nicht verfügbar oder von Ihrer Administration deaktiviert. Eine ausdrücklich angeforderte Benachrichtigung wird nicht stillschweigend entfernt.
mail-compose-account-changed = Dieser Entwurf gehört zu einer anderen Anmeldung. Melden Sie sich beim richtigen Konto an und öffnen Sie den Entwurf erneut.
mail-context-open-message = Nachricht öffnen
mail-context-open-folder = Ordner öffnen
mail-context-download = Nachricht herunterladen
mail-context-refresh = Ordner aktualisieren
mail-context-folder-settings = Ordnereinstellungen
mail-context-failed = Die Aktion konnte nicht abgeschlossen werden. Versuchen Sie es erneut.
mail-sharing-unavailable = Die Ordnerfreigabe ist nicht verfügbar oder Sie haben keine Berechtigung mehr, sie zu verwalten.
mail-trash-cancelling = Anhalten nach der aktuellen Anfrage. Bestätigt gelöscht: { $deleted }; übersprungen: { $skipped }; verbleibend: { $remaining }. Bereits akzeptierte Löschungen können nicht rückgängig gemacht werden.
mail-trash-already-running = Das Löschen des Papierkorbs läuft bereits. Verwenden Sie die Fortschrittsmeldung, um weitere Stapel zu stoppen.
mail-context-search-sender = Nach diesem Absender suchen
mail-context-search-subject = Nach demselben Betreff suchen
mail-drag-too-many = Ziehen Sie höchstens { $count } Nachrichten gleichzeitig. Wählen Sie vor dem Verschieben eine kleinere Gruppe aus.
mail-subtab-reorder-help = Ziehen Sie Tabs, um sie neu anzuordnen, oder drücken Sie Strg+Umschalt+Links bzw. Rechts auf dem aktiven Tab.
mail-compose-ribbon = Editor-Menüband
mail-compose-ribbon-format = Text formatieren
mail-compose-ribbon-insert = Einfügen
mail-compose-font = Schriftart
mail-compose-font-size = Schriftgröße
mail-compose-font-inherit = Standard
mail-compose-undo = Rückgängig
mail-compose-redo = Wiederholen
mail-compose-undo-tip = Rückgängig ({ $shortcut })
mail-compose-redo-tip = Wiederholen ({ $shortcut })
mail-workspace-limit = Schließen Sie einen Tab, bevor Sie einen weiteren öffnen. Es können bis zu 32 Tabs geöffnet bleiben.
mail-compose-private-storage-unavailable = Die lokale Entwurfswiederherstellung ist für diese Sitzung nicht verfügbar. Lassen Sie diesen Editor geöffnet, bis Sie Ihre Nachricht senden oder kopieren.
mail-offline-storage-unavailable = Offline-Änderungen sind erst verfügbar, wenn diese Sitzung die Speicherberechtigung überprüft hat. Verbinden Sie sich erneut und melden Sie sich erneut an.
mail-draft-status-idle = Noch nicht im Konto gespeichert
mail-draft-status-pending = Nicht gespeicherte Änderungen
mail-draft-status-saving = Speichern in Entwürfen …
mail-draft-status-saved = In den Entwürfen des Kontos gespeichert
mail-draft-status-error = Der Entwurf wurde nicht gespeichert. Ihre lokale Wiederherstellungskopie bleibt verfügbar.
mail-draft-status-uncertain = Die Speicherbestätigung fehlt. Prüfen Sie die Entwürfe, bevor Sie eine weitere Kopie erstellen; das automatische Speichern ist angehalten.
mail-draft-status-conflict = Dieser Entwurf wurde auf einem anderen Gerät geändert. Öffnen Sie die Kontokopie erneut, bevor Sie fortfahren.
mail-draft-status-unsupported = Dieses Konto kann das Speichern in seinem Entwurfsordner derzeit nicht bestätigen.
mail-draft-protected = Verschlüsselte oder entschlüsselte zitierte Inhalte werden nicht als Klartext gespeichert. Ein früherer Kontoentwurf bleibt bestehen, bis Sie ihn ausdrücklich verwerfen.
mail-draft-save-now = In Entwürfen speichern
mail-draft-save-failed = Dieser Entwurf konnte nicht in Ihrem Konto gespeichert werden. Prüfen Sie den Editor, bevor Sie ihn schließen.
mail-draft-uncertain = Der Entwurf wurde möglicherweise gespeichert. Automatische Wiederholungen sind angehalten; prüfen Sie Ihren Entwurfsordner.
mail-draft-conflict = Ein anderes Gerät hat diesen Entwurf geändert. Ihre Änderungen haben dessen Kopie nicht überschrieben.
mail-draft-cleanup-pending = Der neue Entwurf wurde gespeichert, aber seine vorherige Kopie konnte nicht entfernt werden. Prüfen Sie die Entwürfe, bevor Sie erneut speichern.
mail-draft-account-list = Kontoentwürfe
mail-draft-loading = Kontoentwürfe werden geladen …
mail-draft-close-recovery = Schließen und nur lokale Wiederherstellung behalten
mail-draft-discard-account = Verwerfen
mail-draft-discard-pending = Das Entfernen des Entwurfs konnte nicht bestätigt werden. Sein Kontoeintrag bleibt bestehen; wiederholen Sie Verwerfen, nachdem Sie den Entwurfsordner geprüft haben.
mail-draft-limit = Dieses Konto hat bereits 100 gespeicherte Entwürfe. Verwerfen Sie einen nicht benötigten Kontoentwurf, bevor Sie einen neuen speichern.
mail-draft-quota-help = Es werden bis zu 100 Kontoentwürfe aufbewahrt. Verwerfen entfernt die bestätigte Kopie auf dem Server, bevor der Platz freigegeben wird.
mail-draft-discard-confirm = Entfernen des Entwurfs bestätigen
mail-draft-discard-confirm-detail = „{ $subject }“ endgültig aus den Kontoentwürfen entfernen? Wenn Sie stattdessen den Editor schließen, bleibt der Entwurf erhalten.
mail-draft-discard-title = Diesen Entwurf verwerfen?
mail-draft-keep-editing = Weiter bearbeiten
mail-draft-discard-action = Verwerfen
mail-draft-close-title = Nicht gespeicherte Änderungen verwerfen?
mail-draft-close-protected = Diese Nachricht enthält verschlüsselte oder entschlüsselte Inhalte und wird daher nicht als Entwurf gespeichert. Beim Schließen werden diese Änderungen verworfen. Ein früherer Kontoentwurf, falls vorhanden, bleibt in den Entwürfen.
mail-draft-close-unstored = In dieser Sitzung können keine Entwürfe gespeichert werden; beim Schließen wird diese Nachricht daher verworfen.
mail-workspace-view-mode = Ansichtsanordnung
mail-workspace-tabs-help = Eine aktive Ansicht anzeigen und die anderen Tabs geöffnet lassen.
mail-workspace-side-by-side-help = Zwei ausgewählte Ansichten nebeneinander anzeigen. Ziehen Sie einen Tab auf eine Ansicht, um sie zu ersetzen.
mail-workspace-stack-help = Zwei ausgewählte Ansichten übereinander anzeigen. Ziehen Sie einen Tab auf eine Ansicht, um sie zu ersetzen.
mail-workspace-masonry-side-by-side = Mauerwerk nebeneinander
mail-workspace-masonry-side-by-side-help = Alle geöffneten Ansichten in versetzten Spalten anordnen. Auf kleinen Bildschirmen wird der aktive Tab angezeigt.
mail-workspace-masonry-stack = Mauerwerk untereinander
mail-workspace-masonry-stack-help = Alle geöffneten Ansichten in versetzten Zeilen anordnen. Auf kleinen Bildschirmen wird der aktive Tab angezeigt.
mail-workspace-view-first = Erste Ansicht
mail-workspace-view-second = Zweite Ansicht
mail-workspace-view-empty = Ziehen Sie einen geöffneten Tab hierher, um diese Ansicht festzulegen.
mail-popup-blocked = Erlauben Sie Pop-up-Fenster für Mailwoman, um diesen Tab zu lösen. Ihr Tab bleibt hier.
mail-popup-unavailable = Das abgelöste Fenster konnte nicht geladen werden. Ihr Tab und nicht gespeicherte Änderungen bleiben im Hauptfenster. Versuchen Sie es nach dem Aktualisieren der App erneut.
mail-popup-reattach = Wieder im Hauptfenster anzeigen
mail-popup-action-reattached = Zum Abschluss dieser Aktion ins Hauptfenster zurückgekehrt.
mail-popup-detached = Dieser Tab ist in einem separaten Fenster geöffnet. Wenn Sie das Fenster schließen, kehrt er hierher zurück.
mail-compose-from-custom = Andere autorisierte Adresse …
mail-compose-from-alias = Absenderadresse für diese Nachricht
mail-compose-from-alias-help = Verwenden Sie einen Alias, mit dem Sie senden dürfen. Dies ändert nur diesen Entwurf; die Berechtigung wird vor dem Senden geprüft.
mail-compose-from-refused = Diese Adresse ist für das Senden von diesem Konto nicht autorisiert.
mail-compose-from-unavailable = Diese Absenderadresse konnte nicht überprüft werden. Ihre Nachricht wurde nicht gesendet.
mail-compose-from-invalid = Geben Sie eine gültige E-Mail-Adresse für den Absender ein.
mail-compose-from-toggle = Absender für diese Nachricht ein- oder ausblenden
mail-compose-ribbon-security = Sicherheit
mail-compose-ribbon-delivery = Zustellung
mail-compose-ribbon-recipients = Empfänger
mail-compose-ribbon-message-type = Nachrichtentyp
mail-compose-recall-help = Nachrichten abbrechen, die noch für Rückgängig oder zeitgesteuerte Zustellung zurückgehalten werden. Bereits zugestellte Nachrichten können nicht zurückgerufen werden.
mail-compose-recall-loading = Nachrichten in der Warteschlange werden geprüft …
mail-compose-recall-empty = Es sind keine Nachrichten in der Warteschlange zum Abbrechen verfügbar.
mail-compose-recall-failed = Der Status der Warteschlange konnte nicht bestätigt werden. Schließen Sie diesen Bereich und öffnen Sie ihn erneut, um ihn zu aktualisieren, bevor Sie eine weitere Aktion ausführen.
mail-compose-importance = Wichtigkeit
mail-compose-importance-high = Hoch
mail-compose-importance-normal = Normal
mail-compose-importance-low = Niedrig
mail-insert-account-title = Kontoelement einfügen
mail-insert-account-help = Einen Kalenderexport, eine Visitenkarte oder eine Originalnachricht aus diesem Konto anhängen.
mail-insert-account-close = Auswahl für Kontoelemente schließen
mail-insert-account-kind = Elementtyp
mail-insert-account-event = Kalendertermin (.ics)
mail-insert-account-calendar = Kalender (.ics)
mail-insert-account-contact = Kontakt-Visitenkarte (.vcf)
mail-insert-account-message = Originalnachricht (.eml)
mail-insert-account-search = Aufgelistete Elemente filtern
mail-insert-account-selection = Element auswählen
mail-insert-account-scope = Es werden nur Elemente des aktuellen Kontos aufgelistet. Originalnachrichten unterliegen der Exportrichtlinie des Servers.
mail-insert-account-limited = Es werden bis zu 100 Elemente angezeigt. Das Filtern durchsucht nur diese Liste.
mail-insert-account-unavailable = Keine unterstützten Kontoexporte verfügbar.
mail-insert-account-empty = Keine passenden Elemente.
mail-insert-account-attach = Ausgewähltes Element anhängen
mail-insert-account-loading = Kontoelement wird geladen …
mail-insert-account-error = Das Element konnte nicht angehängt werden. Prüfen Sie Ihren Zugriff und die Exportrichtlinie und versuchen Sie es erneut. Große Kalender erfordern möglicherweise Exporte einzelner Termine.
mail-compose-html-source = HTML-Quelltext
mail-editor-clear-format = Formatierung entfernen
mail-compose-format-html = HTML
mail-compose-html-visual = Visueller Editor
mail-compose-html-help = HTML direkt bearbeiten. Es wird nur unterstützte E-Mail-Formatierung gespeichert und gesendet; Skripte, externe Bilder und unsicheres Markup werden entfernt.
mail-compose-html-processing = Sicheres HTML wird vorbereitet …
mail-compose-html-failed = Das HTML konnte nicht vorbereitet werden. Bearbeiten Sie es oder versuchen Sie es erneut, bevor Sie speichern oder senden.
mail-compose-check-names = Namen prüfen
mail-compose-check-names-help = Exakte Namen aus Ihren geladenen Kontakten auflösen und das Adressformat prüfen. Die Zustellung an das Postfach wird dadurch nicht überprüft.
mail-compose-names-unresolved = Wählen Sie für diese nicht aufgelösten oder mehrdeutigen Namen eine vollständige E-Mail-Adresse: { $names }
mail-compose-names-checked = Empfängerformat geprüft; { $count } Kontaktname(n) aufgelöst. Die Zustellung an das Postfach wurde nicht überprüft.
mail-compose-ribbon-table = Tabelle
mail-compose-group-clipboard = Zwischenablage
mail-compose-group-paragraph = Absatz
mail-compose-group-editing = Bearbeiten
mail-compose-group-view = Ansicht
mail-compose-dark-background = Dunkler Hintergrund
mail-compose-dark-background-help = Die Nachricht während des Schreibens auf dunklem Hintergrund anzeigen. Nur Ihre Anzeige ändert sich: Die Nachricht wird ohne Hintergrundfarbe gesendet.
mail-compose-group-assist = Schreibassistent
mail-editor-counts = Editor-Statistik
mail-editor-words = { $count ->
    [one] { $count } Wort
   *[other] { $count } Wörter
}
mail-editor-paragraphs = { $count ->
    [one] { $count } Absatz
   *[other] { $count } Absätze
}
mail-editor-characters = { $count ->
    [one] { $count } Zeichen
   *[other] { $count } Zeichen
}
mail-editor-selection-required = Wählen Sie geeigneten Text oder Tabellenzellen aus, bevor Sie diese Aktion verwenden.
mail-editor-clipboard-unavailable = Der Zugriff auf die Zwischenablage war nicht möglich. Verwenden Sie die Tastenkürzel Ihres Browsers zum Kopieren, Ausschneiden oder Einfügen.
mail-editor-selection-changed = Die Auswahl hat sich geändert. Wählen Sie den gewünschten Text aus und versuchen Sie es erneut.
mail-editor-clipboard = Zwischenablage und Formatierung
mail-editor-copy = Kopieren
mail-editor-cut = Ausschneiden
