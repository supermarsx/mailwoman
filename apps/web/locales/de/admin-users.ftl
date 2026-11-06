# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: de · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Benutzer. Spaltenüberschriften sortieren die Liste.
admin-userlist-search = Benutzer suchen
admin-userlist-search-placeholder = Name, Adresse, Alias oder Domain
admin-userlist-count = { $count ->
    [one] { $count } Benutzer
   *[other] { $count } Benutzer
}
admin-userlist-count-filtered = { $count ->
    [one] { $count } von { $total } Benutzern passt
   *[other] { $count } von { $total } Benutzern passen
}
admin-userlist-range = { $from }–{ $to } von { $total }
admin-userlist-loading = Benutzer werden geladen…
admin-userlist-retry = Erneut versuchen
admin-userlist-empty-filtered = Kein Benutzer entspricht dieser Suche und diesen Filtern.
admin-userlist-compact = Kompakte Zeilen
admin-userlist-columns = Spalten
admin-userlist-columns-legend = Spalten anzeigen
admin-userlist-actions = Aktionen
admin-userlist-paging = Seiten der Benutzerliste
admin-userlist-page-size = Zeilen pro Seite
admin-userlist-prev = Zurück
admin-userlist-next = Weiter
admin-userlist-sort-by = Sortieren nach
admin-userlist-sort-asc = { $column }, aufsteigend
admin-userlist-sort-desc = { $column }, absteigend
admin-userlist-col-user = Benutzer
admin-userlist-col-domain = Domain
admin-userlist-col-status = Status
admin-userlist-col-2fa = Zwei-Faktor
admin-userlist-col-role = Konsolenrolle
admin-userlist-col-last-signin = Letzte Anmeldung
admin-userlist-col-last-seen = Zuletzt gesehen
admin-userlist-col-created = Erstellt
admin-userlist-col-quota = Kontingent
admin-userlist-col-profile = Funktionsprofil
admin-userlist-col-linked = Verknüpfte Postfächer
admin-userlist-state-active = Aktiv
admin-userlist-state-pending = Nie angemeldet
admin-userlist-state-blocked = Gesperrt
admin-userlist-state-disabled = Deaktiviert
admin-userlist-state-deleted = Gelöscht
admin-userlist-2fa-passkey = Passkey
admin-userlist-2fa-totp = Authenticator-App
admin-userlist-2fa-none = Keine
admin-userlist-signin-never = Nie
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Keine Aktivität erfasst
admin-userlist-seen-at = Zuletzt gesehen { $relative } ({ $exact })
admin-userlist-quota-unlimited = Unbegrenzt
admin-userlist-profile-default = Standard
admin-userlist-profile-via-user = Diesem Benutzer zugewiesen
admin-userlist-profile-via-group = Aus einer Gruppe
admin-userlist-profile-via-domain = Aus der Domain
admin-userlist-more = Weitere Aktionen
admin-userlist-more-for = Weitere Aktionen für { $account }
admin-userlist-select = { $account } auswählen
admin-userlist-select-page = Alle Benutzer dieser Seite auswählen
admin-userlist-filters = Filter
admin-userlist-add-filter = Filter hinzufügen
admin-userlist-clear-all = Alle löschen
admin-userlist-chip-edit = Filter { $label } bearbeiten
admin-userlist-chip-remove = Filter { $label } entfernen
admin-userlist-filter-domain = Domain
admin-userlist-filter-status = Status
admin-userlist-filter-tfa = Zwei-Faktor
admin-userlist-filter-role = Konsolenrolle
admin-userlist-filter-profile = Funktionsprofil
admin-userlist-filter-signin = Letzte Anmeldung
admin-userlist-filter-created = Erstellt
admin-userlist-filter-no-choices = Noch nichts zur Auswahl.
admin-userlist-filter-remove = Filter entfernen
admin-userlist-apply = Anwenden
admin-userlist-cancel = Abbrechen
admin-userlist-role-any = Beliebige Konsolenrolle
admin-userlist-role-none = Keine Konsolenrolle
admin-userlist-profile-none = Kein Profil (Standard)
admin-userlist-signin-older = { $days ->
    [one] Vor mehr als { $days } Tag
   *[other] Vor mehr als { $days } Tagen
}
admin-userlist-signin-within = { $days ->
    [one] Am letzten { $days } Tag
   *[other] In den letzten { $days } Tagen
}
admin-userlist-signin-custom = Vor mehr als dieser Anzahl Tage:
admin-userlist-signin-days = Tage
admin-userlist-days-invalid = Geben Sie eine ganze Zahl von Tagen zwischen 1 und 36500 ein.
admin-userlist-created-from = Von
admin-userlist-created-to = Bis
admin-userlist-created-between = { $from } bis { $to }
admin-userlist-created-since = seit { $date }
admin-userlist-created-until = bis { $date }
admin-userlist-dates-invalid = Das Startdatum liegt nach dem Enddatum.
admin-userlist-views = Gespeicherte Ansichten
admin-userlist-view-save = Diese Ansicht speichern…
admin-userlist-view-name = Name der Ansicht
admin-userlist-view-save-confirm = Ansicht speichern
admin-userlist-view-delete = Ansicht „{ $name }“ löschen
admin-userlist-bulk = Aktionen für die ausgewählten Benutzer
admin-userlist-selected = { $count ->
    [one] { $count } ausgewählt
   *[other] { $count } ausgewählt
}
admin-userlist-clear-selection = Auswahl aufheben
admin-userlist-block = Anmeldung sperren
admin-userlist-unblock = Anmeldung entsperren
admin-userlist-sign-out = Überall abmelden
admin-userlist-delete = Lokalen Zugang löschen
admin-userlist-bulk-profile = Funktionsprofil zuweisen
admin-userlist-bulk-profile-clear = Eigenes Profil des Benutzers entfernen
admin-userlist-confirm-skipped = { $count ->
    [one] { $count } ausgewählter Benutzer wird übersprungen: Die Aktion trifft auf ihn nicht zu.
   *[other] { $count } ausgewählte Benutzer werden übersprungen: Die Aktion trifft auf sie nicht zu.
}
admin-userlist-confirm-block = { $count ->
    [one] Anmeldung für { $count } Benutzer sperren?
   *[other] Anmeldung für { $count } Benutzer sperren?
}
admin-userlist-confirm-block-one = Anmeldung für { $account } sperren?
admin-userlist-confirm-block-body = Neue Anmeldungen werden abgelehnt und aktuelle Mailwoman-Sitzungen enden. Das Upstream-Postfach bleibt unverändert.
admin-userlist-confirm-block-action = Anmeldung sperren
admin-userlist-confirm-unblock = { $count ->
    [one] Anmeldung für { $count } Benutzer entsperren?
   *[other] Anmeldung für { $count } Benutzer entsperren?
}
admin-userlist-confirm-unblock-one = Anmeldung für { $account } entsperren?
admin-userlist-confirm-unblock-body = Die lokale Anmeldesperre wird aufgehoben. Andere Sicherheitsbeschränkungen gelten weiterhin.
admin-userlist-confirm-unblock-action = Anmeldung entsperren
admin-userlist-confirm-signout = { $count ->
    [one] { $count } Benutzer überall abmelden?
   *[other] { $count } Benutzer überall abmelden?
}
admin-userlist-confirm-signout-one = { $account } überall abmelden?
admin-userlist-confirm-signout-body = Jede offene Mailwoman-Sitzung dieser Benutzer endet. Sie können sich erneut anmelden.
admin-userlist-confirm-signout-action = Abmelden
admin-userlist-confirm-delete = { $count ->
    [one] Lokalen Zugang für { $count } Benutzer löschen?
   *[other] Lokalen Zugang für { $count } Benutzer löschen?
}
admin-userlist-confirm-delete-one = Lokalen Zugang für { $account } löschen?
admin-userlist-confirm-delete-body = Ein gelöschter Zugangseintrag bleibt erhalten, damit der Benutzer nicht automatisch erneut registriert wird. Mailwoman-Einstellungen, Verlauf und das Upstream-Postfach bleiben gespeichert. Geben Sie zur Bestätigung die Adresse ein.
admin-userlist-confirm-delete-action = Lokalen Zugang löschen
admin-userlist-confirm-profile = { $count ->
    [one] Funktionsprofil von { $count } Benutzer ändern?
   *[other] Funktionsprofil von { $count } Benutzern ändern?
}
admin-userlist-confirm-profile-one = Funktionsprofil von { $account } ändern?
admin-userlist-confirm-profile-body = Das Profil wird jedem Benutzer direkt zugewiesen und gilt ab dessen nächster Anfrage. Native, Proxy- und SSO-Postfachidentitäten werden unterstützt.
admin-users-profile-help = Wählen Sie Standard, um die direkte Zuweisung dieses Benutzers zu entfernen und die von seinen Gruppen oder seiner Domain geerbten Profile zu verwenden.
admin-users-profile-ambiguous = Dieser Benutzer hat mehrere Anmeldeidentitäten. Wählen Sie die gewünschte Identität unter Funktionsprofile.
admin-users-profile-unavailable = Keine verifizierte Postfachidentität verfügbar. Dieser Benutzer muss sich anmelden, bevor ein Profil zugewiesen werden kann.
admin-userlist-confirm-profile-action = Profil ändern
admin-userlist-bulk-done-block = { $count ->
    [one] Anmeldung für { $count } Benutzer gesperrt.
   *[other] Anmeldung für { $count } Benutzer gesperrt.
}
admin-userlist-bulk-done-unblock = { $count ->
    [one] Anmeldung für { $count } Benutzer entsperrt.
   *[other] Anmeldung für { $count } Benutzer entsperrt.
}
admin-userlist-bulk-done-signout = { $count ->
    [one] { $count } Benutzer abgemeldet.
   *[other] { $count } Benutzer abgemeldet.
}
admin-userlist-bulk-done-delete = { $count ->
    [one] Lokalen Zugang für { $count } Benutzer gelöscht.
   *[other] Lokalen Zugang für { $count } Benutzer gelöscht.
}
admin-userlist-bulk-done-profile = { $count ->
    [one] Funktionsprofil von { $count } Benutzer geändert.
   *[other] Funktionsprofil von { $count } Benutzern geändert.
}
admin-userlist-bulk-failed = { $done } erledigt, { $failed } fehlgeschlagen: { $reason }
