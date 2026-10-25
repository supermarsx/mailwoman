# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: uk · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Відомості про безпеку повідомлення
security-section-auth = Автентифікація
security-section-received = Шлях доставлення
security-section-signature = Підпис
security-section-attachments = Вкладення
security-section-warnings = Попередження
security-section-controls = Дії щодо відправника
security-received-empty = Ланцюжок Received недоступний.
security-attachments-empty = Немає вкладень.
security-hop-unknown = невідомо
security-unknown = невідомо

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = пройдено
security-auth-fail = не пройдено
security-auth-none = відсутній
security-auth-neutral = нейтрально
security-auth-temperror = тимчасова помилка
security-auth-permerror = постійна помилка

# Expert alignment detail field labels + values.
security-detail-domain = домен
security-detail-selector = селектор
security-detail-policy = політика
security-detail-alignment = узгодження
security-detail-chain-length = довжина ланцюжка
security-aligned = узгоджено
security-not-aligned = не узгоджено

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Підпис перевірено
security-sig-unverified-key = Підписано — ключ підписувача не перевірено
security-sig-invalid = Підпис недійсний
security-sig-none = Не підписано
security-chain-trusted = Довірений ланцюжок
security-chain-untrusted = Недовірений ланцюжок
security-chain-expired = Термін дії ланцюжка минув
security-chain-unknown = Ланцюжок невідомий
security-revocation-good = Не відкликано
security-revocation-revoked = Ключ відкликано
security-revocation-unknown = Стан відкликання невідомий
security-key-changed = Ключ підписувача змінився з часу останньої перевірки
security-fact-signer-key = Ключ підписувача
security-fact-algorithm = Алгоритм
security-fact-key-created = Ключ створено
security-fact-key-expires = Термін дії ключа
security-fact-chain = Ланцюжок
security-fact-revocation = Відкликання
security-fact-key-change = Зміна ключа

# -- Attachment risk --------------------------------------------------------
security-attach-none = Відомих ризиків немає
security-attach-macro = Містить макроси
security-attach-executable = Виконуваний файл
security-attach-encrypted-archive = Зашифрований архів
security-attach-double-extension = Подвійне розширення файлу
security-attach-mismatch = невідповідність типу (заявлено { $declared }, виявлено { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = Адреса Reply-To відрізняється від адреси відправника
security-anomaly-envelopeFromDivergence = Відправник конверта відрізняється від адреси From
security-anomaly-messageIdDomainAnomaly = Домен Message-ID не відповідає відправнику
security-anomaly-dateSkew = Дата надсилання виглядає зміщеною
security-anomaly-punycodeSender = Відправник використовує punycode (можливо, схожий на справжній домен)

# -- Sender controls --------------------------------------------------------
security-control-block = Заблокувати відправника
security-control-silence = Приглушити відправника
security-control-ignore-conversation = Ігнорувати ланцюжок
security-control-report-phishing = Повідомити про фішинг
security-control-report-junk = Позначити як спам
security-control-done-block = Відправника заблоковано
security-control-done-silence = Відправника приглушено
security-control-done-ignore-conversation = Ланцюжок ігнорується
security-control-done-report-phishing = Повідомлено про фішинг
security-control-done-report-junk = Позначено як спам

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = Сховище з нульовим доступом
security-za-on = Увімкнено
security-za-off = Вимкнено
security-za-protects = Нульовий доступ шифрує збережену пошту, нотатки та дані PIM, тож сервер хостингу зберігає лише шифротекст, який не може прочитати. Він захищає вміст збережених даних від цікавого оператора або злому сховища: будь-хто, хто прочитає базу даних, файли на диску чи викрадену резервну копію, побачить шифротекст XChaCha20-Poly1305 і не зможе відновити тексти повідомлень, теми, вкладення, текст нотаток або пошуковий індекс.

security-za-server-sees-title = Що сервер усе ще бачить
security-za-sees-1 = Блоки шифротексту — самі зашифровані рядки.
security-za-sees-2 = Непрозорі ідентифікатори рядків — ідентифікатори, за якими рядки зберігаються та отримуються.
security-za-sees-3 = Розміри — довжина кожного шифротексту (приблизний розмір повідомлення або вкладення).
security-za-sees-4 = Позначки часу — коли рядки записано та оновлено.
security-za-sees-5 = Метадані маршрутизації конверта, потрібні для проксіювання IMAP/SMTP — сервер і далі підключається до вашого вищестоящого поштового провайдера від вашого імені, тому через нього проходять метадані з'єднання та конверт маршрутизації, потрібні для надсилання й отримання пошти.

security-za-active-server-caveat = Нульовий доступ захищає дані В СТАНІ ЗБЕРІГАННЯ від цікавого або зламаного хоста. Повністю зловмисний активний сервер, що проксіює ваш поточний трафік IMAP/SMTP, — сильніший супротивник, і нульовий доступ від нього НЕ захищає: такий сервер перебуває на активному з'єднанні з вашим поштовим провайдером і може спостерігати за поштою або змінювати її під час проходження, незалежно від того, як зашифровано збережену копію. Обирайте нульовий доступ, якщо ваша мета — «я не хочу, щоб той, хто керує сховищем (або зламав його), читав мою збережену пошту». Це не захист від оператора, який втручається в активний шлях пошти.

security-za-no-search-claim = Серверного шифрування з можливістю пошуку тут немає, і такого не стверджується. Пошук повністю виконується на клієнті за вмістом, розшифрованим локально; сервер ніколи не зберігає придатну для пошуку форму вашого відкритого тексту.

security-za-recovery-tradeoff = Ваші ключі виводяться та зберігаються лише на ваших пристроях — сервер ніколи не отримує ключ у відкритому вигляді. Якщо ви втратите парольну фразу (або ключ доступу), І всі пов'язані пристрої, І фразу відновлення, зашифровані дані відновити неможливо. Збережіть фразу відновлення офлайн і ставтеся до неї як до головного секрету облікового запису.

security-za-setup-key = Налаштування ключа
security-za-key-source = Джерело ключа
security-za-passphrase = Парольна фраза
security-za-passkey = Ключ доступу (без пароля)
security-za-passkey-unavailable = ключі доступу недоступні в цьому браузері
security-za-passphrase-label = Парольна фраза нульового доступу
security-za-enable = Увімкнути нульовий доступ
security-za-recovery-title = Фраза відновлення
security-za-recovery-heading = Фраза відновлення — збережіть її офлайн зараз
security-za-recovery-note = Це єдина копія. Будь-хто, хто її має, може прочитати ваші дані; без неї (і без пов'язаного пристрою) ваші дані відновити неможливо.
security-za-disable-section = Вимкнення нульового доступу
security-za-disable-heading = Вимкнути нульовий доступ
security-za-disable-note = Нові дані знову зберігатимуться без шифрування. Наявні зашифровані дані залишаться доступними для читання, лише доки ви можете вивести свій ключ.
security-za-disable-btn = Вимкнути нульовий доступ
security-za-err-passphrase-len = використовуйте парольну фразу щонайменше з 8 символів
security-za-err-enable = не вдалося ввімкнути нульовий доступ
security-za-err-disable = не вдалося вимкнути нульовий доступ

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Зв'язування пристроїв
security-pair-heading = Зв'язати пристрій
security-pair-intro = Перенесіть ключі на інший пристрій, не надсилаючи їх через сервер — він передає лише непрозорий запечатаний конверт. Перш ніж довіряти зв'язуванню, порівняйте шість слів на обох екранах.
security-pair-role = Роль у зв'язуванні
security-pair-new-role = Це новий пристрій
security-pair-existing-role = Зв'язати інший пристрій звідси
security-pair-show-qr = Показати QR-код зв'язування
security-pair-scan = Відскануйте це на наявному пристрої
security-pair-qr-label = QR-код зв'язування пристрою
security-pair-qr-default = QR-код зв'язування
security-pair-copy-code = Або скопіюйте цей код зв'язування
security-pair-paste-envelope = Вставте запечатаний конверт з іншого пристрою
security-pair-envelope-placeholder = конверт…
security-pair-envelope-label = Запечатаний конверт
security-pair-complete = Завершити зв'язування
security-pair-code-from-new = Код зв'язування з нового пристрою
security-pair-code-placeholder = код зв'язування…
security-pair-code-label = Код зв'язування
security-pair-seal = Запечатати мої ключі для цього пристрою
security-pair-envelope-out = Запечатаний конверт — вставте його на новому пристрої
security-pair-compare = Порівняйте ці слова на обох пристроях
security-pair-sas-label = Короткий рядок автентифікації
security-pair-match = Слова збігаються
security-pair-abort = Вони відрізняються — перервати
security-pair-confirmed = Зв'язування підтверджено. Цей канал автентифіковано.
security-pair-err-start = не вдалося почати зв'язування
security-pair-err-complete = не вдалося завершити зв'язування
security-pair-err-unlock = спершу розблокуйте нульовий доступ, щоб зв'язати інший пристрій
security-pair-err-seal = не вдалося запечатати кореневий ключ
