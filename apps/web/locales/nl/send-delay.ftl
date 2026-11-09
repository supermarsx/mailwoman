# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = Verzenden ongedaan maken
send-delay-help = Nadat u op Verzenden hebt gedrukt, wacht het bericht zo lang in uw Postvak UIT. Tot die tijd kunt u het terughalen en verder bewerken. De server bewaart het, dus als u de browser sluit, wordt het niet te vroeg verzonden en gaat het niet verloren.
send-delay-label = Verzonden berichten vasthouden gedurende
send-delay-off = Uit (direct verzenden)
send-delay-seconds = { $n } seconden
send-delay-minutes = { $n ->
    [one] 1 minuut
   *[other] { $n } minuten
}
send-delay-custom = Aangepast…
send-delay-custom-label = Aangepaste vertraging in seconden
send-delay-custom-help = Hele seconden, 0 tot { $max }.
send-delay-capped = Uw beheerder beperkt dit tot { $max }.
send-delay-disabled = Het terughalen van verzonden berichten is door uw beheerder uitgeschakeld. Berichten worden direct verzonden.
send-delay-invalid = Voer een geheel aantal seconden in van 0 tot { $max }.
send-later-button = Later verzenden
send-later-menu = Opties voor later verzenden
send-later-preset-hour = Over 1 uur
send-later-preset-today = Later vandaag, 17:00
send-later-preset-tomorrow = Morgenochtend, 09:00
send-later-preset-monday = Maandagochtend, 09:00
send-later-custom = Datum en tijd kiezen…
send-later-custom-label = Verzenden op
send-later-zone = Tijdzone: { $zone }
send-later-schedule = Verzending plannen
send-later-cancel = Annuleren
send-later-err-invalid = Voer een geldige datum en tijd in.
send-later-err-past = Kies een tijdstip in de toekomst.
send-later-err-horizon = Dat is te ver vooruit. Berichten kunnen maximaal { $days } dagen van tevoren worden gepland.
send-later-err-off = Geplande verzending is door uw beheerder uitgeschakeld.
send-queued = Bericht in de wachtrij — wordt verzonden over
send-queued-undo = Ongedaan maken
send-queued-now = Nu verzenden
send-queued-recalled = Bericht teruggehaald. Het staat weer in uw concepten.
send-queued-too-late = Te laat om terug te halen: het bericht wordt al verzonden.
send-queued-sent-now = Wordt nu verzonden.
send-queued-scheduled = Gepland voor { $when } ({ $zone }).
outbox-scheduled-badge = Gepland
outbox-sends-in = Wordt verzonden over { $time }
outbox-sends-at = Wordt verzonden { $when }
outbox-recall-edit = Terughalen en bewerken
outbox-recall-too-late = Wordt al verzonden; kan niet meer worden teruggehaald.
outbox-failed-drafts = Kon niet worden verzonden en is bewaard in uw concepten.
outbox-queued-count = { $count ->
    [one] 1 bericht wacht op verzending
   *[other] { $count } berichten wachten op verzending
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = Volgende verzending: { $when }

outbox-chip-title = Berichten die wachten om te worden verzonden

outbox-chip-view-all = Open Postvak UIT

outbox-delete = Verwijderen

outbox-delete-confirm = Bericht verwijderen?

outbox-delete-failed = Het bericht kon niet worden verwijderd. { $error }

outbox-deleted = Het bericht in de wachtrij is verwijderd.

outbox-edit-time = Verzendtijd bewerken

outbox-edit-time-conflict = De verzendtijd is gewijzigd of het bericht is al verzonden. De Outbox is opnieuw geladen.

outbox-edit-time-current = Momenteel { $when }

outbox-edit-time-refused = De verzendtijd kon niet worden gewijzigd. { $error }

outbox-edit-time-save = Bespaar verzendtijd

outbox-failed-toast = Er kon geen bericht worden verzonden. { $error }

outbox-failed-uncertain-toast = Het kan zijn dat een bericht wel of niet is afgeleverd. Vink Verzonden aan voordat u het opnieuw verzendt.

outbox-open = Openen

outbox-reader-actions = Berichtacties in de wachtrij

outbox-reader-cc = Cc: { $recipients }

outbox-reader-label = Bericht in wachtrij

outbox-reader-loading = Het bericht laden...

outbox-reader-to = Aan: { $recipients }

outbox-reader-unavailable = Het bericht kon niet worden geladen.

outbox-recall-to-drafts = Terugroepen naar concepten

outbox-send-again = Opnieuw verzenden

outbox-send-again-failed = Kan het bericht niet opnieuw in de wachtrij plaatsen. { $error }

outbox-send-again-queued = Het bericht wordt in de wachtrij geplaatst om opnieuw te worden verzonden.

outbox-sent-copy-folder = Wordt opgeslagen in { $folder }

outbox-sent-copy-missing = Wordt opgeslagen in Verzonden (de gekozen map is niet beschikbaar)

outbox-sent-copy-skip = Er wordt geen kopie opgeslagen

outbox-status-attempts = { $count ->
        [one] Tot nu toe { $count } poging
       *[other] { $count } pogingen tot nu toe
    }

outbox-status-canceled = Geannuleerd

outbox-status-failed = Niet verzonden

outbox-status-held = Wordt vastgehouden totdat je het loslaat

outbox-status-last-error = Laatste fout: { $error }

outbox-status-next-attempt = Volgende poging { $when }

outbox-status-queued = In de wachtrij

outbox-status-retrying = Opnieuw proberen

outbox-status-scheduled = Gepland

outbox-status-sending = Verzenden

outbox-status-sends-at-in = Verzendt { $when } (in { $time })

outbox-status-sent = Verzonden

outbox-status-uncertain = Levering onbekend
