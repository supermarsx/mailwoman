# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Dashboard
admin-shell-group-overview = Overzicht
admin-shell-group-people = Personen
admin-shell-group-security = Beveiliging
admin-shell-group-configuration = Configuratie
admin-shell-group-operations = Beheer
admin-shell-menu = Menu
admin-shell-close-menu = Menu sluiten
admin-shell-search = Zoeken
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Ingelogd als { $name }
admin-shell-language-label = Taal
admin-shell-breadcrumb = Kruimelpad
admin-shell-unknown-section = Dit onderdeel bestaat niet, of je rol geeft er geen toegang toe.
admin-shell-back-to-dashboard = Terug naar het dashboard
admin-shell-cap-missing = Je rol omvat { $capability } niet.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Laden…
admin-shell-retry = Opnieuw proberen
admin-shell-cancel = Annuleren
admin-shell-save = Opslaan
admin-shell-not-available = Niet beschikbaar

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Deze pagina verlaten?
admin-shell-leave-body = Je wijzigingen op deze pagina zijn niet opgeslagen. Als je de pagina verlaat, gaan ze verloren.
admin-shell-leave-confirm = Verlaten en wijzigingen verwerpen

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Typ { $name } om te bevestigen

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = De beheerconsole doorzoeken
admin-shell-palette-label = Onderdeel, gebruiker, domein of instelling
admin-shell-palette-results = Resultaten
admin-shell-palette-none = Niets gevonden.
admin-shell-palette-searching = Gebruikers, domeinen en instellingen doorzoeken…
admin-shell-palette-sections-only = Deze server kan nog niet in gebruikers, domeinen en instellingen zoeken, dus alleen onderdelen worden weergegeven.
admin-shell-palette-error = Het zoeken in gebruikers, domeinen en instellingen is mislukt; alleen onderdelen worden weergegeven.
admin-shell-palette-kind-section = Onderdeel
admin-shell-palette-kind-user = Gebruiker
admin-shell-palette-kind-domain = Domein
admin-shell-palette-kind-setting = Instelling

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filters
admin-shell-table-actions = Acties
admin-shell-table-error = De lijst kon niet worden geladen.
admin-shell-table-paging = Pagina's van { $table }
admin-shell-table-prev = Vorige pagina
admin-shell-table-next = Volgende pagina

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Laatste { $latest }, hoogste { $max }
admin-shell-chart-col-time = Tijd
admin-shell-chart-col-item = Item
admin-shell-chart-col-value = Waarde

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Ingesteld door
admin-shell-pinned-sr = Deze waarde komt uit de omgeving van de server en kan hier niet worden gewijzigd.
admin-shell-lock-text = Vergrendelen voor gebruikers
admin-shell-lock-setting = { $setting }
admin-shell-restore = Vorige waarde herstellen
admin-shell-restore-note = Gewijzigd door { $actor } op { $time }. Vorige waarde: { $value }
admin-shell-restore-done = De vorige waarde is hersteld.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Stappen
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Stap { $n } van { $total }: { $title }
admin-shell-wizard-review = Controleren
admin-shell-wizard-back = Terug
admin-shell-wizard-next = Volgende
admin-shell-wizard-saved = Opgeslagen.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = De server kon niet worden bereikt. Controleer de verbinding en probeer het opnieuw.
admin-shell-err-session = Je beheersessie is beëindigd. Log opnieuw in.
admin-shell-err-forbidden = Je rol staat deze actie niet toe. Vraag het aan een beheerder met de juiste rol.
admin-shell-err-out-of-scope = Dit object hoort bij een domein dat buiten je rol valt.
admin-shell-err-not-found = Het bestaat niet meer. Laad de lijst opnieuw.
admin-shell-err-conflict = Iemand anders heeft het intussen gewijzigd. Laad opnieuw en probeer het nog eens.
admin-shell-err-validation = De server heeft een waarde geweigerd. Controleer de velden en probeer het opnieuw.
admin-shell-err-rate-limited = Te veel verzoeken. Wacht een minuut en probeer het opnieuw.
admin-shell-err-server = De server antwoordde met een fout ({ $status }). Probeer het opnieuw; controleer het serverlogboek als het blijft gebeuren.
admin-shell-err-upstream-dns = De hostnaam kon niet worden omgezet. Controleer de spelling van de host.
admin-shell-err-upstream-connect = Er antwoordde niets op die host en poort. Controleer de poort en eventuele firewalls daartussen.
admin-shell-err-upstream-tls = De server antwoordde, maar de TLS-handshake is mislukt. Controleer de poort en de instelling voor verbindingsbeveiliging.
admin-shell-err-upstream-auth = De server heeft de inloggegevens geweigerd. Controleer de gebruikersnaam en het wachtwoord.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Te veel inlogpogingen. Wacht een paar minuten en probeer het opnieuw.
admin-shell-totp-form = Inloggen als beheerder, tweede stap
admin-shell-totp-note = Voer de 6-cijferige code in uit de authenticator-app die voor { $name } is ingesteld.
admin-shell-totp-code = Verificatiecode
admin-shell-totp-verify = Verifiëren
admin-shell-totp-start-over = Opnieuw beginnen
admin-shell-totp-invalid = Die code is niet geldig. Voer de huidige code uit je authenticator-app in.
admin-shell-totp-expired = Het inloggen duurde te lang. Begin opnieuw en voer je wachtwoord in.
