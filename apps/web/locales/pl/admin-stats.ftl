# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-stats. Source: ../en/admin-stats.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-stats-key-figures = Kluczowe wartości
admin-stats-charts = Wykresy
admin-stats-tables = Tabele
admin-stats-group-traffic = Ruch w tym okresie
admin-stats-group-storage = Przechowywanie i dostarczanie, teraz
admin-stats-refresh = Odśwież statystyki
admin-stats-loading = Wczytywanie statystyk…
admin-stats-requests-total = Żądania
admin-stats-errors-total = Błędy serwera (5xx)
admin-stats-p95-peak = Najwolniejszy 95. percentyl
admin-stats-requests = Żądania
admin-stats-errors = Błędy serwera (5xx)
admin-stats-not-found = Nie znaleziono (404)
admin-stats-latency = Czas odpowiedzi
admin-stats-series-p50 = Mediana
admin-stats-series-p95 = 95. percentyl
admin-stats-unit-count = Liczba na { $bucket }
admin-stats-unit-duration = Czas na { $bucket }, w pokazanej jednostce
admin-stats-unit-accounts = Zablokowane konta na domenę
admin-stats-chart-about = Wykres liniowy od { $from } do { $to }.
admin-stats-bars-about = { $count ->
    [one] Wykres słupkowy z jedną domeną.
    [few] Wykres słupkowy z { $count } domenami.
    [many] Wykres słupkowy z { $count } domenami.
   *[other] Wykres słupkowy z { $count } domeny.
}
admin-stats-view-table = Pokaż jako tabelę
admin-stats-legend = Legenda
admin-stats-empty-chart = W tym okresie nic jeszcze nie zmierzono. Wybierz dłuższy okres lub wróć, gdy serwer obsłuży więcej żądań.
admin-stats-empty-blocked = Żadne konta nie są zablokowane.
admin-stats-delta-vs = w porównaniu z poprzednim okresem
admin-stats-delta-same = Tyle samo co w poprzednim okresie
admin-stats-delta-sr-up = Więcej o { $change } niż w poprzednim okresie.
admin-stats-delta-sr-down = Mniej o { $change } niż w poprzednim okresie.
admin-stats-delta-sr-same = Tyle samo co w poprzednim okresie.
admin-ct-group = Czasy ładowania po stronie klienta
admin-ct-metric-timeToUsable = Czas do użyteczności
admin-ct-metric-ttfb = Pierwszy bajt
admin-ct-metric-dcl = DOM gotowy
admin-ct-metric-load = Wczytanie strony
admin-ct-metric-fcp = Pierwsze wyrenderowanie treści
admin-ct-metric-lcp = Największe wyrenderowanie treści
admin-ct-metric-folderOpen = Otwarcie folderu
admin-ct-metric-folderLoadCold = Wczytanie folderu z sieci
admin-ct-metric-folderLoadCached = Wczytanie folderu z pamięci podręcznej
admin-ct-metric-messageLoadCold = Wczytanie wiadomości z sieci
admin-ct-metric-messageLoadCached = Wczytanie wiadomości z pamięci podręcznej lub wstępnego pobrania
admin-ct-device-desktop = Komputer
admin-ct-device-mobile = Urządzenie mobilne
admin-ct-device-unknown = Nieznane
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } próbka
        [few] { $count } próbki
        [many] { $count } próbek
       *[other] { $count } próbki
}
admin-ct-no-samples = Brak próbek w tym okresie.
admin-ct-few-samples = Tylko { $count } próbek; wymagane jest co najmniej { $min }.
admin-ct-chart-usable = Czas do użyteczności
admin-ct-group-folder = Czasy wczytywania folderów
admin-ct-group-message = Czasy wczytywania wiadomości
admin-ct-chart-folder-cold = Wczytanie folderu z sieci
admin-ct-chart-folder-cached = Wczytanie folderu z pamięci podręcznej
admin-ct-chart-message-cold = Wczytanie wiadomości z sieci
admin-ct-chart-message-cached = Wczytanie wiadomości z pamięci podręcznej lub wstępnego pobrania
admin-ct-folder-off = Czasy wczytywania folderów są domyślnie wyłączone. Mierzone są tylko osoby, które je włączyły.
admin-ct-message-off = Czasy wczytywania wiadomości są domyślnie wyłączone. Mierzone są tylko osoby, które je włączyły.
admin-ct-chart-note = Percentyle są pokazywane tylko dla przedziałów z co najmniej { $min } próbkami.
admin-ct-empty-chart = Żaden przedział nie ma { $min } próbek potrzebnych do pokazania mediany. Wybierz dłuższy okres.
admin-ct-col-samples = Próbki
admin-ct-col-metric = Miara
admin-ct-col-device = Urządzenie
admin-ct-table-title = Czasy ładowania po stronie klienta według urządzenia
admin-ct-table-caption = Próbki, mediana i 95. percentyl dla każdej miary i klasy urządzenia.
admin-ct-empty-table = W tym okresie nie zgłoszono czasów ładowania z przeglądarek.
admin-ct-failed = Nie udało się wczytać czasów ładowania po stronie klienta.
admin-ct-disabled = Zbieranie czasów ładowania po stronie klienta jest wyłączone. Poniższe wartości pochodzą sprzed jego wyłączenia.
admin-ct-note = Tylko z zalogowanych przeglądarek, próbkowanych jest { $rate }% sesji, anonimowo i zbiorczo. Czas do użyteczności liczony jest od początku nawigacji do chwili, gdy lista wiadomości pokaże pierwsze wiersze. Wartości są szacunkowe, w granicach około { $error }% rzeczywistej wartości.
