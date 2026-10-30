# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = التقويم
calendar-prev = السابق
calendar-next = التالي
calendar-today = اليوم
calendar-views = طرق عرض التقويم
calendar-new-event = حدث جديد
calendar-import = استيراد
calendar-export = تصدير
calendar-import-file = استيراد ملف تقويم

# -- View names --------------------------------------------------------------
calendar-view-day = يوم
calendar-view-3day = 3 أيام
calendar-view-work-week = أسبوع العمل
calendar-view-week = أسبوع
calendar-view-month = شهر
calendar-view-tri-month = ربع سنة
calendar-view-schedule = الجدول
calendar-view-agenda = الأجندة
calendar-view-year = سنة

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = التقاويم
calendar-toggle = تبديل { $name }
calendar-color-for = لون { $name }
calendar-add-calendar = إضافة تقويم
calendar-new-calendar-name = تقويم جديد
calendar-holidays-heading = العطلات
calendar-subscribe-holidays = الاشتراك في العطلات
calendar-add-region = إضافة منطقة…
calendar-synced = تقويم متزامن

# -- Grid / views ------------------------------------------------------------
calendar-all-day = طوال اليوم
calendar-all-day-full = طوال اليوم
calendar-more = { $count } أخرى
calendar-conflict = تعارض
calendar-no-events-30 = لا أحداث في الأيام الثلاثين القادمة.
calendar-events-count = { $count ->
    [zero] لا أحداث
    [one] حدث واحد
    [two] حدثان
    [few] { $count } أحداث
    [many] { $count } حدثًا
   *[other] { $count } حدث
}
calendar-month-grid = شهر { $month }
calendar-time-grid = الشبكة الزمنية
calendar-cell = { $date }، { $events }
calendar-announce = { $date }، { $events }
calendar-event-at = { $time } { $title }
calendar-event-allday = طوال اليوم: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = حدث جديد
calendar-editor-edit = تعديل الحدث
calendar-invitation = دعوة
calendar-invited = أنت مدعو ({ $status }).
calendar-accept = قبول
calendar-tentative = مبدئي
calendar-decline = رفض
calendar-counter = اقتراح بديل…
calendar-propose-start = اقتراح وقت بدء جديد
calendar-send-counter = إرسال الاقتراح البديل
calendar-overlaps = يتداخل هذا الحدث مع { $count ->
    [zero] لا أحداث أخرى
    [one] حدث آخر واحد
    [two] حدثين آخرين
    [few] { $count } أحداث أخرى
    [many] { $count } حدثًا آخر
   *[other] { $count } حدث آخر
}.
calendar-field-title = العنوان
calendar-untitled = (بلا عنوان)
calendar-field-calendar = التقويم
calendar-field-all-day = طوال اليوم
calendar-field-start = البدء
calendar-field-duration = المدة (دقيقة)
calendar-field-location = المكان
calendar-field-shows-as = يظهر كـ
calendar-shows-busy = مشغول
calendar-shows-free = متاح
calendar-field-status = الحالة
calendar-status-confirmed = مؤكد
calendar-status-tentative = مبدئي
calendar-status-cancelled = ملغى
calendar-repeats = التكرار
calendar-frequency = التواتر
calendar-freq-daily = يوميًا
calendar-freq-weekly = أسبوعيًا
calendar-freq-monthly = شهريًا
calendar-freq-yearly = سنويًا
calendar-every = كل
calendar-interval = الفاصل
calendar-ends = ينتهي
calendar-end-mode = طريقة الانتهاء
calendar-end-never = أبدًا
calendar-end-count = بعد N مرة
calendar-end-until = في تاريخ
calendar-occurrences = عدد المرات
calendar-until-date = حتى تاريخ
calendar-reminders = التذكيرات
calendar-reminder-before = قبل { $min } د
calendar-remove-reminder = إزالة التذكير { $min }
calendar-add-reminder = إضافة تذكير
calendar-reminder-none = + تذكير
calendar-reminder-5 = 5 دقائق
calendar-reminder-15 = 15 دقيقة
calendar-reminder-30 = 30 دقيقة
calendar-reminder-60 = ساعة واحدة
calendar-reminder-1440 = يوم واحد
calendar-attendees = الحاضرون
calendar-add-attendee = إضافة حاضر
calendar-attendee-invalid = هذا ليس عنوان بريد إلكتروني. أدخل عنوانًا واحدًا، مثل name@example.com.
calendar-save-failed = لم يُحفظ الحدث. رفضه الخادم.
calendar-remove-attendee = إزالة { $email }
calendar-notes = ملاحظات

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = دور { $email }
calendar-attendee-cutype-for = نوع { $email }
calendar-role-chair = رئيس الاجتماع
calendar-role-required = مطلوب
calendar-role-optional = اختياري
calendar-role-non-participant = غير مشارك
calendar-cutype-individual = شخص
calendar-cutype-group = مجموعة
calendar-cutype-resource = مورد
calendar-cutype-room = قاعة
calendar-partstat-needs-action = لا رد
calendar-partstat-accepted = مقبول
calendar-partstat-declined = مرفوض
calendar-partstat-tentative = مبدئي

# -- Schedule view (distinct from Agenda) ------------------------------------
calendar-schedule-row = من { $start } إلى { $end }: { $title }
calendar-schedule-free = { $gap } متاح
calendar-gap-hm = { $h } س { $m } د
calendar-gap-h = { $h } س
calendar-gap-m = { $m } د

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = حلّ { $count ->
    [zero] لا تعارضات
    [one] تعارض واحد
    [two] تعارضين
    [few] { $count } تعارضات
    [many] { $count } تعارضًا
   *[other] { $count } تعارض
}
calendar-resolver-title = حلّ التعارضات
calendar-resolver-none = لا توجد تعارضات لحلّها.
calendar-resolver-pick = التعارض
calendar-resolver-pair-n = { $n } من { $total }: { $a } مقابل { $b }
calendar-resolver-time = الوقت
calendar-resolver-overlap = التداخل { $start } – { $end }
calendar-resolver-reschedule = إعادة جدولة الحدث اللاحق
calendar-resolver-shorten = تقصير الحدث الأسبق
calendar-resolver-tentative = جعل الحدث اللاحق مبدئيًا
calendar-resolver-double-book = حجز مزدوج (وسم كمتاح)
calendar-resolver-keep = الإبقاء على الاثنين
calendar-events-count-people = { $count ->
    [zero] لا حاضرين
    [one] حاضر واحد
    [two] حاضران
    [few] { $count } حاضرين
    [many] { $count } حاضرًا
   *[other] { $count } حاضر
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-whose = التقاويم
calendar-fb-own = تقاويمك
calendar-fb-unavailable = تعذّر تحميل أوقات انشغالك.
calendar-fb-cell = { $principal } في { $hour }: { $status }
calendar-fb-busy = مشغول
calendar-fb-tentative = مبدئي
calendar-fb-free = متاح

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = إضافة حدث سريعة
calendar-quick-add-placeholder = إضافة سريعة، مثل: Lunch Friday 1pm
calendar-quick-add-btn = إضافة
calendar-quick-add-do = إضافة سريعة

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = الفئات
calendar-add-category = إضافة فئة
calendar-category-placeholder = أضف فئة
calendar-remove-category = إزالة الفئة { $name }
calendar-filter-heading = التصفية
calendar-filter-category = التصفية حسب الفئة
calendar-filter-category-placeholder = الفئة المراد عرضها

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = المرفقات
calendar-attachment-title = اسم المرفق
calendar-attachment-title-placeholder = الاسم (اختياري)
calendar-attachment-uri = رابط المرفق
calendar-attachment-add = إضافة مرفق
calendar-remove-attachment = إزالة المرفق { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = الاشتراك عبر URL
calendar-subscribe-url = URL التقويم
calendar-subscribe-add = اشتراك
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = تعذّر جلب التقويم من ذلك العنوان، لذا لم تتم إضافته.
calendar-subscribe-hol-many-locations = يسرد ملف العطلات هذا أكثر من موقع. نزّله، ثم استخدم «استيراد» واختر المواقع التي تريدها.
calendar-subscribe-refresh = تحديث { $name }
calendar-subscribe-refresh-btn = تحديث
calendar-subscribe-refresh-failed = تعذّر جلب موجز التقويم. تُركت أحداثه كما كانت.
calendar-import-failed = تعذّرت قراءة هذا الملف كتقويم. لم يُستورد أي شيء.
calendar-import-done = { $count ->
    [zero] لم يُستورد أي حدث.
    [one] استُورد حدث واحد.
    [two] استُورد حدثان.
    [few] استُوردت { $count } أحداث.
    [many] استُورد { $count } حدثًا.
   *[other] استُورد { $count } حدث.
}
calendar-import-locations-title = اختيار المواقع المراد استيرادها
calendar-import-locations-intro = يحتوي هذا الملف على عطلات لـ { $count } من المواقع. تُستورد المواقع التي تحددها فقط.
calendar-import-locations-filter = تصفية المواقع
calendar-import-locations-list = المواقع في الملف
calendar-import-locations-none = لا يوجد موقع يطابق عامل التصفية.
calendar-import-locations-picked = { $count ->
    [zero] لم يُحدَّد أي موقع.
    [one] حُدِّد موقع واحد.
    [two] حُدِّد موقعان.
    [few] حُدِّدت { $count } مواقع.
    [many] حُدِّد { $count } موقعًا.
   *[other] حُدِّد { $count } موقع.
}
calendar-import-locations-import = استيراد المحدد
calendar-export-failed = تعذّر تصدير التقويم.
calendar-quick-add-failed = تعذّرت إضافة هذا الحدث.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = مشاركة
calendar-share-for = مشاركة { $name }
calendar-share-title = مشاركة { $name }
calendar-share-intro = يمكن للأشخاص الذين تشارك معهم رؤية هذا التقويم. امنحهم صلاحية القراءة والكتابة ليتمكنوا من إضافة الأحداث وتعديلها.
calendar-share-people = تمت المشاركة مع
calendar-share-empty = هذا التقويم غير مُشارك مع أي شخص.
calendar-share-add = إضافة شخص
calendar-share-new-access = صلاحية الشخص الجديد
calendar-share-access-for = صلاحية { $principal }
calendar-share-remove = إزالة { $principal }
calendar-share-read = قراءة فقط
calendar-share-readwrite = قراءة وكتابة
