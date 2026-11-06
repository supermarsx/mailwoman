# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-features. Source: ../en/admin-features.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-features-title = Funktionsprofile
admin-features-intro = Ein Profil ist ein benannter Satz von Funktionsschaltern. Weisen Sie es einer Domain, einer Gruppe oder einem Benutzer zu. Einzelzuweisungen unterstützen native Postfächer und verifizierte Proxy- oder SSO-Identitäten; sie steuern den Zugriff auf Mailwoman-Funktionen.
admin-features-new = Neues Profil
admin-features-search = Profile suchen
admin-features-empty = Noch keine Funktionsprofile.
admin-features-no-match = Kein Profil entspricht der Suche.
admin-features-load-error = Der Funktionskatalog konnte nicht geladen werden.
admin-features-not-loaded = Der Funktionskatalog ist noch nicht geladen. Laden Sie die Seite neu und versuchen Sie es erneut.
admin-features-stale = Ein anderer Administrator hat den Funktionskatalog geändert. Er wurde neu geladen; prüfen Sie die Werte und speichern Sie erneut.
admin-features-not-found = Kein Profil mit der ID { $id }.
admin-features-col-name = Name
admin-features-col-values = Schalter
admin-features-col-assigned = Zugewiesen an
admin-features-values-count = { $count ->
    [one] { $count } Schalter gesetzt
   *[other] { $count } Schalter gesetzt
}
admin-features-assigned-count = { $count ->
    [0] Nicht zugewiesen
    [one] { $count } Stelle
   *[other] { $count } Stellen
}
admin-features-editor = Profileinstellungen
admin-features-name = Profilname
admin-features-name-help = Nur für Administratoren sichtbar, bis zu 128 Zeichen.
admin-features-name-invalid = Geben Sie einen Namen mit 1 bis 128 Zeichen ein.
admin-features-switches = Funktionsschalter
admin-features-switches-help = „Nicht gesetzt“ überlässt die Funktion dem nächsten Profil, der Domain oder dem Serverstandard. Ein gesperrter Wert kann weder vom Benutzer noch von einer niedrigeren Ebene geändert werden.
admin-features-col-feature = Funktion
admin-features-col-value = Wert
admin-features-col-lock = Sperre
admin-features-value-for = Wert für { $feature }
admin-features-unset = Nicht gesetzt
admin-features-on = Erlaubt
admin-features-off = Nicht erlaubt
admin-features-lock = Gesperrt
admin-features-save = Profil speichern
admin-features-create = Profil erstellen
admin-features-delete = Profil löschen
admin-features-delete-title = Profil { $name } löschen?
admin-features-delete-body = { $count ->
    [0] Das Profil ist nirgends zugewiesen. Seine Schalter gelten sofort nicht mehr.
    [one] Das Profil wird von der { $count } Stelle entfernt, der es zugewiesen ist, und seine Schalter gelten sofort nicht mehr.
   *[other] Das Profil wird von den { $count } Stellen entfernt, denen es zugewiesen ist, und seine Schalter gelten sofort nicht mehr.
}
admin-features-saved = Profil { $name } gespeichert.
admin-features-deleted = Profil { $name } gelöscht.
admin-features-assigned = Profil { $name } zugewiesen.
admin-features-unassigned = Zuweisung von { $name } entfernt.
admin-features-assignments = Zugewiesen an
admin-features-assignments-help = Profile gelten in Prioritätsreihenfolge, höhere zuerst. Bei gleicher Priorität gilt eine Benutzerzuweisung vor einer Gruppe und eine Gruppe vor einer Domain.
admin-features-assignments-empty = Dieses Profil ist noch nicht zugewiesen.
admin-features-assign-form = Dieses Profil zuweisen
admin-features-col-scope = Gilt für
admin-features-col-target = Name
admin-features-col-priority = Priorität
admin-features-col-actions = Aktionen
admin-features-scope-domain = Domain
admin-features-scope-group = Gruppe
admin-features-scope-user = Benutzer
admin-features-assign = Zuweisen
admin-features-unassign = Entfernen
admin-features-no-domain = Es sind keine Domains konfiguriert.
admin-features-no-group = Es gibt keine Benutzergruppen.
admin-features-no-user = Noch hat sich kein Postfachkonto angemeldet.
admin-features-tools = Profilwerkzeuge
admin-features-preset = Mit einer Vorlage beginnen
admin-features-preset-choose = Vorlage wählen
admin-features-preset-apply = Vorlage anwenden
admin-features-preset-applied = Vorlage { $name } angewendet. Prüfen Sie die Werte und speichern Sie.
admin-features-preset-full = Vollständig
admin-features-preset-standard = Standard
admin-features-preset-mail-only = Nur E-Mail
admin-features-preset-minimal = Minimal / Kiosk
admin-features-import = JSON importieren
admin-features-export = JSON exportieren
admin-features-exported = Profil als JSON-Datei exportiert.
admin-features-imported = { $count ->
    [one] { $count } Wert importiert. Prüfen Sie ihn und speichern Sie.
   *[other] { $count } Werte importiert. Prüfen Sie sie und speichern Sie.
}
admin-features-import-json = Die Datei ist kein gültiges JSON.
admin-features-import-format = Die Datei ist kein Mailwoman-Funktionsprofil.
admin-features-import-version = Diese Funktionsprofil-Datei stammt aus einer neueren Version und kann nicht importiert werden.
admin-features-clone = Klonen
admin-features-clone-name = Kopie von { $name }
admin-features-filter = Funktion suchen
admin-features-filter-placeholder = Name, Einstellung oder Schlüssel
admin-features-filter-none = Keine Funktion entspricht der Suche.
admin-features-diff-summary = { $differs ->
    [one] { $differs } Wert weicht vom Serverstandard ab
   *[other] { $differs } Werte weichen vom Serverstandard ab
} · { $same } wiederholen ihn
admin-features-differs = Weicht vom Standard ab
admin-features-default-is = Standard: { $value }
admin-features-fix-values = Korrigieren Sie die als ungültig markierten Werte vor dem Speichern.
admin-features-allow-all = Alle erlauben
admin-features-deny-all = Alle nicht erlauben
admin-features-clear-all = Zurücksetzen
admin-features-group-allow = Jede Funktion in { $group } erlauben
admin-features-group-deny = Jede Funktion in { $group } nicht erlauben
admin-features-group-clear = Jeden Wert in { $group } zurücksetzen
admin-features-enforcement-ui = Nur in der App ausgeblendet
admin-features-enforcement-partial = Einige Grenzen werden noch nicht gezählt
admin-features-other = Weitere Einstellungen, die ein Profil enthalten kann
admin-features-other-help = Jede andere Einstellung, die ein Administrator pro Profil festlegen darf. Lassen Sie ein Feld leer, um den Serverwert beizubehalten.
admin-features-cat-compose = E-Mail und Verfassen
admin-features-cat-reading = Lesen und Datenschutz
admin-features-cat-views = Ansichten und Suche
admin-features-cat-organise = Organisieren
admin-features-cat-pim = Kalender, Kontakte, Aufgaben, Notizen und Schlüssel
admin-features-cat-data = Daten
admin-features-cat-accounts = Konten und Zugriff
admin-features-cat-integrations = Integrationen
admin-features-cat-developer = Entwicklerzugriff
admin-features-cat-personal = Personalisierung
admin-features-feature-compose-sending = Sendelimits
admin-features-feature-compose-sending-help = Empfänger pro Nachricht werden vom Server oberhalb des Limits abgelehnt. Nachrichten pro Stunde und pro Tag werden für das Profil gespeichert, aber noch nicht gezählt.
admin-features-feature-storage = Speicherkontingent
admin-features-feature-storage-help = Das Speicherlimit wird für das Profil gespeichert; das eigene Kontingent des Mailservers gilt weiterhin.
admin-features-feature-access-protocols = Protokolle für den E-Mail-Zugriff
admin-features-feature-access-protocols-help = Mit welchen Mailserver-Protokollen sich diese Benutzer anmelden dürfen, durch Kommas getrennt: imap, pop3, ews.
admin-features-feature-accounts-linked = Verknüpfte Postfächer
admin-features-feature-calendar = Kalender
admin-features-feature-contacts = Kontakte
admin-features-feature-tasks = Aufgaben
admin-features-feature-notes = Notizen
admin-features-preview = Wirksame Werte
admin-features-preview-help = Worauf jede Funktion für einen Benutzer oder einen Benutzer einer Domain aufgelöst wird, nach allen Profilen, Gruppen, Domain-Werten und Sperren.
admin-features-preview-kind = Vorschau für
admin-features-preview-target = Wer
admin-features-preview-show = Anzeigen
admin-features-preview-loading = Wird geladen…
admin-features-preview-state = Wirksam
admin-features-preview-from = Entschieden durch
admin-features-preview-profiles = Anwendbare Profile, das erste gewinnt: { $names }
admin-features-preview-no-profiles = Kein Profil trifft zu; die Werte von Domain und Server entscheiden.
admin-features-source-default = Eingebauter Standard
admin-features-source-env-fallback = Umgebungsstandard
admin-features-source-env-pin = Umgebung (festgelegt)
admin-features-source-global = Serverwert
admin-features-source-domain = Domain-Wert
admin-features-source-profile = Profil
admin-features-source-profile-named = Profil { $name }
admin-features-source-group = Gruppenwert
admin-features-source-admin-user = Für diesen Benutzer gesetzter Wert
admin-features-source-user = Eigene Wahl des Benutzers
admin-features-source-locked = { $source } (gesperrt)
admin-retention-col-category = Kategorie
admin-retention-col-kept = Aufbewahrt
admin-retention-col-change = Ändern
admin-retention-cat-default = Standard
admin-retention-cat-admin = Administratoraktionen
admin-retention-cat-security = Sicherheitsereignisse
admin-retention-cat-alert = Benutzerwarnungen
admin-retention-forever = Jeder Eintrag
admin-retention-days = { $days ->
    [one] { $days } Tag
   *[other] { $days } Tage
}
admin-retention-follows-default = Standard: { $kept }
admin-retention-hold-on = Rechtssperre
admin-retention-read-only = Zum Ändern der Aufbewahrung ist die Berechtigung settings.write nötig.
admin-retention-edit-for = Aufbewahrung für { $category }
admin-retention-keep = Behalten
admin-retention-mode-inherit = Dem Standard folgen
admin-retention-mode-forever = Jeder Eintrag
admin-retention-mode-days = Für eine Anzahl Tage
admin-retention-days-label = Tage
admin-retention-hold = Rechtssperre
admin-retention-save = Speichern
admin-retention-bounds = Geben Sie eine ganze Zahl von { $min } bis { $max } ein.
admin-retention-saved = Aufbewahrung für { $category } gespeichert.
admin-retention-task-none = Noch keine Bereinigungsaufgabe. Sie wird täglich um 03:15 UTC angelegt, sobald erstmals eine Tageszahl gespeichert wird.
admin-retention-task-disabled = Die geplante Bereinigungsaufgabe ist deaktiviert; Einträge werden nur mit „Jetzt bereinigen“ bereinigt.
admin-retention-task-next = Nächste geplante Bereinigung: { $when }.
admin-retention-task-daily = Wird täglich um 03:15 UTC bereinigt.
admin-retention-last = Letzte Bereinigung, die Einträge entfernt hat: { $when }.
admin-retention-prune = Jetzt bereinigen
admin-retention-prune-title = Auditprotokoll jetzt bereinigen?
admin-retention-prune-body = Einträge, die älter als die Aufbewahrung ihrer Kategorie sind, werden gelöscht. Kategorien, die jeden Eintrag behalten oder unter Rechtssperre stehen, bleiben unverändert. Dies kann nicht rückgängig gemacht werden.
admin-retention-pruned = { $count ->
    [0] Kein Eintrag war alt genug, um entfernt zu werden.
    [one] { $count } Eintrag entfernt.
   *[other] { $count } Einträge entfernt.
}
admin-auth-attempted = Versucht als { $name }
admin-features-feature-compose-open-tracking = Öffnungsverfolgung von Nachrichten
admin-features-feature-compose-open-tracking-help = Erlaubt das Erfassen von Öffnungen geeigneter formatierter Nachrichten über das Tracking-Bild des Servers. Erfordert eine konfigurierte öffentliche URL.
admin-features-feature-reading-original-html = Ansicht des Original-HTML
admin-features-feature-reading-original-html-help = Erlaubt Benutzern, für einzelne Nachrichten das Original-HTML im isolierten Leserframe zu aktivieren. Bereinigtes HTML bleibt der Standard.
admin-features-inherit = Erben
admin-features-override = Wert setzen
admin-features-setting-mode = Wertmodus für { $setting }
admin-features-inherit-help = „Erben“ entfernt Wert und Sperre dieses Profils. Das nächste anwendbare Profil, die Domain oder der Serverwert entscheidet; die Vorschau der wirksamen Werte zeigt das Ergebnis und seine Quelle.
admin-features-domain-alias = { $alias } (Alias von { $domain })
admin-features-domain-aliases = { $domain } (Aliase: { $aliases })
