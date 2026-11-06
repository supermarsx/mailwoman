# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = Pełna administracja we wszystkich obszarach, w tym administratorzy, role, bezpieczeństwo, routing poczty i konfiguracja usług. Rolę tę należy zarezerwować dla zaufanych operatorów odpowiedzialnych za całe wdrożenie.
admin-role-purpose-domain_admin = Codzienna administracja użytkownikami i domenami: tworzenie, aktualizowanie, wyłączanie lub usuwanie użytkowników; zarządzanie limitami, uprawnieniami użytkowników, regułami logowania domeny, ustawieniami i marką; unieważnianie sesji i zarządzanie blokadami. Obejmuje też diagnostykę poczty oraz eksporty użycia i danych. Nie daje zarządzania rolami administratorów ani zmian serwerów nadrzędnych.
admin-role-purpose-helpdesk = Obsługa kont i wstępna ocena incydentów: przeglądanie użytkowników, aktywności audytu i użycia; zmiana flag użytkowników, wyłączanie kont i unieważnianie sesji; podgląd blokad i uruchamianie symulacji logowania. Nie daje tworzenia ani usuwania użytkowników, zmian limitów ani ogólnego zapisu ustawień.
admin-role-purpose-auditor = Kontrola i raportowanie: daje każde uprawnienie kończące się na .read oraz eksport audytu i użycia. Nie daje zmian konfiguracji ani modyfikacji kont. Wyeksportowane raporty mogą zawierać wrażliwe informacje administracyjne.
admin-role-purpose-security_officer = Operacje bezpieczeństwa: zarządzanie politykami bezpieczeństwa i MFA, polityką logowania i blokadami; wyłączanie użytkowników, unieważnianie sesji lub kluczy API; przeglądanie i eksport aktywności audytu; odczyt dzienników uwierzytelniania i uruchamianie diagnostyki. Konfiguracja SSO jest dostępna do odczytu, ale rola nie daje edycji SSO ani zarządzania rolami administratorów.
admin-role-purpose-custom = Rola niestandardowa z wybranymi uprawnieniami: { $count }. Jej uprawnienia wynikają z dokładnej listy poniżej, a nie z nazwy. Przed przypisaniem sprawdź każdą akcję.
admin-role-purpose-scope = Zakres domeny jest przypisywany, gdy rola jest nadawana administratorowi. Zastąpienia zezwoleń i zakazów na poziomie konta mogą zmienić efektywny dostęp. Opis nie daje dodatkowych uprawnień.
admin-role-purpose-editor = Wybierz zadania, które ten administrator ma wykonywać, a następnie zaznacz odpowiednie uprawnienia. Odczyt, eksport danych i zmiana rekordów to osobne akcje. Przed zapisem sprawdź dokładne identyfikatory uprawnień.
admin-role-purpose-selection = Te opisy wyjaśniają wybrane akcje. Nie rozszerzają uprawnień wysyłanych do serwera.
admin-role-purpose-read = Wyświetlanie informacji w tym obszarze.
admin-role-purpose-export = Eksport informacji z tego obszaru.
admin-role-purpose-revoke = Unieważnianie sesji lub poświadczeń wskazanych przez to uprawnienie.
admin-role-purpose-delete = Usuwanie rekordów objętych tym uprawnieniem.
admin-role-purpose-disable = Wyłączanie kont objętych tym uprawnieniem.
admin-role-purpose-create = Tworzenie rekordów objętych tym uprawnieniem.
admin-role-purpose-specific = Wykonanie konkretnej akcji wskazanej przez to uprawnienie; przed nadaniem sprawdź jego identyfikator.
admin-role-purpose-description = Cel i obowiązki
admin-role-purpose-description-help = Wyjaśnij, kto powinien otrzymać tę rolę niestandardową i jakimi zadaniami się zajmuje (do 1024 znaków). Ten tekst nie zmienia uprawnień.
