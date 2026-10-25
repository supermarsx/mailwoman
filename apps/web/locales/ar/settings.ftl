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
