# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = Telemetria operacyjna
admin-telemetry-help = Eksportuj ślady i metryki operacyjne do kolektora zgodnego z OpenTelemetry, na przykład własnego OpenTelemetry Collector. Eksport jest domyślnie wyłączony. Zapisane zmiany zaczną obowiązywać po ponownym uruchomieniu serwera.
admin-telemetry-back = Wróć do obserwowalności
admin-telemetry-saved = Konfiguracja telemetrii zapisana. Uruchom serwer ponownie, aby zastosować zmiany.
admin-telemetry-restart = Zapisana konfiguracja różni się od działającego serwera. Wymagane jest ponowne uruchomienie.
admin-telemetry-reload = Wczytaj konfigurację ponownie
admin-telemetry-save = Zapisz konfigurację telemetrii
admin-telemetry-collector = Kolektor i sygnały
admin-telemetry-collector-help = Tylko administrator może wybrać to miejsce docelowe dla całego serwera. Użyj prywatnego kolektora lub HTTPS; HTTP przesyła telemetrię bez szyfrowania.
admin-telemetry-enabled = Włącz eksport OTLP
admin-telemetry-traces = Eksportuj ślady
admin-telemetry-metrics = Eksportuj metryki
admin-telemetry-endpoint = Punkt końcowy OTLP gRPC
admin-telemetry-endpoint-help = Bezwzględny adres kolektora HTTP lub HTTPS, zwykle port 4317. Nie podawaj danych uwierzytelniających, zapytania ani fragmentu. To punkt końcowy gRPC, a nie OTLP HTTP na porcie 4318.
admin-telemetry-serviceName = Nazwa usługi
admin-telemetry-serviceName-help = Stała etykieta usługi identyfikująca to wdrożenie Mailwoman w kolektorze. Nie używaj adresu e-mail ani innych danych osobowych.
admin-telemetry-environment = Środowisko
admin-telemetry-environment-help = Opcjonalna etykieta wdrożenia, na przykład production lub staging. Jest wysyłana wraz z eksportowaną telemetrią.
admin-telemetry-pinned = Wyłączone pola są kontrolowane przez zmienne środowiskowe wdrożenia. Aby zmienić ich wartości, zmień te zmienne.
admin-telemetry-limits = Próbkowanie i limity eksportu
admin-telemetry-limits-help = Ogranicza zużycie pamięci i ruch do kolektora. Rozmiar partii nie może przekraczać rozmiaru kolejki. Pełne kolejki mogą odrzucać zakresy zamiast blokować żądania poczty.
admin-telemetry-sampleRatio = Współczynnik próbkowania śladów
admin-telemetry-sampleRatio-help = Ułamek nowych śladów głównych do zachowania, od 0 do 1. Zakresy podrzędne stosują decyzję próbkowania zakresu nadrzędnego.
admin-telemetry-exportTimeoutMs = Limit czasu eksportu (milisekundy)
admin-telemetry-exportTimeoutMs-help = Maksymalny czas żądania eksportu do kolektora, od 100 do 30000 milisekund.
admin-telemetry-metricsIntervalSecs = Odstęp metryk (sekundy)
admin-telemetry-metricsIntervalSecs-help = Czas między eksportami metryk, od 10 do 3600 sekund.
admin-telemetry-spanQueueSize = Rozmiar kolejki zakresów
admin-telemetry-spanQueueSize-help = Maksymalna liczba zakresów oczekujących na eksport, od 128 do 32768.
admin-telemetry-spanBatchSize = Rozmiar partii zakresów
admin-telemetry-spanBatchSize-help = Maksymalna liczba zakresów w jednym eksporcie, od 1 do 4096 i nie więcej niż rozmiar kolejki.
admin-telemetry-spanExportDelayMs = Opóźnienie partii zakresów (milisekundy)
admin-telemetry-spanExportDelayMs-help = Zwykły czas oczekiwania między partiami zakresów, od 100 do 60000 milisekund.
admin-telemetry-prometheus-help = Osobny, uwierzytelniany punkt końcowy dla skrapera zgodnego z Prometheus. Dostęp zawsze wymaga tokenu okaziciela MW_METRICS_TOKEN wdrożenia; jego wartość nigdy nie jest zwracana na tej stronie.
admin-telemetry-prometheusEnabled = Włącz skrapowanie przez Prometheus
admin-telemetry-token-missing = Nie skonfigurowano tokenu skrapowania. Punkt końcowy metryk pozostaje niedostępny, dopóki operator nie ustawi MW_METRICS_TOKEN.
