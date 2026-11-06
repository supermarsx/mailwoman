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

# -- Machine translation (2026-11): added to match en ---------------------
security-stepup-title = Підтвердьте, що це ви
security-stepup-body = Підтвердьте свою особу, щоб продовжити цю дію. Підтвердження діє п’ять хвилин у цьому сеансі.
security-stepup-password = Пароль
security-stepup-code = Код із застосунку-автентифікатора
security-stepup-confirm = Підтвердити й продовжити
security-stepup-passkey = Використати ключ доступу
security-stepup-cancel = Скасувати
security-stepup-failed = Не вдалося підтвердити. Спробуйте ще раз.
security-stepup-unavailable = У цьому сеансі немає доступного способу підтвердження.
security-center-title = Центр безпеки
security-center-history = Історія входів
security-center-result = Результат
security-center-all = Усі результати
security-center-success = Успішні
security-center-failure = Невдалі
security-center-blocked = Заблоковані
security-center-refresh = Оновити
security-center-export = Експортувати історію
security-center-devices = Запам’ятовані пристрої
security-center-remember-help = Якщо адміністратор це дозволяє, запам’ятайте цей пристрій після підтвердження особи. Для важливих змін усе одно потрібне нове підтвердження.
security-center-label = Назва пристрою
security-center-remember = Запам’ятати цей пристрій
security-center-revoke = Відкликати
security-center-activity = Активність облікового запису
security-center-error = Не вдалося завантажити або змінити відомості про безпеку. Можливо, адміністратор вимкнув цю дію.
security-center-first-seen = Уперше помічено: { $when }
security-center-passkey-mode = У режимі рушія ключі доступу дають змогу входити без пароля. У режимі проксі вони підтверджують другий фактор після пароля від пошти.
security-stepup-recovery = Код відновлення (разом із паролем, замість коду із застосунку-автентифікатора)
security-center-signin = Вхід і ключі доступу
security-center-sessions = Сеанси та доступ
security-center-encryption = Шифрування
signin-geo-title = Політика країн для входу
signin-geo-help = За бажанням можна відхиляти нові входи до вашого облікового запису з країн, з яких ви ще не входили, або з будь-якої країни поза вашим списком дозволених. Типово політика вимкнена й ніколи не впливає на сеанси, що вже ввійшли. Дозволені IP-адреси та країни завжди проходять цю перевірку; пароль і другий фактор усе одно потрібні.
signin-geo-accuracy = Країна визначається за базою геолокації IP-адрес, яка може бути неточною або застарілою. VPN, мобільні оператори, проксі та подорожі можуть заблокувати законний вхід або пропустити сторонній. Додайте довірені IP-адреси або країни до списку дозволених.
signin-geo-locked = Цим налаштуванням для вашого облікового запису керує адміністратор.
signin-geo-mode = Політика
signin-geo-mode-off = Вимкнено
signin-geo-mode-block-unusual-countries = Блокувати країни, з яких я не входив
signin-geo-mode-block-foreign-countries = Блокувати країни поза моїм списком дозволених
signin-geo-countries = Дозволені країни (дволітерні коди через кому)
signin-geo-ips = Дозволені IP-адреси або мережі (по одній в рядку)
signin-geo-current = Ваша поточна адреса: { $ip } (визначена країна: { $country }).
signin-geo-no-context = Не вдалося визначити вашу поточну адресу.
signin-geo-unknown-country = невідомо
signin-geo-add-ip = Додати мою поточну IP-адресу
signin-geo-add-country = Додати мою поточну країну
signin-geo-save = Зберегти
signin-geo-saved = Збережено.
signin-geo-invalid = Перевірте коди країн та IP-адреси.
signin-geo-self-lockout = Це заблокує вхід із вашої поточної адреси або країни. Спершу додайте поточну IP-адресу або країну до списку дозволених.
signin-geo-error = Не вдалося зберегти політику країн для входу.
session-alerts-title = Сповіщення про безпеку сеансів
session-alerts-help = Попередження про незвичне використання ваших сеансів. У кожному сповіщенні вказано, що помітив сервер і що він зробив.
session-alerts-none = Сповіщень про безпеку сеансів немає.
session-alerts-banner = { $count ->
    [one] У вас { $count } непрочитане сповіщення про безпеку сеансів.
    [few] У вас { $count } непрочитані сповіщення про безпеку сеансів.
    [many] У вас { $count } непрочитаних сповіщень про безпеку сеансів.
   *[other] У вас { $count } непрочитаного сповіщення про безпеку сеансів.
}
session-alerts-banner-review = Переглянути
session-alerts-kind-countryChange = Сеанс використовувався з іншої країни ({ $from } → { $to }).
session-alerts-kind-impossibleTravel = Сеанс з’явився в іншій країні ({ $from } → { $to }) швидше, ніж це можливо під час подорожі.
session-alerts-kind-concurrentUse = Один сеанс одночасно використовувався з різних країн або мереж ({ $from } і { $to }).
session-alerts-kind-fingerprintMismatch = Сеанс використовувався з браузера або пристрою, що не збігаються з тими, з яких виконано вхід.
session-alerts-kind-rotationReuse = Старий ідентифікатор сеансу було пред’явлено знову після оновлення сеансу.
session-alerts-action-alerted = Дія: вас попереджено.
session-alerts-action-step-up = Дія: для сеансу запитано підтвердження пароля та другого фактора.
session-alerts-action-revoked = Дія: сеанс завершено.
session-alerts-countries = Країни: { $from } → { $to }
session-alerts-unknown-country = невідомо
session-alerts-session = Сеанс { $id }
session-alerts-caveat = Країна може визначатися неточно; VPN і мобільні мережі можуть спричиняти хибні зміни.
session-alerts-not-me = Це був не я
session-alerts-dismiss = Закрити
session-alerts-dismiss-all = Закрити всі
session-alerts-revoked = Решту ваших сеансів завершено. Негайно змініть пароль у розділі пароля на сторінці «Вхід і ключі доступу».
session-alerts-error = Не вдалося оновити сповіщення про безпеку сеансів.
session-alerts-prefs = Налаштування сповіщень
session-alerts-country = Попереджати, коли сеанс використовується з нової країни
session-alerts-country-help = Якщо адміністратор не дозволяє вимикати сповіщення, сервер і далі попереджатиме, навіть коли цей параметр вимкнено.
session-alerts-channels = Де отримувати попередження
session-alerts-channel-app = У застосунку
session-alerts-channel-email = У моїй поштовій скриньці
session-alerts-channel-both = У застосунку та в моїй поштовій скриньці
session-alerts-sensitivity = Чутливість
session-alerts-sensitivity-low = Низька: лише висновки про місцезнаходження, без примусового виходу
session-alerts-sensitivity-normal = Звичайна
session-alerts-sensitivity-high = Висока: суворіша перевірка подорожей, також порівнюються мережі
session-alerts-reset = Використовувати типові значення адміністратора
session-alerts-pref-error = Не вдалося зберегти налаштування.
signin-geo-history-title = Помічені країни
signin-geo-history-help = Країни, з яких ви успішно входили; зберігаються { $days } дн. Режим «незвичні країни» використовує цей список. Зберігаються лише коди країн, дати й лічильники, але не адреси.
signin-geo-history-empty = Країни поки не записано.
signin-geo-history-entry = { $country }: перший вхід { $first }, останній { $last }, входів: { $count }
signin-geo-history-forget = Забути помічені країни
security-severity-info = Інформація
security-severity-caution = Обережність
security-severity-suspicious = Підозріло
security-explain-toggle = Що це означає?
security-explain-label-checked = Що перевірялося
security-explain-label-legit = Типові безпечні причини
security-explain-label-risk = Про що це може свідчити
security-explain-label-action = Рекомендована дія
security-explain-messageIdDomainAnomaly-severity = { security-severity-caution }
security-explain-messageIdDomainAnomaly-meaning = Домен після символу @ у заголовку Message-ID порівняли з доменом в адресі відправника (From). Вони різні.
security-explain-messageIdDomainAnomaly-legit = Розсилки, маркетингові та новинні платформи, служби пересилання, корпоративні шлюзи й хости вебпошти часто формують Message-ID з назвою власного сервера.
security-explain-messageIdDomainAnomaly-risk = Підроблений відправник або лист, створений програмою, яка не є поштовою системою відправника. Саме по собі це слабкий сигнал.
security-explain-messageIdDomainAnomaly-action = Перегляньте результати SPF, DKIM і DMARC вище. Якщо для домену відправника перевірки пройдено, найімовірніше, це звичайна ситуація. Якщо їх не пройдено або вони відсутні, не відкривайте посилання та вкладення й підтвердьте відправника іншим способом.
security-explain-replyToMismatch-severity = { security-severity-caution }
security-explain-replyToMismatch-meaning = Адресу Reply-To порівняли з адресою From. Відповіді надходитимуть на іншу адресу.
security-explain-replyToMismatch-legit = Розсилки, служби підтримки, помічники, що пишуть від чужого імені, і контактні форми навмисно задають інший Reply-To.
security-explain-replyToMismatch-risk = Перенаправлення відповідей на адресу, яку контролює зловмисник, — поширений прийом у підміні особи та шахрайстві з рахунками.
security-explain-replyToMismatch-action = Перевірте адресу Reply-To перед відповіддю. Якщо в листі просять гроші, облікові дані або зміну реквізитів, підтвердьте це каналом, якому ви вже довіряєте.
security-explain-envelopeFromDivergence-severity = { security-severity-info }
security-explain-envelopeFromDivergence-meaning = Відправника конверта (Return-Path), на який надходять повідомлення про недоставку, порівняли з адресою From. Вони різні.
security-explain-envelopeFromDivergence-legit = Сервіси масових розсилок, системи заявок і розсилки використовують власну адресу для повернень, а в полі From показують адресу вашого контакту.
security-explain-envelopeFromDivergence-risk = Підміна відправника, якщо SPF або DMARC також не пройдено. Саме по собі це поширене й зазвичай безпечне явище.
security-explain-envelopeFromDivergence-action = Перевірте, чи пройдено DMARC із вирівнюванням. Якщо так, жодних дій не потрібно.
security-explain-dateSkew-severity = { security-severity-info }
security-explain-dateSkew-meaning = Заголовок Date порівняли з часом отримання листа. Вони сильно розходяться.
security-explain-dateSkew-legit = Комп’ютер відправника з неправильним годинником, лист, що довго лежав у черзі або повторно надсилався, чи відкладене надсилання.
security-explain-dateSkew-risk = Повторно надісланий або датований заднім числом лист. Це буває рідко й саме по собі рідко буває шкідливим.
security-explain-dateSkew-action = Обережно ставтеся до термінових прохань у листі та підтверджуйте їх, якщо вони важливі.
security-explain-punycodeSender-severity = { security-severity-suspicious }
security-explain-punycodeSender-meaning = Домен відправника містить punycode (мітку xn--) — так у доменних іменах записуються нелатинські символи.
security-explain-punycodeSender-legit = Так кодуються справжні домени арабською, китайською, кирилицею або португальською з діакритикою.
security-explain-punycodeSender-risk = Схожий домен, у якому змішано символи різних алфавітів, щоб скидатися на відомий бренд або людину.
security-explain-punycodeSender-action = Порівняйте домен відправника з очікуваним посимвольно. Не переходьте за посиланнями й не відкривайте вкладень, доки не підтвердите відправника іншим способом, і повідомте про лист, якщо він підроблений.
security-explain-auth-fail-severity = { security-severity-suspicious }
security-explain-auth-fail-meaning = Сервер-одержувач перевірив цей метод, і результат не збігся з тим, що публікує домен відправника.
security-explain-auth-fail-legit = Пересилання та розсилки можуть порушити SPF або DKIM, а неправильно налаштований сервіс надсилання може не пройти будь-яку з перевірок.
security-explain-auth-fail-risk = Лист може бути не від того, ким назвався відправник, або його змінили дорогою.
security-explain-auth-fail-action = Не довіряйте відправнику на підставі цієї ознаки. Не відкривайте посилання та вкладення й перевірте іншим каналом.
security-explain-auth-permerror-severity = { security-severity-caution }
security-explain-auth-permerror-meaning = Перевірку не вдалося завершити, бо опублікований запис домену недійсний або надто складний.
security-explain-auth-permerror-legit = Помилка в DNS-записах відправника.
security-explain-auth-permerror-risk = Нічого про сам лист, але домен відправника неможливо перевірити.
security-explain-auth-permerror-action = Вважайте це відсутністю результату й перевірте інші методи.
security-explain-auth-none-severity = { security-severity-info }
security-explain-auth-none-meaning = У заголовках листа не знайдено результату для цього методу.
security-explain-auth-none-legit = Домен відправника не публікує цей метод, або сервер-одержувач його не записує.
security-explain-auth-none-risk = Домен не дає змоги перевірити відправника, тому підміну виключити не можна.
security-explain-auth-none-action = Спирайтеся на інші методи та на те, чи очікували ви цей лист.
security-explain-auth-neutral-severity = { security-severity-info }
security-explain-auth-neutral-meaning = Домен опублікував запис для цього методу, але нічого не заявив щодо цього листа.
security-explain-auth-neutral-legit = Домен, який ще налаштовує свою політику.
security-explain-auth-neutral-risk = Невеликий. Відправника не вдалося ні підтвердити, ні відхилити.
security-explain-auth-neutral-action = Спирайтеся на інші методи та на те, чи очікували ви цей лист.
security-explain-auth-temperror-severity = { security-severity-info }
security-explain-auth-temperror-meaning = Перевірку не вдалося завершити через тимчасову проблему, зазвичай збій DNS-запиту.
security-explain-auth-temperror-legit = Короткий збій DNS у відправника або одержувача.
security-explain-auth-temperror-risk = Нічого про сам лист.
security-explain-auth-temperror-action = Зазвичай дій не потрібно. Якщо лист важливий, перевірте ще раз пізніше.
breach-notice-heading = Повідомлення про порушення захисту персональних даних
breach-notice-reference = Номер звернення { $reference }
breach-notice-ack = Підтвердити ознайомлення
