# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: pl · Module: avatars. Source: ../en/avatars.ftl


# -- Machine translation (2026-11): added to match en ---------------------
avatars-title = Osoby i zdjęcia kontaktów
avatars-mode = Zdjęcia nadawców i kontaktów
avatars-off = Ukryj zdjęcia
avatars-initials = Generuj inicjały lokalnie
avatars-contacts = Używaj lokalnych zdjęć kontaktów, a w razie braku inicjałów
avatars-help = Inicjały są generowane na tym urządzeniu. Zdjęcia kontaktów wykorzystują wyłącznie osadzone zdjęcia PNG lub JPEG zapisane już w Twoich książkach adresowych, do 512 × 512 pikseli i 128 KiB. Adresy URL zdjęć zdalnych ani zewnętrzne usługi awatarów nigdy nie są wywoływane. To ustawienie jest zapisywane na koncie i może być narzucone przez administratora.
avatars-default = Przywróć ustawienie dziedziczone
avatars-sender-icons = Pokazuj dołączone ikony domen nadawców
avatars-sender-icons-help = Rozpoznane domeny społecznościowe, deweloperskie, AI, sklepowe i rządowe używają ikon dołączonych do aplikacji. Pierwszeństwo mają Twoje własne zdjęcia kontaktów. Nieznani nadawcy mają lokalnie generowane inicjały w jednym z 192 kolorów. Ikony są wskazówkami dotyczącymi domeny, a nie dowodem autentyczności wiadomości. Żadna zewnętrzna usługa logo nie jest wywoływana.
avatars-icons-default = Przywróć dziedziczone ustawienie ikon
avatars-brand-avatars = Pokazuj logo marek obok zweryfikowanych nadawców
avatars-brand-avatars-help = W czytniku rozpoznana firma pokazuje swoje logo obok nadawcy tylko wtedy, gdy wiadomość przeszła DKIM i zgodne sprawdzenie DMARC dla domeny tej firmy. W przeciwnym razie używany jest zwykły awatar. Logo są dołączone do aplikacji; nic nie jest pobierane od nadawcy.
avatars-brand-avatars-default = Przywróć dziedziczone ustawienie logo marek
avatars-domain-icon = Ikona domeny { $name } · ta ikona nie weryfikuje tożsamości nadawcy
avatars-favorites-first = Pokazuj ulubione kontakty jako pierwsze w podpowiedziach odbiorców
avatars-favorites-first-help = Kontakty oznaczone gwiazdką są wyświetlane przed innymi dopasowaniami podczas wpisywania odbiorcy. Zmienia się tylko kolejność; żadna podpowiedź nie jest ukrywana.
avatars-favorites-first-default = Przywróć dziedziczone ustawienie ulubionych

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
avatars-require-auth = Uwierzytelnienie wymagane w przypadku logo marki

avatars-require-auth-always = Zawsze (wymagaj pozytywnego sprawdzenia)

avatars-require-auth-default = Przywróć odziedziczone preferencje uwierzytelniania

avatars-require-auth-help = Wybierz, kiedy logo uznanej firmy może zastąpić inicjały obok nadawcy. Nigdy: pokaż go za każdym razem, gdy domena nadawcy będzie zgodna. Tylko jeśli sprawdzenie się nie powiodło (domyślnie): pokaż je, chyba że komunikat nie powiódł się DKIM lub DMARC. Zawsze: pokazuj to tylko wtedy, gdy wiadomość przeszła DKIM i dopasowane sprawdzenie DMARC dla własnej domeny tej firmy, aby poczta, która nie została jeszcze sprawdzona, zawierała inicjały. Dopasowanie domeny obejmuje subdomeny, takie jak mail.example.com. Ta preferencja jest zapisana na Twoim koncie i może być przechowywana przez administratora.

avatars-require-auth-known-failure = Tylko w przypadku niepowodzenia kontroli

avatars-require-auth-never = Nigdy (dopasuj tylko w domenie)

avatars-unverified-sender = Niezweryfikowany nadawca · nie jest wyświetlana żadna ikona marki, ponieważ ta wiadomość nie przeszła kontroli uwierzytelnienia
