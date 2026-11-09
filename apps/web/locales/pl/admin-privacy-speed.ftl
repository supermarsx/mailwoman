# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-privacy-speed. Source: ../en/admin-privacy-speed.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-speed-title = Szybkość
privacy-speed-unavailable = Dane o szybkości nie są dostępne z tego serwera.
privacy-speed-empty = W tym zakresie nie udostępniono jeszcze żadnych obrazów.
privacy-speed-off = Śledzenie szybkości jest wyłączone (ustawienie: Śledzenie szybkości proxy obrazów). Nic nowego nie jest zapisywane.
privacy-speed-verdict-healthy = Dobra
privacy-speed-verdict-slow = Wolna
privacy-speed-verdict-degraded = Pogorszona
privacy-speed-verdict-noData = Za mało danych
privacy-speed-reason-ok = Pobrane obrazy i trafienia w pamięci podręcznej mieszczą się w poniższych limitach, a niewiele żądań zakończyło się błędem.
privacy-speed-reason-failureRate = { $failurePct } % żądań zakończyło się zajętością, przekroczeniem czasu lub błędem źródła.
privacy-speed-reason-missP95 = Obrazy, które trzeba było pobrać, są wolne: ich 95. percentyl przekracza poniższy limit.
privacy-speed-reason-hitP95 = Obrazy z pamięci podręcznej odpowiadają wolno: ich 95. percentyl przekracza poniższy limit.
privacy-speed-reason-fewRequests = Mniej niż { $min } żądań w tym zakresie, zbyt mało do oceny.
privacy-speed-thresholds = Wolna od: 95. percentyl pobranego obrazu { $slowMiss }, 95. percentyl obrazu z pamięci podręcznej { $slowHit } lub { $slowFail } % zajętości, przekroczeń czasu lub błędów. Pogorszona od: 95. percentyl pobranego obrazu { $degradedMiss } lub { $degradedFail } % błędów. Limity stałe, nie ustawienia.
privacy-speed-median = Mediana
privacy-speed-p95 = 95. percentyl
privacy-speed-p99 = 99. percentyl
privacy-speed-max = Najwolniejsze
privacy-speed-requests = Zmierzone żądania
privacy-speed-busy = Odpowiedzi „zajęty”
privacy-speed-timeouts = Przekroczenia czasu
privacy-speed-upstream-errors = Błędy źródła
privacy-speed-hit = Trafienia w pamięci podręcznej
privacy-speed-miss = Pobrane obrazy (chybienia pamięci podręcznej)
privacy-speed-split-count = { $count } żądań
privacy-speed-split-times = Mediana { $p50 }, 95. percentyl { $p95 }, najwolniejsze { $max }.
privacy-speed-chart = Czas udostępnienia obrazu
privacy-speed-chart-miss = Czas udostępnienia pobranego obrazu
privacy-speed-chart-unit = Od odebrania żądania do gotowości odpowiedzi
privacy-speed-phases = Na co schodzi czas
privacy-speed-phases-help = Dla żądań, które nie były trafieniami w pamięci podręcznej: ile trwał każdy krok i jaki ma udział w całkowitym czasie.
privacy-speed-phase = Krok
privacy-speed-share = Udział w czasie
privacy-speed-phase-queue = Oczekiwanie na zasoby
privacy-speed-phase-cache = Przeszukanie pamięci podręcznej
privacy-speed-phase-upstream = Pobranie ze źródła
privacy-speed-phase-decode = Dekodowanie i ponowne kodowanie
privacy-speed-phase-other = Wszystko inne
privacy-speed-note = Zmierzone tylko w procesie tego serwera, przechowywane w pamięci i puste po ponownym uruchomieniu. Percentyle są dokładne z błędem do { $error } %. Nie są przechowywane żadne adresy URL, hosty ani konta.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-work-coalesced = Połączone pobrania i żądania priorytetowe

privacy-work-coalesced-series = Pobieracze łączą się z jednym w locie

privacy-work-handle = Obsługuj współczynnik trafień w pamięci podręcznej

privacy-work-handle-series = Obsługuj trafienia w pamięci podręcznej

privacy-work-per-interval = Na interwał próbkowania

privacy-work-percent = Udział wyszukiwań uchwytów

privacy-work-priority-series = Żądania priorytetowe

privacy-work-resized = Obrazy zmniejszone

privacy-work-resized-series = Zmieniono rozmiar

privacy-work-revalidated-series = Ponowna walidacja

privacy-work-saved = Bajty zaoszczędzone przez zmniejszenie skali

privacy-work-saved-series = Bajty zapisane

privacy-work-stale = Wyświetlono i ponownie zweryfikowano nieaktualne obrazy

privacy-work-stale-series = Podawane nieświeże

privacy-work-title = Uniknięto pracy i zmieniono jej kształt
