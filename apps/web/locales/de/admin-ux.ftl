# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Schließen

# -- Page states (UX11) ---------------------------------------------------------
admin-ux-state-unavailable = Nicht verfügbar — { $reason }
admin-ux-unavailable-default = Dieser Server bietet das noch nicht an.
admin-ux-state-code = Fehlercode: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Öffnen
admin-ux-list-back = Zurück zur Liste

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Keine ungespeicherten Änderungen
admin-ux-form-changes =
    { $count ->
        [one] 1 ungespeicherte Änderung
       *[other] { $count } ungespeicherte Änderungen
    }
admin-ux-form-discard = Änderungen verwerfen
admin-ux-form-saving = Wird gespeichert…
admin-ux-form-saved =
    { $count ->
        [one] 1 Änderung gespeichert.
       *[other] { $count } Änderungen gespeichert.
    }
admin-ux-form-fix =
    { $count ->
        [one] Es wurde nichts gespeichert. Korrigieren Sie das unten markierte Feld.
       *[other] Es wurde nichts gespeichert. Korrigieren Sie die { $count } unten markierten Felder.
    }
admin-ux-learn-more = Mehr erfahren
admin-ux-learn-more-field = über { $label }
admin-ux-propagation-hot = Wird innerhalb von 30 Sekunden in jedem Serverprozess wirksam.
admin-ux-propagation-restart = Wird nach einem Neustart des Servers wirksam.
admin-ux-secret-set = Ein Wert ist gespeichert. Geben Sie einen neuen ein, um ihn zu ersetzen.
admin-ux-secret-unset = Es ist kein Wert gespeichert.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Geben Sie einen Wert ein.
admin-ux-err-number = Geben Sie eine Zahl ein.
admin-ux-err-whole = Geben Sie eine ganze Zahl ein.
admin-ux-err-min = Geben Sie { $min } oder mehr ein.
admin-ux-err-max = Geben Sie { $max } oder weniger ein.
admin-ux-err-max-length = Verwenden Sie höchstens { $max } Zeichen.
admin-ux-err-url = Geben Sie eine vollständige Adresse ein, die mit https:// oder http:// beginnt.
admin-ux-err-email = Geben Sie eine E-Mail-Adresse wie name@example.org ein.
admin-ux-err-pattern = Dieser Wert hat nicht die erwartete Form.
admin-ux-err-option = Wählen Sie eine der aufgeführten Optionen.
admin-ux-err-out-of-range = Der Server hat diesen Wert abgewiesen, weil er außerhalb des zulässigen Bereichs liegt.
admin-ux-err-too-long = Der Server hat diesen Wert abgewiesen, weil er zu lang ist.
admin-ux-err-invalid = Der Server hat diesen Wert abgewiesen. Prüfen Sie ihn und speichern Sie erneut.
admin-ux-err-pinned = Dieser Wert ist in der Umgebung des Servers festgelegt und kann hier nicht geändert werden.
admin-ux-err-locked = Dieser Wert ist auf einer übergeordneten Ebene gesperrt und kann hier nicht geändert werden.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Standard
admin-ux-source-stored = Gespeichert
admin-ux-source-env-pin = Festgelegt durch { $name }
admin-ux-source-env-fallback = Ersatzwert aus { $name }
admin-ux-source-build = Beim Build festgelegt
admin-ux-source-locked = Gesperrt durch { $scope }
admin-ux-source-read-only = Schreibgeschützt
admin-ux-source-restart = Neustart nötig
admin-ux-source-env-pin-reason = Die Umgebung des Servers legt diesen Wert mit { $name } fest. Ändern Sie ihn dort und starten Sie den Server neu.
admin-ux-source-build-reason = Dieser Wert wird beim Build des Servers festgelegt.
admin-ux-source-locked-reason = Eine Sperre auf der Ebene { $scope } legt diesen Wert fest. Heben Sie die Sperre dort auf, um ihn zu ändern.
admin-ux-source-read-only-reason = Die Konsole zeigt diesen Wert an, kann ihn aber nicht ändern. Die Dokumentation beschreibt, wo er festgelegt wird.

# -- History (UX8) ------------------------------------------------------------
admin-ux-history-changed-by-at = Zuletzt geändert von { $who } am { $time }
admin-ux-history-changed-by = Zuletzt geändert von { $who }
admin-ux-history-changed-at = Zuletzt geändert am { $time }
admin-ux-history-never = Nicht über die Konsole geändert
admin-ux-history-link = Verlauf

# -- Revert to default (UX7) --------------------------------------------------
admin-ux-revert-default-is = Standard: { $value }
admin-ux-revert = Auf Standard zurücksetzen
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Auf den Standardwert ({ $value }) zurückgesetzt.

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } hat dies am { $time } geändert, während Sie es bearbeitet haben.
admin-ux-stale-by = { $who } hat dies geändert, während Sie es bearbeitet haben.
admin-ux-stale-unknown = Jemand anderes hat dies geändert, während Sie es bearbeitet haben.
admin-ux-stale-choice = Laden Sie neu, um die andere Fassung zu sehen und Ihre Änderungen zu verwerfen, oder behalten Sie Ihre und speichern Sie sie über die andere.
admin-ux-stale-reload = Neu laden
admin-ux-stale-keep = Meine behalten

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Auswirkungen dieser Änderung werden geprüft…
admin-ux-impact-heading = Diese Änderung betrifft
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] und 1 weiteres
       *[other] und { $count } weitere
    }
admin-ux-impact-none = Sonst ändert sich nichts.
admin-ux-impact-unavailable = Vorschau der Auswirkungen nicht verfügbar — der Server hat nicht gemeldet, was diese Änderung betrifft.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filter
admin-ux-stream-pause = Anhalten
admin-ux-stream-resume = Fortsetzen
admin-ux-stream-clear = Leeren
admin-ux-stream-announce = Neue Einträge vorlesen
admin-ux-stream-connecting = Verbindung wird hergestellt…
admin-ux-stream-live = Live
admin-ux-stream-ended = Beendet
admin-ux-stream-lost = Verbindung verloren
admin-ux-stream-reconnect = Neu verbinden
admin-ux-stream-paused =
    { $count ->
        [0] Angehalten
        [one] Angehalten — 1 neuer Eintrag zurückgehalten
       *[other] Angehalten — { $count } neue Einträge zurückgehalten
    }
admin-ux-stream-no-match = Kein Eintrag passt zum Filter.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Feld

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Darstellung
admin-ux-appearance-title = Darstellung der Konsole
admin-ux-appearance-note = Diese Einstellungen ändern nur die Admin-Konsole, in diesem Browser.
admin-ux-appearance-following = Die Konsole folgt dem Webmail-Design dieses Browsers, bis Sie hier etwas auswählen. Diese Einstellungen ändern nur die Admin-Konsole, in diesem Browser.
admin-ux-appearance-mode = Modus
admin-ux-appearance-mode-light = Hell
admin-ux-appearance-mode-dark = Dunkel
admin-ux-appearance-mode-system = Wie das System
admin-ux-appearance-theme = Design
admin-ux-appearance-accent = Akzentfarbe
admin-ux-appearance-light = Hell
admin-ux-appearance-dark = Dunkel
admin-ux-appearance-reset = Webmail-Design verwenden
