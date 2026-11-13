# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = تفاصيل أمان الرسالة
security-section-auth = المصادقة
security-section-received = مسار التسليم
security-section-signature = التوقيع
security-section-attachments = المرفقات
security-section-warnings = التحذيرات
security-section-controls = إجراءات المرسل
security-received-empty = لا تتوفر سلسلة Received.
security-attachments-empty = لا توجد مرفقات.
security-hop-unknown = غير معروف
security-unknown = غير معروف

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = نجح
security-auth-fail = فشل
security-auth-none = غير موجود
security-auth-neutral = محايد
security-auth-temperror = خطأ مؤقت
security-auth-permerror = خطأ دائم

security-detail-domain = النطاق
security-detail-selector = المحدِّد
security-detail-policy = السياسة
security-detail-alignment = المحاذاة
security-detail-chain-length = طول السلسلة
security-aligned = متحاذٍ
security-not-aligned = غير متحاذٍ

# -- Signature / certificate ------------------------------------------------
security-sig-verified = تم التحقق من التوقيع
security-sig-unverified-key = موقّعة — لم يُتحقق من مفتاح الموقِّع
security-sig-invalid = التوقيع غير صالح
security-sig-none = غير موقّعة
security-chain-trusted = سلسلة موثوقة
security-chain-untrusted = سلسلة غير موثوقة
security-chain-expired = السلسلة منتهية الصلاحية
security-chain-unknown = السلسلة غير معروفة
security-revocation-good = غير ملغى
security-revocation-revoked = المفتاح ملغى
security-revocation-unknown = حالة الإلغاء غير معروفة
security-key-changed = تغيّر مفتاح الموقِّع منذ آخر مرة شوهد فيها
security-fact-signer-key = مفتاح الموقِّع
security-fact-algorithm = الخوارزمية
security-fact-key-created = تاريخ إنشاء المفتاح
security-fact-key-expires = تاريخ انتهاء المفتاح
security-fact-chain = السلسلة
security-fact-revocation = الإلغاء
security-fact-key-change = تغيّر المفتاح

# -- Attachment risk --------------------------------------------------------
security-attach-none = لا يوجد خطر معروف
security-attach-macro = يحتوي على وحدات ماكرو
security-attach-executable = ملف تنفيذي
security-attach-encrypted-archive = أرشيف مشفّر
security-attach-double-extension = امتداد ملف مزدوج
security-attach-mismatch = عدم تطابق النوع (المُعلَن { $declared }، المُكتشَف { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = عنوان Reply-To يختلف عن المرسل
security-anomaly-envelopeFromDivergence = مرسل المغلف يختلف عن عنوان From
security-anomaly-messageIdDomainAnomaly = نطاق Message-ID لا يطابق المرسل
security-anomaly-dateSkew = يبدو تاريخ الإرسال منحرفًا
security-anomaly-punycodeSender = يستخدم المرسل punycode (قد يكون نطاقًا مشابهًا)

# -- Sender controls --------------------------------------------------------
security-control-block = حظر المرسل
security-control-silence = كتم المرسل
security-control-ignore-conversation = تجاهل المحادثة
security-control-report-phishing = الإبلاغ عن تصيّد
security-control-report-junk = الإبلاغ عن بريد عشوائي
security-control-done-block = حُظر المرسل
security-control-done-silence = كُتم المرسل
security-control-done-ignore-conversation = تم تجاهل المحادثة
security-control-done-report-phishing = أُبلغ عنها كتصيّد
security-control-done-report-junk = أُبلغ عنها كبريد عشوائي

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = التخزين بعدم الوصول (zero-access)
security-za-on = مفعّل
security-za-off = معطّل
security-za-protects = يشفّر التخزين بعدم الوصول بريدك وملاحظاتك وبيانات PIM المخزّنة، بحيث لا يحتفظ خادم الاستضافة إلا بنص مشفّر لا يستطيع قراءته. ويحمي محتوى بياناتك المخزّنة من مشغّل فضولي أو من اختراق مضيف التخزين: كل من يقرأ قاعدة البيانات أو الملفات على القرص أو نسخة احتياطية مسروقة يرى نصًا مشفّرًا بـ XChaCha20-Poly1305 ولا يستطيع استرداد نصوص الرسائل أو مواضيعها أو مرفقاتها أو نصوص الملاحظات أو فهرس البحث.

security-za-server-sees-title = ما الذي لا يزال الخادم يراه
security-za-sees-1 = كتل النص المشفّر — الصفوف المشفّرة نفسها.
security-za-sees-2 = معرّفات صفوف مُبهمة — المعرّفات المستخدمة لتخزين الصفوف وجلبها.
security-za-sees-3 = الأحجام — طول كل نص مشفّر (حجم تقريبي للرسالة/المرفق).
security-za-sees-4 = الطوابع الزمنية — وقت كتابة الصفوف وتحديثها.
security-za-sees-5 = بيانات توجيه المغلف اللازمة لتمرير IMAP/SMTP — لا يزال الخادم يتصل بمزوّد البريد الخاص بك نيابةً عنك، لذا تمر عبره بيانات الاتصال الوصفية ومغلف التوجيه اللازم لإرسال البريد واستقباله.

security-za-active-server-caveat = يحمي التخزين بعدم الوصول البيانات المخزّنة (أثناء السكون) من مضيف فضولي أو مُخترَق. أما الخادم النشط الخبيث بالكامل الذي يمرّر حركة IMAP/SMTP الحية الخاصة بك فهو خصم أقوى، والتخزين بعدم الوصول لا يحمي منه: فمثل هذا الخادم يقع على الاتصال الحي بمزوّد بريدك ويمكنه مراقبة البريد أو التلاعب به أثناء مروره، بغض النظر عن كيفية تشفير النسخة المخزّنة. اختر التخزين بعدم الوصول عندما يكون همّك «لا أريد لمن يشغّل مضيف التخزين (أو يخترقه) أن يقرأ بريدي المخزّن». وهو ليس حماية من مشغّل يعبث بمسار البريد الحي.

security-za-no-search-claim = لا يوجد هنا تشفير قابل للبحث على جانب الخادم، ولا يُدّعى ذلك. يجري البحث بالكامل على جهاز العميل على المحتوى الذي فُكّ تشفيره محليًا؛ ولا يحتفظ الخادم أبدًا بصيغة قابلة للبحث من نصك الصريح.

security-za-recovery-tradeoff = تُشتق مفاتيحك وتُحفظ على أجهزتك فقط — لا يتلقى الخادم أبدًا مفتاحًا بنص صريح. إذا فقدت عبارة المرور (أو مفتاح المرور) وكل جهاز مقترن وعبارة الاسترداد معًا، فلا يمكن استرداد البيانات المشفّرة. احفظ عبارة الاسترداد دون اتصال وعاملها كما تعامل السر الرئيسي لحسابك.

security-za-setup-key = إعداد مفتاحك
security-za-key-source = مصدر المفتاح
security-za-passphrase = عبارة المرور
security-za-passkey = مفتاح المرور (بدون كلمة مرور)
security-za-passkey-unavailable = مفاتيح المرور غير متاحة في هذا المتصفح
security-za-passphrase-label = عبارة مرور عدم الوصول
security-za-enable = تفعيل عدم الوصول
security-za-recovery-title = عبارة الاسترداد
security-za-recovery-heading = عبارة الاسترداد — احفظها دون اتصال الآن
security-za-recovery-note = هذه هي النسخة الوحيدة. يمكن لأي شخص يملكها قراءة بياناتك؛ ودونها (ودون جهاز مقترن) لا يمكن استرداد بياناتك.
security-za-disable-section = تعطيل عدم الوصول
security-za-disable-heading = إيقاف عدم الوصول
security-za-disable-note = ستُخزَّن البيانات الجديدة دون تشفير مرة أخرى. تبقى البيانات المشفّرة الحالية قابلة للقراءة فقط ما دمت قادرًا على اشتقاق مفتاحك.
security-za-disable-btn = تعطيل عدم الوصول
security-za-err-passphrase-len = استخدم عبارة مرور من 8 أحرف على الأقل
security-za-err-enable = تعذّر تفعيل عدم الوصول
security-za-err-disable = تعذّر تعطيل عدم الوصول

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = إقران الأجهزة
security-pair-heading = إقران جهاز
security-pair-intro = انقل مفاتيحك إلى جهاز آخر دون إرسالها عبر الخادم — فهو لا ينقل إلا مغلفًا مختومًا مُبهمًا. قارن الكلمات الست على الشاشتين قبل الوثوق بالإقران.
security-pair-role = دور الإقران
security-pair-new-role = هذا هو الجهاز الجديد
security-pair-existing-role = إقران جهاز آخر من هنا
security-pair-show-qr = عرض رمز QR للإقران
security-pair-scan = امسح هذا على جهازك الحالي
security-pair-qr-label = رمز QR لإقران الجهاز
security-pair-qr-default = رمز QR للإقران
security-pair-copy-code = أو انسخ رمز الإقران هذا
security-pair-paste-envelope = الصق المغلف المختوم من الجهاز الآخر
security-pair-envelope-placeholder = المغلف…
security-pair-envelope-label = المغلف المختوم
security-pair-complete = إتمام الإقران
security-pair-code-from-new = رمز الإقران من الجهاز الجديد
security-pair-code-placeholder = رمز الإقران…
security-pair-code-label = رمز الإقران
security-pair-seal = ختم مفاتيحي لذلك الجهاز
security-pair-envelope-out = المغلف المختوم — الصقه على الجهاز الجديد
security-pair-compare = قارن هذه الكلمات على الجهازين
security-pair-sas-label = سلسلة المصادقة القصيرة
security-pair-match = الكلمات متطابقة
security-pair-abort = إنها مختلفة — إلغاء
security-pair-confirmed = تم تأكيد الإقران. هذه القناة مُصادَق عليها.
security-pair-err-start = تعذّر بدء الإقران
security-pair-err-complete = تعذّر إتمام الإقران
security-pair-err-unlock = افتح قفل عدم الوصول أولًا لإقران جهاز آخر
security-pair-err-seal = تعذّر ختم المفتاح الجذر

# -- Machine translation (2026-11): added to match en ---------------------
security-stepup-title = أكّد أنك أنت
security-stepup-body = أكّد هويتك لمتابعة هذا الإجراء. يبقى التأكيد صالحًا لخمس دقائق في هذه الجلسة.
security-stepup-password = كلمة المرور
security-stepup-code = رمز المصادقة
security-stepup-confirm = تأكيد ومتابعة
security-stepup-passkey = استخدام مفتاح مرور
security-stepup-cancel = إلغاء
security-stepup-failed = فشل التأكيد. حاول مرة أخرى.
security-stepup-unavailable = لا توجد طريقة تأكيد متاحة لهذه الجلسة.
security-center-title = مركز الأمان
security-center-history = سجل تسجيل الدخول
security-center-result = النتيجة
security-center-all = كل النتائج
security-center-success = ناجح
security-center-failure = فاشل
security-center-blocked = محظور
security-center-refresh = تحديث
security-center-export = تصدير السجل
security-center-devices = الأجهزة المحفوظة
security-center-remember-help = إذا سمح المسؤول بذلك، يمكنك حفظ هذا الجهاز بعد التحقق من هويتك. تظل التغييرات الحساسة تتطلب تحققًا جديدًا.
security-center-label = اسم الجهاز
security-center-remember = حفظ هذا الجهاز
security-center-revoke = إلغاء
security-center-activity = نشاط الحساب
security-center-error = تعذّر تحميل معلومات الأمان أو تغييرها. قد يكون المسؤول قد عطّل هذا الإجراء.
security-center-first-seen = أول ظهور: { $when }
security-center-passkey-mode = في وضع المحرّك، يمكن لمفاتيح المرور تسجيل دخولك دون كلمة مرور. في وضع الوكيل، تتحقق من عامل ثانٍ بعد كلمة مرور بريدك.
security-stepup-recovery = رمز الاسترداد (مع كلمة المرور، بدلًا من رمز المصادقة)
security-center-signin = تسجيل الدخول ومفاتيح المرور
security-center-sessions = الجلسات والوصول
security-center-encryption = التشفير
signin-geo-title = سياسة بلد تسجيل الدخول
signin-geo-help = يمكنك اختياريًا رفض عمليات تسجيل الدخول الجديدة إلى حسابك من بلدان لم تستخدمها من قبل، أو من أي بلد خارج قائمتك المسموح بها. هذا الخيار متوقف افتراضيًا ولا يؤثر أبدًا في الجلسات المسجَّل دخولها بالفعل. تجتاز عناوين IP والبلدان المسموح بها هذا الفحص دائمًا؛ وتظل كلمة المرور والعامل الثاني مطلوبين.
signin-geo-accuracy = يعتمد كشف البلد على قاعدة بيانات لتحديد الموقع الجغرافي لعناوين IP قد تكون خاطئة أو قديمة. قد تحظر شبكات VPN ومشغّلو الهاتف المحمول والوكلاء والسفر تسجيل دخول مشروعًا أو تسمح بتسجيل دخول أجنبي. أضف عناوين IP أو بلدانًا موثوقة إلى قائمة السماح.
signin-geo-locked = يتحكم المسؤول في هذا الإعداد لحسابك.
signin-geo-mode = السياسة
signin-geo-mode-off = متوقف
signin-geo-mode-block-unusual-countries = حظر البلدان التي لم أسجّل الدخول منها
signin-geo-mode-block-foreign-countries = حظر البلدان خارج قائمتي المسموح بها
signin-geo-countries = البلدان المسموح بها (رمزان من حرفين، مفصولان بفاصلة)
signin-geo-ips = عناوين IP أو الشبكات المسموح بها (واحد في كل سطر)
signin-geo-current = عنوانك الحالي هو { $ip } (البلد المكتشف: { $country }).
signin-geo-no-context = تعذّر تحديد عنوانك الحالي.
signin-geo-unknown-country = غير معروف
signin-geo-add-ip = إضافة عنوان IP الحالي
signin-geo-add-country = إضافة بلدي الحالي
signin-geo-save = حفظ
signin-geo-saved = تم الحفظ.
signin-geo-invalid = تحقق من رموز البلدان وعناوين IP.
signin-geo-self-lockout = سيؤدي هذا إلى حظر تسجيل الدخول من عنوانك أو بلدك الحالي. أضف عنوان IP الحالي أو بلدك إلى قائمة السماح أولًا.
signin-geo-error = تعذّر حفظ سياسة بلد تسجيل الدخول.
session-alerts-title = إشعارات أمان الجلسات
session-alerts-help = تحذيرات بشأن الاستخدام غير المعتاد لجلساتك المسجَّل دخولها. يعرض كل إشعار ما رآه الخادم وما فعله.
session-alerts-none = لا توجد إشعارات أمان للجلسات.
session-alerts-banner = { $count ->
    [zero] ليس لديك إشعارات أمان غير مقروءة للجلسات.
    [one] لديك إشعار أمان واحد غير مقروء للجلسات.
    [two] لديك إشعارا أمان غير مقروءين للجلسات.
    [few] لديك { $count } إشعارات أمان غير مقروءة للجلسات.
    [many] لديك { $count } إشعار أمان غير مقروء للجلسات.
   *[other] لديك { $count } إشعار أمان غير مقروء للجلسات.
}
session-alerts-banner-review = مراجعة
session-alerts-kind-countryChange = استُخدمت جلسة من بلد مختلف ({ $from } إلى { $to }).
session-alerts-kind-impossibleTravel = ظهرت جلسة في بلد آخر ({ $from } إلى { $to }) في وقت أقصر مما يسمح به السفر.
session-alerts-kind-concurrentUse = استُخدمت جلسة واحدة من بلدان أو شبكات مختلفة في الوقت نفسه ({ $from } و{ $to }).
session-alerts-kind-fingerprintMismatch = استُخدمت جلسة من متصفح أو جهاز لا يطابق الذي سجّل الدخول.
session-alerts-kind-rotationReuse = قُدّم معرّف جلسة قديم مرة أخرى بعد تجديد الجلسة.
session-alerts-action-alerted = الإجراء: تم تنبيهك.
session-alerts-action-step-up = الإجراء: طُلب من الجلسة تأكيد كلمة مرورك والعامل الثاني.
session-alerts-action-revoked = الإجراء: تم تسجيل خروج الجلسة.
session-alerts-countries = البلدان: { $from } إلى { $to }
session-alerts-unknown-country = غير معروف
session-alerts-session = الجلسة { $id }
session-alerts-caveat = قد يكون كشف البلد غير دقيق؛ فقد تتسبب شبكات VPN وشبكات الهاتف المحمول في تغييرات زائفة.
session-alerts-not-me = لم أكن أنا
session-alerts-dismiss = تجاهل
session-alerts-dismiss-all = تجاهل الكل
session-alerts-revoked = تم تسجيل خروج جلساتك الأخرى. غيّر كلمة مرورك الآن في قسم كلمة المرور ضمن تسجيل الدخول ومفاتيح المرور.
session-alerts-error = تعذّر تحديث إشعارات أمان الجلسات.
session-alerts-prefs = تفضيلات التنبيه
session-alerts-country = حذّرني عند استخدام جلسة من بلد جديد
session-alerts-country-help = إذا لم يسمح المسؤول بكتم التنبيهات، يواصل الخادم التنبيه حتى لو كان هذا الخيار متوقفًا.
session-alerts-channels = مكان استلام التحذيرات
session-alerts-channel-app = داخل التطبيق
session-alerts-channel-email = في صندوق بريدي
session-alerts-channel-both = داخل التطبيق وفي صندوق بريدي
session-alerts-sensitivity = الحساسية
session-alerts-sensitivity-low = منخفضة: نتائج الموقع فقط، دون تسجيل خروج إجباري
session-alerts-sensitivity-normal = عادية
session-alerts-sensitivity-high = عالية: فحص أكثر صرامة للسفر، ومقارنة الشبكات أيضًا
session-alerts-reset = استخدام الإعدادات الافتراضية للمسؤول
session-alerts-pref-error = تعذّر حفظ التفضيل.
signin-geo-history-title = البلدان التي ظهرت
signin-geo-history-help = البلدان التي سجّلت الدخول منها بنجاح، وتُحفظ لمدة { $days } يومًا. يستخدم وضع "البلدان غير المعتادة" هذه القائمة. تُخزَّن رموز البلدان والتواريخ والأعداد فقط، ولا تُخزَّن العناوين أبدًا.
signin-geo-history-empty = لم تُسجَّل أي بلدان بعد.
signin-geo-history-entry = { $country }: أول مرة { $first }، آخر مرة { $last }، { $count } عمليات تسجيل دخول
signin-geo-history-forget = نسيان البلدان التي ظهرت
security-severity-info = معلومة
security-severity-caution = تنبيه
security-severity-suspicious = مشبوه
security-explain-toggle = ماذا يعني هذا؟
security-explain-label-checked = ما الذي جرى فحصه
security-explain-label-legit = أسباب شائعة غير ضارة
security-explain-label-risk = ما قد يدل عليه
security-explain-label-action = الإجراء المقترح
security-explain-messageIdDomainAnomaly-severity = { security-severity-caution }
security-explain-messageIdDomainAnomaly-meaning = قورن النطاق الذي يلي @ في ترويسة Message-ID بالنطاق في عنوان المرسل (From). وهما مختلفان.
security-explain-messageIdDomainAnomaly-legit = غالبًا ما تنشئ القوائم البريدية ومنصات التسويق والنشرات الإخبارية وخدمات إعادة التوجيه وبوابات الشركات ومضيفو البريد عبر الويب معرّف Message-ID باسم خادمها الخاص.
security-explain-messageIdDomainAnomaly-risk = قد يكون المرسل مزوّرًا، أو الرسالة منشأة ببرنامج ليس نظام البريد الخاص بالمرسل. هذه إشارة ضعيفة بمفردها.
security-explain-messageIdDomainAnomaly-action = راجع نتائج SPF وDKIM وDMARC أعلاه. إذا اجتازت لنطاق المرسل، فالأرجح أن الأمر اعتيادي. وإذا فشلت أو كانت مفقودة، فتجنب الروابط والمرفقات وتأكد من المرسل بطريقة أخرى.
security-explain-replyToMismatch-severity = { security-severity-caution }
security-explain-replyToMismatch-meaning = قورن عنوان الرد (Reply-To) بعنوان المرسل (From). ستذهب الردود إلى عنوان مختلف.
security-explain-replyToMismatch-legit = تضع القوائم البريدية ومكاتب الدعم والمساعدون الذين يكتبون نيابة عن أحد ونماذج الاتصال عنوان Reply-To مختلفًا عن قصد.
security-explain-replyToMismatch-risk = إعادة توجيه الردود إلى عنوان يتحكم فيه المهاجم، وهو نمط شائع في انتحال الهوية واحتيال الفواتير.
security-explain-replyToMismatch-action = تحقق من عنوان Reply-To قبل الرد. وإذا طلبت الرسالة مالًا أو بيانات اعتماد أو تغيير تفاصيل، فتأكد عبر قناة تثق بها بالفعل.
security-explain-envelopeFromDivergence-severity = { security-severity-info }
security-explain-envelopeFromDivergence-meaning = قورن مرسل المظروف (Return-Path)، وهو وجهة الرسائل المرتدة، بعنوان المرسل (From). وهما مختلفان.
security-explain-envelopeFromDivergence-legit = تستخدم خدمات البريد الجماعي وأنظمة التذاكر والقوائم البريدية عنوانًا خاصًا بها للرسائل المرتدة، بينما تعرض عنوان جهة اتصالك في From.
security-explain-envelopeFromDivergence-risk = انتحال المرسل، إذا فشل SPF أو DMARC أيضًا. وبمفرده شائع وغالبًا غير ضار.
security-explain-envelopeFromDivergence-action = تحقق مما إذا اجتاز DMARC مع المحاذاة. إذا اجتازه، فلا حاجة لأي إجراء.
security-explain-dateSkew-severity = { security-severity-info }
security-explain-dateSkew-meaning = قورنت ترويسة Date بوقت استلام الرسالة. وبينهما فارق كبير.
security-explain-dateSkew-legit = ساعة خاطئة في حاسوب المرسل، أو رسالة بقيت في قائمة الانتظار أو أُعيدت محاولة إرسالها طويلًا، أو إرسال مجدول.
security-explain-dateSkew-risk = رسالة مُعاد إرسالها أو مؤرَّخة بتاريخ سابق. هذا غير شائع ونادرًا ما يكون ضارًا بمفرده.
security-explain-dateSkew-action = تعامل بحذر مع الطلبات الحساسة للوقت في الرسالة وتأكد منها إذا كانت مهمة.
security-explain-punycodeSender-severity = { security-severity-suspicious }
security-explain-punycodeSender-meaning = يحتوي نطاق المرسل على punycode (تسمية xn--)، وهي الطريقة التي تُكتب بها الأحرف غير اللاتينية في أسماء النطاقات.
security-explain-punycodeSender-legit = تُرمَّز النطاقات الحقيقية بلغات مثل العربية والصينية والسيريلية والبرتغالية ذات علامات التشكيل بهذه الطريقة.
security-explain-punycodeSender-risk = نطاق مشابه في الشكل يخلط أحرفًا من أبجديات أخرى ليشبه علامة تجارية أو شخصًا معروفًا.
security-explain-punycodeSender-action = قارن نطاق المرسل حرفًا بحرف بالنطاق الذي تتوقعه. لا تنقر على الروابط ولا تفتح المرفقات حتى تتأكد من المرسل بطريقة أخرى، وأبلغ عنه إذا لم يكن حقيقيًا.
security-explain-auth-fail-severity = { security-severity-suspicious }
security-explain-auth-fail-meaning = فحص الخادم المستقبِل هذه الطريقة ولم تطابق النتيجة ما ينشره نطاق المرسل.
security-explain-auth-fail-legit = قد تُفسد إعادة التوجيه والقوائم البريدية SPF أو DKIM، وقد تفشل أي منهما في خدمة إرسال مهيأة بشكل خاطئ.
security-explain-auth-fail-risk = قد لا تأتي الرسالة ممن تدّعي، أو قد تكون عُدّلت أثناء النقل.
security-explain-auth-fail-action = لا تثق بالمرسل استنادًا إلى هذا الدليل. تجنب الروابط والمرفقات وتحقق عبر قناة أخرى.
security-explain-auth-permerror-severity = { security-severity-caution }
security-explain-auth-permerror-meaning = تعذّر إكمال الفحص لأن السجل المنشور للنطاق غير صالح أو شديد التعقيد.
security-explain-auth-permerror-legit = خطأ في سجلات DNS الخاصة بالمرسل.
security-explain-auth-permerror-risk = لا شيء بشأن الرسالة نفسها، لكن لا يمكن التحقق من نطاق المرسل.
security-explain-auth-permerror-action = تعامل معها كأنها نتيجة مفقودة وراجع الطرق الأخرى.
security-explain-auth-none-severity = { security-severity-info }
security-explain-auth-none-meaning = لم يُعثر على نتيجة لهذه الطريقة في ترويسات الرسالة.
security-explain-auth-none-legit = لا ينشر نطاق المرسل هذه الطريقة، أو لا يسجّلها الخادم المستقبِل.
security-explain-auth-none-risk = لا يوفّر النطاق وسيلة للتحقق من المرسل، لذا لا يمكن استبعاد الانتحال.
security-explain-auth-none-action = اعتمد على الطرق الأخرى وعلى ما إذا كنت تتوقع هذه الرسالة.
security-explain-auth-neutral-severity = { security-severity-info }
security-explain-auth-neutral-meaning = نشر النطاق سجلًا لهذه الطريقة لكنه لم يقدّم أي ادعاء بشأن هذه الرسالة.
security-explain-auth-neutral-legit = نطاق ما زال في طور إعداد سياسته.
security-explain-auth-neutral-risk = قليل. لم يمكن تأكيد المرسل ولا رفضه.
security-explain-auth-neutral-action = اعتمد على الطرق الأخرى وعلى ما إذا كنت تتوقع هذه الرسالة.
security-explain-auth-temperror-severity = { security-severity-info }
security-explain-auth-temperror-meaning = تعذّر إنهاء الفحص بسبب مشكلة مؤقتة، وهي عادةً فشل في استعلام DNS.
security-explain-auth-temperror-legit = انقطاع قصير في DNS لدى المرسل أو المستقبِل.
security-explain-auth-temperror-risk = لا شيء بشأن الرسالة نفسها.
security-explain-auth-temperror-action = لا حاجة عادةً لأي إجراء. أعد الفحص لاحقًا إذا كانت الرسالة مهمة.
breach-notice-heading = إشعار بشأن خرق للبيانات الشخصية
breach-notice-reference = المرجع { $reference }
breach-notice-ack = إقرار بالاطلاع

session-evidence-ip = عنوان IP
session-evidence-location = الموقع
session-evidence-device = نوع الجهاز
session-evidence-recognized = التعرف على الجهاز
session-evidence-browser = المتصفح
session-evidence-engine = محرك المتصفح
session-evidence-os = نظام التشغيل
session-evidence-client = نوع العميل
session-evidence-version = إصدار العميل
session-evidence-authentication = المصادقة
session-evidence-fingerprint = بصمة الجهاز
session-evidence-agent = وكيل المستخدم
session-evidence-unavailable = غير متاح
session-evidence-known = جهاز معروف
session-evidence-new = جهاز غير معروف
session-evidence-technical = التفاصيل التقنية
session-evidence-legacy = لم تُسجّل تفاصيل الجهاز لهذا الإشعار القديم.
session-evidence-help = يبلغ العميل عن تفاصيل الجهاز والمتصفح. التعرف معلوماتي ولا يثبت أمان الجهاز. البصمة مرجع خاص بالحساب وليست معرّفاً للأجهزة.
session-evidence-device-desktop = حاسوب
session-evidence-device-phone = هاتف
session-evidence-device-tablet = جهاز لوحي
session-evidence-device-unknown = جهاز غير معروف
session-evidence-client-browser = متصفح ويب
session-alerts-kind-countryUnavailable = تعذر تحديد بلد طلب الجلسة.
session-alerts-kind-unknown = يرجى مراجعة نشاط الجلسة غير المعتاد.

notification-center-help = راجع نشاط الحساب المهم والتحديثات. التجاهل والأرشفة ينظمان القائمة ولا يحلان مشكلات الأمان.
notification-center-active = نشطة
notification-center-archived = مؤرشفة
notification-center-dismissed = متجاهلة
notification-center-type = النوع
notification-center-type-all = كل الأنواع
notification-center-type-security = الأمان
notification-center-type-session = تحذيرات الجلسة
notification-center-type-login = تسجيلات الدخول
notification-center-type-update = تحديثات التطبيق
notification-center-login = تسجيل دخول ناجح
notification-center-security = نشاط أمان الحساب
notification-center-newDevice = تسجيل دخول من جهاز غير معروف
notification-center-newCountry = تسجيل دخول من بلد جديد
notification-center-twoFactorChanged = تغيرت المصادقة الثنائية
notification-center-delegateAdded = تم منح وصول إلى البريد
notification-center-forwardingAdded = أضيفت قاعدة إعادة توجيه
notification-center-apiKeyCreated = أُنشئ مفتاح API
notification-center-error = تعذر تحميل الإشعارات أو تحديثها. حاول التحديث.
notification-center-empty = لا توجد إشعارات في هذا العرض.
notification-center-open = فتح الإجراء المرتبط
notification-center-dismiss = تجاهل
notification-center-archive = أرشفة
notification-center-restore = استعادة
