# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: about. Source: ../en/about.ftl

about-tp-label = Сторонние библиотеки и лицензии
about-tp-search = Поиск по названию, версии или лицензии
about-tp-filter = Фильтр по лицензии
about-tp-filter-all = Все ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Нет подходящих записей.
        [one] Показано { $shown } из { $total } записи.
        [few] Показано { $shown } из { $total } записей.
        [many] Показано { $shown } из { $total } записей.
       *[other] Показано { $shown } из { $total } записей.
    }
about-tp-empty = Ничего не найдено по этому запросу и лицензии.
about-tp-clear = Сбросить поиск и фильтр
about-tp-more = Показать ещё (осталось { $n })

about-tp-kind-crate = Крейт Rust
about-tp-kind-npm = Пакет npm
about-tp-kind-font = Шрифт
about-tp-kind-icons = Значки
about-tp-kind-image = Изображения
about-tp-kind-os = Пакет образа

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Крейты Rust в сервере, веб-приложении и плагинах
about-tp-group-shells = Крейты Rust только в настольном и мобильном приложениях
about-tp-group-web = Пакеты npm в веб-приложении
about-tp-group-assets = Шрифты, значки и изображения
about-tp-group-image = Файлы образа сервера
about-tp-group-build = Инструменты сборки: запускаются при сборке, не поставляются

about-tp-licence = Лицензия:
about-tp-licence-none = не указана
about-tp-licence-in-file = см. файл copyright
about-tp-used-in = Используется в: { $parts }
about-tp-artifact-server = сервер
about-tp-artifact-web = веб-приложение
about-tp-artifact-plugins = плагины
about-tp-artifact-desktop = настольное приложение
about-tp-artifact-mobile = мобильное приложение
about-tp-note-bundled = Входит в сборку веб-приложения.
about-tp-note-fonts-pull = Не входит в сборку: загружается оператором сервера командой `mailwoman fonts pull`.
about-tp-note-first-party = Создано для Mailwoman, распространяется под собственной лицензией проекта.
about-tp-note-static-libc = Компонуется в исполняемые файлы сервера в образе FROM-scratch.
about-tp-note-alpine-ca = Набор сертификатов CA образа FROM-scratch, взятый из Alpine Linux.
about-tp-in-images = В: { $images }
about-tp-image-runtime = образ среды выполнения distroless
about-tp-image-static = образ FROM-scratch

about-tp-repository = Исходный код
about-tp-homepage = Домашняя страница
about-tp-new-tab = для { $name } (открывается в новой вкладке)

about-tp-text-toggle =
    { $count ->
        [one] Текст лицензии ({ $count })
        [few] Тексты лицензий ({ $count })
        [many] Тексты лицензий ({ $count })
       *[other] Тексты лицензий ({ $count })
    }
about-tp-text-open-for = Текст лицензии для { $name }
about-tp-dialog-title = Текст лицензии: { $name } { $version }
about-tp-close = Закрыть
about-tp-text-loading = Загрузка текста лицензии…
about-tp-text-error = Не удалось загрузить тексты лицензий.
about-tp-retry = Повторить попытку
about-tp-text-missing = Этот текст отсутствует в списке. Создайте его заново с помощью scripts/about/gen-third-party.mjs.
about-tp-text-label = Текст лицензии { $name }: { $file }
about-tp-standard = Лицензия { $licence }
about-tp-standard-note = (стандартный текст, в пакете нет файла)
about-tp-no-text = Для этой записи нет текста лицензии.
