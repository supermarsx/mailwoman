# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: directory. Source: ../en/directory.ftl

# -- Directory / GAL search --------------------------------------------------
directory-matches-label = Совпадения в каталоге
directory-search-error = не удалось выполнить поиск в каталоге
directory-group-badge = Группа
directory-load-more = Загрузить ещё

# -- Distribution-group expand-before-send -----------------------------------
directory-members-of = Участники группы { $name }
directory-is-distribution-group = — это группа рассылки.
directory-who-in-this = Кто в неё входит?
directory-recipient-count = { $count ->
    [one] { $count } получатель
    [few] { $count } получателя
    [many] { $count } получателей
   *[other] { $count } получателя
}
directory-replace-with = Заменить группу списком: { $count ->
    [one] { $count } получатель
    [few] { $count } получателя
    [many] { $count } получателей
   *[other] { $count } получателя
}
directory-expand-error = не удалось раскрыть группу

# -- Per-contact security tab ------------------------------------------------
directory-contact-security-label = Безопасность контакта
directory-photo-alt = Фото { $email } из каталога
directory-published-material = Данные безопасности, опубликованные в каталоге
directory-smime-certificates = Сертификаты S/MIME
directory-no-cert = Для этого контакта в каталоге не опубликован сертификат.
directory-valid-until = Действителен до { $date }
directory-no-expiry = Срок действия не указан
directory-cert-expired = Истёк
directory-cert-current = Действителен
directory-looking-up = Поиск в каталоге…
directory-lookup-failed = Не удалось выполнить поиск в каталоге.
