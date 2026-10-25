# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Mailwoman-Verwaltung
admin-nav = Verwaltungsbereiche
admin-sign-out = Abmelden
admin-nav-domains = Domains
admin-nav-users = Benutzer
admin-nav-integrations = Integrationen
admin-nav-observability = Überwachung
admin-nav-plugins = Plugins
admin-nav-assist = KI-Assistent
admin-nav-sso = Single Sign-on
admin-nav-servermeta = Server-Metadaten
admin-nav-rethread = Wartung
admin-nav-2fa = Zwei-Faktor-Pflicht

admin-delete = Löschen
admin-revoke = Widerrufen
admin-remove = Entfernen
admin-saved = Gespeichert.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Admin-Anmeldung
admin-login-note = Dieser Bereich läuft in einer eigenen Admin-Sitzung. Die Sitzung endet nach 30 Minuten ohne Aktivität und spätestens 12 Stunden nach der Anmeldung.
admin-login-session-ended = Ihre Admin-Sitzung ist beendet. Melden Sie sich erneut an.
admin-login-username = Admin-Benutzername
admin-login-password = Passwort
admin-login-sign-in = Anmelden
admin-login-signing-in = Anmeldung läuft…
admin-login-invalid = Ungültige Admin-Zugangsdaten
admin-login-unreachable = Der Server war nicht erreichbar

# -- Domains -----------------------------------------------------------------
admin-domains-title = Domains
admin-domains-load-error = Domains konnten nicht geladen werden
admin-domains-save-error = Die Domain konnte nicht gespeichert werden
admin-domains-delete-error = Die Domain konnte nicht gelöscht werden
admin-domains-add = Domain hinzufügen
admin-domains-name = Domainname
admin-domains-name-placeholder = example.com
admin-domains-note = Eine Domain wird über ihren Namen registriert. Registrierte Namen werden auf der Seite „Zwei-Faktor-Pflicht“ angeboten, wenn eine Regel für eine einzelne Domain gilt. Das Registrieren einer Domain leitet keine E-Mails weiter und filtert sie nicht.
admin-domains-save = Domain speichern
admin-domains-empty = Noch keine Domains.
admin-domains-delete-for = { $name } löschen

# -- Users -------------------------------------------------------------------
admin-users-title = Benutzer
admin-users-load-error = Benutzer konnten nicht geladen werden
admin-users-provision-error = Der Benutzer konnte nicht angelegt werden
admin-users-flag-error = Die Markierung konnte nicht geändert werden
admin-users-revoke-error = Sitzungen konnten nicht widerrufen werden
admin-users-provision = Benutzer anlegen
admin-users-username = Benutzername
admin-users-domain = Domain
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Kontingent in Bytes (0 = unbegrenzt)
admin-users-quota-msgs = Kontingent in Nachrichten (0 = unbegrenzt)
admin-users-empty = Noch keine Benutzer.
admin-users-col-account = Konto
admin-users-col-quota = Kontingent (Bytes/Nachr.)
admin-users-col-zeroaccess = Zero-Access
admin-users-col-flags = Markierungen
admin-users-col-sessions = Sitzungen
admin-users-zeroaccess-for = Zero-Access für { $account }
admin-users-zeroaccess-help = Nur ein Vermerk. Zero-Access-Speicherung schaltet der Benutzer selbst in seinen Einstellungen mit seinem eigenen Schlüssel ein und aus. Das Ankreuzen dieses Felds verschlüsselt die gespeicherten E-Mails des Kontos nicht, und das Entfernen des Häkchens entschlüsselt sie nicht.
admin-users-disable-for = { $account } deaktivieren
admin-users-disabled = deaktiviert
admin-users-disabled-help = Sperrt die Anmeldung und beendet jede offene Sitzung dieses Kontos. Seine API-Schlüssel und Tokens werden abgewiesen. Das Postfach auf dem Mailserver wird dadurch nicht deaktiviert: Andere E-Mail-Programme können es weiterhin erreichen, bis Sie es auch dort deaktivieren.
admin-users-force-change-for = Passwortänderung für { $account } erzwingen
admin-users-force-change = Änderung erzwingen
admin-users-force-change-help = Der Benutzer wird bei der nächsten Anmeldung und in bereits offenen Sitzungen auf einer Seite zur Passwortänderung gehalten und kann erst nach erfolgreicher Änderung auf seine E-Mails zugreifen. Dafür ist ein funktionierendes Backend für Passwortänderungen (MW_PASSWD_BACKEND) nötig. Ohne dieses kann die Änderung nicht gelingen, und der Benutzer bleibt auf dieser Seite, bis Sie dieses Feld wieder leeren.
admin-users-revoke-for = Sitzungen von { $account } widerrufen

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integrationen
admin-integrations-load-error = Integrationen konnten nicht geladen werden
admin-integrations-revoke-error = Der Schlüssel konnte nicht widerrufen werden
admin-integrations-ldap = LDAP-/GAL-Verzeichnis
admin-integrations-nextcloud = Nextcloud-Anbindung
admin-integrations-active = Aktiv
admin-integrations-configured = Konfiguriert
admin-integrations-not-configured = Nicht konfiguriert
admin-integrations-unknown = Status unbekannt
admin-integrations-config-note = „Konfiguriert“ bedeutet, dass diese Installation Einstellungen für die Integration hat: einen aktivierten LDAP-Verzeichniseintrag in der Datenbank oder die drei Umgebungsvariablen MW_NEXTCLOUD_*. Beides wird beim Start des Servers gelesen. Es bedeutet nicht, dass der entfernte Dienst kontaktiert wurde oder erreichbar ist.
admin-integrations-webhooks = Webhooks
admin-integrations-webhooks-empty = Keine Webhooks registriert.
admin-integrations-keys = API- und MCP-Schlüssel
admin-integrations-keys-empty = Keine Schlüssel ausgegeben.
admin-integrations-col-prefix = Präfix
admin-integrations-col-account = Konto
admin-integrations-col-scopes = Berechtigungen
admin-integrations-col-status = Status
admin-integrations-status-revoked = widerrufen
admin-integrations-status-active = aktiv
admin-integrations-revoke-key = Schlüssel { $prefix } widerrufen
admin-integrations-col-unattended = Unbeaufsichtigtes Senden
admin-integrations-unattended-not-requested = Nicht beantragt
admin-integrations-unattended-requested = Beantragt, nicht genehmigt
admin-integrations-unattended-approved = Genehmigt
admin-integrations-unattended-approve = Genehmigen
admin-integrations-unattended-withdraw = Zurückziehen
admin-integrations-unattended-approve-key = Unbeaufsichtigtes Senden für Schlüssel { $prefix } genehmigen
admin-integrations-unattended-withdraw-key = Genehmigung für unbeaufsichtigtes Senden mit Schlüssel { $prefix } zurückziehen
admin-integrations-unattended-approve-title = Unbeaufsichtigtes Senden für diesen Schlüssel genehmigen?
admin-integrations-unattended-withdraw-title = Genehmigung für diesen Schlüssel zurückziehen?
admin-integrations-unattended-owner = Inhaber: { $account }
admin-integrations-unattended-key = Schlüssel: { $prefix }
admin-integrations-unattended-scope = Berechtigungen: { $permissions }. MCP-Werkzeuge: { $tools }.
admin-integrations-unattended-scope-unreadable = Die Berechtigungen dieses Schlüssels konnten nicht gelesen werden.
admin-integrations-scope-read = lesen
admin-integrations-scope-send = senden
admin-integrations-scope-delete = löschen
admin-integrations-scope-mail = E-Mail
admin-integrations-scope-pim = Kalender, Aufgaben, Notizen und Kontakte
admin-integrations-scope-none = keine
admin-integrations-unattended-approve-meaning = Nachrichten, die mit diesem Schlüssel über MCP gesendet werden, werden übertragen, ohne dass eine Person sie freigibt.
admin-integrations-unattended-withdraw-meaning = Ein Senden mit diesem Schlüssel über MCP wird abgewiesen, bis der Schlüssel erneut genehmigt wird. Die Nachricht wird nicht übertragen und nicht in den Postausgang des Inhabers gelegt.
admin-integrations-unattended-approve-effect = Die Genehmigung gilt sofort, ab dem nächsten Senden mit dem Schlüssel. Ein Neustart ist nicht nötig.
admin-integrations-unattended-withdraw-effect = Das Zurückziehen gilt sofort: Ab dem nächsten Senden sendet der Schlüssel nicht mehr unbeaufsichtigt. Ein Neustart ist nicht nötig.
admin-integrations-unattended-cancel = Abbrechen
admin-integrations-unattended-saving = Wird gespeichert…
admin-integrations-unattended-approved-done = Unbeaufsichtigtes Senden für Schlüssel { $prefix } genehmigt. Es gilt ab sofort.
admin-integrations-unattended-withdrawn-done = Genehmigung für Schlüssel { $prefix } zurückgezogen. Dieser Schlüssel sendet nicht mehr unbeaufsichtigt.
admin-integrations-unattended-error-gone = Schlüssel { $prefix } existiert nicht mehr oder wurde widerrufen. Es wurde nichts geändert.
admin-integrations-unattended-error-conflict = Schlüssel { $prefix } ist widerrufen, oder sein Inhaber hat kein unbeaufsichtigtes Senden beantragt. Es wurde nichts genehmigt.
admin-integrations-unattended-error = Die Genehmigung für Schlüssel { $prefix } konnte nicht geändert werden.

# -- Observability -----------------------------------------------------------
admin-obs-title = Überwachung
admin-obs-load-error = Überwachungsdaten konnten nicht geladen werden
admin-obs-export-error = Das Audit-Protokoll konnte nicht exportiert werden
admin-obs-ban-add-error = Die Sperre konnte nicht hinzugefügt werden
admin-obs-unban-error = Die Sperre konnte nicht aufgehoben werden
admin-obs-telemetry-note = Protokollierung und Telemetrie werden nicht hier eingestellt. Der Protokollfilter (MW_LOG), der OTLP-Collector (MW_OTLP_ENDPOINT) und der Metrik-Endpunkt (MW_METRICS_TOKEN) werden beim Start des Servers aus seiner Umgebung gelesen.
admin-obs-audit = Audit-Protokoll
admin-obs-export = JSONL exportieren
admin-obs-audit-empty = Keine Audit-Einträge.
admin-obs-col-time = Zeit
admin-obs-col-actor = Akteur
admin-obs-col-action = Aktion
admin-obs-col-target = Ziel
admin-obs-bans = Anmeldeüberwachung / Sperrliste
admin-obs-bans-note = Diese Liste ist nur ein Vermerk. Mailwoman weist Verbindungen von den hier aufgeführten Adressen nicht ab, und ein Eintrag hat keine Auswirkung auf die Anmeldung. Jede fehlgeschlagene Admin-Anmeldung schreibt eine Zeile in das Serverprotokoll, auf die ein fail2ban-Jail reagieren kann, und fünf Fehlversuche von einer Adresse fügen diese Adresse hier hinzu. Gesperrt wird durch dieses Jail oder durch Ihre Firewall.
admin-obs-ban-add = Sperre hinzufügen
admin-obs-ban-ip = IP-Adresse
admin-obs-ban-reason = Grund
admin-obs-ban-btn = IP sperren
admin-obs-bans-empty = Keine aktiven Sperren.
admin-obs-unban-for = Sperre für { $ip } aufheben
admin-obs-unban-btn = Entsperren

# -- Plugins (§22) -----------------------------------------------------------
admin-plugins-title = Plugins
admin-plugins-intro = Engine-Plugins laufen in einer WebAssembly-Sandbox mit Berechtigungsprüfung. Genehmigen Sie ein Plugin, bevor es aktiviert werden kann, und gewähren Sie nur die Berechtigungen, die es braucht.
admin-plugins-empty = Es sind keine Plugins registriert.
admin-plugins-signed = Signiert
admin-plugins-unsigned = Unsigniert
admin-plugins-approved = Genehmigt
admin-plugins-enabled = Aktiviert
admin-plugins-approve = Genehmigen
admin-plugins-enable = Aktivieren
admin-plugins-disable = Deaktivieren
admin-plugins-net = Netz: { $hosts }
admin-plugins-limits = Limits: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = Limits: { $memory } MiB · { $deadline } ms · { $fuel } Fuel
admin-plugins-allow-unsigned-for = Unsigniertes Plugin { $name } zulassen
admin-plugins-allow-unsigned = Ausführung dieses unsignierten Plugins zulassen
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
admin-allowlist-title = Freigabeliste für Drittanbieter-Plugins
admin-allowlist-intro = Eine Drittanbieter-Komponente wird erst geladen, nachdem ein Administrator ihren exakten SHA-256-Digest genehmigt hat. Der unten angezeigte Digest wird vom Server über die Bytes der Komponente auf dem Datenträger berechnet; die Genehmigung legt genau diese Bytes fest. Eigene Komponenten sind im Build festgelegt und werden hier nicht verwaltet.
admin-allowlist-load-error = Die Plugin-Freigabeliste konnte nicht geladen werden.
admin-allowlist-present-heading = Komponenten auf dem Datenträger
admin-allowlist-present-empty = Es sind keine Drittanbieter-Komponenten vorhanden. Legen Sie eine Komponente im Verzeichnis für Drittanbieter-Plugins ab, damit sie hier erscheint.
admin-allowlist-digest-label = Berechneter SHA-256
admin-allowlist-status-approved = Genehmigt
admin-allowlist-status-pending = Nicht genehmigt
admin-allowlist-status-firstparty = Eigene Komponente
admin-allowlist-unsigned-note = Zugelassen über festgelegten Digest. Diese Komponente trägt keine Signatur; die Genehmigung vertraut genau den Bytes, deren Digest angezeigt wird. Das ist bei einer so genehmigten Komponente der vorgesehene Zustand.
admin-allowlist-highpower-note = Weitreichende Berechtigungen ({ $caps }) können einem Drittanbieter-Plugin nicht gewährt werden. Der Server weist sie unabhängig von der Genehmigung ab; sie stehen nur eigenen Komponenten zur Verfügung.
admin-allowlist-firstparty-note = Diese ID entspricht einer eigenen Komponente. Die Festlegung der eigenen Komponente hat immer Vorrang, daher kann diese ID nicht als Drittanbieter-Komponente genehmigt werden.
admin-allowlist-approve = Digest genehmigen
admin-allowlist-approve-for = Digest für { $id } genehmigen
admin-allowlist-revoke = Widerrufen
admin-allowlist-revoke-for = Festlegung für { $id } widerrufen
admin-allowlist-uninstall = Deinstallieren
admin-allowlist-uninstall-for = { $id } deinstallieren
admin-allowlist-pins-heading = Genehmigte und widerrufene Festlegungen
admin-allowlist-pins-empty = Keine Festlegungen erfasst.
admin-allowlist-pin-approved-by = Genehmigt von { $by } am { $at }
admin-allowlist-pin-revoked = Widerrufen
admin-allowlist-cancel = Abbrechen

admin-allowlist-approve-title = Laden dieser Komponente genehmigen?
admin-allowlist-approve-detail = Die Genehmigung legt genau die Bytes fest, deren SHA-256 unten angezeigt wird. Danach werden für diese ID nur Bytes geladen, die zu diesem Digest passen; jede Änderung an der Komponente ergibt einen anderen Digest und wird erst nach erneuter Genehmigung geladen. Die Genehmigung allein gewährt keine Berechtigungen.
admin-allowlist-approve-confirm = Digest genehmigen

admin-allowlist-revoke-title = Diese Festlegung widerrufen?
admin-allowlist-revoke-detail = Das Widerrufen entfernt die Genehmigung für diesen Digest und deaktiviert das Plugin. Es wirkt beim nächsten Laden; eine bereits laufende Instanz wird bis dahin nicht gestoppt.
admin-allowlist-revoke-confirm = Festlegung widerrufen

admin-allowlist-uninstall-title = Dieses Plugin deinstallieren?
admin-allowlist-uninstall-detail = Das Deinstallieren löscht die gespeicherten Schlüssel-Wert-Daten des Plugins für alle Konten, entfernt seine Festlegungen in der Freigabeliste und deaktiviert es. Die Komponentendatei auf dem Datenträger wird nicht gelöscht; sie kann später erneut genehmigt werden.
admin-allowlist-uninstall-confirm = Plugin deinstallieren

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Single Sign-on
admin-sso-intro = Konfigurieren Sie OIDC- und SAML-2.0-Anmelde-Backends. Aktivierte Backends erscheinen als Schaltflächen „Anmelden mit…“ bei der Postfach-Anmeldung, entweder für die gesamte Installation oder für eine Domain.
admin-sso-add = Anmelde-Backend hinzufügen
admin-sso-edit = Bearbeiten
admin-sso-create = Backend hinzufügen
admin-sso-update = Änderungen speichern
admin-sso-cancel = Abbrechen
admin-sso-empty = Keine SSO-Backends konfiguriert.
admin-sso-load-error = SSO-Backends konnten nicht geladen werden.
admin-sso-save-error = Das Backend konnte nicht gespeichert werden.
admin-sso-delete-error = Das Backend konnte nicht gelöscht werden.

admin-sso-id = Backend-ID
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Anzeigename
admin-sso-display-name-placeholder = Anmelden mit Acme SSO
admin-sso-kind = Protokoll
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Geltungsbereich
admin-sso-scope-deployment = Gesamte Installation
admin-sso-scope-domain = Eine Domain
admin-sso-domain = Domain
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Aktiviert
admin-sso-first-login = Richtlinie für die erste Anmeldung
admin-sso-policy-allowlist = Freigabeliste (unbekannte Benutzer abweisen)
admin-sso-policy-autocreate = Konten bei der ersten Anmeldung automatisch anlegen

admin-sso-issuer = Aussteller-URL
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = Client-ID
admin-sso-client-secret = Client-Geheimnis
admin-sso-secret-unchanged = Leer lassen, um das gespeicherte Geheimnis zu behalten
admin-sso-redirect = Weiterleitungs-URL
admin-sso-scopes = Scopes
admin-sso-metadata = SP-Metadaten

admin-sso-sp-entity-id = SP-Entity-ID
admin-sso-acs-url = ACS-URL
admin-sso-idp-metadata-url = IdP-Metadaten-URL
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = IdP-Metadaten-XML
admin-sso-idp-metadata-xml-placeholder = IdP-Metadaten-XML einfügen oder die URL oben verwenden
admin-sso-idp-sso-url = IdP-SSO-URL
admin-sso-idp-slo-url = IdP-Abmelde-URL (SLO)
admin-sso-idp-certs = IdP-Signaturzertifikate (PEM)
admin-sso-idp-certs-placeholder = Ein PEM-Zertifikat pro Block, getrennt durch eine Leerzeile
admin-sso-nameid-format = NameID-Format
admin-sso-want-signed = Signierte Assertions verlangen
admin-sso-want-encrypted = Verschlüsselte Assertions verlangen

admin-sso-claims = Claim-Zuordnung
admin-sso-claim-email = E-Mail-Claim
admin-sso-claim-username = Benutzername-Claim
admin-sso-claim-display = Anzeigename-Claim
admin-sso-claim-groups = Gruppen-Claim

admin-sso-badge-enabled = Aktiviert
admin-sso-badge-disabled = Deaktiviert
admin-sso-enable = Aktivieren
admin-sso-disable = Deaktivieren
admin-sso-delete = Löschen
admin-sso-enable-for = { $name } aktivieren
admin-sso-disable-for = { $name } deaktivieren
admin-sso-delete-for = { $name } löschen

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
admin-servermeta-title = Server-Metadaten
admin-servermeta-intro = RFC-5464-Server-Annotationen eines angelegten Kontos anzeigen und bearbeiten. Änderungen werden direkt auf den Mailserver geschrieben, der entscheidet, ob das Konto sie setzen darf.
admin-servermeta-account = Konto
admin-servermeta-select-option = Konto auswählen…
admin-servermeta-select-prompt = Wählen Sie ein Konto, um seine Server-Annotationen anzuzeigen und zu bearbeiten.
admin-servermeta-load-error = Die Kontenliste konnte nicht geladen werden.
admin-servermeta-no-accounts = Es sind keine Konten angelegt.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
admin-rethread-title = Postfach neu verketten
admin-rethread-intro = Führt die Konversationsverkettung (JWZ) für die gespeicherten E-Mails eines angelegten Kontos erneut aus und ordnet die Thread-Gruppierung neu zu. Dies ist eine einmalige Wartungsaktion, die nicht automatisch läuft.
admin-rethread-account = Konto
admin-rethread-select-option = Konto auswählen…
admin-rethread-no-accounts = Es sind keine Konten angelegt.
admin-rethread-load-error = Die Kontenliste konnte nicht geladen werden.
admin-rethread-run = Postfach neu verketten
admin-rethread-confirm-title = Dieses Postfach neu verketten?
admin-rethread-confirm-warning = Das Neuverketten ordnet die Konversationsgruppierung dieses Kontos neu zu. Bestehende Threads können zusammengeführt oder geteilt werden, und Benutzer sehen Konversationen möglicherweise neu gruppiert.
admin-rethread-confirm-detail = Dies ist eine einmalige Wartungsaktion. Sie läuft jetzt einmal; sie kann gefahrlos wiederholt werden und löscht keine E-Mails.
admin-rethread-confirm = Postfach neu verketten
admin-rethread-running = Wird neu verkettet…
admin-rethread-cancel = Abbrechen
admin-rethread-summary = { $messages } Nachricht(en) in { $threads } Thread(s) über { $accounts } Konto/Konten neu verkettet; { $reassigned } Nachricht(en) in einen anderen Thread verschoben.
admin-rethread-error = Das Neuverketten ist fehlgeschlagen. Wenn der Server die Anfrage abgewiesen hat, wurde keine Thread-Gruppierung geändert; prüfen Sie die Serverprotokolle und versuchen Sie es erneut.

# -- Search index: status and rebuild (t28-e13) --------------------------------
admin-searchindex-title = Suchindex
admin-searchindex-intro = Die Suche liest einen Index, der aus den gespeicherten E-Mails erstellt wird. Der Server baut ihn beim Start neu auf, wenn er nicht zu den gespeicherten E-Mails passt. E-Mails von Zero-Access-Konten werden nicht indiziert, daher findet die Suche für diese Konten nichts.
admin-searchindex-count = { $documents } von { $messages } gespeicherten Nachricht(en) sind indiziert.
admin-searchindex-on-disk = Der Index wird auf dem Datenträger gespeichert.
admin-searchindex-in-memory = Der Index wird im Arbeitsspeicher gehalten und bei jedem Serverstart neu aufgebaut.
admin-searchindex-progress = Ein Neuaufbau läuft: { $done } von { $total } Nachricht(en) erledigt.
admin-searchindex-run = Suchindex neu aufbauen
admin-searchindex-running = Wird neu aufgebaut…
admin-searchindex-summary = { $messages } gespeicherte Nachricht(en) in { $accounts } Konto/Konten gelesen: { $indexed } indiziert, { $removed } aus dem Index entfernt, { $failed } nicht lesbar. { $zeroAccess } Zero-Access-Konto/Konten nicht indiziert.
admin-searchindex-busy = Ein Neuaufbau läuft bereits. Warten Sie, bis er abgeschlossen ist.
admin-searchindex-error = Der Neuaufbau ist fehlgeschlagen. Der Index behält seinen bisherigen Inhalt; prüfen Sie die Serverprotokolle und versuchen Sie es erneut.
admin-searchindex-status-error = Der Status des Suchindex konnte nicht gelesen werden.
admin-searchindex-unavailable = Dieser Server läuft im Proxy-Modus und hat keinen Suchindex.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
admin-2fa-title = Zwei-Faktor-Pflicht
admin-2fa-intro = Einen zweiten Faktor (Passkey oder Authenticator-App) für die Anmeldung verlangen. Jeder Benutzer kann selbst einen Faktor einrichten; wird er hier verlangt, müssen betroffene Konten ihn bei der nächsten Anmeldung einrichten.
admin-2fa-load-error = Die Zwei-Faktor-Richtlinie konnte nicht geladen werden.
admin-2fa-save-error = Die Zwei-Faktor-Richtlinie konnte nicht gespeichert werden.
admin-2fa-global = Zwei-Faktor für die gesamte Installation verlangen
admin-2fa-global-label = Zwei-Faktor für die gesamte Installation verlangen
admin-2fa-global-note = Wenn aktiviert, muss jedes Konto einen zweiten Faktor haben. Eine Domain-Regel unten kann ihn auch nur für eine Domain verlangen, ohne ihn überall zu verlangen.
admin-2fa-domains-heading = Anforderungen pro Domain
admin-2fa-domains-empty = Keine Anforderungen pro Domain festgelegt.
admin-2fa-col-domain = Domain
admin-2fa-col-require = Verlangt
admin-2fa-require-for = Zwei-Faktor für { $domain } verlangen
admin-2fa-add-domain = Domain
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Zwei-Faktor für diese Domain verlangen
admin-2fa-add-require-label = Zwei-Faktor für diese Domain verlangen
admin-2fa-add-save = Domain-Regel hinzufügen

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Ausgehend
admin-egress-heading = Ausgehende Proxy-Routen
admin-egress-intro = Ausgehende Abrufe des Servers über einen HTTP-CONNECT- oder SOCKS5-Proxy leiten. Mailwoman setzt weiterhin seine eigene Adressrichtlinie durch — der Proxy wird nie gebeten, einen Namen aufzulösen.
admin-egress-empty = Keine Routen konfiguriert. Ausgehende Abrufe erfolgen direkt.
admin-egress-none-active = Keine Route in Gebrauch. Ausgehende Abrufe erfolgen direkt. Eine gespeicherte Route leitet erst Verkehr, nachdem Sie „Diese Route verwenden“ gewählt haben.
admin-egress-in-use = Ausgehende Abrufe verwenden die Route „{ $id }“.
admin-egress-deactivate = Keinen Proxy mehr verwenden
admin-egress-activate = Diese Route verwenden
admin-egress-activate-for = Route { $id } verwenden
admin-egress-activate-error = Konnte nicht zur Route wechseln
admin-egress-deactivate-error = Die Route konnte nicht abgeschaltet werden
admin-egress-col-state = Zustand
admin-egress-state-active = In Gebrauch
admin-egress-state-inactive = Nicht in Gebrauch
admin-egress-test-scope = Nur die Route in Gebrauch kann getestet werden. Ein Test ruft eine feste URL über die Route ab und meldet, wie weit er gekommen ist; er ändert nichts.
admin-egress-probe-url = Abgerufen: { $url }
admin-egress-load-error = Die ausgehenden Routen konnten nicht geladen werden
admin-egress-save-error = Die Route konnte nicht gespeichert werden
admin-egress-delete-error = Die Route konnte nicht gelöscht werden
admin-egress-invalid = Eine ID, ein Host und ein Port größer als null sind erforderlich.
admin-egress-username-required = Ein Passwort braucht einen Benutzernamen; ohne ihn würde sich die Route als niemand authentifizieren.
admin-egress-col-id = ID
admin-egress-col-route = Route
admin-egress-col-username = Benutzername
admin-egress-col-credentials = Passwort
admin-egress-col-plaintext = Klartext
admin-egress-col-actions = Aktionen
admin-egress-cred-set = Gesetzt
admin-egress-cred-none = Keins
admin-egress-plaintext-on = Erlaubt
admin-egress-plaintext-off = Abgewiesen
admin-egress-add-heading = Route hinzufügen
admin-egress-edit-heading = Route bearbeiten
admin-egress-id = ID
admin-egress-scheme = Schema
admin-egress-host = Host
admin-egress-port = Port
admin-egress-username = Benutzername
admin-egress-password = Passwort
admin-egress-password-unchanged = Leer lassen, um das gespeicherte Passwort zu behalten
admin-egress-allow-plaintext = Klartext-http-Ursprünge über diese Route erlauben
admin-egress-allow-plaintext-note = Standardmäßig aus. Ein https-Ursprung verhindert, dass ein Proxy den von ihm transportierten Verkehr liest; wird http erlaubt, sieht der Proxy ihn.
admin-egress-test = Testen
admin-egress-testing = Wird getestet…
admin-egress-test-failed = Der Test konnte nicht ausgeführt werden, daher ist über diese Route nichts bekannt.
admin-egress-outcome-connected = Verbunden
admin-egress-outcome-auth-rejected = Authentifizierung abgewiesen
admin-egress-outcome-refused-by-policy = Durch Richtlinie abgewiesen
admin-egress-outcome-dns-failed = Name nicht aufgelöst
admin-egress-outcome-unreachable = Nicht erreichbar
admin-egress-outcome-origin-tls-failed = TLS zum Ursprung fehlgeschlagen
admin-egress-outcome-route-invalid = Route ist ungültig
admin-egress-outcome-unknown = Unbekanntes Ergebnis
admin-egress-stage-dns = bei der Namensauflösung abgebrochen
admin-egress-stage-connect = beim Verbinden mit dem Proxy abgebrochen
admin-egress-stage-tunnel = beim Öffnen des Tunnels abgebrochen
admin-egress-stage-origin = beim Ursprung abgebrochen
admin-egress-stage-unknown = in einer unbekannten Phase abgebrochen
admin-egress-proxied-yes = Über den Proxy geleitet
admin-egress-proxied-no = Nicht über den Proxy geleitet
