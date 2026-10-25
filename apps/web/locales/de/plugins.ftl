# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = Warnung zu unsignierten UI-Plugins
plugins-unsigned-title = Unsignierte Plugins laufen
plugins-unsigned-text = Diese Plugins wurden ohne geprüfte Signatur zugelassen. Sie laufen in einer Sandbox, aber ihr Code wurde nicht gegen einen vertrauenswürdigen Schlüssel geprüft.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = Plugin: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = Engine-Plugins laufen in einer WebAssembly-Sandbox. Ein Plugin läuft erst, nachdem es registriert, genehmigt, mit mindestens einer Berechtigung ausgestattet und aktiviert wurde. Es kann nur die hier gewährten Berechtigungen nutzen.
plugins-admin-unsigned-banner = Ein Drittanbieter-Plugin ohne Signatur ist geladen. Es läuft, weil ein Administrator es zugelassen hat; der Server prüft seine Datei nur gegen den genehmigten Digest.
plugins-admin-load-error = Die Plugins konnten nicht geladen werden.
plugins-admin-register-heading = Plugin registrieren
plugins-admin-register-component = Komponente
plugins-admin-register-third-party = Drittanbieter-Komponente
plugins-admin-register-id = Plugin-ID
plugins-admin-register-id-hint = Der Server liest die Komponente aus der Datei, die nach dieser ID im Verzeichnis für Drittanbieter-Plugins benannt ist. Genehmigen Sie zuerst den Digest dieser Datei in der Freigabeliste unten.
plugins-admin-register-name = Name
plugins-admin-register-version = Version
plugins-admin-register-capabilities = Vom Plugin angegebene Berechtigungen
plugins-admin-register-hosts = Hosts, die das Plugin kontaktieren darf
plugins-admin-register-hosts-hint-first-party = Hostnamen, durch Kommas getrennt. Leer lassen, um die Standard-Hosts der Komponente zu verwenden.
plugins-admin-register-hosts-hint-third-party = Hostnamen, durch Kommas getrennt. Leer lassen für keinen Netzwerkzugriff.
plugins-admin-register-submit = Registrieren
plugins-admin-trust-first-party = Eigene Komponente, Digest eingebaut
plugins-admin-trust-third-party = Drittanbieter, Digest genehmigt
plugins-admin-status-loaded = Geladen
plugins-admin-status-not-loaded = Nicht geladen
plugins-admin-status-restart = Neustart erforderlich
plugins-admin-restart-to-load = Neustart erforderlich, um dieses Plugin zu laden.
plugins-admin-restart-to-apply = Neustart erforderlich, um diese Änderungen anzuwenden. Einige Konten dieser Anbindung werden bis dahin nicht bedient.
plugins-admin-reason-not-approved = Nicht geladen: Ein Administrator hat dieses Plugin nicht genehmigt.
plugins-admin-reason-disabled = Nicht geladen: Das Plugin ist nicht aktiviert.
plugins-admin-reason-unsigned-not-allowed = Nicht geladen: Das Plugin hat keine Signatur und wurde nicht für die Ausführung ohne Signatur zugelassen.
plugins-admin-reason-no-grant = Nicht geladen: Keine benötigte Berechtigung wurde gewährt.
plugins-admin-reason-component-unavailable = Nicht geladen: Keine Komponentendatei hat die Digest-Prüfung bestanden.
plugins-admin-reason-load-failed = Nicht geladen: Der Server konnte die Komponente nicht laden. Siehe Serverprotokoll.
plugins-admin-reason-proxy-mode = Nicht geladen: Dieser Server läuft im Proxy-Modus und verarbeitet selbst keine E-Mails.
plugins-admin-reason-no-account-binding = Nicht geladen: Dieser Anbindung ist kein Konto zugeordnet.
plugins-admin-reason-no-host-caller = Nicht geladen: Diese Serverversion ruft die Hooks dieses Plugins nicht auf.
plugins-admin-reason-another-classifier-active = Nicht geladen: Ein anderer Spam-Klassifikator ist geladen. Es läuft immer nur einer.
plugins-admin-running-with = Läuft mit: { $capabilities }
plugins-admin-hosts = Hosts: { $hosts }
plugins-admin-hosts-none = Hosts: keine
plugins-admin-grants-heading = Gewährte Berechtigungen
plugins-admin-grants-hint = Das Plugin kann nur die hier angekreuzten Berechtigungen nutzen. Speichern ersetzt die frühere Gewährung.
plugins-admin-grant-for = { $capability } für { $name } gewähren
plugins-admin-grants-save = Berechtigungen speichern
plugins-admin-endpoint = Klassifikator-Adresse
plugins-admin-endpoint-for = Klassifikator-Adresse für { $name }
plugins-admin-endpoint-hint = Wo das Plugin seinen Daemon erreicht. Der Host muss auch in der Host-Liste des Plugins stehen. Leer lassen, um den Standard der Komponente zu verwenden.
plugins-admin-endpoint-save = Adresse speichern
plugins-admin-test = Klassifikator testen
plugins-admin-test-verdict = Testergebnis: { $verdict }
plugins-admin-uninstall = Deinstallieren
plugins-admin-uninstall-confirm = Deinstallation bestätigen
plugins-admin-uninstall-note = Die Deinstallation entfernt die Registrierung, ihre Berechtigungen und ihre gespeicherten Einstellungen.
plugins-admin-uninstall-detail = Die Deinstallation löscht die gespeicherten Daten des Plugins für alle Konten, seine Festlegungen in der Freigabeliste sowie seine Registrierung und Berechtigungen. Die Komponentendatei auf dem Datenträger wird nicht gelöscht.
plugins-admin-revoke-detail = Das Widerrufen entfernt die Genehmigung für diesen Digest und deaktiviert das Plugin. Läuft das Plugin, wird es sofort gestoppt.
plugins-admin-error-already-registered = Dieses Plugin ist bereits registriert. Deinstallieren Sie es, bevor Sie es erneut registrieren.
plugins-admin-error-component-unavailable = Keine Komponentendatei für dieses Plugin passt zum im Server eingebauten Digest.
plugins-admin-error-digest-not-approved = Für diese ID wurde keine Komponentendatei mit genehmigtem Digest gefunden. Genehmigen Sie zuerst den Digest der Datei in der Freigabeliste.
plugins-admin-error-not-approved = Genehmigen Sie das Plugin, bevor Sie es aktivieren.
plugins-admin-error-unsigned-not-allowed = Dieses Plugin hat keine Signatur. Lassen Sie die unsignierte Ausführung zu, bevor Sie es aktivieren.
plugins-admin-error-not-loaded = Das Plugin ist nicht geladen und kann daher nicht getestet werden.
plugins-admin-error-classifier-error = Der Klassifikator-Aufruf ist fehlgeschlagen: { $detail }
plugins-admin-error-other = Der Server hat die Anfrage abgewiesen: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = UI-Plugins
plugins-admin-ui-title = UI-Plugins
plugins-admin-ui-intro = UI-Plugins fügen dem Web-Client Bedienelemente hinzu. Jedes läuft in einem Sandbox-Frame und kann nur die hier gewährten Berechtigungen nutzen. Ein Plugin wird angemeldeten Benutzern angeboten, sobald es genehmigt und aktiviert ist.
plugins-admin-ui-empty = Es sind keine UI-Plugins registriert.
plugins-admin-ui-load-error = Die UI-Plugins konnten nicht geladen werden.
plugins-admin-ui-register-heading = UI-Plugin registrieren
plugins-admin-ui-manifest = Manifest (JSON)
plugins-admin-ui-bundle = Bundle-Datei
plugins-admin-ui-bundle-hint = Für ein signiertes Plugin erforderlich: Die Signatur wird gegen diese Datei geprüft.
plugins-admin-ui-allow-unsigned = Registrierung dieses Plugins ohne Signatur zulassen
plugins-admin-ui-register = Registrieren
plugins-admin-ui-manifest-invalid = Das Manifest ist kein gültiges JSON.
plugins-admin-ui-refused = Der Server hat die Anfrage abgewiesen: { $detail }
plugins-admin-ui-approve = Genehmigen und aktivieren
plugins-admin-ui-extension-points = Erweiterungspunkte: { $points }
plugins-admin-ui-grants-heading = Berechtigung gewähren
plugins-admin-ui-grants-note = Der Server meldet nicht, welche Berechtigungen bereits gewährt sind. Wird eine Berechtigung erneut gewährt, ersetzt das ihre frühere Gewährung.
plugins-admin-ui-grant = { $capability } gewähren
plugins-admin-ui-grant-for = { $capability } für { $name } gewähren
plugins-admin-ui-grant-hosts = Hosts für { $capability } bei { $name }
plugins-admin-ui-grant-hosts-hint = Hostnamen, durch Kommas getrennt, von denen das Plugin abrufen darf.
plugins-admin-ui-granted = { $capability } gewährt.
plugins-admin-ui-delete = Löschen
plugins-admin-ui-delete-confirm = Löschen bestätigen
