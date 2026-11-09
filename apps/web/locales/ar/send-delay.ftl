# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = التراجع عن الإرسال
send-delay-help = بعد الضغط على إرسال، تنتظر الرسالة في صندوق الصادر طوال هذه المدة. وحتى ذلك الحين يمكنك استرجاعها ومتابعة التعديل. يحتفظ الخادم بها، لذا فإن إغلاق المتصفح لا يرسلها مبكرًا ولا يفقدها.
send-delay-label = احتجاز الرسائل المرسلة لمدة
send-delay-off = إيقاف (إرسال فوري)
send-delay-seconds = { $n } ثانية
send-delay-minutes = { $n ->
    [zero] { $n } دقيقة
    [one] دقيقة واحدة
    [two] دقيقتان
    [few] { $n } دقائق
    [many] { $n } دقيقة
   *[other] { $n } دقيقة
}
send-delay-custom = مخصص…
send-delay-custom-label = مدة تأخير مخصصة بالثواني
send-delay-custom-help = ثوانٍ كاملة، من 0 إلى { $max }.
send-delay-capped = يحدّ المسؤول هذا بالقيمة { $max }.
send-delay-disabled = عطّل المسؤول استرجاع الرسائل المرسلة. تُرسل الرسائل فورًا.
send-delay-invalid = أدخل عددًا صحيحًا من الثواني بين 0 و{ $max }.
send-later-button = إرسال لاحقًا
send-later-menu = خيارات الإرسال لاحقًا
send-later-preset-hour = بعد ساعة
send-later-preset-today = لاحقًا اليوم، 17:00
send-later-preset-tomorrow = صباح الغد، 09:00
send-later-preset-monday = صباح الاثنين، 09:00
send-later-custom = اختيار التاريخ والوقت…
send-later-custom-label = الإرسال في
send-later-zone = المنطقة الزمنية: { $zone }
send-later-schedule = جدولة الإرسال
send-later-cancel = إلغاء
send-later-err-invalid = أدخل تاريخًا ووقتًا صالحين.
send-later-err-past = اختر وقتًا في المستقبل.
send-later-err-horizon = هذا بعيد جدًا. يمكن جدولة الرسائل قبل { $days } يومًا كحد أقصى.
send-later-err-off = عطّل المسؤول الإرسال المجدول.
send-queued = الرسالة في قائمة الانتظار — تُرسل خلال
send-queued-undo = تراجع
send-queued-now = إرسال الآن
send-queued-recalled = تم استرجاع الرسالة. وهي الآن في مسوداتك.
send-queued-too-late = فات أوان الاسترجاع: الرسالة قيد الإرسال بالفعل.
send-queued-sent-now = جارٍ الإرسال الآن.
send-queued-scheduled = مجدولة في { $when } ({ $zone }).
outbox-scheduled-badge = مجدولة
outbox-sends-in = تُرسل خلال { $time }
outbox-sends-at = تُرسل { $when }
outbox-recall-edit = استرجاع وتعديل
outbox-recall-too-late = قيد الإرسال بالفعل؛ لم يعد استرجاعها ممكنًا.
outbox-failed-drafts = تعذّر الإرسال وتم الاحتفاظ بالرسالة في مسوداتك.
outbox-queued-count = { $count ->
    [zero] لا توجد رسائل بانتظار الإرسال
    [one] رسالة واحدة بانتظار الإرسال
    [two] رسالتان بانتظار الإرسال
    [few] { $count } رسائل بانتظار الإرسال
    [many] { $count } رسالة بانتظار الإرسال
   *[other] { $count } رسالة بانتظار الإرسال
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = الإرسال التالي: { $when }

outbox-chip-title = رسائل في انتظار الإرسال

outbox-chip-view-all = افتح صندوق الصادر

outbox-delete = حذف

outbox-delete-confirm = هل تريد حذف الرسالة؟

outbox-delete-failed = لا يمكن حذف الرسالة. { $error }

outbox-deleted = تم حذف الرسالة الموضوعة في قائمة الانتظار.

outbox-edit-time = تعديل وقت الإرسال

outbox-edit-time-conflict = تم تغيير وقت الإرسال أو تم إرسال الرسالة بالفعل. تم إعادة تحميل علبة الصادر.

outbox-edit-time-current = حاليا { $when }

outbox-edit-time-refused = لا يمكن تغيير وقت الإرسال. { $error }

outbox-edit-time-save = حفظ وقت الإرسال

outbox-failed-toast = لا يمكن إرسال الرسالة. { $error }

outbox-failed-uncertain-toast = ربما تم تسليم الرسالة أو لم يتم تسليمها. تحقق من الإرسال قبل إرساله مرة أخرى.

outbox-open = مفتوح

outbox-reader-actions = إجراءات الرسائل في قائمة الانتظار

outbox-reader-cc = نسخة: { $recipients }

outbox-reader-label = رسالة في قائمة الانتظار

outbox-reader-loading = جارٍ تحميل الرسالة...

outbox-reader-to = إلى: { $recipients }

outbox-reader-unavailable = لا يمكن تحميل الرسالة.

outbox-recall-to-drafts = أذكر للمسودات

outbox-send-again = أرسل مرة أخرى

outbox-send-again-failed = لا يمكن وضع الرسالة في قائمة الانتظار مرة أخرى. { $error }

outbox-send-again-queued = يتم وضع الرسالة في قائمة الانتظار لإرسالها مرة أخرى.

outbox-sent-copy-folder = سيتم الحفظ في { $folder }

outbox-sent-copy-missing = سيتم حفظه في "المرسلة" (المجلد المختار غير متوفر)

outbox-sent-copy-skip = لن يتم حفظ أي نسخة

outbox-status-attempts = { $count ->
        [zero] { $count } محاولات حتى الآن
        [one] { $count } محاولة حتى الآن
        [two] محاولتان حتى الآن
        [few] { $count } محاولات حتى الآن
        [many] { $count } محاولة حتى الآن
       *[other] { $count } محاولة حتى الآن
    }

outbox-status-canceled = تم الإلغاء

outbox-status-failed = لم يتم إرسالها

outbox-status-held = عقد حتى يمكنك الافراج عنه

outbox-status-last-error = الخطأ الأخير: { $error }

outbox-status-next-attempt = المحاولة التالية { $when }

outbox-status-queued = في قائمة الانتظار

outbox-status-retrying = إعادة المحاولة

outbox-status-scheduled = المقرر

outbox-status-sending = إرسال

outbox-status-sends-at-in = يرسل { $when } (في { $time })

outbox-status-sent = تم الإرسال

outbox-status-uncertain = التسليم غير معروف
