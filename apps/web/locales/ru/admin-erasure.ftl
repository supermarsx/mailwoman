# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ru · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = Удаление и обезличивание
compliance-card-erasure-coverage = Замените идентификаторы одного человека в истории аудита и журналах и посмотрите, в каких резервных копиях ещё остались удалённые люди.
admin-erasure-title = Удаление и обезличивание
admin-erasure-intro = Обезличивание человека заменяет его адрес и идентификаторы учётной записи в истории аудита и в ротируемых файлах журналов постоянным псевдонимом с ключом. Цепочка хешей аудита пересчитывается и запечатывается заново, поэтому проверка проходит. Каждое удаление и обезличивание также оставляет метку, так что восстановление старой резервной копии снова удалит этих людей.
admin-erasure-docs = Реестр того, что удаляется, экспортируется и сохраняется для каждой таблицы, находится в docs/admin/erasure-coverage.md.
admin-erasure-scrub = Обезличить человека
admin-erasure-scrub-note = Сначала запустите предпросмотр: он подсчитывает строки и записи журнала, которые изменились бы, и ничего не меняет.
admin-erasure-address = Адрес
admin-erasure-preview = Предпросмотр
admin-erasure-run = Обезличить человека
admin-erasure-dry-note = Только предпросмотр. Ничего не изменено.
admin-erasure-done-note = Обезличено. Теперь человек отображается как { $pseudonym }.
admin-erasure-nothing = В истории этот человек не упоминается.
admin-erasure-col-table = Таблица или файл
admin-erasure-col-rows = Строки
admin-erasure-confirm-title = Обезличить этого человека в истории?
admin-erasure-confirm-body = Записи аудита и журналов, где назван этот человек, переписываются с псевдонимом. Это нельзя отменить; без ключа адрес по псевдониму восстановить невозможно.
admin-erasure-settings = Настройки
admin-erasure-keys = Ключи псевдонимов
admin-erasure-keys-note = Псевдонимы создаются текущим поколением ключа. Ротация добавляет поколение; прежние остаются, пока их не удалит заданный выше срок хранения.
admin-erasure-rotate = Сменить ключ
admin-erasure-keys-none = Ключ ещё не создан. Первое обезличивание создаст его.
admin-erasure-col-gen = Поколение
admin-erasure-col-created = Создан
admin-erasure-col-state = Состояние
admin-erasure-key-current = Текущий
admin-erasure-key-retired = Выведен, сохранён
admin-erasure-key-destroy = Выведен, будет удалён { $when }
admin-erasure-chain = Цепочка аудита
admin-erasure-chain-note = Каждая запись аудита содержит хеш предыдущей. Обезличивание переписывает записи и заново запечатывает цепочку, сохраняя прежний и новый хеш вершины.
admin-erasure-seal = Запечатать новые записи
admin-erasure-chain-ok = Цепочка проверена: запечатано записей — { $rows }, ожидают запечатывания — { $unsealed }.
admin-erasure-chain-bad = Цепочка не проходит проверку; первая неверная запись на позиции { $seq }.
admin-erasure-col-time = Время
admin-erasure-col-range = Позиции
admin-erasure-backups = Резервные копии
admin-erasure-backups-note = Резервная копия записывает водяной знак меток при создании. Текущий водяной знак: { $watermark }.
admin-erasure-col-label = Метка
admin-erasure-col-location = Расположение
admin-erasure-col-watermark = Водяной знак
admin-erasure-col-marker = Удалено с
admin-erasure-col-actions = Действия
admin-erasure-backups-none = Резервные копии не зарегистрированы.
admin-erasure-contains-erased = Содержит удалённых субъектов ({ $count })
admin-erasure-no-marker = Нет
admin-erasure-forget = Забыть
admin-erasure-register = Зарегистрировать резервную копию
