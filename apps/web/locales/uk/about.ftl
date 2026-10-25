# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: about. Source: ../en/about.ftl

about-tp-label = Сторонні бібліотеки та ліцензії
about-tp-search = Пошук за назвою, версією або ліцензією
about-tp-filter = Фільтр за ліцензією
about-tp-filter-all = Усі ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] Немає відповідних записів.
        [one] Показано { $shown } з { $total } запису.
        [few] Показано { $shown } з { $total } записів.
        [many] Показано { $shown } з { $total } записів.
       *[other] Показано { $shown } з { $total } запису.
    }
about-tp-empty = Ніщо не відповідає цьому пошуку та ліцензії.
about-tp-clear = Очистити пошук і фільтр
about-tp-more = Показати більше (залишилося { $n })

about-tp-kind-crate = Crate Rust
about-tp-kind-npm = Пакет npm
about-tp-kind-font = Шрифт
about-tp-kind-icons = Значки
about-tp-kind-image = Зображення
about-tp-kind-os = Пакет образу

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = Crate Rust у сервері, вебзастосунку та плагінах
about-tp-group-shells = Crate Rust лише в настільному та мобільному застосунках
about-tp-group-web = Пакети npm у вебзастосунку
about-tp-group-assets = Шрифти, значки та зображення
about-tp-group-image = Файли образу сервера
about-tp-group-build = Інструменти збирання: працюють під час збирання, не постачаються

about-tp-licence = Ліцензія:
about-tp-licence-none = не вказано
about-tp-licence-in-file = див. файл з авторськими правами
about-tp-used-in = Використовується в: { $parts }
about-tp-artifact-server = сервер
about-tp-artifact-web = вебзастосунок
about-tp-artifact-plugins = плагіни
about-tp-artifact-desktop = настільний застосунок
about-tp-artifact-mobile = мобільний застосунок
about-tp-note-bundled = Входить до складу вебзастосунку.
about-tp-note-fonts-pull = Не входить до складу: завантажується оператором сервера командою `mailwoman fonts pull`.
about-tp-note-first-party = Створено для Mailwoman, під власною ліцензією проєкту.
about-tp-note-static-libc = Скомпоновано в бінарні файли сервера образу FROM-scratch.
about-tp-note-alpine-ca = Набір сертифікатів CA образу FROM-scratch, узятий з Alpine Linux.
about-tp-in-images = В: { $images }
about-tp-image-runtime = образ виконання distroless
about-tp-image-static = образ FROM-scratch

about-tp-repository = Вихідний код
about-tp-homepage = Домашня сторінка
about-tp-new-tab = для { $name } (відкривається в новій вкладці)

about-tp-text-toggle =
    { $count ->
        [one] Текст ліцензії ({ $count })
        [few] Тексти ліцензій ({ $count })
        [many] Тексти ліцензій ({ $count })
       *[other] Тексти ліцензій ({ $count })
    }
about-tp-text-open-for = Текст ліцензії для { $name }
about-tp-dialog-title = Текст ліцензії: { $name } { $version }
about-tp-close = Закрити
about-tp-text-loading = Завантаження тексту ліцензії…
about-tp-text-error = Не вдалося завантажити тексти ліцензій.
about-tp-retry = Спробувати ще раз
about-tp-text-missing = Цього тексту немає в списку. Згенеруйте його знову за допомогою scripts/about/gen-third-party.mjs.
about-tp-text-label = Текст ліцензії { $name }: { $file }
about-tp-standard = Ліцензія { $licence }
about-tp-standard-note = (стандартний текст, файлу в пакеті немає)
about-tp-no-text = Для цього запису текст ліцензії недоступний.
