# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Kerncijfers
admin-stats-charts = Grafieken
admin-stats-tables = Tabellen
admin-stats-group-traffic = Verkeer in deze periode
admin-stats-group-storage = Opslag en bezorging, nu
admin-stats-refresh = Statistieken vernieuwen
admin-stats-loading = Statistieken laden…
admin-stats-requests-total = Verzoeken
admin-stats-errors-total = Serverfouten (5xx)
admin-stats-p95-peak = Traagste 95e percentiel
admin-stats-requests = Verzoeken
admin-stats-errors = Serverfouten (5xx)
admin-stats-not-found = Niet gevonden (404)
admin-stats-latency = Responstijd
admin-stats-series-p50 = Mediaan
admin-stats-series-p95 = 95e percentiel
admin-stats-unit-count = Aantal per { $bucket }
admin-stats-unit-duration = Tijd per { $bucket }, in de getoonde eenheid
admin-stats-unit-accounts = Geblokkeerde accounts per domein
admin-stats-chart-about = Lijngrafiek van { $from } tot { $to }.
admin-stats-bars-about = { $count ->
    [one] Staafdiagram met één domein.
   *[other] Staafdiagram met { $count } domeinen.
}
admin-stats-view-table = Als tabel bekijken
admin-stats-legend = Legenda
admin-stats-empty-chart = In deze periode is nog niets gemeten. Kies een langere periode, of kom terug zodra de server enkele verzoeken heeft afgehandeld.
admin-stats-empty-blocked = Er zijn geen accounts geblokkeerd.
admin-stats-delta-vs = t.o.v. vorige periode
admin-stats-delta-same = Gelijk aan de vorige periode
admin-stats-delta-sr-up = { $change } hoger dan in de vorige periode.
admin-stats-delta-sr-down = { $change } lager dan in de vorige periode.
admin-stats-delta-sr-same = Gelijk aan de vorige periode.
admin-ct-group = Laadtijden van de client
admin-ct-metric-timeToUsable = Tijd tot bruikbaar
admin-ct-metric-ttfb = Eerste byte
admin-ct-metric-dcl = DOM gereed
admin-ct-metric-load = Pagina geladen
admin-ct-metric-fcp = First contentful paint
admin-ct-metric-lcp = Largest contentful paint
admin-ct-metric-folderOpen = Map openen
admin-ct-metric-folderLoadCold = Map laden, via het netwerk
admin-ct-metric-folderLoadCached = Map laden, uit cache
admin-ct-metric-messageLoadCold = Bericht laden, via het netwerk
admin-ct-metric-messageLoadCached = Bericht laden, uit cache of prefetch
admin-ct-device-desktop = Desktop
admin-ct-device-mobile = Mobiel
admin-ct-device-unknown = Onbekend
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } meting
       *[other] { $count } metingen
    }
admin-ct-no-samples = Geen metingen in deze periode.
admin-ct-few-samples = Slechts { $count } metingen; er zijn er minimaal { $min } nodig.
admin-ct-chart-usable = Tijd tot bruikbaar
admin-ct-group-folder = Laadtijden van mappen
admin-ct-group-message = Laadtijden van berichten
admin-ct-chart-folder-cold = Map laden via het netwerk
admin-ct-chart-folder-cached = Map laden uit cache
admin-ct-chart-message-cold = Bericht laden via het netwerk
admin-ct-chart-message-cached = Bericht laden uit cache of prefetch
admin-ct-folder-off = Laadtijden van mappen staan standaard uit. Alleen mensen die ze hebben ingeschakeld worden gemeten.
admin-ct-message-off = Laadtijden van berichten staan standaard uit. Alleen mensen die ze hebben ingeschakeld worden gemeten.
admin-ct-chart-note = Percentielen worden alleen getoond voor intervallen met minstens { $min } metingen.
admin-ct-empty-chart = Geen enkel interval heeft de { $min } metingen die nodig zijn om een mediaan te tonen. Kies een langere periode.
admin-ct-col-samples = Metingen
admin-ct-col-metric = Maat
admin-ct-col-device = Apparaat
admin-ct-table-title = Laadtijden van de client per apparaat
admin-ct-table-caption = Metingen, mediaan en 95e percentiel per maat en apparaatklasse.
admin-ct-empty-table = In deze periode zijn geen laadtijden van browsers gemeld.
admin-ct-failed = De laadtijden van de client konden niet worden geladen.
admin-ct-disabled = Het verzamelen van laadtijden van de client staat uit. De onderstaande cijfers zijn van vóór het uitschakelen.
admin-ct-note = Alleen van ingelogde browsers, { $rate }% van de sessies gesampled, anoniem en geaggregeerd. Tijd tot bruikbaar loopt van het begin van de navigatie tot de berichtenlijst haar eerste rijen toont. De cijfers zijn schattingen, binnen ongeveer { $error }% van de werkelijke waarde.
