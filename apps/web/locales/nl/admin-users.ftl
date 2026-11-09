# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = Gebruikers. Met de kolomkoppen sorteert u de lijst.
admin-userlist-search = Gebruikers zoeken
admin-userlist-search-placeholder = Naam, adres, alias of domein
admin-userlist-count = { $count ->
    [one] { $count } gebruiker
   *[other] { $count } gebruikers
 }
admin-userlist-count-filtered = { $count ->
    [one] { $count } van { $total } gebruikers komt overeen
   *[other] { $count } van { $total } gebruikers komen overeen
 }
admin-userlist-range = { $from }–{ $to } van { $total }
admin-userlist-loading = Gebruikers laden…
admin-userlist-retry = Opnieuw proberen
admin-userlist-empty-filtered = Geen gebruikers komen overeen met deze zoekopdracht en filters.
admin-userlist-compact = Compacte rijen
admin-userlist-columns = Kolommen
admin-userlist-columns-legend = Kolommen weergeven
admin-userlist-actions = Acties
admin-userlist-paging = Pagina's met gebruikers
admin-userlist-page-size = Rijen per pagina
admin-userlist-prev = Vorige
admin-userlist-next = Volgende
admin-userlist-sort-by = Sorteren op
admin-userlist-sort-asc = { $column }, oplopend
admin-userlist-sort-desc = { $column }, aflopend
admin-userlist-col-user = Gebruiker
admin-userlist-col-domain = Domein
admin-userlist-col-status = Status
admin-userlist-col-2fa = Tweestapsverificatie
admin-userlist-col-role = Consolerol
admin-userlist-col-last-signin = Laatste aanmelding
admin-userlist-col-last-seen = Laatst gezien
admin-userlist-col-created = Aangemaakt
admin-userlist-col-quota = Quotum
admin-userlist-col-profile = Functieprofiel
admin-userlist-col-linked = Gekoppelde mailboxen
admin-userlist-state-active = Actief
admin-userlist-state-pending = Nooit aangemeld
admin-userlist-state-blocked = Geblokkeerd
admin-userlist-state-disabled = Uitgeschakeld
admin-userlist-state-deleted = Verwijderd
admin-userlist-2fa-passkey = Passkey
admin-userlist-2fa-totp = Authenticator-app
admin-userlist-2fa-none = Geen
admin-userlist-signin-never = Nooit
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = Geen activiteit geregistreerd
admin-userlist-seen-at = Laatst gezien { $relative } ({ $exact })
admin-userlist-quota-unlimited = Onbeperkt
admin-userlist-profile-default = Standaard
admin-userlist-profile-via-user = Aan deze gebruiker toegewezen
admin-userlist-profile-via-group = Via een groep
admin-userlist-profile-via-domain = Via het domein
admin-userlist-more = Meer acties
admin-userlist-more-for = Meer acties voor { $account }
admin-userlist-select = { $account } selecteren
admin-userlist-select-page = Alle gebruikers op deze pagina selecteren
admin-userlist-filters = Filters
admin-userlist-add-filter = Filter toevoegen
admin-userlist-clear-all = Alles wissen
admin-userlist-chip-edit = Filter { $label } bewerken
admin-userlist-chip-remove = Filter { $label } verwijderen
admin-userlist-filter-domain = Domein
admin-userlist-filter-status = Status
admin-userlist-filter-tfa = Tweestapsverificatie
admin-userlist-filter-role = Consolerol
admin-userlist-filter-profile = Functieprofiel
admin-userlist-filter-signin = Laatste aanmelding
admin-userlist-filter-created = Aangemaakt
admin-userlist-filter-no-choices = Nog niets om uit te kiezen.
admin-userlist-filter-remove = Filter verwijderen
admin-userlist-apply = Toepassen
admin-userlist-cancel = Annuleren
admin-userlist-role-any = Elke consolerol
admin-userlist-role-none = Geen consolerol
admin-userlist-profile-none = Geen profiel (standaard)
admin-userlist-signin-older = { $days ->
    [one] Meer dan { $days } dag geleden
   *[other] Meer dan { $days } dagen geleden
 }
admin-userlist-signin-within = { $days ->
    [one] In de afgelopen { $days } dag
   *[other] In de afgelopen { $days } dagen
 }
admin-userlist-signin-custom = Langer dan dit aantal dagen geleden:
admin-userlist-signin-days = Dagen
admin-userlist-days-invalid = Voer een geheel aantal dagen in van 1 tot 36500.
admin-userlist-created-from = Van
admin-userlist-created-to = Tot
admin-userlist-created-between = { $from } tot { $to }
admin-userlist-created-since = sinds { $date }
admin-userlist-created-until = tot { $date }
admin-userlist-dates-invalid = De begindatum ligt na de einddatum.
admin-userlist-views = Opgeslagen weergaven
admin-userlist-view-save = Deze weergave opslaan…
admin-userlist-view-name = Naam van de weergave
admin-userlist-view-save-confirm = Weergave opslaan
admin-userlist-view-delete = Weergave “{ $name }” verwijderen
admin-userlist-bulk = Acties voor de geselecteerde gebruikers
admin-userlist-selected = { $count ->
    [one] { $count } geselecteerd
   *[other] { $count } geselecteerd
 }
admin-userlist-clear-selection = Selectie wissen
admin-userlist-block = Aanmelden blokkeren
admin-userlist-unblock = Aanmelden deblokkeren
admin-userlist-sign-out = Overal afmelden
admin-userlist-delete = Lokale toegang verwijderen
admin-userlist-bulk-profile = Functieprofiel toewijzen
admin-userlist-bulk-profile-clear = Eigen profiel van de gebruiker verwijderen
admin-userlist-confirm-skipped = { $count ->
    [one] { $count } geselecteerde gebruiker wordt overgeslagen: de actie is hierop niet van toepassing.
   *[other] { $count } geselecteerde gebruikers worden overgeslagen: de actie is hierop niet van toepassing.
 }
admin-userlist-confirm-block = { $count ->
    [one] Aanmelden blokkeren voor { $count } gebruiker?
   *[other] Aanmelden blokkeren voor { $count } gebruikers?
 }
admin-userlist-confirm-block-one = Aanmelden blokkeren voor { $account }?
admin-userlist-confirm-block-body = Nieuwe aanmeldingen worden geweigerd en huidige Mailwoman-sessies worden beëindigd. De mailbox bij de upstream-server wordt niet gewijzigd.
admin-userlist-confirm-block-action = Aanmelden blokkeren
admin-userlist-confirm-unblock = { $count ->
    [one] Aanmelden deblokkeren voor { $count } gebruiker?
   *[other] Aanmelden deblokkeren voor { $count } gebruikers?
 }
admin-userlist-confirm-unblock-one = Aanmelden deblokkeren voor { $account }?
admin-userlist-confirm-unblock-body = De lokale aanmeldblokkade wordt opgeheven. Andere beveiligingsbeperkingen blijven gelden.
admin-userlist-confirm-unblock-action = Aanmelden deblokkeren
admin-userlist-confirm-signout = { $count ->
    [one] { $count } gebruiker overal afmelden?
   *[other] { $count } gebruikers overal afmelden?
 }
admin-userlist-confirm-signout-one = { $account } overal afmelden?
admin-userlist-confirm-signout-body = Alle open Mailwoman-sessies van deze gebruikers worden beëindigd. Ze kunnen zich opnieuw aanmelden.
admin-userlist-confirm-signout-action = Afmelden
admin-userlist-confirm-delete = { $count ->
    [one] Lokale toegang verwijderen voor { $count } gebruiker?
   *[other] Lokale toegang verwijderen voor { $count } gebruikers?
 }
admin-userlist-confirm-delete-one = Lokale toegang verwijderen voor { $account }?
admin-userlist-confirm-delete-body = Een verwijderd toegangsrecord blijft bewaard, zodat de gebruiker niet automatisch opnieuw wordt ingeschreven. Mailwoman-instellingen, geschiedenis en de upstream-mailbox blijven opgeslagen. Typ het adres ter bevestiging.
admin-userlist-confirm-delete-action = Lokale toegang verwijderen
admin-userlist-confirm-profile = { $count ->
    [one] Het functieprofiel van { $count } gebruiker wijzigen?
   *[other] Het functieprofiel van { $count } gebruikers wijzigen?
 }
admin-userlist-confirm-profile-one = Het functieprofiel van { $account } wijzigen?
admin-userlist-confirm-profile-body = Het profiel wordt rechtstreeks aan elke gebruiker toegewezen en geldt vanaf het volgende verzoek. Native, proxy- en SSO-mailboxidentiteiten worden ondersteund.
admin-users-profile-help = Selecteer Standaard om de directe toewijzing van deze gebruiker te verwijderen en de profielen te gebruiken die van hun groepen of domein worden geërfd.
admin-users-profile-ambiguous = Deze gebruiker heeft meerdere aanmeldidentiteiten. Kies de bedoelde identiteit bij Functieprofielen.
admin-users-profile-unavailable = Er is geen geverifieerde mailboxidentiteit beschikbaar. Deze gebruiker moet zich eerst aanmelden voordat een profiel kan worden toegewezen.
admin-userlist-confirm-profile-action = Profiel wijzigen
admin-userlist-bulk-done-block = { $count ->
    [one] Aanmelden geblokkeerd voor { $count } gebruiker.
   *[other] Aanmelden geblokkeerd voor { $count } gebruikers.
 }
admin-userlist-bulk-done-unblock = { $count ->
    [one] Aanmelden gedeblokkeerd voor { $count } gebruiker.
   *[other] Aanmelden gedeblokkeerd voor { $count } gebruikers.
 }
admin-userlist-bulk-done-signout = { $count ->
    [one] { $count } gebruiker afgemeld.
   *[other] { $count } gebruikers afgemeld.
 }
admin-userlist-bulk-done-delete = { $count ->
    [one] Lokale toegang verwijderd voor { $count } gebruiker.
   *[other] Lokale toegang verwijderd voor { $count } gebruikers.
 }
admin-userlist-bulk-done-profile = { $count ->
    [one] Functieprofiel van { $count } gebruiker gewijzigd.
   *[other] Functieprofiel van { $count } gebruikers gewijzigd.
 }
admin-userlist-bulk-failed = { $done } voltooid, { $failed } mislukt: { $reason }

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-user-compliance-due = verschuldigd { $date }

admin-user-compliance-empty = Geen gegevensverzoeken voor dit account.

admin-user-compliance-help = Verzoeken ingediend voor de adressen van dit account. Open er een in Gegevensverzoeken om ernaar te handelen.

admin-user-compliance-open-all = Open alle gegevensverzoeken

admin-user-compliance-title = Gegevensverzoeken voor dit account

admin-user-never = Nooit

admin-user-overview-last-signin = Laatste aanmelding

admin-user-overview-status = Status

admin-user-status-active = Actief

admin-user-status-deleted = Verwijderd

admin-user-status-deleting = Verwijderen

admin-user-status-disabled = Geblokkeerd

admin-user-tab-access = Toegang en verwijdering

admin-user-tab-compliance = Naleving

admin-user-tab-overview = Overzicht

admin-user-tab-profile = Functieprofiel

admin-user-tab-security = Beveiliging

admin-user-tab-sessions = Sessies

admin-user-tabs-label = Accountsecties
