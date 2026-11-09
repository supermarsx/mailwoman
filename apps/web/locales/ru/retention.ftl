# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: retention. Source: ../en/retention.ftl


# -- Machine translation (2026-11): added to match en ---------------------
retention-title = Хранение
retention-intro = Правила, которые перемещают или удаляют вашу почту через заданное число дней. Правило может учитывать папку, метку, отправителя, размер, прочитанность или наличие вложения. Помеченные флагом письма и письма с меткой keep не затрагиваются, пока вы этого не измените. Удалённая почта сначала попадает в корзину и безвозвратно удаляется после периода отмены.
retention-disabled = Правила хранения отключены на этом сервере. Ваши существующие правила не выполняются.
retention-never = никогда
retention-error = Не удалось загрузить или сохранить настройки хранения. Повторите попытку.
retention-last-run = Последний запуск: { $when }
retention-next-run = следующий запуск примерно через { $days } дн.
retention-next-run-unknown = следующий запуск ещё не запланирован
retention-enabled = Выполнять мои правила хранения
retention-trash-first = Сначала перемещать удалённую почту в корзину
retention-trash-forced = Этот сервер всегда сначала перемещает почту в корзину и безвозвратно удаляет её после периода отмены.
retention-undo = Период отмены в днях
retention-undo-help = Почта, перемещённая правилом в корзину, хранится там столько времени, прежде чем будет удалена безвозвратно. От 0 до { $max } дн.
retention-pause = Приостановить все правила на 30 дней
retention-paused-until = Приостановлено до { $when }
retention-resume = Возобновить
retention-policy = Что разрешает этот сервер
retention-limits-both = Правило, удаляющее почту, ждёт не менее { $min } дн. и не более { $max } дн.
retention-limits-min = Правило, удаляющее почту, ждёт не менее { $min } дн.
retention-max-rules = Можно хранить до { $count } правил.
retention-permanent-allowed = Правило может удалять почту безвозвратно, если отключить шаг с корзиной.
retention-permanent-denied = Правила не могут удалять почту безвозвратно. Почта попадает в корзину и удаляется безвозвратно через { $days } дн.
retention-protected = Папки, которых правила никогда не касаются: { $folders }.
retention-mandatory = Задано администратором: почта в папке { $folder } старше { $days } дн. удаляется.
retention-rules = Ваши правила
retention-add = Новое правило
retention-no-rules = Правил пока нет. Примените шаблон ниже или добавьте своё правило.
retention-rule-paused = приостановлено
retention-rule-on = Включено
retention-rule-enabled = Правило { $name } включено
retention-edit-short = Изменить
retention-delete = Удалить
retention-new = Новое правило
retention-edit = Изменить правило
retention-name = Название
retention-action = Что сделать
retention-action-keep = Хранить всегда
retention-action-delete = Удалить
retention-action-trash = Переместить в корзину
retention-action-archive = Переместить в архив
retention-action-move = Переместить в папку
retention-action-markRead = Пометить как прочитанное
retention-days = Через сколько дней
retention-target = Папка
retention-choose-folder = Выберите папку
retention-scope = Какая почта
retention-folder = Папка
retention-any-folder = Любая папка
retention-subfolders = Включая вложенные папки
retention-labels = Метки (через запятую)
retention-senders = Адреса отправителей (через запятую)
retention-domains = Домены отправителей (через запятую)
retention-min-size = Минимальный размер в КБ
retention-max-size = Максимальный размер в КБ
retention-read = Состояние прочтения
retention-read-any = Прочитанные или непрочитанные
retention-read-read = Только прочитанные
retention-read-unread = Только непрочитанные
retention-attachment = Вложения
retention-attachment-any = С вложением или без
retention-attachment-with = С вложением
retention-attachment-without = Без вложения
retention-exclude = Не трогать
retention-exclude-flagged = Письма с флагом (со звёздочкой)
retention-exclude-unread = Непрочитанные письма
retention-exclude-tags = Письма с этими метками (через запятую)
retention-exclude-note = Почта, находящаяся под юридическим удержанием, не затрагивается никогда, независимо от настроек здесь.
retention-role-inbox = Входящие
retention-role-sent = Отправленные
retention-role-archive = Архив
retention-role-junk = Спам
retention-role-trash = Корзина
retention-role-drafts = Черновики
retention-btn-preview = Предпросмотр
retention-save = Сохранить
retention-cancel = Отмена
retention-preview-none = Ни одно сообщение не изменится.
retention-preview-count = Это изменит сообщений: { $count }.
retention-preview-truncated = Для этого предпросмотра прочитана только часть почтового ящика.
retention-preview-sample = Примеры
retention-no-subject = (без темы)
retention-do-trash = в корзину
retention-do-move = перемещено
retention-do-delete = удалено
retention-do-markRead = помечено как прочитанное
retention-skipped-legalHold = Ваша учётная запись находится под юридическим удержанием. Ничего не изменено.
retention-skipped-restricted = Обработка вашей учётной записи ограничена. Ничего не изменено.
retention-skipped-noRules = У вас нет активных правил, и администратор их не задал.
retention-error-name = Укажите название правила.
retention-error-days = Число дней должно быть целым числом от 0 до 3650.
retention-error-min = Правило, удаляющее почту, должно ждать не менее { $days } дн.
retention-error-max = Правило, удаляющее почту, может ждать не более { $days } дн.
retention-error-target = Выберите папку, в которую нужно перемещать почту.
retention-summary-keep = хранить всегда
retention-summary-delete = через { $days } дн. в корзину, затем удаляется безвозвратно
retention-summary-delete-permanent = через { $days } дн. удаляется безвозвратно
retention-summary-trash = через { $days } дн. в корзину
retention-summary-archive = через { $days } дн. в архив
retention-summary-move = через { $days } дн. в папку
retention-summary-markRead = через { $days } дн. помечается как прочитанное
retention-templates = Шаблоны
retention-templates-help = Шаблон добавляет готовое правило. Проверьте его метки и число дней, прежде чем на него полагаться.
retention-adopt = Добавить это правило
retention-run-heading = Запустить сейчас
retention-preview-all = Предпросмотр всех правил
retention-btn-run = Запустить мои правила сейчас
retention-run-result = Изменено сообщений: { $count }.
retention-history = Недавние запуски
retention-history-when = Когда
retention-history-matched = Найдено
retention-history-changed = Изменено
retention-history-errors = Ошибки
retention-context-action = Хранение…

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
pim-cal-calendars = Календари

pim-cal-days = Через несколько дней после окончания мероприятия

pim-cal-enabled = Удалить завершенные мероприятия

pim-cal-help = Удалять события в выбранных вами календарях через заданное количество дней после их окончания. Повторяющиеся события никогда не удаляются.

pim-cal-title = События календаря

pim-days-range = Выберите между { $min } и { $max } днями.

pim-held-title = Удаленные заметки, которые можно восстановить

pim-held-until = Хранится до { $when }

pim-limits = Между { $min } и { $max } днями.

pim-notes-days = Количество дней с момента последнего изменения заметки

pim-notes-enabled = Удалить нетронутые заметки

pim-notes-help = Удалите заметки, которые вы не меняли в течение заданного количества дней. Закрепленные заметки сохраняются. Удаленную заметку можно восстановить в течение { $days } дней.

pim-notes-title = Примечания

pim-off = Отключено администратором.

pim-preview = Предварительный просмотр

pim-restore = Восстановить

pim-result = Событие(я) { $events } и примечание(я) { $notes }.

pim-run = Беги сейчас

pim-run-title = Беги сейчас

pim-save = Сохранить

pim-skipped = Варианты { $count } были пропущены, поскольку администратор их отключил.

pim-untitled = Заметка без названия

retention-action-stripAttachments = Удаление больших вложений

retention-do-stripAttachments = вложения удалены

retention-error-strip-disabled = Удаление вложений отключено администратором.

retention-error-strip-min = Правило, удаляющее вложения, должно ждать не менее { $days } дней.

retention-fd-action = Что произойдет с почтой в { $folder }

retention-fd-days = За несколько дней до обработки { $folder }

retention-fd-help = Установите, как долго почта будет оставаться в папке, прежде чем она будет перемещена или удалена. Папка, для которой установлено значение «Наследовать», не имеет собственного значения по умолчанию; приведенные выше правила и ограничения администратора по-прежнему применяются. Подпапки следуют за своей папкой.

retention-fd-inherit = Наследовать

retention-fd-mandatory = Администратор хранит эту папку { $days } дней.

retention-fd-protected = Защищено администратором

retention-fd-save = Сохранить настройки папки по умолчанию

retention-fd-title = Папка по умолчанию

retention-rule-strip-off = Отключено администратором

retention-strip-allowed = Правило может удалять вложения размером не менее { $kb } КБ из почты не старше { $days } дней. Каждое вложение заменяется небольшой заметкой.

retention-strip-denied = Удаление вложений отключено администратором.

retention-strip-done = Вложения удалены из сообщений { $count }, освобождая { $size }.

retention-strip-inline-kept = Изображения, показанные внутри сообщения, сохраняются.

retention-strip-inline-removed = Изображения, показанные внутри сообщения, также будут удалены.

retention-strip-limits = Удаляет вложения размером не менее { $kb } КБ из почты не старше { $days } дней. Оригинал хранится в корзине { $undo } дней.

retention-strip-preview = Сообщения { $count } потеряют вложения, освобождая { $size }.

retention-strip-preview-estimate = Около { $count } сообщений потеряют вложения, освободив до { $size }.

retention-strip-skipped = Правила { $count }, удаляющие вложения, были пропущены, поскольку администратор отключил эту функцию.

retention-summary-stripAttachments = через { $days } дней большие вложения удалены
