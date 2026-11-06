# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-webhooks. Source: ../en/admin-webhooks.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-webhooks-title = Webhooks
admin-webhooks-intro = يرسل webhook الصادر إشعارًا موقَّعًا إلى عنوان URL عند حدوث شيء في حساب واحد. ويتيح webhook الوارد لنظام آخر تشغيل إجراء قاعدة بريد في حساب واحد.
admin-webhooks-empty = لا توجد webhooks بعد.
admin-webhooks-unavailable = لا يوفّر هذا الخادم إدارة webhooks.
admin-webhooks-load-error = تعذّر تحميل webhooks.
admin-webhooks-direction-outbound = صادر
admin-webhooks-direction-inbound = وارد
admin-webhooks-col-direction = الاتجاه
admin-webhooks-col-label = الاسم
admin-webhooks-col-account = الحساب
admin-webhooks-col-target = يرسل إلى أو يسمح بـ
admin-webhooks-col-created = تاريخ الإنشاء
admin-webhooks-no-label = (بلا اسم)
admin-webhooks-created-by = بواسطة { $who }
admin-webhooks-last-used = آخر استخدام { $when }
admin-webhooks-never-used = لم يُستخدم بعد
admin-webhooks-delete = حذف
admin-webhooks-delete-named = حذف webhook { $name }
admin-webhooks-delete-title = حذف webhook { $name }؟
admin-webhooks-delete-body = تتوقف عمليات التسليم إليه، أو تُرفض الاستدعاءات الموقَّعة بسره. لا يمكن التراجع عن هذا؛ وتحصل webhook جديدة على سر جديد.
admin-webhooks-deleted = تم حذف webhook.
admin-webhooks-add = إضافة webhook
admin-webhooks-add-direction = الاتجاه
admin-webhooks-add-outbound-help = يستدعي Mailwoman عنوان URL الخاص بك.
admin-webhooks-add-inbound-help = يستدعي نظامك Mailwoman.
admin-webhooks-add-account = الحساب
admin-webhooks-add-account-help = الحساب الذي تُرسل أحداثه، أو الحساب الوحيد الذي يجوز للاستدعاء الوارد أن يعمل عليه.
admin-webhooks-add-label = الاسم
admin-webhooks-add-label-help = الغرض من webhook ليتمكن الآخرون من التمييز بينها. اختياري.
admin-webhooks-add-url = URL
admin-webhooks-add-url-help = يجب أن يصل إلى عنوان إنترنت عام. تُرفض العناوين الخاصة وعناوين الحلقة الراجعة وعناوين الربط المحلي.
admin-webhooks-add-events = الأحداث
admin-webhooks-add-actions = الإجراءات المسموح بها
admin-webhooks-add-submit = إنشاء webhook
admin-webhooks-add-saving = جارٍ الإنشاء…
admin-webhooks-add-check = فحص
admin-webhooks-add-check-ok = اجتاز عنوان URL الفحص. لم يُحفظ شيء.
admin-webhooks-event-message-received = رسالة جديدة
admin-webhooks-event-message-flags = رسالة مقروءة أو مميّزة بعلامة أو بتسمية
admin-webhooks-event-message-moved = نُقلت رسالة
admin-webhooks-event-message-deleted = حُذفت رسالة
admin-webhooks-event-submission-sent = أُرسلت رسالة
admin-webhooks-event-submission-failed = تعذّر إرسال رسالة
admin-webhooks-event-pim-changed = تغيّر التقويم أو جهات الاتصال أو المهام أو الملاحظات
admin-webhooks-action-run-rules = تشغيل قواعد بريد الحساب على وصف رسالة
admin-webhooks-action-sieve = تطبيق إجراء قاعدة واحد على رسالة
admin-webhooks-secret-title = سر { $name }
admin-webhooks-secret-once = انسخ هذا السر الآن. لن يُعرض مرة أخرى.
admin-webhooks-secret-outbound = يحمل كل تسليم X-Mailwoman-Signature: sha256= متبوعًا بـ HMAC-SHA256 لمتن الطلب بهذا السر.
admin-webhooks-secret-inbound = أرسل X-Mailwoman-Webhook: { $id } وX-Mailwoman-Timestamp (ثوانٍ بنظام Unix) وX-Mailwoman-Nonce (من 16 إلى 128 حرفًا أو رقمًا أو - أو _، ولا يُعاد استخدامه) وX-Mailwoman-Signature: sha256= متبوعًا بـ HMAC-SHA256 للنص timestamp.nonce.body بهذا السر. تُرفض الاستدعاءات التي يزيد فرقها عن خمس دقائق أو التي تعيد استخدام nonce.
admin-webhooks-secret-done = لقد نسخته
admin-webhooks-error-stale = غيّر أحدهم webhooks منذ فتحت هذه الصفحة. أُعيد تحميل القائمة؛ حاول مرة أخرى.
admin-webhooks-error-account = أدخل الحساب.
admin-webhooks-error-url = أدخل عنوان URL يبدأ بـ https:// أو http://.
admin-webhooks-error-events = اختر حدثًا واحدًا على الأقل.
admin-webhooks-error-actions = اختر إجراءً واحدًا على الأقل.
