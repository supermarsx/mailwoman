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
