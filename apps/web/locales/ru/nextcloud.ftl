# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = Вверх
nextcloud-loading = Загрузка…
nextcloud-list-error = Не удалось получить содержимое этой папки.
nextcloud-empty = Эта папка пуста.
nextcloud-file-list = Файлы Nextcloud
nextcloud-open-folder = Открыть папку { $name }
nextcloud-select-file = Выбрать { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Вложить из Nextcloud
nextcloud-attach-action = { $count ->
    [0] Вложить
    [one] Вложить { $count } файл
    [few] Вложить { $count } файла
    [many] Вложить { $count } файлов
   *[other] Вложить { $count } файла
  }
nextcloud-error-select-file = выберите хотя бы один файл для вложения
nextcloud-error-attach-failed = не удалось вложить выбранные файлы

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Сохранить в Nextcloud
nextcloud-save-title = Сохранить «{ $name }» в Nextcloud
nextcloud-save-action = Сохранить сюда ({ $dir })
nextcloud-saved = Сохранено в { $path }
nextcloud-error-save-failed = не удалось сохранить в Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Создать ссылку для общего доступа
nextcloud-share-title = Ссылка для общего доступа
nextcloud-share-intro = Создайте общедоступную ссылку на { $path } вместо того, чтобы вкладывать файл.
nextcloud-protect-password = Защитить паролем
nextcloud-password-label = Пароль
nextcloud-share-password = Пароль для общего доступа
nextcloud-set-expiry = Задать срок действия
nextcloud-expires-on = Действует до
nextcloud-expiry-date = Дата окончания срока действия
nextcloud-create-link = Создать ссылку
nextcloud-password-protected = Защищено паролем
nextcloud-no-password = Без пароля
nextcloud-expires = действует до { $date }
nextcloud-no-expiry = без срока действия
nextcloud-error-need-password = введите пароль или отключите защиту паролем
nextcloud-error-need-expiry = выберите дату окончания или отключите срок действия
nextcloud-error-share-failed = не удалось создать ссылку для общего доступа
