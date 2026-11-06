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
