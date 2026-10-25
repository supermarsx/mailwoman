# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = إغلاق

# -- Page states (UX11) ---------------------------------------------------------
admin-ux-state-unavailable = غير متاح — { $reason }
admin-ux-unavailable-default = لا يوفّر هذا الخادم ذلك بعد.
admin-ux-state-code = رمز الخطأ: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = فتح
admin-ux-list-back = العودة إلى القائمة

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = لا توجد تغييرات غير محفوظة
admin-ux-form-changes =
    { $count ->
        [zero] لا توجد تغييرات غير محفوظة
        [one] تغيير واحد غير محفوظ
        [two] تغييران غير محفوظين
        [few] { $count } تغييرات غير محفوظة
        [many] { $count } تغييرًا غير محفوظ
       *[other] { $count } تغيير غير محفوظ
    }
admin-ux-form-discard = تجاهل التغييرات
admin-ux-form-saving = جارٍ الحفظ…
admin-ux-form-saved =
    { $count ->
        [zero] لم يُحفظ أي تغيير.
        [one] تم حفظ تغيير واحد.
        [two] تم حفظ تغييرين.
        [few] تم حفظ { $count } تغييرات.
        [many] تم حفظ { $count } تغييرًا.
       *[other] تم حفظ { $count } تغيير.
    }
admin-ux-form-fix =
    { $count ->
        [zero] لم يُحفظ شيء. صحّح الحقول المحددة أدناه.
        [one] لم يُحفظ شيء. صحّح الحقل المحدد أدناه.
        [two] لم يُحفظ شيء. صحّح الحقلين المحددين أدناه.
        [few] لم يُحفظ شيء. صحّح { $count } حقول محددة أدناه.
        [many] لم يُحفظ شيء. صحّح { $count } حقلًا محددًا أدناه.
       *[other] لم يُحفظ شيء. صحّح { $count } حقل محدد أدناه.
    }
admin-ux-learn-more = معرفة المزيد
admin-ux-learn-more-field = حول { $label }
admin-ux-propagation-hot = يسري خلال 30 ثانية في كل عمليات الخادم.
admin-ux-propagation-restart = يسري بعد إعادة تشغيل الخادم.
admin-ux-secret-set = توجد قيمة محفوظة. اكتب قيمة جديدة لاستبدالها.
admin-ux-secret-unset = لا توجد قيمة محفوظة.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = أدخل قيمة.
admin-ux-err-number = أدخل رقمًا.
admin-ux-err-whole = أدخل عددًا صحيحًا.
admin-ux-err-min = أدخل { $min } أو أكثر.
admin-ux-err-max = أدخل { $max } أو أقل.
admin-ux-err-max-length = استخدم { $max } حرفًا على الأكثر.
admin-ux-err-url = أدخل عنوانًا كاملًا يبدأ بـ https:// أو http://.
admin-ux-err-email = أدخل عنوان بريد إلكتروني مثل name@example.org.
admin-ux-err-pattern = هذه القيمة ليست بالشكل المتوقع.
admin-ux-err-option = اختر أحد الخيارات المدرجة.
admin-ux-err-out-of-range = رفض الخادم هذه القيمة لأنها خارج النطاق المسموح به.
admin-ux-err-too-long = رفض الخادم هذه القيمة لأنها طويلة جدًا.
admin-ux-err-invalid = رفض الخادم هذه القيمة. تحقّق منها واحفظ مرة أخرى.
admin-ux-err-pinned = هذه القيمة مُعيَّنة في بيئة الخادم ولا يمكن تغييرها هنا.
admin-ux-err-locked = هذه القيمة مقفلة على مستوى أوسع ولا يمكن تغييرها هنا.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = افتراضي
admin-ux-source-stored = محفوظ
admin-ux-source-env-pin = مُعيَّن بواسطة { $name }
admin-ux-source-env-fallback = قيمة احتياطية من { $name }
admin-ux-source-build = مُعيَّن عند البناء
admin-ux-source-locked = مقفل بواسطة { $scope }
admin-ux-source-read-only = للقراءة فقط
admin-ux-source-restart = يلزم إعادة التشغيل
admin-ux-source-env-pin-reason = تعيّن بيئة الخادم هذه القيمة عبر { $name }. غيّرها هناك وأعد تشغيل الخادم.
admin-ux-source-build-reason = تُثبَّت هذه القيمة عند بناء الخادم.
admin-ux-source-locked-reason = قفل على مستوى { $scope } يثبّت هذه القيمة. أزل القفل هناك لتغييرها.
admin-ux-source-read-only-reason = تعرض وحدة التحكم هذه القيمة لكنها لا تستطيع تغييرها. توضّح الوثائق مكان تعيينها.

# -- History (UX8) ------------------------------------------------------------
admin-ux-history-changed-by-at = آخر تغيير بواسطة { $who } في { $time }
admin-ux-history-changed-by = آخر تغيير بواسطة { $who }
admin-ux-history-changed-at = آخر تغيير في { $time }
admin-ux-history-never = لم يُغيَّر من وحدة التحكم
admin-ux-history-link = السجل

# -- Revert to default (UX7) --------------------------------------------------
admin-ux-revert-default-is = الافتراضي: { $value }
admin-ux-revert = الرجوع إلى الافتراضي
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = تم الرجوع إلى القيمة الافتراضية ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = غيّر { $who } هذا في { $time } أثناء تحريرك.
admin-ux-stale-by = غيّر { $who } هذا أثناء تحريرك.
admin-ux-stale-unknown = غيّر شخص آخر هذا أثناء تحريرك.
admin-ux-stale-choice = أعد التحميل لرؤية نسختهم وفقدان تعديلاتك، أو احتفظ بنسختك واحفظها فوق نسختهم.
admin-ux-stale-reload = إعادة التحميل
admin-ux-stale-keep = الاحتفاظ بنسختي

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = جارٍ التحقق مما يؤثر فيه هذا التغيير…
admin-ux-impact-heading = يؤثر هذا التغيير في
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [zero] ولا شيء آخر
        [one] وعنصر آخر
        [two] وعنصران آخران
        [few] و{ $count } عناصر أخرى
        [many] و{ $count } عنصرًا آخر
       *[other] و{ $count } عنصر آخر
    }
admin-ux-impact-none = لا شيء آخر يتغير.
admin-ux-impact-unavailable = معاينة التأثير غير متاحة — لم يُبلغ الخادم بما يؤثر فيه هذا التغيير.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = تصفية
admin-ux-stream-pause = إيقاف مؤقت
admin-ux-stream-resume = استئناف
admin-ux-stream-clear = مسح
admin-ux-stream-announce = قراءة الإدخالات الجديدة بصوت عالٍ
admin-ux-stream-connecting = جارٍ الاتصال…
admin-ux-stream-live = مباشر
admin-ux-stream-ended = انتهى
admin-ux-stream-lost = انقطع الاتصال
admin-ux-stream-reconnect = إعادة الاتصال
admin-ux-stream-paused =
    { $count ->
        [0] متوقف مؤقتًا
        [zero] متوقف مؤقتًا
        [one] متوقف مؤقتًا — إدخال جديد واحد محتجز
        [two] متوقف مؤقتًا — إدخالان جديدان محتجزان
        [few] متوقف مؤقتًا — { $count } إدخالات جديدة محتجزة
        [many] متوقف مؤقتًا — { $count } إدخالًا جديدًا محتجزًا
       *[other] متوقف مؤقتًا — { $count } إدخال جديد محتجز
    }
admin-ux-stream-no-match = لا يوجد إدخال يطابق عامل التصفية.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = حقل

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = المظهر
admin-ux-appearance-title = مظهر وحدة التحكم
admin-ux-appearance-note = تغيّر هذه الخيارات وحدة تحكم المسؤول فقط، في هذا المتصفح.
admin-ux-appearance-following = تتبع وحدة التحكم سمة البريد الإلكتروني على الويب في هذا المتصفح حتى تختار هنا. تغيّر هذه الخيارات وحدة تحكم المسؤول فقط، في هذا المتصفح.
admin-ux-appearance-mode = الوضع
admin-ux-appearance-mode-light = فاتح
admin-ux-appearance-mode-dark = داكن
admin-ux-appearance-mode-system = اتباع النظام
admin-ux-appearance-theme = السمة
admin-ux-appearance-accent = لون التمييز
admin-ux-appearance-light = فاتح
admin-ux-appearance-dark = داكن
admin-ux-appearance-reset = استخدام سمة البريد على الويب
