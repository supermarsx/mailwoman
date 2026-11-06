# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = Niet gevonden (404) per interval
admin-history-nf-total = Niet gevonden (404) in deze periode
admin-history-nf-share = { $share } van de verzoeken
admin-history-nf-not-tracked-window = 404's zijn in deze periode niet bijgehouden.
admin-history-nf-not-tracked-before = 404's zijn vóór { $time } niet bijgehouden. Eerdere periodes worden als niet beschikbaar getoond, niet als nul.
admin-history-nf-paths-title = Vaakst voorkomende niet-gevonden paden
admin-history-nf-paths-help = Maximaal { $limit } paden per uur, van { $from } tot { $to }, afgerond naar hele uren. Querystrings worden verwijderd en e-mailadressen, getallen, id's, tokens en lange waarden worden vervangen door tijdelijke aanduidingen zoals {"{"}token{"}"}. De eigen verwachte missers van de app, zoals verzoeken om browserpictogrammen, worden niet meegeteld.
admin-history-nf-paths-approximate = Er kwamen meer verschillende paden binnen dan er worden bewaard, dus sommige aantallen zijn een bereik in plaats van een exact getal.
admin-history-nf-paths-empty = Geen niet-gevonden-antwoorden in deze periode.
admin-history-nf-paths-error = De niet-gevonden-paden konden niet worden geladen. Vernieuw om het opnieuw te proberen.
admin-history-nf-paths-method = Methode
admin-history-nf-paths-path = Pad
admin-history-nf-paths-count = Aantal
admin-history-nf-paths-range = { $min } tot { $max }
admin-history-nf-paths-client = Van de eigen clients van de app
admin-history-nf-paths-client-help = Paden die de webapp, haar service worker of de desktop- en mobiele apps opvragen: buildbestanden, het workerscript, API-, JMAP-, DAV- en consoleroutes. Een misser hier betekent meestal dat een client verouderd is of dat een route niet klopt.
admin-history-nf-paths-external = Overig verkeer
admin-history-nf-paths-external-help = Paden die de app nooit opvraagt, zoals scanners die zoeken naar PHP-bestanden, dotfiles of andere software, en links van elders.
admin-history-nf-paths-all = Paden
admin-history-nf-paths-all-help = Deze server maakt geen onderscheid tussen clientpaden en overig verkeer.
