# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: privacy. Source: ../en/privacy.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-title = Datenschutz
privacy-help = Externe Bilder können verraten, wann Sie eine Nachricht öffnen. Legen Sie fest, wann sie geladen werden und ob der Server sie für Sie abruft.
privacy-remote-mode = Externe Inhalte
privacy-mode-block = Externe Inhalte blockieren
privacy-mode-ask = Vor dem Laden über den Server fragen
privacy-mode-proxy = Über den Server laden
privacy-mode-direct = Direkt von den Servern des Absenders laden
privacy-trackers = Tracking-Bilder und Beacons blockieren
privacy-clean-links = Tracking-Parameter aus Links entfernen
privacy-direct-unavailable = Direktes Laden erfordert die Erlaubnis Ihrer Administration. Dabei wird Ihre Netzwerkadresse dem Bildhost offengelegt.
privacy-direct-native = Direktes Laden ist in dieser nativen App nicht verfügbar. Bilder können über den Server geladen werden.
privacy-inherit = Geerbte Einstellung verwenden
privacy-content-failed = Einige Bilder konnten nicht autorisiert werden. Die Nachricht bleibt lesbar, diese Bilder bleiben blockiert.
privacy-frame-failed = Die isolierte Bildansicht konnte nicht geladen werden. Laden Sie die Nachricht neu, um es erneut zu versuchen.
privacy-download-file = Über den Server herunterladen
privacy-file-failed = Diese Datei konnte nicht über den Server heruntergeladen werden.
privacy-admin-title = Datenschutz und externe Inhalte
privacy-admin-help = Legen Sie Standardwerte, Sperren und Limits über die Einstellungsrichtlinie fest. Die Diagnosezähler betreffen nur diesen Serverprozess.
privacy-cache-entries = Zwischengespeicherte Bilder
privacy-cache-bytes = Cache-Größe (Bytes)
privacy-replaced-trackers = Ersetzte Tracking-Pixel
privacy-edit-setting = Richtlinie bearbeiten
privacy-outbound-title = Ausgehende Verbindungen
privacy-fail-closed = Eine fehlgeschlagene konfigurierte Route blockiert die Anfrage. Es erfolgt nie ein Rückfall auf eine direkte Verbindung.
privacy-linked-files = In dieser Nachricht verlinkte Dateien
privacy-allow-direct = Direktes Laden von Bildern erlauben
privacy-max-bytes = Maximale Bildgröße (Bytes)
privacy-cache-limit = Maximale Cache-Größe (Bytes)
privacy-rate = Anfragen pro Minute
privacy-on = Ein
privacy-off = Aus
privacy-locked = Durch Administratorrichtlinie gesperrt
privacy-process-stats = Seit dem Start dieses Serverprozesses
privacy-refresh = Aktualisieren
privacy-traffic-class = Verkehrsklasse
privacy-allowed = Erlaubt
privacy-refused = Durch Richtlinie abgelehnt
privacy-failed = Fehlgeschlagen
privacy-policy-values = Richtlinie für externe Inhalte
privacy-cache-summary = Cache und Tracking-Schutz
privacy-traffic-summary = Verarbeitung des ausgehenden Verkehrs
privacy-metric = Messwert
privacy-value = Wert
privacy-total-duration = Gesamtzeit
privacy-no-traffic = Seit dem Start dieses Prozesses wurde kein ausgehender Verkehr aufgezeichnet.
privacy-analytics-title = Optionale Nutzungszähler
privacy-analytics-help = Standardmäßig aus. Wenn Ihre Administration einen Collector aktiviert, können Sie feste Bildschirmnamen wie Mail oder Kalender teilen. Nachrichten, E-Mail-Adressen, Suchanfragen, Routendetails und Browserkennungen werden nie gesendet. Es handelt sich um Ereigniszähler, nicht um Statistiken zu eindeutigen Besuchern.
privacy-analytics-consent = Optionale Zähler zur Bildschirmnutzung für dieses Konto erlauben
privacy-analytics-signal = Das Datenschutzsignal Ihres Browsers verhindert Analysen, auch wenn die Einwilligung gespeichert ist.
privacy-load-time-title = Ladezeiten
privacy-load-time-help = Helfen Sie Ihrer Administration zu sehen, wie schnell sich Mailwoman anfühlt. Gemeldet werden nur die Millisekunden und ob die Daten aus dem Cache oder dem Netzwerk kamen, anonym und aggregiert. Ordner, Nachrichten, Absender und Betreff sind nicht enthalten.
privacy-load-time-folder = Messen, wie lange das Öffnen von Ordnern dauert
privacy-load-time-folder-help = Standardmäßig an. Meldet die Zeit von der Ordnerauswahl bis zur Anzeige der Nachrichtenliste.
privacy-load-time-message = Messen, wie lange das Öffnen von Nachrichten dauert
privacy-load-time-message-help = Standardmäßig an. Meldet die Zeit vom Öffnen einer Nachricht bis zur Anzeige von Kopfzeilen und Text.
privacy-analytics-save-error = Die Einwilligung konnte nicht synchronisiert werden. Dieser Browser pausiert neue Analyseanfragen, bis die Änderung gespeichert ist.
privacy-analytics-saving = Einwilligung wird gespeichert …
privacy-back-policies = Zurück zu den Datenschutzrichtlinien
privacy-image-toast = Eine Benachrichtigung anzeigen, während externe Bilder geladen werden
privacy-image-downloads = Manuelle Downloads externer Bilder
privacy-image-download-help = Wenn ein Bild nicht angezeigt wird, laden Sie eine geprüfte Kopie über diesen Server herunter. Der Server prüft, ob es zu dieser Nachricht gehört, und wendet weiterhin seine Datenschutz- und Sicherheitsregeln an. Clientseitig entschlüsselte Bilder sind hier nicht verfügbar.
privacy-image-download-ready = Das Bild ist bereit, der Download wurde gestartet.
privacy-image-download-failed = Das Bild konnte nicht heruntergeladen werden. Es ist möglicherweise nicht verfügbar oder durch Ihre Datenschutzrichtlinie blockiert.
privacy-strip-mailer-headers = Mailer-Kopfzeilen ausgehender Nachrichten entfernen
privacy-strip-mailer-help = Beim nativen SMTP- und EWS-Versand werden Fingerprint-Kopfzeilen des Clients auf oberster Ebene entfernt. Nachrichtentexte, angehängte Nachrichten und Berichtsdaten bleiben unverändert. Das gilt nicht für den Versand über den vorgeschalteten JMAP-Proxy oder für Kopfzeilen, die später ein Relay hinzufügt. Vorsignierte Nachrichten, die eine entfernte Kopfzeile abdecken, erfordern vor dem Senden das Deaktivieren dieser Einstellung.
privacy-frame-compatible = Kompatible Ansicht: Bilder, die direkt von anderen Websites geladen werden, bleiben aus Datenschutzgründen blockiert. Eingebettete Bilder und über den Server geladene Bilder werden unterstützt.
privacy-analytics-tag-custom = ein benutzerdefiniertes Analyseskript
privacy-analytics-tag-title = Browser-Analyse: { $provider }
privacy-analytics-tag-help = Ihre Administration bietet außerdem { $provider } an, das in dieser Seite in Ihrem Browser läuft. Anders als die obigen Zähler kann dessen Skript die Seiten sehen, die Sie in dieser App öffnen, einschließlich angezeigter Nachrichten und Adressen, Ihren Browser und Ihre IP-Adresse, und es kann eigene Kennungen setzen. Der Anbieter kann sein Skript aktualisieren, ohne Sie erneut zu fragen. Aus, solange Sie es nicht hier einschalten; beim Ausschalten wird die Seite in jedem Tab ohne das Skript neu geladen.
privacy-analytics-tag-consent = { $provider } in dieser App für dieses Konto ausführen lassen
privacy-analytics-tag-changed = Der Analyseanbieter oder die Art, wie dieser Server ihn lädt, hat sich seit Ihrer letzten Zustimmung geändert. Er bleibt aus, bis Sie erneut zustimmen.
privacy-trusted-links-title = Vertrauenswürdige Link-Domains
privacy-trusted-links-help = Links zu diesen Domains werden direkt aus „Links prüfen“ geöffnet, ohne den Zielschritt. Andere Links, auch täuschend ähnliche Domains, werden weiterhin geprüft. Links mit reinem HTTP oder einer IP-Adresse werden immer geprüft.
privacy-trusted-links-domains = Domains
privacy-trusted-links-domains-help = Eine Domain pro Zeile, zum Beispiel example.com. Der Abgleich ist exakt: example.com vertraut weder examples.com noch example.com.other.net. Verwenden Sie für internationale Namen die ASCII-Form (xn--).
privacy-trusted-links-invalid = Geben Sie bis zu 100 Domainnamen mit mindestens zwei Labels und ohne Schema, Pfad, Port oder Platzhalter ein. Nicht akzeptiert: { $entries }
privacy-trusted-links-save = Vertrauenswürdige Domains speichern
privacy-trusted-links-subdomains = Auch Subdomains vertrauen (mail.example.com für example.com)
privacy-trusted-links-subdomains-help = Aus: Nur die exakte Domain ist vertrauenswürdig. An: Jeder Name, der auf .example.com endet, ist ebenfalls vertrauenswürdig.
privacy-linked-files-count = { $count ->
    [one] Eine in dieser Nachricht verlinkte Datei
   *[other] { $count } in dieser Nachricht verlinkte Dateien
}
privacy-linked-links = Links in dieser Nachricht
privacy-linked-files-show = Dateien anzeigen
privacy-linked-links-show = Einen Link herunterladen
privacy-linked-files-help = Downloads laufen über diesen Server, sodass der Dateihost Ihre Adresse nicht sieht. Die Größe einer Datei ist erst nach dem Download bekannt.
privacy-linked-links-help = Ein Link in dieser Nachricht kann über diesen Server als Datei heruntergeladen werden, sodass sein Host Ihre Adresse nicht sieht.
privacy-linked-file-item = { $name } von { $host } herunterladen
privacy-linked-other-item = { $link } herunterladen
privacy-links-enable = Links aktivieren
privacy-links-options = Link-Optionen
privacy-links-enable-once = Einmal aktivieren
privacy-links-trust-sender = Links von diesem Absender immer erlauben
privacy-links-revoke-sender = Links von diesem Absender nicht mehr immer erlauben
privacy-links-trust-failed = Die Link-Einstellungen konnten nicht gespeichert werden.
privacy-trusted-link-senders-title = Vertrauenswürdige Link-Absender
privacy-trusted-link-senders-help = Nachrichtenlinks für diese exakten Absenderadressen automatisch erlauben. Die Linkziele werden weiterhin geprüft.
privacy-trusted-link-senders-list = Absenderadressen
privacy-trusted-link-senders-list-help = Eine vollständige E-Mail-Adresse pro Zeile. Der Teil vor dem @ wird exakt abgeglichen, auch in der Groß- und Kleinschreibung. Entfernen Sie eine Adresse, um das Vertrauen zu widerrufen.
privacy-trusted-link-senders-save = Vertrauenswürdige Absender speichern
privacy-trusted-link-senders-remove = { $sender } entfernen
privacy-trusted-link-senders-invalid = Geben Sie gültige einzelne E-Mail-Adressen ein: { $entries }
privacy-links-disable = Links deaktivieren
privacy-links-enable-help = Erlaubt, dass Links in dieser Nachricht in einem neuen Tab geöffnet werden. Sie können sie wieder deaktivieren.
privacy-links-loading = Links werden vorbereitet …
privacy-body-blocked = Nachrichten-HTML aus Sicherheitsgründen blockiert
privacy-body-blocked-detail = Dieser Nachrichtentext enthält möglicherweise unsichere Inhalte oder konnte nicht geprüft werden. Aktive Inhalte bleiben blockiert. Die Originalnachricht bleibt unverändert.
privacy-body-show-sanitized = Bereinigtes HTML anzeigen
privacy-body-hide-sanitized = Sicheren Text anzeigen
privacy-body-sanitized-shown = Bereinigtes HTML wird angezeigt
privacy-unsafe-html = Möglicherweise unsicheres Nachrichten-HTML
privacy-unsafe-html-ask = Vor der Anzeige von bereinigtem HTML fragen
privacy-unsafe-html-strict = Immer sicheren Text anzeigen
privacy-unsafe-html-sanitized = Immer bereinigtes HTML anzeigen
privacy-unsafe-html-help = Bereinigtes HTML entfernt aktive Inhalte. Externe Ressourcen, Links und automatische Bestätigungen bleiben bei diesen Nachrichten blockiert. Die Originale bleiben unverändert.
privacy-original-html-allow = Original-HTML pro Nachricht erlauben
privacy-original-html-help = Bietet für jede Nachricht eine ausdrückliche Ansicht „Original-HTML“ an. Skripte und Formulare bleiben deaktiviert, Bilder und Links folgen weiterhin Ihren Datenschutzeinstellungen. Bereinigtes HTML bleibt der Standard.
privacy-original-html = Original-HTML
privacy-original-html-show = Original-HTML anzeigen
privacy-original-html-hide = Zurück zur bereinigten Ansicht
privacy-original-html-active = Original-HTML wird angezeigt – Skripte deaktiviert, Ressourcen kontrolliert
privacy-original-html-offer = Das ursprüngliche Layout der Nachricht nur für diese Nachricht beibehalten.
privacy-tracker-choice = Tracker-Auswahl pro Nachricht erlauben
privacy-tracker-choice-help = Bietet eine ausdrückliche Aktion an, um blockierte Tracker für eine Nachricht zu laden. Tracker bleiben blockiert, bis Sie das wählen, und eine Administration kann den Schutz vorschreiben.
privacy-image-retry-attempts = Automatische Wiederholungsversuche für Bilder
privacy-image-retry-help = Wiederholt nur vorübergehende Bildfehler, nachdem die aktuellen Bilder fertig geladen sind. Null deaktiviert automatische Wiederholungen; manuell können Sie weiterhin wiederholen. Sperren der Administration haben Vorrang.
privacy-image-retry-delay = Wartezeit vor erneutem Bildversuch (Sekunden)
privacy-concurrency-title = Adaptive Bildkapazität
privacy-concurrency-mode-auto = Automatisch
privacy-concurrency-mode-conservative = Ressourcenschonend
privacy-concurrency-mode-balanced = Ausgewogen
privacy-concurrency-mode-throughput = Hoher Durchsatz
privacy-concurrency-mode-manual = Manuell
privacy-concurrency-apply = Profil anwenden
privacy-concurrency-saved = Profil für die Bildparallelität gespeichert.
privacy-concurrency-unavailable = Nicht verfügbar
privacy-concurrency-sizing = CPU-Kerne: { $cpu } · Speicherlimit: { $limit } · Verfügbarer Speicher: { $available }
privacy-concurrency-running = Aktives Profil: { $mode } · Adaptive Dimensionierung: { $state }
privacy-concurrency-cap-perMessage = Pro Nachricht
privacy-concurrency-cap-perAccount = Pro Konto
privacy-concurrency-cap-global = Netzwerk global
privacy-concurrency-cap-pipeline = Verarbeitungspipeline
privacy-concurrency-cap-decoders = Decoder
privacy-concurrency-effective-ceiling = { $effective } effektiv / { $ceiling } Obergrenze
privacy-concurrency-message-bound = Der Wert pro Nachricht ist eine Obergrenze der Instanz. Jede Nachricht nutzt höchstens diese Grenze oder die Zahl ihrer genehmigten Bilder, je nachdem, was kleiner ist.
privacy-concurrency-active = Aktiv: { $network } Netzwerk · { $pipeline } Pipelines · { $decoders } Decoder · { $queued } in der Warteschlange
privacy-image-rate-readout = Abrufkontingent pro Konto ({ $mode }): Burst { $burst } (konfiguriert { $configuredBurst }), Auffüllung { $perMin } pro Minute (konfiguriert { $configuredPerMin })
privacy-image-cache-readout = Bild-Cache ({ $mode }): bis zu { $bytes } und { $entries } Einträge, Lebensdauer { $ttl } s, Trefferquote { $hit }
privacy-image-hit-unknown = noch nicht genug Abfragen
privacy-tab-overview = Übersicht
privacy-tab-remote = Externe Inhalte und Tracker
privacy-tab-image-proxy = Bild-Proxy
privacy-tab-fetch-limits = Abruflimits
privacy-tab-traffic = Datenverkehr
privacy-tab-cookies = Cookies
privacy-tab-denied = Sie haben keinen Zugriff auf diesen Tab.
privacy-tab-remote-help = Standardwerte, Sperren und domainspezifische Werte für externe Inhalte, Tracking-Schutz und Link-Bereinigung. Jede Einstellung wird einzeln gespeichert.
privacy-settings-denied = Zum Lesen der Einstellungen ist die Berechtigung settings.read erforderlich, die Sie nicht besitzen. Die Statistiken in den anderen Tabs sind weiterhin verfügbar.
privacy-overview-status = Status
privacy-overview-figures = Kennzahlen
privacy-status-ok = Der Bild-Proxy meldet sich.
privacy-status-unavailable = Die Statistiken des Bild-Proxys konnten nicht gelesen werden.
privacy-history-title = Bild-Proxy im Zeitverlauf
privacy-history-worker-title = Worker-Verlauf
privacy-history-cache-title = Cache und Limits
privacy-history-traffic-title = Ausgehender Datenverkehr
privacy-history-empty = In diesem Zeitraum liegen noch keine Messwerte vor.
privacy-history-unavailable = Der Verlauf ist von diesem Server nicht verfügbar.
privacy-history-summary = { $series }: zuletzt { $latest }, Höchstwert { $max }.
privacy-history-range = Zeitraum
privacy-history-range-15m = 15 Minuten
privacy-history-range-1h = 1 Stunde
privacy-history-range-6h = 6 Stunden
privacy-history-range-24h = 24 Stunden
privacy-history-updated = Aktualisiert { $time }
privacy-history-not-updated = Noch nicht aktualisiert
privacy-history-memory-note = Der Server hält alle { $interval } Sekunden einen Messwert für 24 Stunden ausschließlich im Arbeitsspeicher. Der Verlauf ist nach einem Neustart leer und wird nicht zwischen Serverprozessen geteilt.
privacy-chart-requests = Ausgelieferte Bilder
privacy-chart-requests-unit = Anfragen pro Sekunde
privacy-chart-bytes = Ausgelieferte Daten
privacy-chart-bytes-unit = Bytes pro Sekunde
privacy-chart-hit = Cache-Trefferquote
privacy-chart-hit-unit = Prozent der Abfragen
privacy-chart-ram = Arbeitsspeicher
privacy-chart-disk = Datenträger
privacy-chart-queue = Laufende und wartende Abrufe
privacy-chart-queue-unit = Anzahl
privacy-chart-active = Laufend
privacy-chart-queued = Wartend
privacy-chart-busy = Ausgelastet-Antworten
privacy-chart-busy-unit = Antworten pro Intervall
privacy-chart-trackers = Ersetzte Tracking-Pixel
privacy-chart-trackers-unit = Anzahl pro Intervall
privacy-chart-size = Cache-Größe
privacy-chart-size-unit = Gehaltene Bytes
privacy-chart-entries = Cache-Einträge
privacy-chart-entries-unit = Anzahl
privacy-chart-wait = Wartezeit durch Ratenbegrenzung
privacy-chart-wait-unit = Durchschnittliche Millisekunden pro wartender Anfrage
privacy-chart-rate = Budget der Ratenbegrenzung
privacy-chart-rate-unit = Burst-Kontingent und aktive Konten
privacy-chart-burst = Burst-Kontingent
privacy-chart-accounts = Aktive Konten
privacy-chart-allowed = Erlaubte Anfragen
privacy-chart-refused = Abgelehnte und fehlgeschlagene Anfragen
privacy-chart-traffic-unit = Anzahl pro Intervall
privacy-image-group-rate = Anfragerate
privacy-image-group-cache = Cache
privacy-image-group-ram = Speicherebene
privacy-image-group-disk = Datenträgerebene
privacy-image-group-tiering = Tiering und Wartung

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-error-reports-help = Wenn ein Teil von Mailwoman ausfällt, kann der Browser Ihrem Administrator einen kurzen technischen Bericht senden, damit das Problem behoben werden kann. Es enthält den Fehlertyp, den Ort in der App, an dem er aufgetreten ist, die App-Version und den Browsertyp. Es enthält niemals Nachrichteninhalte, Adressen, Anhangsnamen oder Ihre Eingaben.

privacy-error-reports-send = Senden Sie Fehlerberichte von diesem Browser

privacy-error-reports-send-help = Standardmäßig aktiviert. Deaktivieren Sie diese Option, um diese Berichte auf Ihrem Gerät zu behalten. Im Fehlerbereich wird weiterhin eine Referenz angezeigt, die Sie kopieren können.

privacy-error-reports-title = Fehlerberichte

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-links-revoke-domain = Hören Sie auf, immer Links von dieser Absenderdomäne zuzulassen

privacy-links-trust-domain = Erlauben Sie immer Links von dieser Absenderdomäne

privacy-trusted-link-sender-domains = Absenderdomänen

privacy-trusted-link-sender-domains-help = Erlauben Sie überprüfte Links in Nachrichten von genau diesen Absenderdomänen. Eine Domain pro Zeile; Subdomains sind getrennt. Dadurch werden Zielwebsites nicht vertrauenswürdig und Bilder werden nicht geladen. Für jede Nachricht wird eine leere Liste abgefragt.

privacy-trusted-link-sender-domains-save = Speichern Sie vertrauenswürdige Absenderdomänen

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-feedback-image-host = Image-Host-Präferenz gespeichert.

privacy-feedback-images-off = Bildberechtigung entfernt.

privacy-feedback-images-on = Bildberechtigung gespeichert. Zulässige Bilder können jetzt geladen werden.

privacy-feedback-link-domain-off = Domain-Link-Vertrauen für { $address } entfernt.

privacy-feedback-link-domain-on = Links sind in Nachrichten von der Domäne { $address } zulässig.

privacy-feedback-link-sender-off = Die Vertrauenswürdigkeit des Absender-Links wurde für { $address } entfernt.

privacy-feedback-link-sender-on = Links sind in Nachrichten von { $address } erlaubt.

privacy-feedback-links-off = Links für diese Nachricht deaktiviert.

privacy-feedback-links-on = Für diese Nachricht aktivierte Links.
