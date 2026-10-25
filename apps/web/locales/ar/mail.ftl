# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = الإعدادات
mail-compose = إنشاء
mail-nav-mailboxes = صناديق البريد
mail-nav-attachments = المرفقات
mail-nav-outbox = صندوق الصادر
mail-nav-sharing = مشاركة المجلد
mail-nav-apps = التطبيقات
mail-logout = تسجيل الخروج
mail-offline = غير متصل
mail-module-loading = جارٍ تحميل { $module }…

# -- Search ------------------------------------------------------------------
mail-search = بحث
mail-search-label = البحث في البريد
mail-search-placeholder = البحث في البريد — from:alice subject:invoice larger:1mb
mail-search-clear = مسح

# -- Message list ------------------------------------------------------------
mail-list-label = الرسائل
mail-loading = جارٍ تحميل الرسائل…
mail-empty = لا توجد رسائل
mail-unknown-sender = (مرسل غير معروف)
mail-no-subject = (بلا موضوع)
mail-pinned = مثبّتة
mail-has-attachment = تحتوي على مرفق
mail-answered = تم الرد
mail-forwarded = تمت إعادة التوجيه
mail-unread = غير مقروءة
mail-flagged = موسومة بعلامة
mail-selected = محدّدة
mail-row-position = الرسالة { $pos } من { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [zero] لا توجد رسائل
    [one] رسالة واحدة
    [two] رسالتان
    [few] { $count } رسائل
    [many] { $count } رسالة
   *[other] { $count } رسالة
}
mail-thread-expand = توسيع المحادثة، { $count ->
    [zero] لا توجد رسائل
    [one] رسالة واحدة
    [two] رسالتان
    [few] { $count } رسائل
    [many] { $count } رسالة
   *[other] { $count } رسالة
}
mail-thread-collapse = طي المحادثة، { $count ->
    [zero] لا توجد رسائل
    [one] رسالة واحدة
    [two] رسالتان
    [few] { $count } رسائل
    [many] { $count } رسالة
   *[other] { $count } رسالة
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = خيارات العرض
mail-reading-pane = جزء القراءة
mail-reading-pane-right = يمين
mail-reading-pane-bottom = أسفل
mail-reading-pane-off = إيقاف

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = الرسائل المحدّدة
mail-bulk-count = محدّد: { $count }
mail-mark-read = تعليم كمقروءة
mail-mark-unread = تعليم كغير مقروءة
mail-flag-message = وضع علامة
mail-unflag-message = إزالة العلامة
mail-move-to = نقل إلى مجلد
mail-move-to-placeholder = نقل إلى…
mail-select-all = تحديد الكل
mail-clear-selection = إلغاء التحديد

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-focused-enable = البريد الوارد المركّز
mail-inbox-filter = عامل تصفية البريد الوارد
mail-inbox-focused = المركّز
mail-inbox-other = أخرى
mail-inbox-turn-off = إيقاف

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = علامات التبويب المفتوحة
mail-subtab-pin = تثبيت { $title }
mail-subtab-unpin = إلغاء تثبيت { $title }
mail-subtab-tearoff = فتح { $title } في نافذة جديدة
mail-subtab-close = إغلاق { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = تثبيت
mail-unpin = إلغاء التثبيت
mail-snooze = تأجيل
mail-snooze-menu = تأجيل حتى
mail-snooze-later = لاحقًا اليوم
mail-snooze-tomorrow = غدًا
mail-snooze-next-week = الأسبوع القادم
mail-unsnooze = إلغاء التأجيل
mail-label = تصنيف
mail-labels-menu = التصنيفات
mail-flag = وضع علامة للمتابعة
mail-clear-flag = مسح المتابعة
mail-archive = أرشفة
mail-delete = حذف
mail-more-actions = إجراءات أخرى

# -- Reader ------------------------------------------------------------------
mail-reader-label = الرسالة
mail-reader-empty = اختر رسالة لقراءتها
mail-back = رجوع
mail-reader-from = من: { $addr }
mail-reader-to = إلى: { $addr }
mail-reader-cc = نسخة: { $addr }
mail-reader-actions = إجراءات الرسالة
mail-reply = رد
mail-reply-all = رد على الكل
mail-forward = إعادة توجيه
mail-reply-failed = تعذّر تجهيز الرسالة. حاول مرة أخرى.
mail-reply-unthreaded = تعذّرت قراءة معرّف الرسالة الأصلية. لن يُضم هذا الرد إلى سلسلتها.
mail-spam = بريد عشوائي
mail-export = تصدير
mail-sweep-sender = تنظيف رسائل المرسل
mail-message-body = نص الرسالة
mail-no-content = لا يوجد محتوى
mail-sanitizing = جارٍ التعقيم…
mail-attachments = المرفقات
mail-attachment-unnamed = (بلا اسم)
mail-attachment-open = المرفق { $name }
mail-attachment-close = إغلاق المرفق
mail-attachment-loading = جارٍ تحميل المرفق…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = رسالة مشفّرة
mail-encrypted-note = 🔒 هذه الرسالة مشفّرة بين الطرفين. افتح قفلها على هذا الجهاز لقراءتها.
mail-key-passphrase = عبارة مرور المفتاح
mail-decrypt = فك التشفير
mail-decrypting = جارٍ فك التشفير…
mail-decrypt-no-key = لا يتوفر مفتاح خاص لفك تشفير هذه الرسالة.
mail-decrypt-failed = فشل فك التشفير

# -- Reply and forward: text written into the outgoing message --------------
mail-quote-attribution = في { $date }، كتب { $name }:
mail-quote-attribution-undated = كتب { $name }:
mail-forward-heading = ---------- رسالة مُعاد توجيهها ----------
mail-forward-from = من: { $addr }
mail-forward-date = التاريخ: { $date }
mail-forward-subject = الموضوع: { $subject }
mail-forward-to = إلى: { $addr }
mail-forward-cc = نسخة: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = إنشاء رسالة
mail-compose-title = رسالة جديدة
mail-compose-title-reply = رد
mail-compose-title-reply-all = رد على الكل
mail-compose-title-forward = إعادة توجيه
mail-compose-close = إغلاق
mail-compose-from = من
mail-compose-from-default = الافتراضي
mail-compose-to = إلى
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = اقتراحات جهات الاتصال
mail-compose-cc = نسخة
mail-compose-bcc = نسخة مخفية
mail-compose-show-cc-bcc = إضافة نسخة أو نسخة مخفية
mail-compose-bcc-encrypted = تسرد الرسالة المشفّرة مفتاح كل مستلم، لذا سيكون مستلم النسخة المخفية مرئيًا للآخرين. أزل مستلمي النسخة المخفية أو أوقف التشفير.
mail-compose-plaintext-confirm-title = إرسال نص مفكوك التشفير دون تشفير؟
mail-compose-plaintext-confirm-body = تقتبس هذه الرسالة نصًا من رسالة مشفّرة. وهي غير مُعدّة للتشفير، لذا سيُرسل النص المقتبس دون حماية.
mail-compose-plaintext-confirm-send = إرسال دون تشفير
mail-compose-plaintext-confirm-cancel = رجوع
mail-compose-subject = الموضوع
mail-compose-body = النص
mail-compose-attachments = المرفقات
mail-compose-remove-attachment = إزالة { $name }
mail-compose-attach-nextcloud = إرفاق من Nextcloud
mail-compose-close-nextcloud = إغلاق Nextcloud
mail-compose-attach-file = إرفاق ملف
mail-compose-uploading = جارٍ الرفع…
mail-compose-upload-unavailable = رفع الملفات غير متاح حاليًا.
mail-compose-upload-failed = تعذّر رفع { $name }.
mail-compose-upload-too-large = حجم { $name } هو { $size } ميغابايت. الحد الأقصى لحجم الرفع هو { $max } ميغابايت.
mail-compose-send-later = إرسال لاحقًا
mail-compose-cancel = إلغاء
mail-compose-send = إرسال
mail-compose-schedule = جدولة
mail-compose-sending = جارٍ الإرسال…
mail-compose-encrypted-subject = رسالة مشفّرة
mail-compose-dlp-blocked = الإرسال محظور بموجب قاعدة لمنع فقدان البيانات (انظر التحذير أعلاه).
mail-compose-send-failed = فشل الإرسال
mail-compose-send-later-past = اختر وقت إرسال في المستقبل، أو امسح الحقل للإرسال الآن.
mail-send-not-signed-in = لم تسجّل الدخول.
mail-send-no-draft-folder = لا يوجد في هذا الحساب مجلد لحفظ المسودة.
mail-send-no-recipient = أضف مستلمًا واحدًا على الأقل.
mail-send-bad-recipient = { $count ->
    [zero] هذه ليست عناوين بريد إلكتروني: { $addresses }
    [one] هذا ليس عنوان بريد إلكتروني: { $addresses }
    [two] هذان ليسا عنواني بريد إلكتروني: { $addresses }
    [few] هذه ليست عناوين بريد إلكتروني: { $addresses }
    [many] هذه ليست عناوين بريد إلكتروني: { $addresses }
   *[other] هذه ليست عناوين بريد إلكتروني: { $addresses }
}
mail-send-later-past = وقت الإرسال ليس في المستقبل. لم يُرسل شيء.
mail-send-draft-refused = رفض الخادم هذه الرسالة: { $reason }. لم يُرسل شيء.
mail-send-submission-refused = رفض الخادم إرسال هذه الرسالة: { $reason }. لم يُرسل شيء.
mail-compose-sign-unlock-title = فتح قفل مفتاح التوقيع
mail-compose-sign-unlock-note = أدخل عبارة مرور مفتاحك لتوقيع هذه الرسالة. يبقى مفتوحًا لنافذة الإنشاء هذه فقط.
mail-compose-sign-unlock = فتح القفل
mail-compose-sign-unlocking = جارٍ فتح القفل…
mail-compose-sign-no-key = لا يتوفر مفتاح توقيع على هذا الجهاز.
mail-compose-sign-unlock-failed = تعذّر فتح قفل مفتاح التوقيع.
mail-compose-sign-unlock-required = افتح قفل مفتاح التوقيع لإرسال هذه الرسالة الموقّعة.

mail-compose-format-plain = نص عادي
mail-compose-format-rich = نص منسّق
mail-compose-rt-toolbar = التنسيق
mail-compose-rt-bold = غامق
mail-compose-rt-italic = مائل
mail-compose-rt-underline = تسطير
mail-compose-rt-strike = يتوسطه خط
mail-compose-rt-bullet = قائمة نقطية
mail-compose-rt-ordered = قائمة مرقّمة
mail-compose-rt-quote = اقتباس
mail-compose-rt-link = رابط
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = تطبيق

mail-compose-signature = التوقيع
mail-compose-signature-none = إضافة توقيع…

mail-compose-recall = استرجاع
mail-compose-recall-action = استرجاع
mail-compose-recall-holding = محتجزة، بانتظار الإرسال
mail-compose-recall-scheduled = مجدولة في { $when }

mail-compose-drafts = المسودات
mail-compose-drafts-empty = لا توجد مسودات محفوظة
mail-compose-drafts-delete = حذف المسودة
mail-compose-drafts-no-recipient = (بلا مستلم)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = صندوق الصادر
mail-outbox-refresh = تحديث
mail-outbox-empty = لا شيء بانتظار الإرسال
mail-outbox-send-now = إرسال الآن
mail-outbox-cancel = إلغاء
mail-outbox-scheduled = مجدولة
mail-outbox-holding = سيُرسل قريبًا
mail-outbox-sent = مُرسلة
mail-outbox-canceled = ملغاة
mail-outbox-held = محتجزة
mail-outbox-failed = لم تُرسل
mail-outbox-release = تحرير
mail-outbox-discard = تجاهل
mail-outbox-held-note = محتجزة. لن تُرسل حتى تُحرَّر.
mail-outbox-held-note-api-key = محتجزة — أنشأها مفتاح API { $name }. لن تُرسل حتى تُحرَّر.
mail-outbox-held-note-oauth = محتجزة — أنشأها التطبيق المتصل { $name }. لن تُرسل حتى تُحرَّر.
mail-outbox-origin-api-key = أنشأها مفتاح API { $name }
mail-outbox-origin-oauth = أنشأها التطبيق المتصل { $name }
mail-outbox-to = إلى { $recipients }
mail-outbox-no-subject = (بلا موضوع)
mail-outbox-attempts = { $count ->
    [zero] لم تُجرَ أي محاولة. آخر خطأ: { $error }
    [one] جرت محاولة واحدة. آخر خطأ: { $error }
    [two] جرت محاولتان. آخر خطأ: { $error }
    [few] جرت { $count } محاولات. آخر خطأ: { $error }
    [many] جرت { $count } محاولة. آخر خطأ: { $error }
   *[other] جرت { $count } محاولة. آخر خطأ: { $error }
}
mail-outbox-error = خطأ: { $error }
mail-outbox-toast-canceled = أُلغي الإرسال
mail-outbox-toast-discarded = تم التجاهل. لم تُرسل الرسالة.
mail-outbox-toast-sent = أُرسلت
mail-outbox-toast-retrying = لم تُرسل بعد: { $error }. ستُعاد المحاولة.
mail-outbox-toast-failed = لم تُرسل: { $error }
mail-outbox-toast-not-released = تعذّر الإرسال: { $error }
mail-outbox-toast-not-canceled = تعذّر الإلغاء: { $error }

# -- Network status and the offline queue ------------------------------------
mail-net-lost = انقطع الاتصال — جارٍ إعادة المحاولة…
mail-net-back = عاد الاتصال
mail-queue-sent = { $count ->
    [zero] لم يُرسل أي تغيير من قائمة الانتظار
    [one] أُرسل تغيير واحد من قائمة الانتظار
    [two] أُرسل تغييران من قائمة الانتظار
    [few] أُرسلت { $count } تغييرات من قائمة الانتظار
    [many] أُرسل { $count } تغييرًا من قائمة الانتظار
   *[other] أُرسل { $count } تغيير من قائمة الانتظار
}
mail-queue-failed = { $count ->
    [zero] لم يفشل أي تغيير في قائمة الانتظار
    [one] فشل تغيير واحد في قائمة الانتظار
    [two] فشل تغييران في قائمة الانتظار
    [few] فشلت { $count } تغييرات في قائمة الانتظار
    [many] فشل { $count } تغييرًا في قائمة الانتظار
   *[other] فشل { $count } تغيير في قائمة الانتظار
}
mail-queue-notice-label = تغييرات تعذّر تطبيقها
mail-queue-notice-title = { $count ->
    [zero] لا توجد تغييرات دون اتصال تعذّر تطبيقها
    [one] تعذّر تطبيق تغيير واحد أُجري دون اتصال
    [two] تعذّر تطبيق تغييرين أُجريا دون اتصال
    [few] تعذّر تطبيق { $count } تغييرات أُجريت دون اتصال
    [many] تعذّر تطبيق { $count } تغييرًا أُجري دون اتصال
   *[other] تعذّر تطبيق { $count } تغيير أُجري دون اتصال
}
mail-queue-kind-send = إرسال “{ $subject }”
mail-queue-kind-send-untitled = إرسال رسالة بلا موضوع
mail-queue-kind-move = نقل رسالة
mail-queue-kind-flag = تغيير تصنيف أو علامة
mail-queue-kind-draft = حفظ مسودة
mail-queue-kind-pim = تغيير في التقويم أو المهام أو الملاحظات أو جهات الاتصال
mail-queue-reason = السبب: { $reason }
mail-queue-reason-unknown = لم يذكر الخادم السبب.
mail-queue-retry = إعادة المحاولة
mail-queue-discard = تجاهل
mail-queue-retry-item = إعادة المحاولة: { $what }
mail-queue-discard-item = تجاهل: { $what }
mail-queue-discarded = تم التجاهل. لم يُطبَّق التغيير.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = تنظيف الرسائل
mail-sweep-title = تنظيف رسائل { $sender }
mail-sweep-all = حذف كل الرسائل من هذا المرسل
mail-sweep-keep-latest = الاحتفاظ بالأحدث وحذف الباقي
mail-sweep-older-than = حذف الأقدم من N يومًا
mail-sweep-block = حذف الكل وحظر هذا المرسل
mail-sweep-days = الأيام
mail-sweep-count = { $count ->
    [zero] لن تُنقل أي رسالة إلى المهملات
    [one] ستُنقل رسالة واحدة إلى المهملات
    [two] ستُنقل رسالتان إلى المهملات
    [few] ستُنقل { $count } رسائل إلى المهملات
    [many] ستُنقل { $count } رسالة إلى المهملات
   *[other] ستُنقل { $count } رسالة إلى المهملات
}
mail-sweep-cancel = إلغاء
mail-sweep-run = تنظيف ({ $count })
mail-sweeping = جارٍ التنظيف…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = إغلاق
mail-undo-action = تراجع
mail-undo-send-cancel = إلغاء
mail-undo-failed = تعذّر التراجع

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = أُضيف التصنيف
mail-toast-label-removed = أُزيل التصنيف
mail-toast-pinned = تم التثبيت
mail-toast-unpinned = أُلغي التثبيت
mail-toast-snoozed = تم التأجيل
mail-toast-follow-up-set = تم تعيين المتابعة
mail-toast-follow-up-cleared = مُسحت المتابعة
mail-toast-archived = تمت الأرشفة
mail-toast-trashed = نُقلت إلى المهملات
mail-toast-marked-spam = عُلّمت كبريد عشوائي
mail-toast-moved-to = نُقلت إلى { $folder }
mail-toast-moved = نُقلت إلى المجلد
mail-toast-archived-count = { $count ->
    [zero] لم تُؤرشف أي رسالة
    [one] أُرشفت رسالة واحدة
    [two] أُرشفت رسالتان
    [few] أُرشفت { $count } رسائل
    [many] أُرشفت { $count } رسالة
   *[other] أُرشفت { $count } رسالة
}
mail-toast-trashed-count = { $count ->
    [zero] لم تُنقل أي رسالة إلى المهملات
    [one] نُقلت رسالة واحدة إلى المهملات
    [two] نُقلت رسالتان إلى المهملات
    [few] نُقلت { $count } رسائل إلى المهملات
    [many] نُقلت { $count } رسالة إلى المهملات
   *[other] نُقلت { $count } رسالة إلى المهملات
}
mail-toast-marked-spam-count = { $count ->
    [zero] لم تُعلَّم أي رسالة كبريد عشوائي
    [one] عُلّمت رسالة واحدة كبريد عشوائي
    [two] عُلّمت رسالتان كبريد عشوائي
    [few] عُلّمت { $count } رسائل كبريد عشوائي
    [many] عُلّمت { $count } رسالة كبريد عشوائي
   *[other] عُلّمت { $count } رسالة كبريد عشوائي
}
mail-toast-moved-count = { $count ->
    [zero] لم تُنقل أي رسالة
    [one] نُقلت رسالة واحدة
    [two] نُقلت رسالتان
    [few] نُقلت { $count } رسائل
    [many] نُقلت { $count } رسالة
   *[other] نُقلت { $count } رسالة
}
mail-toast-moved-to-count = { $count ->
    [zero] لم تُنقل أي رسالة إلى { $folder }
    [one] نُقلت رسالة واحدة إلى { $folder }
    [two] نُقلت رسالتان إلى { $folder }
    [few] نُقلت { $count } رسائل إلى { $folder }
    [many] نُقلت { $count } رسالة إلى { $folder }
   *[other] نُقلت { $count } رسالة إلى { $folder }
}
mail-toast-marked-read = { $count ->
    [zero] لم تُعلَّم أي رسالة كمقروءة
    [one] عُلّمت كمقروءة
    [two] عُلّمت رسالتان كمقروءتين
    [few] عُلّمت { $count } رسائل كمقروءة
    [many] عُلّمت { $count } رسالة كمقروءة
   *[other] عُلّمت { $count } رسالة كمقروءة
}
mail-toast-marked-unread = { $count ->
    [zero] لم تُعلَّم أي رسالة كغير مقروءة
    [one] عُلّمت كغير مقروءة
    [two] عُلّمت رسالتان كغير مقروءتين
    [few] عُلّمت { $count } رسائل كغير مقروءة
    [many] عُلّمت { $count } رسالة كغير مقروءة
   *[other] عُلّمت { $count } رسالة كغير مقروءة
}
mail-toast-flagged = { $count ->
    [zero] لم توضع علامة على أي رسالة
    [one] وُضعت العلامة
    [two] وُضعت علامة على رسالتين
    [few] وُضعت علامة على { $count } رسائل
    [many] وُضعت علامة على { $count } رسالة
   *[other] وُضعت علامة على { $count } رسالة
}
mail-toast-unflagged = { $count ->
    [zero] لم تُزل العلامة عن أي رسالة
    [one] أُزيلت العلامة
    [two] أُزيلت العلامة عن رسالتين
    [few] أُزيلت العلامة عن { $count } رسائل
    [many] أُزيلت العلامة عن { $count } رسالة
   *[other] أُزيلت العلامة عن { $count } رسالة
}
mail-toast-move-failed = تعذّر نقل الرسائل. لم يُنقل شيء.
mail-toast-update-failed = تعذّر تغيير الرسائل. لم يتغير شيء.
mail-toast-update-refused = { $count ->
    [zero] لم يرفض الخادم التغيير لأي رسالة
    [one] رفض الخادم التغيير لرسالة واحدة
    [two] رفض الخادم التغيير لرسالتين
    [few] رفض الخادم التغيير لـ { $count } رسائل
    [many] رفض الخادم التغيير لـ { $count } رسالة
   *[other] رفض الخادم التغيير لـ { $count } رسالة
}
mail-toast-no-archive-folder = لا يوجد مجلد أرشيف
mail-toast-no-trash-folder = لا يوجد مجلد مهملات
mail-toast-no-spam-folder = لا يوجد مجلد بريد عشوائي
mail-toast-sweep-nothing = لا شيء للتنظيف
mail-toast-swept = { $count ->
    [zero] لم تُنظَّف أي رسالة
    [one] نُظّفت رسالة واحدة
    [two] نُظّفت رسالتان
    [few] نُظّفت { $count } رسائل
    [many] نُظّفت { $count } رسالة
   *[other] نُظّفت { $count } رسالة
}
mail-toast-load-more-failed = تعذّر تحميل المزيد من الرسائل
mail-toast-export-nothing = لا شيء للتصدير
mail-toast-exported = صُدّر ملف .eml
mail-toast-export-failed = فشل التصدير
mail-toast-send-queued = في قائمة الانتظار — ستُرسل عند عودة الاتصال
mail-toast-send-scheduled = جُدولت للإرسال
mail-toast-sent = أُرسلت الرسالة
mail-toast-send-canceled = أُلغي الإرسال

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = الشريط
mail-ribbon-tab-home = الرئيسية
mail-ribbon-tab-view = عرض
mail-ribbon-tab-folder = مجلد
mail-ribbon-expand = توسيع
mail-ribbon-collapse = طي
mail-ribbon-group-new = جديد
mail-ribbon-compose = إنشاء
mail-ribbon-group-session = الجلسة
mail-ribbon-logout = تسجيل الخروج
mail-ribbon-group-theme = السمة
mail-ribbon-group-density = الكثافة
mail-ribbon-group-settings = الإعدادات
mail-ribbon-more = المزيد…
mail-ribbon-group-folders = المجلدات
mail-density-compact = مضغوط
mail-density-cozy = مريح
mail-density-relaxed = فسيح

# -- Message security level: the three-position switch in the reader toolbar
mail-maxsec-label = مستوى أمان الرسالة
mail-maxsec-full = كامل
mail-maxsec-full-hint = HTML معقّم مع الصور والتنسيق.
mail-maxsec-no-media = بلا وسائط
mail-maxsec-no-media-hint = HTML معقّم مع إزالة جميع الصور والوسائط.
mail-maxsec-plain-text = نص عادي
mail-maxsec-plain-text-hint = لا يُعرض HTML — تُعرض الرسالة كنص عادي فقط.
