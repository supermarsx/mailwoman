# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: reader. Source: ../en/reader.ftl


# -- Machine translation (2026-11): added to match en ---------------------
reader-settings-title = Отображение сообщений
reader-settings-help = Выберите, как отображаются сообщения. Значения администратора по умолчанию действуют, пока Вы не выберете свои. Содержимое сообщений остаётся изолированным от приложения.
reader-colorMode = Цвета сообщений
reader-colorMode-help = «Автоматически» подстраивает цвета сообщений в тёмных темах. «Исходные» сохраняют цвета отправителя; «адаптивные» используют активную тему; «инверсия» меняет цвета на обратные, сохраняя цвета изображений.
reader-fontSize = Размер шрифта для чтения (px)
reader-fontSize-help = Базовый размер шрифта текста сообщения, от 12 до 24 пикселей. Форматирование отправителя по-прежнему может задавать размеры заголовков.
reader-lineHeight = Межстрочный интервал при чтении
reader-lineHeight-help = Расстояние между строками текста сообщения, от 1,2 до 2,2 размера шрифта.
reader-contentWidth = Ширина при чтении
reader-contentWidth-help = Ограничьте длину строк для удобного чтения или используйте всю область сообщения.
reader-loading = Вид индикатора загрузки
reader-loading-help = Выберите спокойные строки-заполнители, текстовый индикатор или отсутствие визуального индикатора. Программы чтения с экрана по-прежнему получают сведения о загрузке.
reader-loadingDelayMs = Задержка индикатора загрузки (мс)
reader-loadingDelayMs-help = Не показывать мигающий индикатор при быстрых запросах. Задерживается только индикатор, но не само сообщение.
reader-option-auto = Автоматически по теме
reader-option-original = Исходные цвета сообщения
reader-option-adaptive = Подстраивать под тему
reader-option-invert = Инвертировать цвета сообщения
reader-option-comfortable = Комфортная
reader-option-wide = Широкая
reader-option-full = Вся область
reader-option-skeleton = Строки-заполнители
reader-option-indicator = Текстовый индикатор
reader-option-none = Без визуального индикатора
reader-inherit = Использовать значение администратора по умолчанию
reader-actionStyle = Лента действий с сообщением
reader-actionStyle-help = Компактные значки оставляют больше места для сообщения. У каждого действия сохраняются название для клавиатуры и пояснительная подсказка. Выберите видимые подписи, если предпочитаете текст рядом со значками.
reader-option-icons = Компактные значки
reader-option-labels = Значки и подписи
reader-zoom-group = Масштаб сообщения
reader-zoom-in = Увеличить
reader-zoom-in-help = Увеличивает сообщение. Меняется только размер сообщения.
reader-zoom-out = Уменьшить
reader-zoom-out-help = Уменьшает сообщение. Меняется только размер сообщения.
reader-zoom-level = { $level }%
reader-zoom-reset-label = Масштаб сообщения { $level }%. Сбросить до 100%
reader-zoom-reset-help = Сбросить масштаб сообщения до 100%
reader-zoom-reset-menu = Сбросить масштаб ({ $level }%)
reader-zoom = Масштаб сообщения (%)
reader-zoom-help = Размер, в котором открываются сообщения, от 50% до 200%. Масштабируется только текст сообщения, но не приложение, а печать всегда выполняется в масштабе 100%. Кнопки масштаба рядом с действиями с сообщением тоже меняют его.
reader-attachments-summary = { $count ->
    [one] { $count } вложение
    [few] { $count } вложения
    [many] { $count } вложений
   *[other] { $count } вложения
  } · { $size }
reader-attachments-show = Показать вложения
reader-attachments-hide = Скрыть вложения
reader-attachmentsDensity = Плотность вложений
reader-attachmentsDensity-help = «Компактная» показывает небольшие кнопки файлов; «комфортная» при раскрытии показывает более крупные превью.
reader-option-compact = Компактная
reader-attachmentsCollapsed = Сворачивать вложения по умолчанию
reader-attachmentsCollapsed-help = Начинать каждое сообщение с компактной сводки вложений. Можно развернуть конкретное сообщение, не меняя это значение по умолчанию.

# -- Machine translation (2026-11): added to match en ---------------------
reader-card-text = Текст и цвета
reader-card-text-help = Размер, интервалы, ширина и цвета текста сообщения.
reader-card-actions = Действия и вложения
reader-card-actions-help = Лента действий с сообщением и способ начала работы с вложениями.
reader-card-loading = Загрузка
reader-card-loading-help = Что вы видите, пока сообщение открывается.
settings-mail-card-list = Список сообщений
settings-mail-card-list-help = Действия в строках и поведение списка после действия с сообщением.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
reader-attachments-download = Скачать { $name }

reader-attachments-download-all = Скачать все

reader-attachments-download-all-blocked = Политика вложений блокирует здесь все файлы.

reader-attachments-download-all-busy = Подготовка файла ZIP

reader-attachments-download-all-label = Загрузите все вложения в виде файла ZIP.

reader-attachments-download-all-none = Нет вложений для скачивания

reader-attachments-download-all-warned = { $count ->
        [one] Один файл, требующий подтверждения, был пропущен. Загрузите его самостоятельно.
        [few] { $count } файла, требующие подтверждения, были исключены. Загрузите их один за другим.
        [many] { $count } файлов, требующих подтверждения, были исключены. Загрузите их один за другим.
       *[other] { $count } файла, требующие подтверждения, были исключены. Загрузите их один за другим.
    }

reader-attachments-download-blocked = { $name } невозможно скачать: { $reason }

reader-attachments-download-label = Скачать { $name } ({ $size })

reader-attachments-download-off = Загрузки отключены

reader-attachments-inline-group = { $count ->
        [one] { $count } встроенное изображение
        [few] { $count } встроенных изображения
        [many] { $count } встроенных изображений
       *[other] { $count } встроенных изображения
    }

reader-attachments-inline-summary = { $count ->
        [one] { $count } встроенное изображение
        [few] { $count } встроенных изображения
        [many] { $count } встроенных изображений
       *[other] { $count } встроенных изображения
    } · { $size }

reader-attachmentsBarClickToToggle = Нажмите на панель, чтобы развернуть или свернуть

reader-attachmentsBarClickToToggle-help = Щелчок в любом месте панели вложений переключает ее. В выключенном состоянии работает только кнопка со стрелкой.

reader-attachmentsBarDownloadAll = Кнопка «Загрузить все»

reader-attachmentsBarDownloadAll-help = Покажите кнопку на панели, которая загружает вложения как один файл ZIP.

reader-attachmentsBarShowDownloadButtons = Кнопки загрузки во вложениях

reader-attachmentsBarShowDownloadButtons-help = Показывать кнопку загрузки каждого вложения при наведении курсора или фокусе, всегда или никогда.

reader-attachmentsBarZipInlineImages = Включите встроенные изображения в ZIP.

reader-attachmentsBarZipInlineImages-help = Изображения, которые появляются внутри текста сообщения, не включены в параметр «Загрузить все», если этот параметр не включен.

reader-card-attachments-bar = Панель вложений

reader-card-attachments-bar-help = Как реагирует панель над вложениями к сообщению и что она предлагает. Линии, помеченные как заблокированные, устанавливаются администратором.

reader-option-always = Всегда

reader-option-hover = При наведении или фокусировке

reader-option-never = Никогда
