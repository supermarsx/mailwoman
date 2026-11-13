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
mail-inbox-filter = عامل تصفية البريد الوارد
mail-inbox-focused = المركّز
mail-inbox-other = أخرى

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
mail-encrypted-note = هذه الرسالة مشفّرة بين الطرفين. افتح قفلها على هذا الجهاز لقراءتها.
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
mail-compose-options = خيارات الإرسال
mail-compose-request-receipt = طلب إشعار بالقراءة
mail-compose-request-receipt-hint = قد يسأل برنامج البريد لدى المستلم صاحبه، أو يرسل إشعارًا دون سؤال، أو يتجاهل الطلب.
mail-compose-track-opens = تسجيل وقت فتح هذه الرسالة
mail-compose-track-opens-hint = تُضاف إلى الرسالة صورة بحجم 1×1 تُحمَّل من هذا الخادم. كثير من برامج البريد تحظر الصور أو تحمّلها مسبقًا، لذا فإن تسجيل الفتح لا يثبت أن الرسالة قُرئت، وغياب التسجيل لا يثبت أنها لم تُقرأ. لا يُبلَّغ المستلم بذلك.
mail-compose-image-type = لا يمكن وضع { $name } في النص. استخدم صورة PNG أو JPEG أو GIF أو WebP.
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
mail-send-tracking-failed = لا يمكن تسجيل مرات الفتح الآن، لذا لم يُرسل شيء. ألغِ تحديد «تسجيل وقت فتح هذه الرسالة» للإرسال بدونه.
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
mail-compose-rt-image = إدراج صورة

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
mail-toast-pinned = تم تثبيت الرسالة «{ $target }»
mail-toast-unpinned = أُلغي تثبيت الرسالة «{ $target }»
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
mail-toast-tracking-not-bound = أُرسلت الرسالة، لكن لن تُسجَّل مرات فتحها.
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
mail-feature-off = تم إيقافه من إعداداتك أو من المسؤول
mail-send-disabled = تم إيقاف الإرسال من إعداداتك أو من المسؤول. ما زال بإمكانك حفظ مسودة.

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = الفعاليات
mail-events-upcoming = القادمة فقط
mail-events-filter = تصفية الفعاليات
mail-loading-more = جارٍ تحميل الرسائل الأقدم…
mail-event-state-invitation = دعوة
mail-event-state-updated = محدَّثة
mail-event-state-cancelled = ملغاة
mail-event-state-reply = رد
mail-event-state-counter = اقتُرح وقت جديد
mail-event-state-event = فعالية
mail-event-hint = { $state }، { $when }
mail-row-actions-expand = عرض المزيد من الإجراءات
mail-remote-moved = نُقلت هذه الرسالة إلى { $folder } من جهاز أو تطبيق آخر. ما زلت تقرأ النسخة التي فتحتها.
mail-remote-moved-unknown = نُقلت هذه الرسالة من جهاز أو تطبيق آخر. ما زلت تقرأ النسخة التي فتحتها.
mail-remote-deleted = حُذفت هذه الرسالة من جهاز أو تطبيق آخر. يمكنك متابعة قراءة هذه النسخة إلى أن تغلقها.
mail-remote-dismiss = إغلاق
mail-reply-draft = ردّ قيد الكتابة
mail-reply-draft-open = فتح المسودة
mail-draft-load-failed = تعذّر فتح المسودة. حاول مرة أخرى.
mail-reply-images-omitted =
    { $count ->
        [zero] تعذّر تضمين أي صورة في الرد.
        [one] تعذّر تضمين صورة واحدة في الرد.
        [two] تعذّر تضمين صورتين في الرد.
        [few] تعذّر تضمين { $count } صور في الرد.
        [many] تعذّر تضمين { $count } صورة في الرد.
       *[other] تعذّر تضمين { $count } صورة في الرد.
    }
mail-export-forbidden = عطّل المسؤول تصدير الرسائل لهذا الحساب.
mail-export-open-failed = تعذّر فتح خيارات التصدير. أغلق علامة تبويب في مساحة العمل وحاول مرة أخرى.
mail-headers-title = ترويسات الرسالة الأصلية
mail-headers-action-help = اعرض ترويسات الرسالة الأصلية، بما فيها تفاصيل التسليم والمصادقة.
mail-headers-help = الترويسات الأصلية للرسالة. تُحفظ الأسطر المطوية، وتُعرض الترويسات فقط هنا.
mail-headers-loading = جارٍ تحميل ترويسات الرسالة…
mail-headers-failed = تعذّر تحميل ترويسات الرسالة. تحقق من اتصالك وحاول مرة أخرى.
mail-headers-copy = نسخ الترويسات
mail-headers-copied = تم نسخ الترويسات.
mail-headers-copy-failed = تعذّر نسخ الترويسات. حدّد النص لنسخه يدويًا.
mail-attachment-failed = تعذّر تنزيل هذا المرفق. حاول مرة أخرى عند توفر الاتصال.
mail-compose-rt-rule = خط أفقي
mail-compose-signature-new = الرسائل الجديدة
mail-compose-signature-replies = الردود
mail-editor-body-size = { $size } كيبيبايت
mail-editor-body-size-label = حجم النص
mail-editor-body-size-help = حجم النص الحالي بترميز UTF-8، بما في ذلك وسوم HTML والصور المضمّنة. لا يشمل ترويسات الرسالة ولا المرفقات المنفصلة.
mail-outbox-load-failed = تعذّر فحص صندوق الصادر. جرّب «تحديث» بعد التحقق من اتصالك.
mail-outbox-unavailable = صندوق الصادر غير متاح. استخدم «تحديث» للمحاولة مرة أخرى.
mail-maxsec-view = العرض: { $mode }
mail-maxsec-menu = وضع العرض
mail-toast-tab-pinned = تم تثبيت علامة التبويب «{ $target }»
mail-toast-tab-unpinned = تم إلغاء تثبيت علامة التبويب «{ $target }»
mail-workspace-message = رسالة
mail-opened-announcement = تم فتح: { $subject }
mail-workspace-layout = تخطيط مساحة العمل
mail-workspace-tabs = علامات التبويب
mail-workspace-side-by-side = جنبًا إلى جنب
mail-workspace-stack = متراصّة
mail-workspace-refresh = تحديث المجلد
mail-workspace-load-error = تعذّر تحميل هذه الرسالة. أغلق علامة التبويب وحاول مرة أخرى.
mail-resize-folders = تغيير حجم الشريط الجانبي للمجلدات
mail-resize-list = تغيير حجم قائمة الرسائل
mail-resize-tiles = تغيير حجم أجزاء مساحة العمل
mail-show-folders = إظهار الشريط الجانبي للمجلدات
mail-hide-folders = إخفاء الشريط الجانبي للمجلدات
mail-compact-folders = أيقونات مجلدات مضغوطة
mail-toggle-list = إظهار قائمة الرسائل أو إخفاؤها
mail-toggle-reader = إظهار جزء القراءة أو إخفاؤه
mail-folder-counts = { $unread } غير مقروءة، { $total } رسالة
mail-print = طباعة / حفظ بصيغة PDF
mail-go-to-top = الانتقال إلى الأعلى
mail-print-help = اطبع هذه الرسالة أو اختر «حفظ بصيغة PDF» في مربع حوار الطباعة. تبقى الصور الخارجية محظورة.
mail-file-details = تفاصيل الملف وبصمته
mail-file-type = نوع الملف
mail-file-extension = الامتداد
mail-file-detected = التوقيع المكتشف
mail-file-unknown = لا يوجد توقيع معروف
mail-file-size = الحجم
mail-file-size-bytes = { $count } بايت
mail-file-hash-unavailable = البصمة غير متاحة (تتطلب متصفحًا آمنًا وملفًا لا يتجاوز 64 ميبيبايت).
mail-file-fingerprint-help = تحدد البصمة هذه البايتات بالضبط. وهي لا تثبت أن الملف آمن ولا من أنشأه.
mail-file-warning-executable = قد يشغّل هذا الملف برامج أو يغيّر إعدادات النظام. نزّله فقط إذا كنت تثق بمصدره.
mail-file-warning-macro = قد يحتوي هذا المستند على وحدات ماكرو. لا تفعّل وحدات الماكرو ما لم تتحقق من المرسل والغرض.
mail-file-warning-disguisedName = قد يخفي اسم الملف امتداده الحقيقي. راجع تفاصيل الملف قبل التنزيل.
mail-file-warning-typeMismatch = لا يطابق توقيع الملف المكتشف نوعه المعلن. تم حظر المعاينة التلقائية.
mail-file-warning-activeDocument = قد تحتوي هذه الصيغة على محتوى نشط. معاينتها معزولة، وقد يختلف سلوك النسخة المنزّلة عند فتحها.
mail-file-preview-blocked = المعاينة التلقائية معطّلة لنوع الملف هذا أو بسبب عدم تطابق النوع.
mail-quote-header-from = من:
mail-quote-header-sent = أُرسلت:
mail-quote-header-to = إلى:
mail-quote-header-cc = نسخة:
mail-quote-header-subject = الموضوع:
mail-spam-score = تحليل البريد غير المرغوب فيه
mail-spam-score-loading = جارٍ فحص ترويسات البريد غير المرغوب فيه…
mail-spam-score-unavailable = درجة البريد غير المرغوب فيه غير متاحة
mail-spam-score-unverified = أبلغت بها ترويسات الرسالة ولم يتم التحقق منها بشكل مستقل. قد تكون الترويسات التي يوفرها المرسل مزوّرة.
mail-spam-score-positive = تفيد الترويسة بأن الرسالة غير مرغوب فيها.
mail-spam-score-negative = تفيد الترويسة بأن الرسالة غير مصنفة كبريد غير مرغوب فيه.
mail-spam-score-unknown = لم يُقدَّم أي حكم مع هذه الدرجة.
mail-external-recipients = المستلمون الخارجيون
mail-external-count = { $count } مستلم(ين) خارج النطاقات الموثوقة
mail-external-help = تحقق من المستلمين في «إلى» و«نسخة» و«نسخة مخفية» قبل إرسال معلومات سرية. تعتمد الثقة على هوية الإرسال المحددة وسياسة النطاق السارية لديك.
mail-external-reviewed = لقد راجعت هؤلاء المستلمين الخارجيين.
mail-external-confirm-required = راجع المستلمين الخارجيين وأكّد ذلك قبل الإرسال.
mail-quick-reply = رد سريع
mail-quick-reply-help = اعرض علامة تبويب رد مضغوطة مع تعبئة المستلمين مسبقًا. متوقفة افتراضيًا. تظل فحوصات الأمان المعتادة وخيارات الإرسال سارية.
mail-quick-reply-expand = تحرير المستلمين والموضوع
mail-attachment-policy-loading = جارٍ فحص سياسة المرفقات…
mail-attachment-policy-unavailable = تعذّر فحص سياسة المرفقات. حاول مرة أخرى قبل فتح هذا الملف.
mail-attachment-policy-blocked = حظر المسؤول هذا المرفق أو تنزيله الأصلي.
mail-attachment-policy-warning = ينصح المسؤول بالحذر مع هذا المرفق. تابع فقط إذا كنت تثق بمصدره.
mail-attachment-policy-acknowledge = أفهم — متابعة
mail-attachment-policy-no-preview = عطّل المسؤول معاينات المرفقات. قد تظل التنزيلات الأصلية متاحة.
mail-time-sent = وقت الإرسال
mail-time-received = وقت الاستلام
mail-time-unknown = غير متوفر
mail-time-comparison = الوقت في { $zone }
mail-time-privacy = المنطقة الزمنية للمقارنة من اختيارك. لا تحدد الطوابع الزمنية للرسائل موقع أي شخص.
mail-time-event-comparison = مقارنة وقت الفعالية
mail-time-ambiguous = يتكرر هذا الوقت المحلي مرتين عند تغيير الساعات. يُعرض كلا الوقتين المحتملين؛ تحقق من الإزاحة المقصودة للفعالية قبل الحفظ.
mail-bulk-add-label = إضافة تصنيف
mail-select-messages = تحديد الرسائل
mail-bulk-remove-label = إزالة التصنيف
mail-mark-spam = تعليم كرسالة غير مرغوب فيها
mail-bulk-forward = إعادة توجيه الرسائل المحددة كمرفقات
mail-bulk-download = تنزيل الرسائل المحددة بصيغة mbox
mail-bulk-forward-subject = رسائل معاد توجيهها ({ $count })
mail-bulk-transfer-failed = تعذّر إعداد الرسائل المحددة. حدّد حتى 100 رسالة بإجمالي لا يتجاوز 32 ميبيبايت، وتحقق من أذونات التصدير في حسابك.
mail-toast-labels-added = تمت إضافة التصنيف { $label } إلى { $count } رسالة
mail-toast-labels-removed = تمت إزالة التصنيف { $label } من { $count } رسالة
mail-action-help-reply = الرد على المرسل في علامة تبويب إنشاء جديدة.
mail-action-help-reply-all = الرد على المرسل وبقية المستلمين في علامة تبويب إنشاء جديدة.
mail-action-help-forward = راجع هذه الرسالة وأعد توجيهها إلى مستلمين تختارهم.
mail-action-help-pin = ثبّت هذه الرسالة لسهولة الوصول إليها، أو أزل تثبيتها.
mail-action-help-read = غيّر ما إذا كانت هذه الرسالة معلَّمة كمقروءة. يمكنك التراجع عن التغيير.
mail-action-help-flag = ضع علامة المتابعة على الرسالة أو أزلها.
mail-action-help-archive = انقل هذه الرسالة إلى مجلد الأرشيف المضبوط.
mail-action-help-delete = انقل هذه الرسالة إلى مجلد المهملات. يمكنك التراجع عن النقل.
mail-action-help-spam = علّم هذه الرسالة كغير مرغوب فيها وانقلها إلى مجلد البريد غير المرغوب فيه المضبوط.
mail-action-help-export = افتح صفحة التصدير لتنزيل هذه الرسالة بصيغة تختارها.
mail-action-help-sweep = راجع رسائل هذا المرسل الأخرى واختر ما يجب تنظيفه (حذف الكل، أو الإبقاء على الأحدث، أو حذف الأقدم، أو حظر المرسل) قبل تطبيق أي شيء.
mail-empty-trash = إفراغ المهملات
mail-trash-help = راجع الرسائل الموجودة حاليًا في المهملات واحذفها نهائيًا. لا يمكن التراجع عن ذلك.
mail-trash-preparing = جارٍ إعداد لقطة للرسائل الموجودة في المهملات…
mail-trash-prepare-failed = تعذّر إعداد معاينة المهملات. تحقق من اتصالك وأعد فتحها. لم تُحذف أي رسائل.
mail-trash-confirm = حذف { $count } رسالة نهائيًا من المهملات؟ لا يمكن التراجع عن ذلك. تُستثنى الرسائل الواردة بعد هذه المعاينة، وتُتخطى الرسائل التي نُقلت خارج المهملات.
mail-trash-limit = تشمل هذه العملية حتى { $count } رسالة. تبقى رسائل أخرى في المهملات؛ راجع عملية أخرى لاحقًا لإزالتها.
mail-trash-permanent = حذف نهائي
mail-trash-stop = التوقف بعد الطلب الحالي
mail-trash-progress = تم حذف { $deleted }؛ وتم تخطي { $skipped } رسالة لم تعد في المهملات؛ وتبقى { $remaining } في هذه المعاينة.
mail-trash-stopped = توقف الحذف. المؤكَّد حذفه: { $deleted }؛ المتخطى: { $skipped }؛ المتبقي: { $remaining }؛ النتيجة غير معروفة: { $uncertain }. تحقق من المهملات قبل بدء عملية أخرى.
mail-links-title = روابط الرسالة
mail-links-review = مراجعة الروابط ({ $count })
mail-links-help = لا تُفتح روابط الرسالة إلا بعد مراجعة وجهتها هنا. لا يتم جلب أي وجهة تلقائيًا.
mail-links-inert = استخدم «مراجعة الروابط» أعلى الرسالة لفحص هذه الوجهة وفتحها.
mail-links-limit = تُعرض أول 100 وجهة مدعومة مختلفة. تبقى الروابط الإضافية غير نشطة في الرسالة.
mail-links-destination = الوجهة المراد فتحها
mail-links-external = هذه الوجهة من مرسل الرسالة. فتحها يخرجك من Mailwoman؛ راجع العنوان قبل المتابعة.
mail-links-open = فتح في علامة تبويب جديدة
mail-links-compose = مراجعة رسالة جديدة
mail-links-warning-http = يستخدم هذا العنوان HTTP غير المشفر.
mail-links-warning-address = تستخدم هذه الوجهة عنوان IP بدلًا من اسم نطاق.
mail-links-warning-international = يحتوي هذا النطاق على اسم مدوَّل. تحقق بعناية من تهجئته بحروف ASCII.
mail-links-warning-mismatch = عنوان الويب الظاهر يسمّي مضيفًا مختلفًا عن الوجهة الفعلية.
mail-links-warning-direction = يحتوي نص الرابط على محارف تغيّر الاتجاه قد تخفي عنوانه.
mail-outbox-uncertain = حالة التسليم غير معروفة
mail-outbox-uncertain-note = بدأ الإرسال لكن تعذّر تأكيد التسليم. لن تُعاد إرسال هذه الرسالة تلقائيًا. تحقق من «المرسلة» أو اسأل المستلم قبل الإرسال مرة أخرى.
mail-conversation-title = عرض المحادثة
mail-conversation-help = تصفّح رسائل المحادثة المتاحة في هذا الحساب عبر المجلدات. فتح رسالة لا يغيّر المجلد الذي اخترته.
mail-conversation-count = عرض { $loaded } من أصل { $total } رسالة في المحادثة.
mail-conversation-other-folder = مجلد آخر
mail-conversation-loading = جارٍ تحميل المحادثة…
mail-conversation-missing = لم تعد { $count } رسالة متاحة.
mail-conversation-error = تعذّر تحميل المحادثة. حاول مرة أخرى.
mail-conversation-more = تحميل المزيد من الرسائل
mail-conversation-limit = هذا العرض محدود بـ { $count } رسالة.
mail-compose-request-delivery = طلب إشعارات التسليم
mail-compose-request-delivery-hint = اطلب من خوادم البريد المستلمة الإبلاغ عن التسليم أو الفشل أو التأخير. لا يطلب ذلك ولا يثبت أن أحدًا قرأ الرسالة. تتضمن التقارير المرتجعة الترويسات دون النص الكامل.
mail-compose-delivery-unavailable = إشعارات التسليم غير متاحة أو معطّلة من المسؤول لهذا الحساب. لن يُزال الإشعار المطلوب صراحةً بصمت.
mail-compose-account-changed = تعود هذه الرسالة قيد الإنشاء إلى تسجيل دخول مختلف. سجّل الدخول إلى الحساب الصحيح وأعد فتح المسودة.
mail-context-open-message = فتح الرسالة
mail-context-open-folder = فتح المجلد
mail-context-download = تنزيل الرسالة
mail-context-refresh = تحديث المجلد
mail-context-folder-settings = إعدادات المجلد
mail-context-failed = تعذّر إكمال الإجراء. حاول مرة أخرى.
mail-sharing-unavailable = مشاركة المجلد غير متاحة أو لم تعد لديك صلاحية إدارتها.
mail-trash-cancelling = جارٍ التوقف بعد الطلب الحالي. المؤكَّد حذفه: { $deleted }؛ المتخطى: { $skipped }؛ المتبقي: { $remaining }. لا يمكن التراجع عن عمليات الحذف التي تم قبولها بالفعل.
mail-trash-already-running = حذف المهملات قيد التنفيذ بالفعل. استخدم إشعار التقدم لإيقاف الدفعات المقبلة.
mail-context-search-sender = البحث عن هذا المرسل
mail-context-search-subject = البحث عن الموضوع نفسه
mail-drag-too-many = اسحب حتى { $count } رسالة في كل مرة. حدّد مجموعة أصغر قبل النقل.
mail-subtab-reorder-help = اسحب لإعادة ترتيب علامات التبويب، أو اضغط Ctrl+Shift+السهم الأيسر أو الأيمن على علامة التبويب النشطة.
mail-compose-ribbon = شريط أدوات المحرر
mail-compose-ribbon-format = تنسيق النص
mail-compose-ribbon-insert = إدراج
mail-compose-font = الخط
mail-compose-font-size = حجم الخط
mail-compose-font-inherit = الافتراضي
mail-compose-undo = تراجع
mail-compose-redo = إعادة
mail-compose-undo-tip = تراجع ({ $shortcut })
mail-compose-redo-tip = إعادة ({ $shortcut })
mail-workspace-limit = أغلق علامة تبويب قبل فتح أخرى. يمكن الاحتفاظ بما يصل إلى 32 علامة تبويب.
mail-compose-private-storage-unavailable = استرداد المسودات المحلية غير متاح لهذه الجلسة. أبقِ هذا المحرر مفتوحًا حتى ترسل رسالتك أو تنسخها.
mail-offline-storage-unavailable = التغييرات دون اتصال غير متاحة إلى أن تتحقق هذه الجلسة من صلاحية التخزين. أعد الاتصال وسجّل الدخول مرة أخرى.
mail-draft-status-idle = لم تُحفظ في الحساب بعد
mail-draft-status-pending = تغييرات غير محفوظة
mail-draft-status-saving = جارٍ الحفظ في المسودات…
mail-draft-status-saved = تم الحفظ في مسودات الحساب
mail-draft-status-error = لم تُحفظ المسودة. تبقى نسخة الاسترداد المحلية متاحة.
mail-draft-status-uncertain = تأكيد الحفظ مفقود. تحقق من المسودات قبل إنشاء نسخة أخرى؛ الحفظ التلقائي متوقف مؤقتًا.
mail-draft-status-conflict = تغيّرت هذه المسودة على جهاز آخر. أعد فتح نسخة الحساب قبل المتابعة.
mail-draft-status-unsupported = لا يستطيع هذا الحساب حاليًا تأكيد عمليات الحفظ في مجلد المسودات.
mail-draft-protected = لا يُحفظ المحتوى المقتبس المشفر أو المفكوك تشفيره كنص عادي. تبقى أي مسودة سابقة في الحساب إلى أن تتجاهلها صراحةً.
mail-draft-save-now = حفظ في المسودات
mail-draft-save-failed = تعذّر حفظ هذه المسودة في حسابك. تحقق من نافذة الإنشاء قبل إغلاقها.
mail-draft-uncertain = ربما حُفظت المسودة. إعادة المحاولة التلقائية متوقفة مؤقتًا؛ تحقق من مجلد المسودات.
mail-draft-conflict = غيّر جهاز آخر هذه المسودة. لم تستبدل تعديلاتك نسخته.
mail-draft-cleanup-pending = حُفظت المسودة الجديدة، لكن تعذّرت إزالة نسختها السابقة. راجع المسودات قبل الحفظ مرة أخرى.
mail-draft-account-list = مسودات الحساب
mail-draft-loading = جارٍ تحميل مسودات الحساب…
mail-draft-close-recovery = إغلاق مع الاحتفاظ بالاسترداد المحلي فقط
mail-draft-discard-account = تجاهل
mail-draft-discard-pending = تعذّر تأكيد إزالة المسودة. يبقى سجلها في الحساب؛ أعد محاولة «تجاهل» بعد مراجعة مجلد المسودات.
mail-draft-limit = لدى هذا الحساب بالفعل 100 مسودة محفوظة. تجاهل مسودة حساب غير مرغوبة قبل حفظ مسودة جديدة.
mail-draft-quota-help = يُحتفظ بما يصل إلى 100 مسودة في الحساب. يزيل التجاهل النسخة المؤكدة على الخادم قبل تحرير مكانها.
mail-draft-discard-confirm = تأكيد إزالة المسودة
mail-draft-discard-confirm-detail = إزالة «{ $subject }» نهائيًا من مسودات الحساب؟ إغلاق نافذة الإنشاء بدلًا من ذلك يُبقي المسودة.
mail-draft-discard-title = تجاهل هذه المسودة؟
mail-draft-keep-editing = متابعة التحرير
mail-draft-discard-action = تجاهل
mail-draft-close-title = تجاهل التغييرات غير المحفوظة؟
mail-draft-close-protected = تحتوي هذه الرسالة على محتوى مشفر أو مفكوك تشفيره، لذا لا تُحفظ كمسودة. يؤدي إغلاقها إلى تجاهل هذه التغييرات. وتبقى أي مسودة سابقة في الحساب ضمن المسودات.
mail-draft-close-unstored = لا يمكن حفظ المسودات في هذه الجلسة، لذا يؤدي إغلاق هذه الرسالة إلى تجاهلها.
mail-workspace-view-mode = ترتيب العرض
mail-workspace-tabs-help = اعرض طريقة عرض نشطة واحدة مع إبقاء علامات التبويب الأخرى مفتوحة.
mail-workspace-side-by-side-help = اعرض طريقتي عرض محددتين جنبًا إلى جنب. اسحب علامة تبويب إلى أي منهما لاستبدالها.
mail-workspace-stack-help = اعرض طريقتي عرض محددتين إحداهما فوق الأخرى. اسحب علامة تبويب إلى أي منهما لاستبدالها.
mail-workspace-masonry-side-by-side = بلاطات متدرجة جنبًا إلى جنب
mail-workspace-masonry-side-by-side-help = رتّب جميع طرق العرض المفتوحة في أعمدة متدرجة. تعرض الشاشات الصغيرة علامة التبويب النشطة.
mail-workspace-masonry-stack = بلاطات متدرجة متراصّة
mail-workspace-masonry-stack-help = رتّب جميع طرق العرض المفتوحة في صفوف متدرجة. تعرض الشاشات الصغيرة علامة التبويب النشطة.
mail-workspace-view-first = العرض الأول
mail-workspace-view-second = العرض الثاني
mail-workspace-view-empty = اسحب علامة تبويب مفتوحة إلى هنا لاختيار طريقة العرض هذه.
mail-popup-blocked = اسمح بالنوافذ المنبثقة لـ Mailwoman لفصل علامة التبويب هذه. تبقى علامة تبويبك هنا.
mail-popup-unavailable = تعذّر تحميل النافذة المنفصلة. تبقى علامة تبويبك وتغييراتك غير المحفوظة في النافذة الرئيسية. حاول مرة أخرى بعد تحديث التطبيق.
mail-popup-reattach = إعادة الإرفاق بالنافذة الرئيسية
mail-popup-action-reattached = تمت العودة إلى النافذة الرئيسية لإكمال هذا الإجراء.
mail-popup-detached = علامة التبويب هذه مفتوحة في نافذة منفصلة. إغلاق تلك النافذة يعيدها إلى هنا.
mail-compose-from-custom = عنوان آخر مصرَّح به…
mail-compose-from-alias = عنوان المرسل لهذه الرسالة
mail-compose-from-alias-help = استخدم اسمًا مستعارًا مصرَّحًا لك بالإرسال منه. يغيّر هذا هذه المسودة فقط؛ ويُتحقق من التصريح قبل الإرسال.
mail-compose-from-refused = هذا العنوان غير مصرَّح له بالإرسال من هذا الحساب.
mail-compose-from-unavailable = تعذّر التحقق من عنوان المرسل هذا. لم تُرسل رسالتك.
mail-compose-from-invalid = أدخل عنوان بريد إلكتروني واحدًا صالحًا للمرسل.
mail-compose-from-toggle = إظهار المرسل لهذه الرسالة أو إخفاؤه
mail-compose-ribbon-security = الأمان
mail-compose-ribbon-delivery = التسليم
mail-compose-ribbon-recipients = المستلمون
mail-compose-ribbon-message-type = نوع الرسالة
mail-compose-recall-help = ألغِ الرسائل التي لا تزال محتجزة للتراجع أو المجدولة للتسليم. لا يمكن استدعاء الرسائل التي سُلّمت بالفعل.
mail-compose-recall-loading = جارٍ فحص الرسائل في قائمة الانتظار…
mail-compose-recall-empty = لا توجد رسائل في قائمة الانتظار يمكن إلغاؤها.

# -- Machine translation (2026-11): added to match en ---------------------
mail-compose-recall-failed = تعذّر تأكيد حالة قائمة الانتظار. أغلق هذه اللوحة وأعد فتحها لتحديثها قبل أي إجراء آخر.
mail-compose-importance = الأهمية
mail-compose-importance-high = عالية
mail-compose-importance-normal = عادية
mail-compose-importance-low = منخفضة
mail-insert-account-title = إدراج عنصر من الحساب
mail-insert-account-help = أرفق تصدير تقويم أو بطاقة عمل لجهة اتصال أو رسالة أصلية من هذا الحساب.
mail-insert-account-close = إغلاق منتقي عناصر الحساب
mail-insert-account-kind = نوع العنصر
mail-insert-account-event = فعالية تقويم (.ics)
mail-insert-account-calendar = تقويم (.ics)
mail-insert-account-contact = بطاقة عمل لجهة اتصال (.vcf)
mail-insert-account-message = رسالة أصلية (.eml)
mail-insert-account-search = تصفية العناصر المعروضة
mail-insert-account-selection = اختر عنصرًا
mail-insert-account-scope = تُعرض فقط عناصر الحساب الحالي. تخضع الرسائل الأصلية لسياسة التصدير على الخادم.
mail-insert-account-limited = يتم عرض ما يصل إلى 100 عنصر. تبحث التصفية في هذه القائمة فقط.
mail-insert-account-unavailable = لا توجد تصديرات حساب مدعومة متاحة.
mail-insert-account-empty = لا توجد عناصر مطابقة.
mail-insert-account-attach = إرفاق العنصر المحدد
mail-insert-account-loading = جارٍ تحميل عنصر الحساب…
mail-insert-account-error = تعذّر إرفاق العنصر. تحقق من صلاحيات الوصول وسياسة التصدير ثم حاول مرة أخرى. قد تحتاج التقويمات الكبيرة إلى تصدير كل فعالية على حدة.
mail-compose-html-source = مصدر HTML
mail-editor-clear-format = مسح التنسيق
mail-compose-format-html = HTML
mail-compose-html-visual = المحرر المرئي
mail-compose-html-help = حرّر HTML مباشرةً. يُحفظ ويُرسل فقط تنسيق البريد المدعوم؛ وتُزال النصوص البرمجية والصور الخارجية والترميز غير الآمن.
mail-compose-html-processing = جارٍ إعداد HTML آمن…
mail-compose-html-failed = تعذّر إعداد HTML. عدّله أو حاول مرة أخرى قبل الحفظ أو الإرسال.
mail-compose-check-names = التحقق من الأسماء
mail-compose-check-names-help = حلّ الأسماء المطابقة تمامًا من جهات الاتصال المحمّلة وتحقق من تنسيق العناوين. لا يتحقق هذا من إمكانية التسليم إلى صندوق البريد.
mail-compose-names-unresolved = اختر عنوان بريد إلكتروني كاملًا لهذه الأسماء غير المحلولة أو الملتبسة: { $names }
mail-compose-names-checked = تم التحقق من تنسيق المستلمين؛ تم حل { $count } من أسماء جهات الاتصال. لم يتم التحقق من التسليم إلى صندوق البريد.
mail-compose-ribbon-table = جدول
mail-compose-group-clipboard = الحافظة
mail-compose-group-paragraph = الفقرة
mail-compose-group-editing = التحرير
mail-compose-group-view = العرض
mail-compose-dark-background = خلفية داكنة
mail-compose-dark-background-help = اعرض الرسالة على خلفية داكنة أثناء الكتابة. تتغير شاشتك فقط: تُرسل الرسالة دون لون خلفية.
mail-compose-group-assist = مساعد الكتابة
mail-editor-counts = إحصاءات المحرر
mail-editor-words = { $count ->
    [zero] لا كلمات
    [one] كلمة واحدة
    [two] كلمتان
    [few] { $count } كلمات
    [many] { $count } كلمة
   *[other] { $count } كلمة
}
mail-editor-paragraphs = { $count ->
    [zero] لا فقرات
    [one] فقرة واحدة
    [two] فقرتان
    [few] { $count } فقرات
    [many] { $count } فقرة
   *[other] { $count } فقرة
}
mail-editor-characters = { $count ->
    [zero] لا أحرف
    [one] حرف واحد
    [two] حرفان
    [few] { $count } أحرف
    [many] { $count } حرفًا
   *[other] { $count } حرف
}
mail-editor-selection-required = حدّد نصًا أو خلايا جدول مناسبة قبل استخدام هذا الإجراء.
mail-editor-clipboard-unavailable = تعذّر الوصول إلى الحافظة. استخدم اختصار النسخ أو القص أو اللصق في المتصفح.
mail-editor-selection-changed = تغيّر التحديد. حدّد النص المقصود وحاول مرة أخرى.
mail-editor-clipboard = الحافظة والتنسيق
mail-editor-copy = نسخ
mail-editor-cut = قص
mail-editor-paste = لصق
mail-editor-copy-format = نسخ التنسيق
mail-editor-apply-format = تطبيق التنسيق
mail-editor-style = نمط الفقرة
mail-editor-text-color = لون الخط
mail-editor-clear-color = اللون الافتراضي
mail-editor-highlight = تمييز
mail-editor-clear-highlight = مسح التمييز
mail-editor-paragraph = خيارات الفقرة
mail-editor-alignment = المحاذاة
mail-editor-line-height = ارتفاع السطر
mail-editor-space-before = المسافة قبل
mail-editor-space-after = المسافة بعد
mail-editor-indent = المسافة البادئة
mail-editor-shading = تظليل الفقرة
mail-editor-clear-shading = مسح التظليل
mail-editor-case = تغيير حالة الأحرف
mail-editor-sort = فرز الفقرات المحددة
mail-editor-sort-alpha = أبجديًا
mail-editor-sort-numeric = ترتيب رقمي
mail-editor-table-rows = صفوف الجدول
mail-editor-table-columns = أعمدة الجدول
mail-editor-table-insert = إدراج جدول
mail-editor-table-limits = ما يصل إلى 20 صفًا و10 أعمدة و200 خلية لكل جدول؛ وعلى الأكثر 10 جداول. الجداول المتداخلة غير مدعومة.
mail-editor-table-row-before = صف أعلاه
mail-editor-table-row-after = صف أسفله
mail-editor-table-row-delete = حذف الصف
mail-editor-table-column-before = عمود قبله
mail-editor-table-column-after = عمود بعده
mail-editor-table-column-delete = حذف العمود
mail-editor-table-merge = دمج الخلايا
mail-editor-table-split = تقسيم الخلية
mail-editor-table-header = تبديل صف الترويسة
mail-editor-table-delete = حذف الجدول
mail-editor-table-shading = تظليل الخلية
mail-editor-find-replace = بحث واستبدال
mail-editor-find = البحث عن نص
mail-editor-replacement = استبدال بـ
mail-editor-match-case = مطابقة حالة الأحرف
mail-editor-previous = التطابق السابق
mail-editor-next = التطابق التالي
mail-editor-replace = استبدال التطابق المحدد
mail-editor-replace-all = استبدال كل التطابقات
mail-editor-matches = { $count } تطابقًا (حتى 1,000)
mail-editor-select-all = تحديد الكل
mail-editor-clear-selection = مسح التحديد
mail-editor-history = سجل التراجع
mail-editor-undo-groups = التراجع عن آخر { $count } من مجموعات التغييرات
mail-editor-zoom = تكبير المحرر
mail-editor-markers = علامات الفقرات
mail-editor-style-normal = عادي
mail-editor-style-no-spacing = بلا تباعد
mail-editor-style-title = عنوان
mail-editor-style-subtitle = عنوان فرعي
mail-editor-style-heading1 = عنوان رئيسي 1
mail-editor-style-heading2 = عنوان رئيسي 2
mail-editor-style-heading3 = عنوان رئيسي 3
mail-editor-style-heading4 = عنوان رئيسي 4
mail-editor-style-heading5 = عنوان رئيسي 5
mail-editor-style-heading6 = عنوان رئيسي 6
mail-editor-align-left = يسار
mail-editor-align-center = وسط
mail-editor-align-right = يمين
mail-editor-align-justify = ضبط
mail-editor-case-lower = أحرف صغيرة
mail-editor-case-upper = أحرف كبيرة
mail-editor-case-title = أحرف العناوين
mail-editor-case-sentence = أحرف الجمل
mail-editor-action-history = سجل { $action }
mail-editor-history-empty = لا توجد إجراءات متاحة
mail-editor-redo-groups = إعادة { $count } من التغييرات
mail-compose-identities-failed = تعذّر تحميل هويات الإرسال. أغلق هذه المسودة وأعد فتحها للمحاولة مرة أخرى.
mail-folders-loading = جارٍ تحميل المجلدات…
mail-initial-sync-error = تعذّرت مزامنة رسائلك حتى الآن.
mail-initial-sync-error-help = انقطع الاتصال بخادم بريدك. حاول مرة أخرى، أو اطلب من المسؤول فحص سجلات الاتصال إذا استمرت المشكلة.
mail-links-trusted = نطاق موثوق: يُفتح دون مراجعة
mail-links-trust-domain = الوثوق بـ { $domain }
mail-links-trust-failed = تعذّر حفظ النطاق الموثوق. حاول مرة أخرى.
mail-focus-always-focused = النقل دائمًا إلى «المهم»
mail-focus-always-other = النقل دائمًا إلى «أخرى»
mail-focus-moved-focused = ستذهب الرسائل من { $sender } الآن إلى «المهم».
mail-focus-moved-other = ستذهب الرسائل من { $sender } الآن إلى «أخرى».
mail-focus-save-failed = تعذّر حفظ اختيار «المهم» لهذا المرسل. حاول مرة أخرى.
mail-context-search-domain = البحث عن الرسائل من هذا النطاق
mail-loading-mailbox = جارٍ تحميل { $name }…
mail-user-folders = مجلدات المستخدم
mail-context-hide-folder = إخفاء من الشريط الجانبي
mail-folder-fallback = المجلد
mail-toast-moving = { $count ->
    [zero] لا رسائل لنقلها إلى { $folder }…
    [one] جارٍ نقل رسالة واحدة إلى { $folder }…
    [two] جارٍ نقل رسالتين إلى { $folder }…
    [few] جارٍ نقل { $count } رسائل إلى { $folder }…
    [many] جارٍ نقل { $count } رسالة إلى { $folder }…
   *[other] جارٍ نقل { $count } رسالة إلى { $folder }…
    }
mail-toast-move-queueing = { $count ->
    [zero] لا رسائل لإضافتها إلى قائمة النقل إلى { $folder }…
    [one] جارٍ إضافة رسالة واحدة إلى قائمة النقل إلى { $folder }…
    [two] جارٍ إضافة رسالتين إلى قائمة النقل إلى { $folder }…
    [few] جارٍ إضافة { $count } رسائل إلى قائمة النقل إلى { $folder }…
    [many] جارٍ إضافة { $count } رسالة إلى قائمة النقل إلى { $folder }…
   *[other] جارٍ إضافة { $count } رسالة إلى قائمة النقل إلى { $folder }…
    }
mail-toast-move-queued = { $count ->
    [zero] لا رسائل في قائمة النقل إلى { $folder }
    [one] أُضيفت رسالة واحدة إلى قائمة النقل إلى { $folder }
    [two] أُضيفت رسالتان إلى قائمة النقل إلى { $folder }
    [few] أُضيفت { $count } رسائل إلى قائمة النقل إلى { $folder }
    [many] أُضيفت { $count } رسالة إلى قائمة النقل إلى { $folder }
   *[other] أُضيفت { $count } رسالة إلى قائمة النقل إلى { $folder }
    }
mail-toast-move-partial = تم نقل { $count } من أصل { $total } رسالة إلى { $folder }؛ وتعذّر نقل { $failed }
mail-send-copy-folder-unavailable = مجلد نسخة المرسَل المحدد غير متاح. اختر مجلدًا آخر قبل الإرسال.
mail-send-copy-failed = قُبلت الرسالة، لكن تعذّر حفظ نسخة المرسَل منها. لا تُرسلها مرة أخرى.
mail-compose-attach-public-key = إرفاق مفتاحي العام أو شهادتي
mail-compose-include-hashes = تضمين تجزئات SHA-256 للملفات المرفقة
mail-compose-hashes-pending = جارٍ حساب تجزئات المرفقات. أرسل عند الجاهزية.
mail-compose-hashes-failed = تعذّر التحقق من بايتات المرفقات. أوقف التجزئات أو أعد المحاولة بتفعيلها مجددًا.
mail-compose-crypto-pending = يجري تحديث التشفير. يُرجى الانتظار قبل الإرسال.
mail-compose-public-key-unavailable = لا يوجد مفتاح عام أو شهادة حالية تطابق عنوان الإرسال.
mail-compose-sent-copy = حفظ الرسالة المرسلة في
mail-compose-sent-copy-default = الافتراضي للحساب
folder-create = إنشاء مجلد
folder-name = اسم المجلد
folder-parent = المجلد الأصل
folder-top-level = المستوى الأعلى
folder-created = تم إنشاء المجلد.
folder-create-unavailable = إنشاء المجلدات غير متاح لهذا الحساب أو المجلد الأصل.
folder-create-invalid = اختر اسم مجلد صالحًا وفريدًا دون فواصل تسلسل هرمي أو أحرف تحكم.
folder-create-failed = تعذّر إنشاء المجلد. حدّث الصفحة وحاول مرة أخرى.
folder-cleanup = تنظيف المجلد…
folder-cleanup-title = تنظيف { $name }
folder-cleanup-help = ابحث عن الرسائل الأقدم من العمر المحدد. راجع العدد قبل الحذف. المجلدات الفرعية مستثناة.
folder-cleanup-age = أقدم من
folder-age-days = أيام
folder-age-weeks = أسابيع
folder-age-months = أشهر
folder-age-years = سنوات
folder-cleanup-preview = معاينة التنظيف
folder-cleanup-confirm = حذف الرسائل المطابقة
folder-cleanup-permanent = حذف { $count } رسالة مطابقة نهائيًا. لا يمكن التراجع عن ذلك.
folder-cleanup-trash = نقل { $count } رسالة مطابقة إلى المهملات.
folder-cleanup-before = استُلمت قبل { $date }
folder-cleanup-done = تم تنظيف { $count } رسالة.
mail-from = المرسل
mail-subject = الموضوع
mail-date = التاريخ
mail-cancel = إلغاء
mail-bulk-selected = تم تحديد { $count } محادثة
mail-bulk-select-folder = تحديد الكل في { $folder }
mail-list-order = ترتيب الرسائل

# -- Machine translation (2026-11): added to match en ---------------------
mail-list-order-newest = الأحدث أولًا
mail-list-order-oldest = الأقدم أولًا
mail-list-order-sender = المرسل
mail-list-order-subject = الموضوع
mail-list-order-error = تعذّر حفظ ترتيب الرسائل.
mail-bulk-selection-load-failed = تعذّر تحديد المجلد بالكامل. تحقق من اتصالك وحاول مرة أخرى.
mail-privacy-bar = خصوصية الرسالة
mail-privacy-trackers-allow = السماح بالمتتبعات لهذه الرسالة
mail-privacy-trackers-block = حظر المتتبعات مجددًا
mail-privacy-images-loading = عرض { $loaded }/{ $total } صور · جارٍ تحميل { $pending }
mail-privacy-images-failed = عرض { $loaded }/{ $total } صور · فشل { $failed }
mail-privacy-images-loaded = عرض { $loaded }/{ $total } صور
mail-privacy-images-loading-failed = عرض { $loaded }/{ $total } صور · جارٍ تحميل { $pending } · فشل { $failed }
mail-reader-folder-title = المجلد: { $folder }
mail-reader-move-to = نقل إلى…
mail-action-help-move = انقل هذه الرسالة إلى مجلد آخر.
folder-ops-new-subfolder = مجلد فرعي جديد
folder-ops-mark-read = تعليم الكل كمقروء
folder-ops-this-folder = هذا المجلد
folder-ops-with-subfolders = تضمين المجلدات الفرعية
folder-ops-no-unread = لا توجد رسائل غير مقروءة.
folder-ops-read-unavailable = لا يمكنك تغيير الرسائل في هذا المجلد.
folder-ops-more = المزيد
folder-ops-system-folder = لا يمكن تغيير مجلد النظام بهذه الطريقة.
folder-ops-sort-subfolders = ترتيب المجلدات الفرعية حسب
folder-ops-sort-folders = ترتيب المجلدات حسب
folder-ops-sort-name = الاسم
folder-ops-sort-unread = عدد غير المقروءة
folder-ops-sort-total = العدد الإجمالي
folder-ops-sort-date = آخر نشاط
folder-ops-sort-custom = ترتيب مخصص
folder-ops-sort-asc = تصاعدي
folder-ops-sort-desc = تنازلي
folder-ops-delete-with-subfolders = حذف المجلد والمجلدات الفرعية
folder-ops-delete-unavailable = لا يمكن حذف هذا المجلد.
folder-ops-delete-tree-body = سيؤدي هذا إلى حذف { $folders } مجلدًا و{ $count } رسالة بداخلها نهائيًا. لا يمكن التراجع عن ذلك.
folder-ops-copy-all-menu = نسخ كل العناصر إلى…
folder-ops-move-all-menu = نقل كل العناصر إلى…
folder-ops-move-folder-menu = نقل المجلد إلى…
folder-ops-move-up = نقل لأعلى
folder-ops-move-down = نقل لأسفل
folder-ops-move-unavailable = لا يمكنك نقل هذا المجلد أو عناصره.
folder-ops-move-duplicate = توجد في الوجهة بالفعل مجلد بهذا الاسم.
folder-ops-move-folder-title = نقل { $name }
folder-ops-move-folder-body = يُنقل المجلد مع مجلداته الفرعية ورسائله. اختر المجلد الذي سيوضع بداخله، أو المستوى الأعلى.
folder-ops-move-here = نقل إلى هنا
folder-ops-moved-folder = تم نقل { $name }.
folder-ops-choose-folder = اختر مجلد الوجهة.
folder-ops-destination = مجلد الوجهة
folder-ops-include-subfolders = تضمين رسائل { $count } من المجلدات الفرعية
folder-ops-copy-all-title = نسخ كل عناصر { $name }
folder-ops-move-all-title = نقل كل عناصر { $name }
folder-ops-copy-all-body = تُضاف { $count } رسالة إلى مجلد الوجهة. وتبقى في مكانها الأصلي.
folder-ops-move-all-body = تُنقل { $count } رسالة إلى مجلد الوجهة. يمكنك التراجع عن ذلك لاحقًا.
folder-ops-copy-all = نسخ العناصر
folder-ops-move-all = نقل العناصر
folder-ops-undo = تراجع
folder-ops-stop = إيقاف
folder-ops-marking = جارٍ تعليم { $name } كمقروء…
folder-ops-marking-progress = جارٍ تعليم { $name } كمقروء: { $done } من { $total }
folder-ops-marked = تم تعليم { $count } رسالة في { $name } كمقروءة.
folder-ops-copying = جارٍ نسخ { $name }…
folder-ops-copying-progress = جارٍ نسخ { $name }: { $done } من { $total }
folder-ops-copied = تم نسخ { $count } رسالة من { $name }.
folder-ops-moving = جارٍ نقل { $name }…
folder-ops-moving-progress = جارٍ نقل { $name }: { $done } من { $total }
folder-ops-moved = تم نقل { $count } رسالة من { $name }.
folder-ops-stopped = توقفت عملية { $name } بعد { $count } من أصل { $total } رسالة.
folder-ops-partial = تمت معالجة { $count } من أصل { $total } رسالة في { $name }؛ ورُفضت { $refused }.
folder-ops-failed = تعذّر إكمال { $name }. حدّث الصفحة وحاول مرة أخرى.
folder-ops-error-forbidden = ليست لديك صلاحية إجراء هذا التغيير.
folder-ops-error-exists = يوجد هناك مجلد بهذا الاسم بالفعل.
folder-ops-error-mailboxHasEmail = لا يزال المجلد يحتوي على رسائل.
folder-ops-error-notFound = لم يعد المجلد موجودًا.
folder-ops-error-failed = تعذّر إجراء التغيير. حدّث الصفحة وحاول مرة أخرى.
folder-ops-expand = إظهار المجلدات الفرعية لـ { $name }
folder-ops-collapse = إخفاء المجلدات الفرعية لـ { $name }
mail-context-reply = رد
mail-context-reply-all = رد على الجميع
mail-context-forward = إعادة توجيه
mail-context-quick-print = طباعة سريعة
mail-context-copy = نسخ
mail-context-copy-to-folder = نسخ الرسالة إلى مجلد…
mail-context-copy-sender = نسخ عنوان المرسل
mail-context-copy-subject = نسخ الموضوع
mail-context-copy-link = نسخ رابط الرسالة
mail-context-copied = تم النسخ.
mail-context-copy-failed = تعذّر النسخ إلى الحافظة.
mail-context-print-failed = تعذّرت طباعة الرسالة.
mail-reply-images-title = صور الرسالة الأصلية
mail-reply-images-ask = هل تريد تضمين صور الرسالة الأصلية في هذا الرد؟
mail-reply-images-include = تضمين الصور
mail-reply-images-not-now = ليس الآن
mail-reply-images-trust = الوثوق دائمًا بـ { $domain }
mail-reply-images-loading = جارٍ إضافة الصور…
mail-reply-images-cancelled = أُلغي التحميل. لم تُضف الصور.
mail-reply-images-failed = تعذّرت إضافة الصور.
mail-reply-images-retry = تحميل مرة أخرى
mail-image-cancel = إلغاء
mail-compose-from-why-match = الإرسال من { $identity } لأن الرسالة أُرسلت إلى { $address }.
mail-compose-from-why-match-plus = الإرسال من { $identity } لأن الرسالة أُرسلت إلى { $address }، وهو عنوان بعلامة + مشتق منه.
mail-compose-from-why-ambiguous = تتطابق عدة هويات مع { $address }. الإرسال من { $identity }؛ اختر هوية أخرى:
mail-compose-from-why-default = الإرسال من هويتك الافتراضية { $identity } لأنه لا توجد هوية تطابق مستلمي الرسالة الأصلية.
mail-compose-from-why-forced-default = الإرسال من هويتك الافتراضية { $identity } كما هو مضبوط في إعداداتك.
mail-compose-from-why-keep = الإرسال من { $identity }.
mail-compose-from-change = تغيير
reader-attachments-reason-encrypted = هذه الرسالة مشفرة. تظهر مرفقاتها بعد فتح قفلها.
reader-attachments-reason-list-missing = تعذّر تحميل قائمة مرفقات هذه الرسالة، لذا فإن شريط المرفقات مخفي.
reader-attachments-reason-inline-only = مرفقات هذه الرسالة ({ $count }) هي صور مضمّنة تُعرض في الرسالة نفسها.
reader-attachments-reason-unavailable-parts = { $count ->
    [zero] لا مرفقات في هذه الرسالة بلا محتوى قابل للتنزيل، لذا فإن شريط المرفقات مخفي.
    [one] مرفق واحد في هذه الرسالة بلا محتوى قابل للتنزيل، لذا فإن شريط المرفقات مخفي.
    [two] مرفقان في هذه الرسالة بلا محتوى قابل للتنزيل، لذا فإن شريط المرفقات مخفي.
    [few] { $count } مرفقات في هذه الرسالة بلا محتوى قابل للتنزيل، لذا فإن شريط المرفقات مخفي.
    [many] { $count } مرفقًا في هذه الرسالة بلا محتوى قابل للتنزيل، لذا فإن شريط المرفقات مخفي.
   *[other] { $count } مرفق في هذه الرسالة بلا محتوى قابل للتنزيل، لذا فإن شريط المرفقات مخفي.
  }
reader-attachments-reason-policy-blocked = { $count ->
    [zero] تحظر سياسة المرفقات لديك تنزيل أي مرفق.
    [one] تحظر سياسة المرفقات لديك تنزيل هذا المرفق.
    [two] تحظر سياسة المرفقات لديك تنزيل هذين المرفقين.
    [few] تحظر سياسة المرفقات لديك تنزيل هذه المرفقات.
    [many] تحظر سياسة المرفقات لديك تنزيل هذه المرفقات.
   *[other] تحظر سياسة المرفقات لديك تنزيل هذه المرفقات.
  }
reader-attachments-reason-policy-unavailable = تعذّر فحص سياسة المرفقات، لذا لا يمكن فتح المرفقات بعد.
mail-drafts-popup-count = { $count ->
    [zero] لا مسودات
    [one] مسودة واحدة
    [two] مسودتان
    [few] { $count } مسودات
    [many] { $count } مسودة
   *[other] { $count } مسودة
  }
mail-drafts-popup-search = البحث بالموضوع أو المستلم
mail-drafts-popup-tab-local = المسودات المحلية
mail-drafts-popup-local-hint = تُحفظ المسودات المحلية في هذا المتصفح فقط.
mail-drafts-popup-badge-account = الحساب
mail-drafts-popup-badge-local = محلية
mail-drafts-popup-current = مفتوحة الآن
mail-drafts-popup-select = تحديد
mail-drafts-popup-select-all = تحديد الكل
mail-drafts-popup-clear = مسح
mail-drafts-popup-done = تم
mail-drafts-popup-selected = تم تحديد { $count }
mail-drafts-popup-select-row = تحديد { $subject }
mail-drafts-popup-open = فتح
mail-drafts-popup-discard = تجاهل
mail-drafts-popup-discard-selected = تجاهل المحدد
mail-drafts-popup-more = المزيد من الإجراءات لـ { $subject }
mail-drafts-popup-retry = إعادة المحاولة
mail-drafts-popup-load-failed = تعذّر تحميل مسودات الحساب.
mail-drafts-popup-empty-account = لا توجد مسودات في الحساب
mail-drafts-popup-no-match = لا توجد مسودات تطابق «{ $query }»
mail-drafts-popup-discarding = جارٍ التجاهل…
mail-drafts-popup-row-failed = تعذّر التجاهل. تبقى نسخته في الحساب.
mail-drafts-popup-confirm-title = { $count ->
    [zero] تجاهل أي مسودة؟
    [one] تجاهل هذه المسودة؟
    [two] تجاهل المسودتين؟
    [few] تجاهل { $count } مسودات؟
    [many] تجاهل { $count } مسودة؟
   *[other] تجاهل { $count } مسودة؟
  }
mail-drafts-popup-confirm-account = تُزال مسودات الحساب أولًا من مجلد المسودات في حسابك، ولا يُحرَّر مكانها إلا بعد تأكيد هذه الإزالة.
mail-drafts-popup-confirm-local = تُزال المسودات المحلية من هذا المتصفح.
mail-drafts-popup-announce-done = { $count ->
    [zero] لم تُتجاهل أي مسودة
    [one] تم تجاهل مسودة واحدة
    [two] تم تجاهل مسودتين
    [few] تم تجاهل { $count } مسودات
    [many] تم تجاهل { $count } مسودة
   *[other] تم تجاهل { $count } مسودة
  }
mail-drafts-popup-announce-failed = تم تجاهل { $done }، وتعذّر تجاهل { $failed }
mail-check-title = تحقق قبل الإرسال
mail-check-intro = قد تحتاج هذه الأمور إلى نظرة ثانية:
mail-check-invalid = تبدو بعض عناوين المستلمين غير صالحة: { $list }
mail-check-subject = الموضوع فارغ.
mail-check-attachment = تذكر الرسالة مرفقًا، لكن لم يُرفق شيء.
mail-check-external = مستلمون خارج النطاقات الموثوقة: { $list }
mail-check-send-anyway = إرسال على أي حال
mail-check-review = مراجعة
mail-check-remember = عدم السؤال مرة أخرى عن هذه
mail-check-denied-invalid = الإرسال محظور بحسب إعداداتك. صحّح عناوين المستلمين غير الصالحة: { $list }
mail-check-denied-subject = الإرسال محظور بحسب إعداداتك. أضف موضوعًا.
mail-check-denied-attachment = الإرسال محظور بحسب إعداداتك. أرفق الملف الذي تذكره، أو أعد صياغة الرسالة.
mail-check-denied-external = الإرسال محظور بحسب إعداداتك. خارج النطاقات الموثوقة: { $list }
mail-alerts-label = مشكلات الرسالة
mail-alert-invalid = عنوان غير صالح: { $address }
mail-alert-subject = بلا موضوع
mail-recipient-chip-remove = إزالة { $address }
mail-recipient-chips = المستلمون في { $field }
mail-recipient-invalid-title = ليس عنوان بريد إلكتروني صالحًا
mail-recipient-chip-name = { $name }، { $address }. اضغط Enter لعرض تفاصيل جهة الاتصال، وBackspace أو F2 للتحرير، وDelete للإزالة
mail-recipient-chip-name-invalid = { $address }، ليس عنوانًا صالحًا. اضغط Enter للتحرير وDelete للإزالة
mail-recipient-added = تمت إضافة { $name }
mail-recipient-removed = تمت إزالة { $name }
mail-recipient-removed-many = { $count ->
    [zero] لم تُزل أي مستلمين
    [one] تمت إزالة مستلم واحد
    [two] تمت إزالة مستلمين
    [few] تمت إزالة { $count } مستلمين
    [many] تمت إزالة { $count } مستلمًا
   *[other] تمت إزالة { $count } مستلم
}
mail-recipient-editing = جارٍ تحرير { $name }
mail-recipient-all-selected = { $count ->
    [zero] لم يُحدَّد أي مستلم
    [one] تم تحديد مستلم واحد
    [two] تم تحديد مستلمين
    [few] تم تحديد { $count } مستلمين
    [many] تم تحديد { $count } مستلمًا
   *[other] تم تحديد { $count } مستلم
}
mail-external-line = { $count } خارج النطاقات الموثوقة: { $domains }
mail-external-dismiss = إغلاق
mail-status-label = حالة المحرر

# -- Machine translation (2026-11): added to match en ---------------------
mail-status-more = { $count } إشعارات أخرى
mail-status-more-label = المزيد من الإشعارات
mail-status-no-recipients = لا مستلمين
mail-status-no-attachment = لا مرفق
mail-status-external = خارجي: { $count }
mail-status-external-reviewed = تمت مراجعة المستلمين الخارجيين
mail-status-external-review = تعليم كمُراجَع
mail-status-from-match = من { $identity }
mail-status-from-match-plus = من { $identity }
mail-status-from-ambiguous = من { $identity } (عدة تطابقات)
mail-status-from-default = من الهوية الافتراضية
mail-status-from-forced-default = من الهوية الافتراضية
mail-status-from-keep = من { $identity }
mail-status-held = { $count } في قائمة الانتظار
mail-status-storage = لا استرداد محلي
mail-status-protected = لا تُحفظ كنص عادي
mail-status-saving = جارٍ الحفظ…
mail-status-saved = تم الحفظ { $time }
mail-status-saved-full = تم الحفظ في مسودات الحساب · { $time }
mail-status-not-saved = لم تُحفظ المسودة
mail-status-retry = إعادة المحاولة
mail-status-conflict = تغيّرت على جهاز آخر
mail-status-uncertain = لم يتأكد الحفظ
mail-status-close-recovery = إغلاق مع الإبقاء على الاسترداد
mail-status-unsupported = حفظ المسودة غير مؤكد
mail-compose-header = ترويسة الرسالة
mail-header-label = ترويسة الرسالة
mail-header-collapse = طي ترويسة الرسالة
mail-header-expand = توسيع ترويسة الرسالة: { $summary }
mail-header-from-chip = من { $address }. تغيير المرسل
mail-compose-group-timing = توقيت الإرسال
mail-compose-group-priority = الأولوية
mail-compose-group-receipts = الإيصالات والتتبع
mail-compose-group-keys-files = المفاتيح والملفات
mail-compose-group-sent-copy = نسخة المرسَل
mail-editor-style-quote = اقتباس
mail-editor-style-code = رمز برمجي
mail-compose-ribbon-table-layout = تخطيط الجدول
mail-compose-ribbon-picture = صورة
mail-editor-table-group-rows = الصفوف
mail-editor-table-group-columns = الأعمدة
mail-editor-table-group-cells = الخلايا
mail-editor-table-row-height = ارتفاع الصف
mail-editor-table-column-width = عرض العمود
mail-editor-table-size-auto = تلقائي
mail-editor-table-padding = حشوة الخلية
mail-editor-table-padding-compact = مضغوطة
mail-editor-table-padding-normal = عادية
mail-editor-table-padding-roomy = واسعة
mail-editor-table-align-left = محاذاة الخلية لليسار
mail-editor-table-align-center = محاذاة الخلية للوسط
mail-editor-table-align-right = محاذاة الخلية لليمين
mail-picture-group-size = الحجم
mail-picture-group-arrange = الترتيب
mail-picture-group-text = النص
mail-picture-group-style = النمط
mail-picture-group-edit = تحرير
mail-picture-width = العرض
mail-picture-height = الارتفاع
mail-picture-lock = الحفاظ على النسب
mail-picture-reset-size = الحجم الأصلي
mail-picture-align-inline = ضمن النص
mail-picture-align-left = محاذاة لليسار
mail-picture-align-center = وسط
mail-picture-align-right = محاذاة لليمين
mail-picture-alt = النص البديل
mail-picture-title = العنوان
mail-picture-border = الحد
mail-picture-radius = الزوايا
mail-picture-none = بلا
mail-picture-round = مستديرة
mail-picture-rotate-ccw = تدوير لليسار
mail-picture-rotate-cw = تدوير لليمين
mail-picture-flip-h = قلب أفقي
mail-picture-flip-v = قلب عمودي
mail-picture-crop = اقتصاص
mail-picture-crop-left = اليسار (%)
mail-picture-crop-top = الأعلى (%)
mail-picture-crop-right = اليمين (%)
mail-picture-crop-bottom = الأسفل (%)
mail-picture-crop-apply = اقتصاص
mail-picture-replace = استبدال
mail-picture-remove = إزالة
mail-picture-failed-no-image = حدّد صورة أولًا.
mail-picture-failed-no-source = لا توجد لهذه الصورة نسخة محلية للتحرير. استبدلها بدلًا من ذلك.
mail-picture-failed-failed = تعذّر تغيير الصورة.
mail-context-download-attachments = تنزيل المرفقات
mail-context-download-named = تنزيل { $name }
mail-context-download-all = تنزيل الكل بصيغة ZIP
mail-context-loading-attachments = جارٍ تحميل المرفقات…
mail-context-attachments-failed = تعذّر تحميل المرفقات.
mail-context-searching = جارٍ البحث عن الرسالة…
mail-context-go-reply = الانتقال إلى الرد
mail-context-go-forward = الانتقال إلى إعادة التوجيه
mail-context-reply-not-found = لم يُعثر على الرد
mail-context-forward-not-found = لم يُعثر على إعادة التوجيه
mail-no-recipients = (لا مستلمين)
mail-attachment-downloaded = تم تنزيل { $name }
mail-attachment-download-failed = فشل التنزيل.
mail-attachments-zip-done = تم تنزيل { $count } مرفقًا بصيغة ZIP.
mail-attachments-zip-capped = يحتوي ملف ZIP على أول { $count } مرفقًا؛ الحد هو { $max } ملفًا أو { $size }.
mail-focus-not-found = لم يُعثر على الرسالة في { $folder }
mail-focus-go-to = الانتقال إلى { $folder }
mail-nav-views-queues = العروض وقوائم الانتظار
mail-badge-unread = { $unread ->
    [zero] لا رسائل غير مقروءة
    [one] رسالة واحدة غير مقروءة
    [two] رسالتان غير مقروءتين
    [few] { $unread } رسائل غير مقروءة
    [many] { $unread } رسالة غير مقروءة
   *[other] { $unread } رسالة غير مقروءة
  }
mail-badge-total = { $total ->
    [zero] لا رسائل في هذا المجلد
    [one] رسالة واحدة في هذا المجلد
    [two] رسالتان في هذا المجلد
    [few] { $total } رسائل في هذا المجلد
    [many] { $total } رسالة في هذا المجلد
   *[other] { $total } رسالة في هذا المجلد
  }
mail-badge-both = { $unread ->
    [zero] لا رسائل غير مقروءة
    [one] رسالة واحدة غير مقروءة
    [two] رسالتان غير مقروءتين
    [few] { $unread } رسائل غير مقروءة
    [many] { $unread } رسالة غير مقروءة
   *[other] { $unread } رسالة غير مقروءة
  } من أصل { $total ->
    [zero] لا شيء في هذا المجلد
    [one] واحدة في هذا المجلد
    [two] اثنتين في هذا المجلد
    [few] { $total } في هذا المجلد
    [many] { $total } في هذا المجلد
   *[other] { $total } في هذا المجلد
  }
mail-badge-drafts = { $total ->
    [zero] المسودات: لا رسائل
    [one] المسودات: رسالة واحدة
    [two] المسودات: رسالتان
    [few] المسودات: { $total } رسائل
    [many] المسودات: { $total } رسالة
   *[other] المسودات: { $total } رسالة
  }
mail-badge-compact-hint = تستخدم المجلدات غير صندوق الوارد شارة أصغر. غيّر ذلك من الإعدادات، ثم المجلدات والشريط الجانبي.
mail-badge-outbox = { $count ->
    [zero] صندوق الصادر: لا رسائل بانتظار الإرسال
    [one] صندوق الصادر: رسالة واحدة بانتظار الإرسال
    [two] صندوق الصادر: رسالتان بانتظار الإرسال
    [few] صندوق الصادر: { $count } رسائل بانتظار الإرسال
    [many] صندوق الصادر: { $count } رسالة بانتظار الإرسال
   *[other] صندوق الصادر: { $count } رسالة بانتظار الإرسال
  }
mail-color-menu = ألوان الرسالة
mail-color-button = ألوان الرسالة: { $mode }
mail-color-use-default = استخدام الافتراضي الخاص بي ({ $mode })
mail-color-make-default = جعل هذا الافتراضي الخاص بي
mail-color-locked = مقفل من المسؤول
mail-color-confirm-title = تغيير ألوان الرسائل الافتراضية؟
mail-color-confirm-body = ستستخدم كل رسالة «{ $mode }» ما لم تغيّره لرسالة واحدة.
mail-color-confirm = جعله الافتراضي
mail-color-default-failed = تعذّر حفظ ألوان الرسائل الافتراضية.

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
mail-attachments-none = الرسائل المحددة لا تحتوي على مرفقات للتنزيل.

mail-attachments-zip-all-blocked = لم يتم تنزيل أي شيء: سياسة المرفقات تحظر جميع الملفات التي يبلغ عددها { $skipped }.

mail-attachments-zip-gathering = تجميع مرفقات { $count } الرسائل...

mail-attachments-zip-skipped = { $skipped ->
        [zero] تم استبعاد { $skipped } ملفات محظورة بواسطة سياسة المرفقات.
        [one] تم استبعاد ملف واحد تم حظره بواسطة سياسة المرفقات.
        [two] تم استبعاد ملفين حظرتهما سياسة المرفقات.
        [few] تم استبعاد { $skipped } ملفات تم حظرها بواسطة سياسة المرفقات.
        [many] تم استبعاد { $skipped } ملفًا تم حظرها بواسطة سياسة المرفقات.
       *[other] تم استبعاد { $skipped } ملف تم حظرها بواسطة سياسة المرفقات.
    }

mail-attachments-zip-unreadable = تعذرت قراءة مرفقات رسائل { $count } وتم استبعادها.

mail-bulk-download-attachments = تنزيل المرفقات بصيغة ZIP

mail-compose-from-via = لقد تطابق مع رأس { $header }.

mail-compose-from-why-match-domain = يتم الإرسال من { $identity } لأنه تم تسليم الرسالة إلى { $address }، وهو عنوان في نفس النطاق.

mail-compose-from-why-match-pattern = يتم الإرسال من { $identity } لأنه تم إرسال الرسالة إلى { $address }، وهو ما يناسب نمط عنوان تلك الهوية.

mail-compose-ribbon-autohide = إخفاء الشريط تلقائيًا (Ctrl+F1)

mail-compose-ribbon-pin = إظهار الشريط دائمًا (Ctrl+F1)

mail-context-download-selected-attachments = { $count ->
        [zero] قم بتنزيل مرفقات { $count } رسالة محددة كـ ZIP
        [one] قم بتنزيل مرفقات الرسالة المحددة كـ ZIP
        [two] تنزيل مرفقات الرسالتين المحددتين بصيغة ZIP
        [few] قم بتنزيل مرفقات { $count } رسائل مختارة كـ ZIP
        [many] قم بتنزيل مرفقات { $count } رسالة مختارة كـ ZIP
       *[other] قم بتنزيل مرفقات { $count } رسالة محددة كـ ZIP
    }

mail-context-download-thread-attachments = قم بتنزيل جميع { $count } مرفقات المحادثة كـ ZIP

mail-context-open-new-tab = افتح في علامة تبويب جديدة

mail-delete-sender-confirm = انقل { $count } الرسائل إلى سلة المهملات

mail-delete-sender-detail = هل تريد نقل { $count } الرسائل من { $sender } إلى سلة المهملات؟ يتم استبعاد الرسائل الموجودة بالفعل في سلة المهملات. يمكنك التراجع عن هذا الإجراء.

mail-delete-sender-empty = لا توجد رسائل من هذا المرسل خارج سلة المهملات.

mail-delete-sender-failed = تعذر التحقق من كل رسالة مطابقة وأذوناتها. لم يتم نقل أي شيء. حاول ثانية.

mail-delete-sender-help = ابحث عن رسائل هذا المرسل في جميع المجلدات وانقلها إلى سلة المهملات بعد التأكيد.

mail-delete-sender-title = حذف كافة الرسائل من هذا المرسل

mail-draft-continue = مواصلة التحرير

mail-draft-edit = تحرير المسودة

mail-draft-edit-failed = لا يمكن فتح المسودة للتحرير

mail-draft-edit-help = افتح هذه المسودة في الملحن لمواصلة تحريرها

mail-draft-notice = هذه مسودة

mail-file-copied = تم النسخ إلى الحافظة.

mail-file-copy-failed = تعذر النسخ إلى الحافظة.

mail-file-copy-fingerprint = نسخ بصمة الإصبع

mail-file-copy-name = انسخ اسم الملف

mail-file-created = تم إنشاؤها

mail-file-creator = تم الإنشاء باستخدام

mail-file-details-button = تفاصيل الملف

mail-file-details-tooltip = إظهار تفاصيل الملف وبصمة الإصبع

mail-file-encryption = حماية كلمة المرور

mail-file-encryption-none = لم يتم طلب كلمة المرور

mail-file-encryption-password = كانت كلمة المرور مطلوبة لفتحه

mail-file-fingerprint = بصمة الإصبع

mail-file-name = اسم الملف

mail-file-pages = الصفحات

mail-file-pdf-version = PDF الإصدار

mail-file-producer = منتج

mail-file-verify-help = قارن هذه المجموعات ببصمة إصبع قدمها لك المرسل بطريقة أخرى، مثل مكالمة أو موقعه على الويب. إذا تطابقت كل مجموعة، فسيكون لديك نفس البايتات تمامًا.

mail-file-verify-title = التحقق من بصمة الإصبع

mail-follow-up-finished = انتهت المتابعة

mail-group-by = تجميع حسب

mail-header-attachments = { $count ->
        [zero] { $count } مرفقات
        [one] { $count } مرفق
        [two] مرفقان
        [few] { $count } مرفقات
        [many] { $count } مرفقات
       *[other] { $count } مرفق
    }

mail-header-expand-short = توسيع رأس الرسالة (Alt+H)

mail-list-group-date-unknown = تاريخ غير معروف

mail-list-group-importance = أهمية

mail-list-group-locked = تم تأمين تجميع الرسائل من قبل المسؤول.

mail-list-group-no-subject = (لا يوجد موضوع)

mail-list-group-none = لا يوجد تجميع

mail-list-group-priority-high = أهمية عالية

mail-list-group-priority-low = أهمية منخفضة

mail-list-group-priority-normal = أهمية عادية

mail-list-group-received = تاريخ الاستلام

mail-list-group-sender = من

mail-list-group-sent = تاريخ الإرسال

mail-list-group-size = الحجم

mail-list-group-size-huge = 10 ميغابايت وما فوق

mail-list-group-size-large = 1-10 ميجابايت

mail-list-group-size-medium = 100 كيلو بايت – 1 ميجابايت

mail-list-group-size-small = 25-100 كيلو بايت

mail-list-group-size-tiny = أقل من 25 كيلو بايت

mail-list-group-subject = الموضوع

mail-list-order-importance = الأهمية، عالية أولاً

mail-list-order-importance-asc = الأهمية، منخفضة أولاً

mail-list-order-locked = تم تأمين ترتيب الرسائل من قبل المسؤول لديك.

mail-list-order-sender-desc = المرسل، Z إلى A

mail-list-order-sent-newest = أرسلت، الأحدث أولا

mail-list-order-sent-oldest = أرسلت، الأقدم أولا

mail-list-order-size = الحجم، الأكبر أولاً

mail-list-order-size-asc = الحجم، الأصغر أولاً

mail-list-order-subject-desc = الموضوع، Z إلى A

mail-mark-answered = وضع علامة كإجابة

mail-mark-as = وضع علامة كـ

mail-mark-finished = وضع علامة على أنه انتهى

mail-mark-forwarded = وضع علامة كمعاد توجيهها

mail-mark-not-forwarded = وضع علامة "لم تتم إعادة توجيهها".

mail-mark-unanswered = وضع علامة "بدون إجابة".

mail-mentions-me = يذكرني

mail-mentions-me-help = ابحث عن الرسائل التي تشير إلى أحد عناوين بريدك الإلكتروني

mail-move-search = مجلدات البحث

mail-order-by = اطلب بواسطة

mail-reader-folder-go = مخزنة في { $folder } · انتقل إلى المجلد

mail-reply-draft-discard = تجاهل الرد

mail-reply-draft-discard-detail = هل تريد إزالة هذا الرد نهائيًا؟ لا يمكن التراجع عن هذا.

mail-reply-draft-discard-title = هل تريد تجاهل هذا الرد؟

mail-status-from-match-domain = من { $identity }

mail-status-from-match-pattern = من { $identity }

mail-status-from-via = الرد من { $identity } لأنه مطابق { $header }

mail-toast-follow-up-finished = تم وضع علامة على المتابعة على أنها منتهية

mail-toast-marked-answered = { $count ->
        [zero] { $count } رسائل تم وضع علامة "تم الرد عليها".
        [one] تم وضع علامة "تم الرد" على رسالة واحدة
        [two] تم وضع علامة الرد على رسالتين
        [few] تم وضع علامة على { $count } رسائل تم الرد عليها
        [many] تم وضع علامة على { $count } رسالة تم الرد عليها
       *[other] تم وضع علامة "تم الرد" على { $count } رسالة
    }

mail-toast-marked-forwarded = { $count ->
        [zero] تم وضع علامة على { $count } رسالة كمعاد توجيهها
        [one] تم وضع علامة على رسالة واحدة كمُعاد توجيهها
        [two] تم وضع علامة إعادة التوجيه على رسالتين
        [few] تم وضع علامة على { $count } رسائل كمعاد توجيهها
        [many] تم وضع علامة على { $count } رسالة كمعاد توجيهها
       *[other] تم وضع علامة على { $count } رسالة كمعاد توجيهها
    }

mail-toast-marked-not-forwarded = { $count ->
        [zero] { $count } رسائل تم وضع علامة "لم تتم إعادة توجيهها".
        [one] تم وضع علامة "لم تتم إعادة توجيه" على رسالة واحدة
        [two] تمت إزالة علامة إعادة التوجيه من رسالتين
        [few] تم وضع علامة على { $count } رسائل بأنها لم تتم إعادة توجيهها
        [many] تم وضع علامة على { $count } رسالة بأنها لم تتم إعادة توجيهها
       *[other] تم وضع علامة "لم تتم إعادة توجيهها" على { $count } رسالة
    }

mail-toast-marked-unanswered = { $count ->
        [zero] { $count } رسائل تم وضع علامة "بدون إجابة".
        [one] تم وضع علامة "لم يتم الرد" على رسالة واحدة
        [two] تمت إزالة علامة الرد من رسالتين
        [few] تم وضع علامة "بدون إجابة" على { $count } رسائل
        [many] تم وضع علامة "بدون إجابة" على { $count } رسالة
       *[other] تم وضع علامة "بدون إجابة" على { $count } رسالة
    }

mail-toast-move-failed-reason = تعذر نقل الرسائل ({ $reason }). لم يتم نقل أي شيء.

mail-toast-update-refused-reason = { $count ->
        [zero] رفض الخادم النقل لـ { $count } رسالة: { $reason }
        [one] رفض الخادم النقل لرسالة واحدة: { $reason }
        [two] رفض الخادم نقل رسالتين: { $reason }
        [few] رفض السيرفر النقل لـ { $count } رسائل: { $reason }
        [many] رفض الخادم النقل لـ { $count } رسالة: { $reason }
       *[other] رفض الخادم النقل لـ { $count } رسالة: { $reason }
    }

reader-badge-hide = إخفاء هذه الشارة

reader-badge-locked = تم تعيينه من قبل المسؤول الخاص بك

reader-badge-menu = شارة الرسالة

reader-badge-notable = تظهر فقط عندما تكون ملحوظة

reader-sanitized-always = يظل المحتوى النشط مثل البرامج النصية معطلاً.

reader-sanitized-clean = الرسالة HTML لا تحتاج إلى أي تغييرات

reader-sanitized-clean-detail = لم تتم إزالة أو حظر أي شيء من HTML لهذه الرسالة.

reader-sanitized-counts = { $blocked } العناصر البعيدة المحظورة، { $trackers } التتبع العناصر.

reader-sanitized-detail = تم تعقيم الرسالة HTML قبل ظهورها. المحتوى البعيد يتبع إعدادات الخصوصية الخاصة بك.

reader-sanitized-hosts = المضيفين عن بعد

reader-sanitized-remote = تم حظر المحتوى البعيد

reader-sanitized-short = معقمة HTML

reader-sanitized-short-blocked = HTML محظور

reader-sanitized-short-clean = HTML نظيف

reader-sanitized-trackers = تتبع العناصر

mail-draft-view = عرض المسودة
mail-reply-draft-view = عرض الرد
