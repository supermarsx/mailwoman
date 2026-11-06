# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = Dostawca tożsamości
admin-identity-description = Pozwól zatwierdzonym aplikacjom logować się przez Mailwoman. To osobna funkcja od zewnętrznych dostawców logowania i dostępu do API poczty. Wymagany jest kod autoryzacji z PKCE; inne typy przyznania są niedostępne.
admin-identity-configure = Skonfiguruj wystawcę
admin-identity-new = Dodaj klienta tożsamości
admin-identity-edit = Edytuj klienta
admin-identity-enabled = Włączony
admin-identity-disabled = Wyłączony
admin-identity-all = Wszystkie
admin-identity-discovery = Otwórz dokument wykrywania
admin-identity-search = Szukaj klientów
admin-identity-status = Stan
admin-identity-clients = Klienci tożsamości
admin-identity-client = Tożsamość klienta
admin-identity-client-id = Identyfikator klienta
admin-identity-name = Nazwa
admin-identity-method = Uwierzytelnianie klienta
admin-identity-public = Klient publiczny (PKCE, bez sekretu)
admin-identity-pkce = Każdy klient musi używać PKCE S256. Sekrety klientów poufnych są generowane tutaj i pokazywane jednorazowo.
admin-identity-rotate-secret = Zastąp istniejący sekret klienta
admin-identity-redirects = Zarejestrowane miejsca docelowe
admin-identity-redirect-help = Jeden dokładny URI HTTPS w wierszu; numeryczny HTTP pętli zwrotnej jest dozwolony dla aplikacji lokalnych. Symbole wieloznaczne i fragmenty są odrzucane.
admin-identity-callbacks = URI zwrotne autoryzacji
admin-identity-logout-uris = URI przekierowania po wylogowaniu
admin-identity-scopes = Dozwolone zakresy tożsamości
admin-identity-actions = Akcje
admin-identity-delete = Usuń klienta
admin-identity-delete-confirm = Usunąć dostęp tożsamości dla { $name }? Istniejące przyznania tożsamości przestaną działać, a rejestracja klienta tylko tożsamości zostanie usunięta.
admin-identity-empty = Brak pasujących klientów tożsamości.
admin-identity-cancel = Anuluj
admin-identity-save = Zapisz
admin-identity-done = Gotowe
admin-identity-secret = Wygenerowany sekret klienta
admin-identity-secret-once = Skopiuj ten sekret teraz do aplikacji ufającej. Nie można go wyświetlić ponownie. Nigdy nie umieszczaj sekretu poufnego w kodzie przeglądarki.
admin-identity-issuer = Publiczny adres URL wystawcy HTTPS
admin-identity-issuer-help = Użyj dokładnego źródła HTTPS tego wdrożenia i ścieżki /oidc, wraz z ewentualnym prefiksem wdrożenia. Zmiana wystawcy lub konfiguracji unieważnia istniejące przyznania tożsamości.
admin-identity-rotate-key = Obróć klucz podpisu
admin-identity-key-help = Obrót wymaga świeżej weryfikacji administratora. Poprzednie klucze publiczne pozostają dostępne przez 15 minut, aby niedawno wydane tokeny mogły dokończyć swój krótki czas życia.
