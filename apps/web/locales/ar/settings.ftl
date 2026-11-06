# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: settings. Source: ../en/settings.ftl

settings-title = الإعدادات
settings-appearance = المظهر
settings-close = إغلاق الإعدادات

settings-theme = السمة
settings-density = الكثافة
settings-accent = اللون المميز
settings-font = خط الواجهة
settings-layout = التخطيط

settings-density-compact = مضغوط
settings-density-cozy = مريح
settings-density-relaxed = فسيح

settings-font-default = الافتراضي
settings-font-system = النظام
settings-font-serif = مذيّل
settings-font-mono = أحادي المسافة

settings-layout-default = الافتراضي
settings-layout-ribbon = شريط

settings-cancel = إلغاء
settings-edit = تحرير
settings-delete = حذف
settings-save = حفظ
settings-saved = تم الحفظ

settings-2fa-title = المصادقة الثنائية
settings-2fa-intro = أضِف عاملاً ثانياً ليطلب أكثر من كلمة مرور عند تسجيل الدخول.
settings-2fa-policy-required = تتطلب مؤسستك عاملاً ثانياً على هذا الحساب.
settings-2fa-enabled = مُفعّل
settings-2fa-recovery-title = رموز الاسترداد
settings-2fa-recovery-once = احفظها الآن. تظهر مرة واحدة ولا يمكن استرجاعها. يعمل كل رمز مرة واحدة.
settings-2fa-recovery-ack = لقد حفظت هذه الرموز
settings-2fa-totp-title = تطبيق المصادقة
settings-2fa-totp-enrol = إعداد تطبيق مصادقة
settings-2fa-totp-scan = أضِف هذا السر إلى تطبيق المصادقة، ثم أدخل الرمز الذي يعرضه.
settings-2fa-totp-uri-link = فتح في تطبيق مصادقة
settings-2fa-totp-code-label = رمز المصادقة
settings-2fa-totp-confirm = تأكيد
settings-2fa-totp-disable = إزالة تطبيق المصادقة
settings-2fa-passkey-title = مفاتيح المرور
settings-2fa-passkey-remove = إزالة
settings-2fa-passkey-unsupported = هذا المتصفح لا يدعم مفاتيح المرور.
settings-2fa-passkey-label-placeholder = اسم مفتاح المرور (اختياري)
settings-2fa-passkey-add = إضافة مفتاح مرور
settings-2fa-recovery-remaining = { $count ->
    [zero] لم يتبقَّ أي رمز استرداد
    [one] تبقّى رمز استرداد واحد
    [two] تبقّى رمزا استرداد
    [few] تبقّت { $count } رموز استرداد
    [many] تبقّى { $count } رمزًا للاسترداد
   *[other] تبقّى { $count } رمز استرداد
}
settings-2fa-recovery-regenerate = توليد رموز استرداد جديدة
settings-2fa-recovery-code-label = رمز الاسترداد
settings-2fa-error-generic = لم ينجح ذلك. حاول مرة أخرى.
settings-2fa-error-code = لم يتم التحقق من الرمز.
settings-2fa-error-passkey = تعذّر تسجيل مفتاح المرور.

settings-2fa-challenge-title = المصادقة الثنائية
settings-2fa-challenge-intro = أكِّد عاملك الثاني لإكمال تسجيل الدخول.
settings-2fa-challenge-failed = لم يتم التحقق. حاول مرة أخرى.
settings-2fa-challenge-use-passkey = استخدام مفتاح مرور
settings-2fa-challenge-method = اختر طريقة
settings-2fa-challenge-totp-tab = رمز المصادقة
settings-2fa-challenge-recovery-tab = رمز الاسترداد
settings-2fa-challenge-verify = تحقّق

settings-sessions-title = الجلسات النشطة
settings-sessions-intro = الجلسات المسجّلة دخولها حالياً على هذا الحساب.
settings-sessions-last-seen = آخر نشاط { $when }
settings-sessions-revoke = تسجيل الخروج
settings-sessions-current = هذه الجلسة
settings-sessions-revoke-others = { $count ->
    [zero] تسجيل الخروج من { $count } جلسة أخرى
    [one] تسجيل الخروج من جلسة أخرى واحدة
    [two] تسجيل الخروج من جلستين أخريين
    [few] تسجيل الخروج من { $count } جلسات أخرى
    [many] تسجيل الخروج من { $count } جلسة أخرى
   *[other] تسجيل الخروج من { $count } جلسة أخرى
}
settings-sessions-error = تعذّر تغيير الجلسة.

settings-sig-title = التواقيع
settings-sig-intro = نص يُضاف إلى الرسائل التي ترسلها. يمكن جعل توقيع واحد افتراضياً.
settings-sig-default = افتراضي
settings-sig-new = توقيع جديد
settings-sig-name-label = الاسم
settings-sig-body-label = التوقيع
settings-sig-default-label = استخدامه كافتراضي
settings-sig-error-name = أعطِ التوقيع اسماً.
settings-sig-error-generic = تعذّر حفظ التوقيع.

settings-ident-title = الهويات
settings-ident-intro = الاسم والعنوان اللذان ترسل بهما. يمكن لكل هوية استخدام توقيعها.
settings-ident-new = هوية جديدة
settings-ident-name-label = الاسم المعروض
settings-ident-email-label = عنوان البريد الإلكتروني
settings-ident-replyto-label = عنوان الرد (اختياري)
settings-ident-signature-label = التوقيع الافتراضي
settings-ident-signature-none = بلا
settings-ident-error-fields = أدخل اسماً وعنوان بريد إلكتروني صالحاً.
settings-ident-error-generic = تعذّر حفظ الهوية.

settings-notif-title = الإشعارات
settings-notif-intro = اختر أي الرسائل الجديدة تُشعرك، ومتى تبقى هادئاً.
settings-notif-enabled-label = إظهار إشعارات البريد الجديد
settings-notif-quiet-title = ساعات الهدوء
settings-notif-quiet-enabled-label = كتم الإشعارات خلال وقت محدد
settings-notif-quiet-start = من
settings-notif-quiet-end = إلى
settings-notif-rules-title = القواعد
settings-notif-rule-match-placeholder = المرسِل أو المجلد أو الموضوع يحتوي…
settings-notif-rule-action-label = الإجراء
settings-notif-rule-notify = إشعار
settings-notif-rule-mute = كتم
settings-notif-rule-add = إضافة قاعدة
settings-notif-error = تعذّر حفظ إعدادات الإشعارات.

settings-search-title = عمليات البحث المحفوظة
settings-search-intro = أظهِر بحثاً محفوظاً كمجلد في قائمة صندوق بريدك.
settings-search-empty = لا توجد لديك عمليات بحث محفوظة بعد.
settings-search-as-folder = إظهار { $name } كمجلد
settings-search-as-folder-label = إظهار كمجلد
settings-search-error = تعذّر تغيير البحث المحفوظ.

settings-kbd-title = اختصارات لوحة المفاتيح
settings-kbd-intro = اختر مجموعة اختصارات. تُطبَّق التغييرات على هذا الجهاز.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = إيقاف (بلا اختصارات)
settings-kbd-action-compose = إنشاء
settings-kbd-action-archive = أرشفة
settings-kbd-action-reply = رد
settings-kbd-action-next = الرسالة التالية
settings-kbd-action-previous = الرسالة السابقة
settings-kbd-action-search = بحث

settings-offline-title = التخزين المؤقت دون اتصال
settings-offline-intro = مقدار البريد المحفوظ على هذا الجهاز للاستخدام دون اتصال، وكيفية استعادة المساحة.
settings-offline-budget-label = حد التخزين (ميغابايت)
settings-offline-retention-label = الاحتفاظ لمدة (أيام)
settings-offline-strategy-label = عند بلوغ الحد
settings-offline-lru = إزالة الأقل استخداماً مؤخراً
settings-offline-oldest = إزالة الأقدم أولاً
settings-offline-manual = فقط عند مسحه يدوياً
settings-offline-purge = مسح التخزين المؤقت الآن

settings-dir-title = اتجاه الواجهة
settings-dir-intro = اتبع اللغة، أو افرض اتجاهاً. من اليمين إلى اليسار يعكس التخطيط.
settings-dir-auto = تلقائي
settings-dir-ltr = من اليسار إلى اليمين
settings-dir-rtl = من اليمين إلى اليسار
settings-dir-preview-title = معاينة
settings-dir-preview-body = ينعكس هذا النص عندما يكون الاتجاه من اليمين إلى اليسار.
settings-mail-features = ميزات البريد
settings-mail-feature-allowSend = السماح بإرسال الرسائل
settings-mail-feature-allowSend-help = عند إيقافه، يتعطل «إرسال» واختصاره. ما زال بالإمكان حفظ المسودات.
settings-mail-feature-allowReply = السماح بالرد والرد على الكل
settings-mail-feature-allowReply-help = عند إيقافه، لا تُتاح إجراءات «رد» و«رد على الكل» ولا اختصاراتها.
settings-mail-feature-allowForward = السماح بإعادة التوجيه
settings-mail-feature-allowForward-help = عند إيقافه، لا تُتاح إجراءات «إعادة توجيه» ولا اختصارها.
settings-mail-feature-allowDownload = السماح بتنزيل الرسائل والمرفقات
settings-mail-feature-allowDownload-help = عند إيقافه، لا يُتاح تنزيل الرسائل والمرفقات.
settings-mail-feature-allowPrint = السماح بالطباعة
settings-mail-feature-allowPrint-help = عند إيقافه، لا يُتاح إجراء «طباعة».
settings-mail-feature-showSpam = إظهار مجلد البريد العشوائي
settings-mail-feature-showSpam-help = عند إيقافه، يُخفى مجلد البريد العشوائي والرسائل المصنفة كبريد عشوائي من قائمة المجلدات ومن نتائج البحث. لا يُحذف أي شيء.
settings-mail-feature-showAuthStatus = إظهار نتائج مصادقة المرسل
settings-mail-feature-showAuthStatus-help = يعرض في القارئ نتائج SPF وDKIM وDMARC للرسالة.
settings-mail-feature-closeOnDelete = إغلاق الرسالة عند حذفها
settings-mail-feature-closeOnDelete-help = يغلق الرسالة المفتوحة أو علامة التبويب أو النافذة عند حذف تلك الرسالة أو نقلها إلى المهملات.

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = مسار الإعدادات
settings-crumb-loading = جارٍ تحميل الاسم
settings-crumb-new = جديد
settings-crumb-edit = تعديل
settings-badge-color = لون شارة غير المقروء
settings-badge-color-accent = مثل لون التمييز
settings-badge-color-accent-lighter = لون تمييز أفتح
settings-badge-color-accent-darker = لون تمييز أغمق
settings-badge-color-option = شارات { $color }
settings-appearance-theme-card = السمة والتبديل
settings-appearance-theme-card-help = اختر سمة وحدد هل تتبع النظام أم وقت اليوم.
settings-appearance-colours-card = الألوان
settings-appearance-colours-card-help = لون التمييز المستخدم للإجراءات والتحديد، ولون عدّادات غير المقروء.
settings-appearance-text-card = النص والتخطيط
settings-appearance-text-card-help = مدى كثافة الواجهة، والخط المستخدم، وكيفية عرض البريد.
settings-sig-create-variant = إنشاء صيغة
settings-sig-error-existing = يوجد توقيع بهذا الاسم بالفعل. اختر اسمًا مختلفًا.
settings-sig-save-failed = تعذّر حفظ التوقيع. { $reason }
settings-ident-signature-new = الرسائل الجديدة
settings-ident-signature-account = استخدام توقيع الحساب
settings-ident-signature-replies = الردود
settings-ident-signature-inherit = استخدام توقيع الرسائل الجديدة
settings-ident-signature-unavailable = توقيع غير متاح: { $name }
settings-ident-signature-help = أنشئ صيغ التوقيع في التواقيع، ثم عيّن إحداها هنا. يمكن استخدام الصيغة نفسها للاثنين.
settings-ident-save-failed = تعذّر حفظ الهوية. { $reason }
settings-search-save-failed = تعذّر حفظ البحث المحفوظ. { $reason }
settings-kbd-preset-title = إعداد الاختصارات المسبق
settings-editor-discard-title = تجاهل التغييرات غير المحفوظة؟
settings-editor-discard-detail = لم تُحفظ تغييراتك. تابع التحرير أو تجاهل هذه المسودة لمغادرة المحرر.
settings-editor-keep = متابعة التحرير
settings-editor-discard = تجاهل التغييرات
settings-editor-missing = لم يعد هذا العنصر موجودًا. ارجع إلى القائمة واختر عنصرًا آخر.
settings-editor-back = العودة إلى القائمة
settings-sig-edit-title = تعديل التوقيع
settings-ident-edit-title = تعديل الهوية
settings-sections = أقسام الإعدادات
settings-category-account = الحساب
settings-category-mail = البريد
settings-category-interface = الواجهة
settings-category-integrations = التكاملات
settings-category-about = حول
settings-nav-preferences = لوحة المفاتيح والعرض
settings-nav-rules = قواعد البريد
settings-nav-access = الوصول عبر API
settings-nav-metadata = معلومات الخادم
settings-folders-title = المجلدات والشريط الجانبي
settings-folders-help = ثبّت المجلدات فوق غيرها، واختر أيقونة، واسحب أو استخدم أزرار النقل لترتيب المجلدات داخل كل مجموعة. تُزامَن هذه الاختيارات مع هذا الحساب ولا تنقل الرسائل ولا تعيد تسمية المجلدات.
settings-folders-pin = تثبيت { $name }
settings-folders-icon = أيقونة { $name }
settings-folders-automatic = دور المجلد التلقائي
settings-folders-drag = اسحب { $name } لإعادة الترتيب
settings-folders-up = نقل { $name } لأعلى
settings-folders-down = نقل { $name } لأسفل
settings-folders-sync-idle = سجّل الدخول لمزامنة تفضيلات المجلدات.
settings-folders-sync-loading = جارٍ تحميل تفضيلات الحساب…
settings-folders-sync-saving = جارٍ حفظ تفضيلات الحساب…
settings-folders-sync-synced = زُوملت التفضيلات مع هذا الحساب.
settings-folders-sync-error = تعذّرت مزامنة التفضيلات. تبقى تغييراتك في هذه الجلسة وستُعاد المحاولة عند عودة الاتصال.
settings-folder-icon-folder = مجلد
settings-folder-icon-star = نجمة
settings-folder-icon-flag = علم
settings-folder-icon-tag = وسم
settings-folder-icon-clock = ساعة
settings-folder-icon-calendar = تقويم
settings-folder-icon-file = مستند
settings-folder-icon-image = صورة
settings-folder-icon-lock = قفل
settings-folder-icon-users = أشخاص
settings-folder-icon-archive = أرشيف
settings-folder-icon-mail = مظروف
settings-folder-icon-attachment = مرفق
settings-folder-icon-check = علامة صح
settings-folder-icon-alert = تنبيه
settings-search-new = مجلد بحث جديد
settings-search-editor = مجلد بحث
settings-search-editor-help = يجب أن تطابق الرسائل جميع الشروط. اترك الشرط فارغًا لتجاهله. تعرض مجلدات البحث الرسائل المطابقة دون نقلها أو نسخها.
settings-search-name = اسم المجلد
settings-search-name-required = أدخل اسمًا لمجلد البحث هذا.
settings-search-invalid = يحتوي هذا البحث المحفوظ على عامل تصفية غير صالح. لا يمكن تعديله بأمان.
settings-search-saved = تم حفظ مجلد البحث "{ $name }".
settings-search-field-text = البحث في نص كل الرسائل
settings-search-field-from = المرسل يحتوي على
settings-search-field-to = المستلم يحتوي على
settings-search-field-subject = الموضوع يحتوي على
settings-search-field-body = نص الرسالة يحتوي على
settings-search-field-filename = اسم ملف المرفق يحتوي على
settings-search-field-inMailbox = التقييد بمعرّف صندوق بريد (اختياري)
settings-search-field-after = استُلمت بعد (تاريخ ووقت UTC)
settings-search-field-before = استُلمت قبل (تاريخ ووقت UTC)
settings-search-attachment = المرفقات
settings-search-any = أي رسالة
settings-search-with-attachments = مع مرفقات
settings-search-without-attachments = بدون مرفقات
settings-search-unread = الرسائل غير المقروءة فقط
settings-search-flagged = الرسائل ذات العلامة فقط
settings-sessions-search = البحث عن حساب أو جهاز أو عنوان IP
settings-sessions-account = الحساب
settings-sessions-device = الجهاز والعميل
settings-sessions-location = عنوان IP وآخر بلد معروف
settings-sessions-location-unavailable = البلد غير متاح
settings-sessions-activity = النشاط
settings-sessions-actions = الإجراءات
settings-sessions-revoke-device = تسجيل الخروج من { $device }
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = الكتابة والردود
settings-quote-style = نمط ترويسة الرد وإعادة التوجيه
settings-quote-classic = رد كلاسيكي مع اقتباس
settings-quote-outlook = ترويسة بنمط Outlook
settings-quote-help = يضيف نمط Outlook كلًّا من المرسل وتاريخ الإرسال والمستلم والنسخة والموضوع فوق الرسالة الأصلية في النص المنسّق والنص العادي. لا تتغير المسودات الحالية.
settings-external-warning = تحذيرات المستلمين الخارجيين
settings-external-off = متوقف
settings-external-warn = عرض تحذير
settings-external-confirm = طلب التأكيد قبل الإرسال
settings-external-help = تُقارَن عناوين إلى ونسخة ونسخة مخفية بنطاق الهوية المحددة وبقائمة النطاقات الموثوقة. يحمي هذا الإرسال من هذه النافذة؛ وهو ليس قاعدة تسليم على الخادم.
settings-trusted-domains = نطاقات المستلمين الموثوقة
settings-trusted-domains-help = أدخل حتى 100 نطاق مطابق تمامًا، واحدًا في كل سطر. يجب إدراج النطاقات الفرعية على حدة. نطاق هوية الإرسال نفسها موثوق دائمًا. لا تُقبل عناوين URL ولا عناوين البريد الإلكتروني ولا أحرف البدل.
settings-trusted-domains-invalid = أدخل حتى 100 اسم نطاق صالح، دون عناوين URL أو عناوين بريد إلكتروني أو أحرف بدل.
settings-trusted-domains-save = حفظ النطاقات الموثوقة
settings-shortcuts-enabled = تفعيل اختصارات التطبيق
settings-shortcuts-action = الإجراء
settings-shortcuts-binding = الاختصار
settings-shortcuts-edit = تعديل الاختصار
settings-shortcuts-edit-action = تعديل اختصار { $action }
settings-shortcuts-help = استخدم inherit لاتباع الإعداد المسبق المحدد، أو disabled لعدم وجود اختصار، أو توليفة مثل Ctrl+Shift+K. يعني Ctrl مفتاح Cmd على macOS. الاختصارات المتعارضة غير نشطة. تحتفظ حقول الكتابة والنوافذ بمفاتيحها المعتادة؛ وقد لا تتوفر التوليفات المحجوزة للمتصفح.
settings-shortcuts-invalid = أدخل inherit أو disabled أو مفتاحًا واحدًا صالحًا مع مفاتيح تعديل اختيارية Ctrl وAlt وShift.
settings-shortcuts-conflict = يستخدم إجراءان الاختصار نفسه. غيّر أحد الاختصارين؛ ولن تعمل الاختصارات المتعارضة.
settings-shortcuts-discard = تجاهل تغيير الاختصار غير المحفوظ؟
settings-shortcuts-save = حفظ الاختصار
settings-shortcuts-back = العودة إلى الاختصارات
settings-shortcuts-off = يعطّل إعدادك المسبق "متوقف" كل الاختصارات، بما فيها المخصصة. اختر إعدادًا مسبقًا آخر في التفضيلات لاستخدام هذه الاختصارات.
settings-kbd-action-replyAll = رد على الجميع
settings-kbd-action-forward = إعادة توجيه
settings-kbd-action-refresh = تحديث الرسائل
settings-kbd-action-settings = فتح الإعدادات
settings-kbd-action-trash = نقل إلى المهملات
settings-kbd-action-markRead = وضع علامة مقروء
settings-kbd-action-markUnread = وضع علامة غير مقروء
settings-kbd-action-spam = وضع علامة كرسالة غير مرغوب فيها
settings-kbd-action-zoomIn = تكبير الرسالة
settings-kbd-action-zoomOut = تصغير الرسالة
settings-kbd-action-zoomReset = إعادة ضبط تكبير الرسالة
settings-kbd-action-renameFolder = إعادة تسمية المجلد
settings-kbd-action-openInTab = فتح الرسالة في علامة تبويب مستقلة
settings-notif-device-title = الإذن على هذا الجهاز
settings-notif-device-help = تتبع تفضيلات الإشعارات حسابك. أما إذن المتصفح فيخص هذا الجهاز؛ ولا يسأل Mailwoman إلا عندما تختار السماح.
settings-notif-ask-permission = عرض طلب إذن الإشعارات على الأجهزة الجديدة
settings-notif-ask-permission-help = عرض طلب الإذن عندما لا يكون هذا الجهاز قد سمح بالإشعارات أو حظرها. مفعّل افتراضيًا؛ ويُحفظ هذا التفضيل فورًا.
settings-notif-permission-default = لم يُطلب إذن المتصفح بعد.
settings-notif-permission-granted = إشعارات المتصفح مسموح بها على هذا الجهاز.
settings-notif-permission-denied = الإشعارات محظورة. غيّر إذن هذا الموقع في إعدادات المتصفح للسماح بها.
settings-notif-permission-unsupported = لا يدعم هذا المتصفح إشعارات النظام هنا.
settings-notif-permission-request = السماح بالإشعارات على هذا الجهاز
settings-notif-permission-error = تعذّر على المتصفح طلب إذن الإشعارات.
settings-notif-timezone = المنطقة الزمنية لساعات الهدوء (IANA)
settings-notif-timezone-device = استخدام المنطقة الزمنية لهذا الجهاز
settings-permissions-background-title = الإشعارات في الخلفية على هذا المتصفح
settings-permissions-background-help = احفظ تفضيلات الإشعارات واسمح بإشعارات المتصفح أولًا. سجّل هذا المتصفح لتلقي نشاط الحساب عندما يكون التطبيق مغلقًا.
settings-permissions-push-unsupported = إشعارات Web Push في الخلفية غير متاحة في هذا المتصفح.
settings-permissions-push-registered = لدى هذا المتصفح اشتراك دفع. سجّله لربطه بالحساب الحالي.
settings-permissions-push-unregistered = ليس لدى هذا المتصفح اشتراك دفع.
settings-permissions-push-register = تسجيل إشعارات الخلفية
settings-permissions-push-remove = إزالة اشتراك المتصفح
settings-permissions-push-policy = لا يسمح المسؤول بإشعارات الدفع في الخلفية.
settings-permissions-storage-title = تخزين المتصفح
settings-permissions-storage-help = يحمي التخزين الدائم بيانات هذا الموقع المحلية من الإزالة التلقائية عندما يمنحه المتصفح. وهو خاص بالجهاز ولا يغيّر مدة الاحتفاظ على الخادم. يمكنك مسح بيانات الموقع من إعدادات المتصفح؛ وقد تُفقد التغييرات غير المرسلة أثناء عدم الاتصال.
settings-permissions-storage-unsupported = طلبات التخزين الدائم غير متاحة في هذا المتصفح.
settings-permissions-storage-persistent = منح المتصفح هذا الموقع تخزينًا دائمًا.
settings-permissions-storage-evictable = قد يزيل المتصفح بيانات هذا الموقع المحلية تلقائيًا.
settings-permissions-storage-usage = التخزين الحالي للموقع: { $mib } ميبي بايت.
settings-permissions-storage-request = طلب تخزين دائم
settings-permissions-storage-policy = لا يسمح المسؤول بطلبات التخزين الدائم.
settings-permissions-error = تعذّر إكمال إجراء المتصفح. تحقق من إعدادات الحساب المحفوظة وأذونات المتصفح لهذا الموقع.
settings-notif-background-rules = تُقيَّم قواعد المحتوى أثناء فتح Mailwoman. تُكتم إشعارات النظام في الخلفية ما دامت أي قواعد محتوى مهيأة، لأن الإيقاظ المعتم في الخلفية لا يتضمن تفاصيل الرسالة.
settings-time-title = المناطق الزمنية
settings-time-help = اختر كيفية ظهور أوقات الرسائل ومنطقة مقارنة اختيارية للجدولة. لا تُستنتج المواقع أبدًا من عناوين البريد الإلكتروني.
settings-time-displayZone = المنطقة الزمنية للعرض
settings-time-displayZone-help = استخدم local لهذا الجهاز، أو اسم IANA مثل Europe/Lisbon.
settings-time-comparisonZone = منطقة المقارنة الزمنية
settings-time-comparisonZone-help = أدخل اسم IANA مثل America/New_York، أو اتركه فارغًا لإخفاء المقارنات.
settings-time-helpers = عرض مساعدات المنطقة الزمنية
settings-time-save = حفظ المنطقة الزمنية
settings-time-invalid = أدخل منطقة زمنية IANA صالحة. استخدم local لمنطقة العرض فقط، أو اترك منطقة المقارنة فارغة.
settings-time-discard = تجاهل تغييرات المنطقة الزمنية غير المحفوظة؟
settings-folders-name = المجلد
settings-folders-actions = الإجراءات
settings-folders-edit = تعديل مظهر { $name }
settings-folders-icon-help = اختر واحدة من خمسين أيقونة مواضيعية، أو أبقِ الاختيار التلقائي بحسب غرض المجلد. يتبع هذا حسابك ولا يعيد تسمية صندوق البريد.
settings-folders-save = حفظ مظهر المجلد
settings-folders-back = العودة إلى المجلدات
settings-folders-discard = تجاهل تغييرات مظهر المجلد غير المحفوظة؟
settings-folder-labels = تسميات المجلد
settings-folder-labels-help = نظّم هذا المجلد بما يصل إلى 32 تسمية حساب. لا يضع هذا تسميات على الرسائل داخله ولا ينقلها. أدِر أسماء التسميات وألوانها في التسميات.
settings-folders-leave-pending = ما زالت هذه التغييرات في قائمة انتظار المزامنة. مغادرة هذا المحرر؟
settings-folders-retry = إعادة محاولة الحفظ
settings-folder-role-title = المجلدات الخاصة
settings-folder-role-help = اختر المجلد الفعلي المستخدم لكل دور بريدي. يستخدم "تلقائي" اكتشاف الخادم. يزيل "استخدام الإعداد الافتراضي للمسؤول" تجاوزك. تؤدي الوجهات المتعارضة أو المفقودة إلى تعطيل ذلك الدور بدلًا من اختيار مجلد آخر.

# -- Machine translation (2026-11): added to match en ---------------------
cache-clear = مسح البريد المخزَّن مؤقتًا الآن
settings-compose-window = فتح الرسائل الجديدة في
settings-compose-window-tab = علامة تبويب في مساحة العمل
settings-compose-window-new = نافذة جديدة
settings-compose-window-help = إذا حظر متصفحك النافذة الجديدة، تُفتح نافذة الكتابة في علامة تبويب بمساحة العمل.
settings-compose-read-receipt = طلب إشعارات القراءة دائمًا
settings-compose-delivery-receipt = طلب إشعارات التسليم دائمًا
settings-compose-reply-copy = حفظ نسخة من الردود في المجلد الأصلي
settings-compose-receipts-help = الإشعارات مجرد طلبات. قد يرفضها المستلم أو خادم بريده أو يتجاهلها.
settings-compose-forward = صيغة إعادة التوجيه الافتراضية
settings-compose-forward-inline = نص الرسالة مضمَّنًا
settings-compose-forward-attachment = الرسالة الأصلية كمرفق بصيغة ‎.eml
settings-compose-saving = جارٍ حفظ تفضيل الكتابة…
settings-compose-saved = تم حفظ تفضيل الكتابة
settings-ident-organization = المؤسسة
settings-ident-auto-bcc = نسخة مخفية تلقائية
settings-ident-auto-bcc-help = أضف حتى 20 عنوانًا إلى النسخة المخفية عند الإرسال من هذه الهوية. افصل بين العناوين بفواصل.
settings-ident-pgp-key = مفتاح OpenPGP
settings-ident-pgp-none = استخدام المفتاح الافتراضي
settings-ident-pgp-unavailable = المفتاح المحفوظ غير متاح — اختر مفتاحًا آخر
settings-ident-pgp-help = اختر أحد مفاتيح التوقيع OpenPGP الموجودة لديك. تبقى مادة المفتاح الخاص في خزنة مفاتيحك.
settings-ident-error-optional = تحقق من عناوين الرد والنسخة المخفية التلقائية والمؤسسة والمفتاح المحدد.
settings-compose-open-tracking = تسجيل فتح الرسائل افتراضيًا
settings-compose-open-tracking-help = يضيف صورة تتبع إلى الرسائل المنسَّقة المؤهلة. يعتمد تسجيل الفتح على تحميل المستلم للصور ولا يثبت أن الرسالة قُرئت. يمكن للمسؤول تعطيل التتبع أو قفل هذا الإعداد الافتراضي.
settings-compose-open-tracking-inherit = استخدام إعداد التتبع الافتراضي الموروث
settings-reply-images = الصور في الردود وإعادة التوجيه
settings-reply-images-ask = السؤال في كل مرة
settings-reply-images-allow = التضمين دائمًا
settings-reply-images-trusted = التضمين لنطاقات المرسلين الموثوقة، والسؤال في غيرها
settings-reply-images-deny = عدم التضمين أبدًا
settings-reply-images-help = اختر هل يحمل الرد أو إعادة التوجيه صور الرسالة الأصلية. تُنسخ الصور المضمَّنة إلى المسودة كمرفقات.
settings-reply-image-domains = نطاقات مرسلي الصور الموثوقة
settings-reply-image-domains-help = أدخل حتى 200 نطاق مرسل مطابق تمامًا، واحدًا في كل سطر. يُستخدم عندما يكون الاختيار أعلاه هو النطاقات الموثوقة.
settings-reply-image-domains-save = حفظ نطاقات الصور الموثوقة
settings-ident-group-basics = الأساسيات
settings-ident-group-replies = معالجة الردود
settings-ident-group-signature = التوقيع
settings-ident-group-sending = خيارات الإرسال
settings-ident-name-required = أدخل اسم عرض.
settings-ident-email-invalid = أدخل عنوان بريد إلكتروني صالحًا، مثل name@example.org.
settings-ident-email-duplicate = لديك هوية بهذا العنوان بالفعل.
settings-ident-replyto-invalid = أدخل عنوان بريد إلكتروني صالحًا أو اترك الحقل فارغًا.
settings-ident-replyto-help = تذهب الردود على الرسائل المرسلة من هذه الهوية إلى هذا العنوان بدلًا من عنوان المرسل.
settings-ident-bcc-invalid = تحقق من قائمة النسخة المخفية: استخدم عناوين صالحة مفصولة بفواصل، بحد أقصى 20.
settings-ident-delete-title = حذف الهوية { $email }؟
settings-ident-delete-body = لن تتمكن بعد الآن من الإرسال من هذه الهوية. لا تتأثر المسودات والرسائل المرسلة.
settings-ident-empty = ليس لديك أي هويات بعد. أضف واحدة لاختيار اسمها وعنوانها وتوقيعها عند الكتابة.
settings-sig-delete-title = حذف التوقيع { $name }؟
settings-sig-delete-body = تعود الهويات التي تستخدمه إلى توقيع الحساب. لا تتأثر الرسائل المرسلة بالفعل.
settings-sig-deleted = تم حذف التوقيع.
settings-reply-from = الرد من
settings-reply-from-match = العنوان الذي أُرسلت إليه الرسالة
settings-reply-from-default = هويتي الافتراضية دائمًا
settings-reply-from-ask = اسألني عند تطابق عدة هويات
settings-reply-from-keep = إبقاء الهوية المحددة بالفعل
settings-reply-from-help = عند الرد أو إعادة التوجيه، اختر الهوية المستخدمة. إذا لم تطابق أي هوية العنوان الذي أُرسلت إليه الرسالة، تُستخدم هويتك الافتراضية. العناوين ذات علامة الزائد مثل name+tag@example.org تطابق name@example.org.
settings-ident-default-badge = افتراضية
settings-ident-make-default = جعلها افتراضية
settings-ident-default-set = أصبحت { $email } هويتك الافتراضية.
settings-ident-create-variant = إنشاء صيغة
settings-ident-variant-count = { $count ->
        [zero] لا صيغ
        [one] صيغة واحدة
        [two] صيغتان
        [few] { $count } صيغ
        [many] { $count } صيغة
       *[other] { $count } صيغة
    }
settings-ident-inherits = تستخدم { $base } في: { $fields }
settings-ident-field-replyTo = عنوان الرد
settings-ident-field-autoBcc = النسخة المخفية التلقائية
settings-ident-field-organization = المؤسسة
settings-ident-field-pgpKeyId = مفتاح OpenPGP
settings-ident-field-signatureName = التوقيع
settings-ident-delete-default-body = هذه هي هويتك الافتراضية. ستصبح أقدم هوية متبقية هي الافتراضية. لن تتمكن بعد الآن من الإرسال من هذه الهوية. لا تتأثر المسودات والرسائل المرسلة.
settings-ident-variants-choice-title = حذف { $email }؟
settings-ident-variants-choice-body = لهذه الهوية { $count ->
        [zero] لا صيغ
        [one] صيغة واحدة
        [two] صيغتان
        [few] { $count } صيغ
        [many] { $count } صيغة
       *[other] { $count } صيغة
    }. احذف { $count ->
        [zero] لا شيء
        [one] الصيغة
        [two] الصيغتين
        [few] الصيغ
        [many] الصيغ
       *[other] الصيغ
    } معها، أو أبقِ { $count ->
        [zero] لا شيء
        [one] الصيغة
        [two] الصيغتين
        [few] الصيغ
        [many] الصيغ
       *[other] الصيغ
    } كهويات منفصلة. تنسخ الصيغة المحفوظة الإعدادات التي ورثتها من هذه الهوية.
settings-ident-delete-with-variants = حذف الهوية والصيغ
settings-ident-delete-keep-variants = حذف الهوية وإبقاء الصيغ
settings-ident-new-variant = صيغة جديدة
settings-ident-edit-variant-title = تعديل الصيغة
settings-ident-variant-intro = صيغة من { $base }. اترك الحقل فارغًا لاستخدام القيمة من تلك الهوية.
settings-ident-label-label = تسمية الصيغة
settings-ident-label-required = أدخل تسمية لهذه الصيغة، مثل النشرات الإخبارية.
settings-ident-inherit-hint = فارغ: يستخدم { $value } من الهوية الأساسية.
settings-ident-inherit-none = لا شيء
settings-check-policy-allow = الإرسال دون سؤال
settings-check-policy-warn = السؤال في كل مرة
settings-check-policy-warn-remember = السؤال، مع إتاحة التوقف عن السؤال
settings-check-policy-deny = حظر الإرسال
settings-check-invalid = عناوين مستلمين غير صالحة
settings-check-attachment = ذُكر مرفق لكنه غير موجود
settings-check-subject = موضوع فارغ
settings-check-help = ما يُفعل عندما يجد الفحص مشكلة. يعرض السؤال دائمًا خيار الإرسال على أي حال؛ ولا يوقف الرسالة إلا حظر الإرسال.
settings-external-warn-remember = السؤال، مع إتاحة التوقف عن السؤال
settings-external-deny = حظر الإرسال
settings-auto-chip = إنهاء المستلم عند كتابة مسافة أو فاصلة أو فاصلة منقوطة
settings-auto-chip-help = يتحول العنوان الصالح إلى مستلم مكتمل ويُضاف الفاصل نيابةً عنك.
settings-recipient-separator = فاصل المستلمين
settings-recipient-separator-comma = فاصلة
settings-recipient-separator-semicolon = فاصلة منقوطة
settings-header-collapse = ترويسة الرسالة (إلى، نسخة، الموضوع)
settings-header-collapse-expanded = إظهار دائمًا
settings-header-collapse-collapsed = طيّ إلى سطر ملخص
settings-header-collapse-auto = الطيّ أثناء كتابة النص
settings-header-collapse-help = عند الطيّ، تعرض الترويسة سطرًا واحدًا مثل إلى: Ana +2 وتُفتح عند النقر عليها أو عندما يحتاج حقل إلى انتباه.
settings-retention-title = الاحتفاظ
settings-folder-icon-camera = كاميرا
settings-folder-icon-music = موسيقى
settings-folder-icon-film = فيلم
settings-folder-icon-lightbulb = فكرة
settings-folder-icon-rocket = صاروخ
settings-folder-icon-trophy = كأس
settings-folder-icon-leaf = ورقة شجر
settings-folder-icon-globe = كرة أرضية
settings-folder-icon-cloud = سحابة
settings-folder-icon-bookmark = إشارة مرجعية
settings-folder-icon-wallet = محفظة
settings-folder-icon-medical = طبي
settings-folder-icon-bell = جرس
settings-folder-icon-key = مفتاح
settings-folder-icon-receipt = إيصال
settings-folder-icon-cart = عربة تسوق
settings-folder-icon-package = طرد
settings-folder-icon-wrench = أدوات
settings-folder-icon-chart = مخطط
settings-folder-icon-team = فريق
settings-folder-icon-store = متجر
settings-folder-icon-office = مكتب
settings-folder-icon-laptop = حاسوب محمول
settings-folder-icon-crown = تاج
settings-folder-icon-family = عائلة
settings-folder-icon-group-general = عام
settings-folder-icon-group-mail = البريد
settings-folder-icon-group-work = العمل
settings-folder-icon-group-finance = المال والتسوق
settings-folder-icon-group-travel = السفر والمنزل
settings-folder-icon-group-personal = شخصي وترفيه
settings-folder-icon-group-tech = التقنية والأدوات
settings-folder-icon-search-label = البحث في الأيقونات
settings-folder-icon-no-match = لا أيقونات مطابقة
