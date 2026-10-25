# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = لوحة المعلومات
admin-shell-group-overview = نظرة عامة
admin-shell-group-people = الأشخاص
admin-shell-group-security = الأمان
admin-shell-group-configuration = الإعدادات
admin-shell-group-operations = العمليات
admin-shell-menu = القائمة
admin-shell-close-menu = إغلاق القائمة
admin-shell-search = بحث
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = تم تسجيل الدخول باسم { $name }
admin-shell-language-label = اللغة
admin-shell-breadcrumb = مسار التنقل
admin-shell-unknown-section = هذا القسم غير موجود، أو أن دورك لا يشمله.
admin-shell-back-to-dashboard = العودة إلى لوحة المعلومات
admin-shell-cap-missing = دورك لا يشمل { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = جارٍ التحميل…
admin-shell-retry = إعادة المحاولة
admin-shell-cancel = إلغاء
admin-shell-save = حفظ
admin-shell-not-available = غير متاح

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = مغادرة هذه الصفحة؟
admin-shell-leave-body = لم تُحفظ التغييرات في هذه الصفحة. المغادرة تتجاهلها.
admin-shell-leave-confirm = المغادرة وتجاهل التغييرات

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = اكتب { $name } للتأكيد

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = البحث في وحدة تحكم المسؤول
admin-shell-palette-label = قسم أو مستخدم أو نطاق أو إعداد
admin-shell-palette-results = النتائج
admin-shell-palette-none = لا توجد نتائج مطابقة.
admin-shell-palette-searching = جارٍ البحث في المستخدمين والنطاقات والإعدادات…
admin-shell-palette-sections-only = لا يستطيع هذا الخادم البحث في المستخدمين والنطاقات والإعدادات بعد، لذا تُعرض الأقسام فقط.
admin-shell-palette-error = فشل البحث في المستخدمين والنطاقات والإعدادات؛ تُعرض الأقسام فقط.
admin-shell-palette-kind-section = قسم
admin-shell-palette-kind-user = مستخدم
admin-shell-palette-kind-domain = نطاق
admin-shell-palette-kind-setting = إعداد

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = عوامل التصفية
admin-shell-table-actions = الإجراءات
admin-shell-table-error = تعذّر تحميل القائمة.
admin-shell-table-paging = صفحات { $table }
admin-shell-table-prev = الصفحة السابقة
admin-shell-table-next = الصفحة التالية

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = الأحدث { $latest }، الأعلى { $max }
admin-shell-chart-col-time = الوقت
admin-shell-chart-col-item = العنصر
admin-shell-chart-col-value = القيمة

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = مُعيَّن بواسطة
admin-shell-pinned-sr = هذه القيمة مصدرها بيئة الخادم ولا يمكن تغييرها هنا.
admin-shell-lock-text = قفل للمستخدمين
admin-shell-lock-setting = { $setting }
admin-shell-restore = استعادة القيمة السابقة
admin-shell-restore-note = غيّرها { $actor } في { $time }. القيمة السابقة: { $value }
admin-shell-restore-done = تمت استعادة القيمة السابقة.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = الخطوات
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = الخطوة { $n } من { $total }: { $title }
admin-shell-wizard-review = المراجعة
admin-shell-wizard-back = رجوع
admin-shell-wizard-next = التالي
admin-shell-wizard-saved = تم الحفظ.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = تعذّر الوصول إلى الخادم. تحقّق من الاتصال وحاول مرة أخرى.
admin-shell-err-session = انتهت جلسة المسؤول. سجّل الدخول مرة أخرى.
admin-shell-err-forbidden = دورك لا يسمح بهذا الإجراء. اطلب ذلك من مسؤول لديه الدور المناسب.
admin-shell-err-out-of-scope = هذا العنصر ينتمي إلى نطاق لا يغطيه دورك.
admin-shell-err-not-found = لم يعد موجودًا. أعد تحميل القائمة.
admin-shell-err-conflict = غيّره شخص آخر في هذه الأثناء. أعد التحميل وحاول مرة أخرى.
admin-shell-err-validation = رفض الخادم إحدى القيم. تحقّق من الحقول وحاول مرة أخرى.
admin-shell-err-rate-limited = طلبات كثيرة جدًا. انتظر دقيقة وحاول مرة أخرى.
admin-shell-err-server = ردّ الخادم بخطأ ({ $status }). حاول مرة أخرى؛ وإذا تكرر ذلك، فتحقّق من سجل الخادم.
admin-shell-err-upstream-dns = تعذّر تحليل اسم المضيف. تحقّق من كتابة اسم المضيف.
admin-shell-err-upstream-connect = لم يستجب أي شيء على هذا المضيف والمنفذ. تحقّق من المنفذ ومن أي جدار حماية بينهما.
admin-shell-err-upstream-tls = استجاب الخادم لكن مصافحة TLS فشلت. تحقّق من المنفذ ومن إعداد أمان الاتصال.
admin-shell-err-upstream-auth = رفض الخادم بيانات الاعتماد. تحقّق من اسم المستخدم وكلمة المرور.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = محاولات تسجيل دخول كثيرة جدًا. انتظر بضع دقائق وحاول مرة أخرى.
admin-shell-totp-form = تسجيل دخول المسؤول، الخطوة الثانية
admin-shell-totp-note = أدخل الرمز المكوّن من 6 أرقام من تطبيق المصادقة المُعدّ لـ { $name }.
admin-shell-totp-code = رمز المصادقة
admin-shell-totp-verify = تحقّق
admin-shell-totp-start-over = البدء من جديد
admin-shell-totp-invalid = هذا الرمز غير صالح. أدخل الرمز الحالي من تطبيق المصادقة.
admin-shell-totp-expired = استغرق تسجيل الدخول وقتًا طويلًا. ابدأ من جديد وأدخل كلمة المرور.
