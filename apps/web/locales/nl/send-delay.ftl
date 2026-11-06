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
