# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = KI-Assistent
assist-heading = KI-Assistent
assist-transcript-label = Unterhaltung mit dem KI-Assistenten
assist-proposed-action = Vorgeschlagene Aktion
assist-outbox-note = Dadurch würde eine Nachricht in Ihren Postausgang gelegt. Es wird nichts gesendet, bis Sie es dort bestätigen.
assist-review-outbox = Im Postausgang prüfen
assist-review = Prüfen
assist-empty = Bitten Sie den KI-Assistenten, etwas zusammenzufassen, zu entwerfen oder zu finden. Er kann Aktionen vorschlagen, aber Sie bestätigen sie immer selbst.
assist-error = Der KI-Assistent konnte gerade nicht antworten.
assist-input-label = Nachricht an den KI-Assistenten
assist-input-placeholder = Den KI-Assistenten fragen…
assist-thinking = Denkt nach…
assist-ask = Fragen

# -- Composer tools ---------------------------------------------------------
assist-composer-label = Schreibhilfe
assist-composer-toolbar = Editor-Werkzeuge
assist-tool-grammar = Grammatik korrigieren
assist-tool-rewrite = Umschreiben
assist-tool-tone = Ton anpassen
assist-tool-translate = Übersetzen
assist-toolarg-tone = Ton
assist-toolarg-translate = Sprache
assist-tone-neutral = Neutral
assist-tone-friendly = Freundlich
assist-tone-formal = Förmlich
assist-tone-concise = Knapp
assist-tone-assertive = Bestimmt
assist-busy = { $label }…
assist-composer-empty-error = Schreiben Sie zuerst etwas.
assist-composer-error = Der KI-Assistent konnte das gerade nicht erledigen.
assist-suggested-edit = Vorgeschlagene Änderung
assist-apply-draft = In Entwurf übernehmen
assist-discard = Verwerfen

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = Vorgeschlagene Labels
assist-apply-auto = Automatisch anwenden
assist-apply-label = { $label } anwenden
assist-remove-label = { $label } entfernen
assist-apply = Anwenden
assist-undo = Rückgängig

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = Zum Diktieren gedrückt halten
assist-dictate-stop = Diktat beenden
assist-dictate-hold-text = Zum Diktieren gedrückt halten
assist-dictate-listening = ● Hört zu…
assist-dictate-endpoint-note = Audio wird von Ihrem KI-Assistenten-Endpunkt transkribiert.
assist-dictate-err = Diktierfehler.
assist-transcribe-err = Audio konnte nicht transkribiert werden.
assist-mic-err = Mikrofon nicht verfügbar.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = Semantische Suche
assist-semantic-note = Der Suchtext wird an Ihren KI-Assistenten-Endpunkt gesendet, um nach Bedeutung zu sortieren.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = Was dieses Gerät verlassen kann
assist-left-1 = der ausgewählte Nachrichtentext (Betreff + Inhalt) für die gewählte Funktion
assist-left-2 = der Endpunkt-Host, an den er gesendet wurde
assist-left-3 = nie: Ende-zu-Ende-verschlüsselte Inhalte (standardmäßig ausgeschlossen)
assist-left-4 = nie: Anhänge (standardmäßig ausgeschlossen)
assist-left-5 = nie: Ihre Zugangsdaten oder andere Konten
assist-disclosure-off = Der KI-Assistent ist aus. Kein Nachrichteninhalt verlässt dieses Gerät.
assist-disclosure-sentence = Wenn Sie ein Werkzeug des KI-Assistenten verwenden, wird der ausgewählte Nachrichtentext über einen Proxy an { $host } weitergeleitet. { $excl } Senden ist nie automatisiert — Sie bestätigen immer, bevor etwas Ihren Postausgang verlässt.
assist-disclosure-excl-one = Er sendet nie { $a }.
assist-disclosure-excl-two = Er sendet nie { $a } oder { $b }.
assist-disclosure-admin-allowed = Ihr Admin hat erlaubt, dass verschlüsselte Inhalte und Anhänge gesendet werden.
assist-excluded-e2ee = Ende-zu-Ende-verschlüsselte Inhalte
assist-excluded-attachments = Anhänge

# -- Admin → AI assistant (deployment configuration) ------------------------------
assist-admin-title = KI-Assistent
assist-admin-intro = Der KI-Assistent sendet von Benutzern ausgewählten Nachrichtentext an den hier konfigurierten KI-Endpunkt. Er kann keine E-Mails senden, löschen oder annehmen. Ende-zu-Ende-verschlüsselte Inhalte und Anhänge werden zurückgehalten, sofern Sie sie unten nicht erlauben.
assist-admin-load-unreadable = Die gespeicherte Konfiguration des KI-Assistenten kann nicht gelesen werden: Ihr API-Schlüssel lässt sich mit dem Schlüssel dieses Servers (MW_SERVER_KEY) nicht öffnen. Sie wurde unter einem anderen Schlüssel gespeichert oder wurde verändert. Der KI-Assistent ist aus. Füllen Sie das Formular einschließlich des API-Schlüssels aus und speichern Sie, um sie zu ersetzen.
assist-admin-load-error = Die Konfiguration des KI-Assistenten konnte nicht geladen werden. Speichern ist deaktiviert, damit die gespeicherte Konfiguration nicht überschrieben wird.
assist-admin-status-running = Der KI-Assistent läuft. Anfragen gehen an { $host }.
assist-admin-status-running-pending = Der KI-Assistent läuft mit der Konfiguration, mit der der Server gestartet wurde. Gespeicherte Änderungen gelten nach dem Neustart des Servers.
assist-admin-status-off = Der KI-Assistent ist aus. Jede Anfrage an den KI-Assistenten wird abgewiesen.
assist-admin-status-pending = Der KI-Assistent läuft nicht. Die gespeicherte Konfiguration gilt nach dem Neustart des Servers.
assist-admin-status-local-program-not-allowed = Der KI-Assistent läuft nicht. Das konfigurierte Programm { $program } ist auf dem Server nicht in MW_ASSIST_LOCAL_PROGRAMS aufgeführt, daher hat der Server es nicht gestartet.
assist-admin-status-no-endpoint = Der KI-Assistent ist eingeschaltet, läuft aber nicht, weil kein nutzbarer Endpunkt konfiguriert ist.
assist-admin-stop = KI-Assistent jetzt stoppen
assist-admin-stop-note = Das Stoppen gilt sofort für alle Benutzer und bleibt über Neustarts hinweg erhalten.
assist-admin-resume = KI-Assistent einschalten
assist-admin-resume-note = Wurde der Server mit ausgeschaltetem KI-Assistenten gestartet, gilt das Einschalten nach dem Neustart des Servers.
assist-admin-kill-error = Es konnte nicht geändert werden, ob der KI-Assistent eingeschaltet ist.
assist-admin-endpoint = Endpunkt
assist-admin-endpoint-note = Der Server kontaktiert diesen Endpunkt; Browser tun das nie. Änderungen gelten nach dem Neustart des Servers.
assist-admin-kind = Endpunkttyp
assist-admin-kind-none = Keiner
assist-admin-kind-open-ai-compatible = OpenAI-kompatibel
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = Lokales Programm
assist-admin-local-program-enable = Damit der KI-Assistent ein Programm auf diesem Server ausführt, setzen Sie MW_ASSIST_LOCAL_PROGRAMS dort, wo der Server gestartet wird, auf den absoluten Pfad des Programms (mehrere Pfade durch Kommas trennen), und starten Sie dann den Server neu.
assist-admin-local-program-note = Der Server startet dieses Programm für jede Anfrage an den KI-Assistenten und übergibt ihm die Anfrage über die Standardeingabe, einschließlich des vom Benutzer ausgewählten Nachrichtentexts. Die Antwort liest er aus der Standardausgabe des Programms. Nur Programme, die auf dem Server in MW_ASSIST_LOCAL_PROGRAMS aufgeführt sind, können gewählt werden.
assist-admin-local-program = Programm
assist-admin-local-program-unlisted = { $program } (nicht aufgeführt)
assist-admin-local-program-unlisted-note = Dieses Programm ist nicht in MW_ASSIST_LOCAL_PROGRAMS aufgeführt, daher führt der Server es nicht aus und nimmt es beim Speichern nicht an. Wählen Sie ein aufgeführtes Programm oder einen anderen Endpunkttyp.
assist-admin-local-program-required = Wählen Sie das auszuführende Programm.
assist-admin-local-args = Argumente, eines pro Zeile
assist-admin-local-args-note = Jede Zeile wird dem Programm als ein Argument übergeben. Es wird keine Shell verwendet.
assist-admin-base-url = Basis-URL
assist-admin-base-url-required = Geben Sie die Basis-URL des Endpunkts ein, beginnend mit http:// oder https://.
assist-admin-api-key = API-Schlüssel
assist-admin-api-key-stored = Ein Schlüssel ist gespeichert. Der Server zeigt ihn nicht erneut an.
assist-admin-api-key-replace = API-Schlüssel ersetzen
assist-admin-api-key-replace-placeholder = Leer lassen, um den gespeicherten Schlüssel zu behalten
assist-admin-api-key-remove = Gespeicherten Schlüssel entfernen
assist-admin-api-key-keep = Gespeicherten Schlüssel behalten
assist-admin-api-key-remove-pending = Der gespeicherte Schlüssel wird beim Speichern entfernt.
assist-admin-chat-model = Chat-Modell
assist-admin-embed-model = Embedding-Modell
assist-admin-stt-model = Sprache-zu-Text-Modell
assist-admin-model = Modell
assist-admin-anthropic-version = API-Version
assist-admin-max-tokens = Maximale Antwort-Tokens
assist-admin-default-placeholder = Server-Standard
assist-admin-grants = Funktionen
assist-admin-grants-note = Nur die hier angekreuzten Funktionen können verwendet werden.
assist-admin-cap-summarize = Zusammenfassen
assist-admin-cap-draft = Entwerfen und umschreiben
assist-admin-cap-grammar = Grammatik
assist-admin-cap-dictation = Diktat
assist-admin-cap-search-semantic = Semantische Suche
assist-admin-cap-auto-tag = Automatische Labels
assist-admin-cap-recap = Rückblick
assist-admin-cap-assistant = Chat mit dem KI-Assistenten
assist-admin-ceilings = Datengrenzen
assist-admin-ceilings-note = Diese Grenzen gelten für jede Anfrage, unabhängig von der Funktion.
assist-admin-accounts = Konto-IDs, deren E-Mails gesendet werden dürfen, eine pro Zeile
assist-admin-accounts-note = Ist kein Konto aufgeführt, wird kein E-Mail-Inhalt an den Endpunkt gesendet.
assist-admin-folders = Ordner-IDs, auf die das Senden beschränkt wird, eine pro Zeile
assist-admin-folders-note = Ist kein Ordner aufgeführt, ist jeder Ordner der aufgeführten Konten erlaubt.
assist-admin-allow-e2ee = Senden Ende-zu-Ende-verschlüsselter Inhalte erlauben
assist-admin-allow-attachments = Senden von Anhängen erlauben
assist-admin-save = Konfiguration speichern
assist-admin-saved = Gespeichert.
assist-admin-saved-pending = Gespeichert. Die Änderung gilt nach dem Neustart des Servers.
assist-admin-save-error = Die Konfiguration wurde nicht gespeichert.
assist-admin-save-not-allowed = Die Konfiguration wurde nicht gespeichert, weil der Server dieses Programm nicht erlaubt. Nur Programme, die in MW_ASSIST_LOCAL_PROGRAMS aufgeführt sind, können verwendet werden.
