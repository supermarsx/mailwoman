# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: nl · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Snelheid
privacy-speed-unavailable = Snelheidscijfers zijn niet beschikbaar van deze server.
privacy-speed-empty = In dit bereik zijn nog geen afbeeldingen geleverd.
privacy-speed-off = Snelheidsregistratie staat uit (instelling: Snelheidsregistratie van de afbeeldingsproxy). Er wordt niets nieuws vastgelegd.
privacy-speed-verdict-healthy = Gezond
privacy-speed-verdict-slow = Traag
privacy-speed-verdict-degraded = Verslechterd
privacy-speed-verdict-noData = Onvoldoende gegevens
privacy-speed-reason-ok = Opgehaalde afbeeldingen en cachetreffers blijven binnen de onderstaande limieten en weinig verzoeken zijn mislukt.
privacy-speed-reason-failureRate = { $failurePct } % van de verzoeken eindigde met bezet, een time-out of een upstreamfout.
privacy-speed-reason-missP95 = Afbeeldingen die opgehaald moesten worden zijn traag: hun 95e percentiel ligt boven de onderstaande limiet.
privacy-speed-reason-hitP95 = Gecachete afbeeldingen reageren traag: hun 95e percentiel ligt boven de onderstaande limiet.
privacy-speed-reason-fewRequests = Minder dan { $min } verzoeken in dit bereik, te weinig om te beoordelen.
privacy-speed-thresholds = Traag vanaf: 95e percentiel van opgehaalde afbeelding { $slowMiss }, 95e percentiel van gecachete afbeelding { $slowHit }, of { $slowFail } % bezet, time-out of mislukt. Verslechterd vanaf: 95e percentiel van opgehaalde afbeelding { $degradedMiss } of { $degradedFail } % mislukt. Vaste limieten, geen instellingen.
privacy-speed-median = Mediaan
privacy-speed-p95 = 95e percentiel
privacy-speed-p99 = 99e percentiel
privacy-speed-max = Langzaamst
privacy-speed-requests = Gemeten verzoeken
privacy-speed-busy = Bezet-antwoorden
privacy-speed-timeouts = Time-outs
privacy-speed-upstream-errors = Upstreamfouten
privacy-speed-hit = Cachetreffers
privacy-speed-miss = Opgehaalde afbeeldingen (cachemissers)
privacy-speed-split-count = { $count } verzoeken
privacy-speed-split-times = Mediaan { $p50 }, 95e percentiel { $p95 }, langzaamst { $max }.
privacy-speed-chart = Tijd om een afbeelding te leveren
privacy-speed-chart-miss = Tijd om een opgehaalde afbeelding te leveren
privacy-speed-chart-unit = Van ontvangen verzoek tot klaar antwoord
privacy-speed-phases = Waar de tijd naartoe gaat
privacy-speed-phases-help = Voor verzoeken die geen cachetreffer waren: hoe lang elke stap duurde en welk aandeel die had in hun totale tijd.
privacy-speed-phase = Stap
privacy-speed-share = Aandeel van de tijd
privacy-speed-phase-queue = Wachten op capaciteit
privacy-speed-phase-cache = Cache-opzoeking
privacy-speed-phase-upstream = Upstream ophalen
privacy-speed-phase-decode = Decoderen en opnieuw coderen
privacy-speed-phase-other = Al het overige
privacy-speed-note = Alleen gemeten op dit serverproces, in het geheugen bewaard en leeg na een herstart. Percentielen zijn nauwkeurig tot op { $error } %. Er worden geen URL's, hosts of accounts opgeslagen.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-work-coalesced = Gezamenlijke ophaalacties en prioriteitsverzoeken

privacy-work-coalesced-series = Fetches samengevoegd met één tijdens de vlucht

privacy-work-handle = Behandel het cachetrefferpercentage

privacy-work-handle-series = Verwerk cachehits

privacy-work-per-interval = Per bemonsteringsinterval

privacy-work-percent = Aandeel van handle-zoekopdrachten

privacy-work-priority-series = Prioritaire verzoeken

privacy-work-resized = Afbeeldingen verkleind

privacy-work-resized-series = Formaat gewijzigd

privacy-work-revalidated-series = Opnieuw gevalideerd

privacy-work-saved = Bytes bespaard door downscaling

privacy-work-saved-series = Bytes opgeslagen

privacy-work-stale = Oude afbeeldingen weergegeven en opnieuw gevalideerd

privacy-work-stale-series = Oud geserveerd

privacy-work-title = Werk vermeden en hervormd
