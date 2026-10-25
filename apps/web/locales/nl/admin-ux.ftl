# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = Sluiten

# -- Page states (UX11) ---------------------------------------------------------
# $reason — a sentence saying why the data is not there.
admin-ux-state-unavailable = Niet beschikbaar — { $reason }
admin-ux-unavailable-default = deze server biedt dit nog niet aan.
# $code — the server's error code or "HTTP <status>".
admin-ux-state-code = Foutcode: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Openen
admin-ux-list-back = Terug naar de lijst

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = Geen niet-opgeslagen wijzigingen
admin-ux-form-changes =
    { $count ->
        [one] 1 niet-opgeslagen wijziging
       *[other] { $count } niet-opgeslagen wijzigingen
    }
admin-ux-form-discard = Wijzigingen verwerpen
admin-ux-form-saving = Opslaan…
admin-ux-form-saved =
    { $count ->
        [one] 1 wijziging opgeslagen.
       *[other] { $count } wijzigingen opgeslagen.
    }
admin-ux-form-fix =
    { $count ->
        [one] Er is niets opgeslagen. Corrigeer het hieronder gemarkeerde veld.
       *[other] Er is niets opgeslagen. Corrigeer de { $count } hieronder gemarkeerde velden.
    }
admin-ux-learn-more = Meer informatie
# Added for screen readers after "Learn more". $label — the field label.
admin-ux-learn-more-field = over { $label }
admin-ux-propagation-hot = Geldt binnen 30 seconden in elk serverproces.
admin-ux-propagation-restart = Geldt na een herstart van de server.
admin-ux-secret-set = Er is een waarde opgeslagen. Typ een nieuwe om deze te vervangen.
admin-ux-secret-unset = Er is geen waarde opgeslagen.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Voer een waarde in.
admin-ux-err-number = Voer een getal in.
admin-ux-err-whole = Voer een geheel getal in.
# $min, $max — the bounds the server declares for this field.
admin-ux-err-min = Voer { $min } of meer in.
admin-ux-err-max = Voer { $max } of minder in.
admin-ux-err-max-length = Gebruik maximaal { $max } tekens.
admin-ux-err-url = Voer een volledig adres in dat begint met https:// of http://.
admin-ux-err-email = Voer een e-mailadres in, zoals name@example.org.
admin-ux-err-pattern = Deze waarde heeft niet de verwachte vorm.
admin-ux-err-option = Kies een van de vermelde opties.
admin-ux-err-out-of-range = De server heeft deze waarde geweigerd omdat die buiten het toegestane bereik valt.
admin-ux-err-too-long = De server heeft deze waarde geweigerd omdat die te lang is.
admin-ux-err-invalid = De server heeft deze waarde geweigerd. Controleer de waarde en sla opnieuw op.
admin-ux-err-pinned = Deze waarde is ingesteld in de serveromgeving en kan hier niet worden gewijzigd.
admin-ux-err-locked = Deze waarde is vergrendeld op een breder niveau en kan hier niet worden gewijzigd.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Standaard
admin-ux-source-stored = Opgeslagen
# $name — the environment variable, e.g. MW_LOG_LEVEL.
admin-ux-source-env-pin = Ingesteld door { $name }
admin-ux-source-env-fallback = Terugvalwaarde uit { $name }
admin-ux-source-build = Ingesteld bij het bouwen
# $scope — "global" or a domain name.
admin-ux-source-locked = Vergrendeld door { $scope }
admin-ux-source-read-only = Alleen-lezen
admin-ux-source-restart = Herstart nodig
admin-ux-source-env-pin-reason = De serveromgeving stelt deze waarde in met { $name }. Wijzig de waarde daar en herstart de server.
admin-ux-source-build-reason = Deze waarde wordt vastgelegd wanneer de server wordt gebouwd.
admin-ux-source-locked-reason = Een vergrendeling op niveau { $scope } legt deze waarde vast. Verwijder de vergrendeling daar om de waarde te wijzigen.
admin-ux-source-read-only-reason = De console toont deze waarde maar kan die niet wijzigen. De documentatie vermeldt waar de waarde wordt ingesteld.

# -- History (UX8) ------------------------------------------------------------
# $who — an admin name; $time — date and time in the reader's locale.
admin-ux-history-changed-by-at = Laatst gewijzigd door { $who } op { $time }
admin-ux-history-changed-by = Laatst gewijzigd door { $who }
admin-ux-history-changed-at = Laatst gewijzigd op { $time }
admin-ux-history-never = Niet gewijzigd via de console
admin-ux-history-link = Geschiedenis

# -- Revert to default (UX7) --------------------------------------------------
# $value — the default as shown, e.g. "587".
admin-ux-revert-default-is = Standaard: { $value }
admin-ux-revert = Standaardwaarde herstellen
# Added for screen readers after "Revert to default". $label — the field label.
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Standaardwaarde ({ $value }) hersteld.

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } heeft dit op { $time } gewijzigd terwijl je aan het bewerken was.
admin-ux-stale-by = { $who } heeft dit gewijzigd terwijl je aan het bewerken was.
admin-ux-stale-unknown = Iemand anders heeft dit gewijzigd terwijl je aan het bewerken was.
admin-ux-stale-choice = Laad opnieuw om hun versie te zien en je wijzigingen te verliezen, of behoud de jouwe en sla die op over die van hen.
admin-ux-stale-reload = Opnieuw laden
admin-ux-stale-keep = De mijne behouden

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Controleren wat deze wijziging beïnvloedt…
admin-ux-impact-heading = Deze wijziging beïnvloedt
# $label — what is affected (e.g. "Users refused"); $count — how many.
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] en nog 1
       *[other] en nog { $count }
    }
admin-ux-impact-none = Verder verandert er niets.
admin-ux-impact-unavailable = Impactvoorbeeld niet beschikbaar — de server heeft niet gemeld wat deze wijziging beïnvloedt.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filter
admin-ux-stream-pause = Pauzeren
admin-ux-stream-resume = Hervatten
admin-ux-stream-clear = Wissen
admin-ux-stream-announce = Nieuwe items voorlezen
admin-ux-stream-connecting = Verbinden…
admin-ux-stream-live = Live
admin-ux-stream-ended = Beëindigd
admin-ux-stream-lost = Verbinding verbroken
admin-ux-stream-reconnect = Opnieuw verbinden
admin-ux-stream-paused =
    { $count ->
        [0] Gepauzeerd
        [one] Gepauzeerd — 1 nieuw item vastgehouden
       *[other] Gepauzeerd — { $count } nieuwe items vastgehouden
    }
admin-ux-stream-no-match = Geen enkel item komt overeen met het filter.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Veld

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Weergave
admin-ux-appearance-title = Weergave van de console
admin-ux-appearance-note = Deze keuzes wijzigen alleen de beheerconsole, in deze browser.
admin-ux-appearance-following = De console volgt het webmailthema van deze browser totdat je hier een keuze maakt. Deze keuzes wijzigen alleen de beheerconsole, in deze browser.
admin-ux-appearance-mode = Modus
admin-ux-appearance-mode-light = Licht
admin-ux-appearance-mode-dark = Donker
admin-ux-appearance-mode-system = Systeem volgen
admin-ux-appearance-theme = Thema
admin-ux-appearance-accent = Accentkleur
admin-ux-appearance-light = Licht
admin-ux-appearance-dark = Donker
admin-ux-appearance-reset = Het webmailthema gebruiken
