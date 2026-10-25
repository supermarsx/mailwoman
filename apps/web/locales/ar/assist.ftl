# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = مساعد الذكاء الاصطناعي
assist-heading = مساعد الذكاء الاصطناعي
assist-transcript-label = محادثة مساعد الذكاء الاصطناعي
assist-proposed-action = إجراء مقترح
assist-outbox-note = سيضع هذا رسالة في صندوق الصادر. لا يُرسل شيء حتى تؤكده هناك.
assist-review-outbox = المراجعة في صندوق الصادر
assist-review = مراجعة
assist-empty = اطلب من مساعد الذكاء الاصطناعي التلخيص أو الصياغة أو البحث عن أشياء. يمكنه اقتراح إجراءات، لكنك تؤكدها بنفسك دائمًا.
assist-error = تعذّر على مساعد الذكاء الاصطناعي الرد الآن.
assist-input-label = مراسلة مساعد الذكاء الاصطناعي
assist-input-placeholder = اسأل مساعد الذكاء الاصطناعي…
assist-thinking = جارٍ التفكير…
assist-ask = اسأل

# -- Composer tools ---------------------------------------------------------
assist-composer-label = مساعدة في الكتابة
assist-composer-toolbar = أدوات المحرّر
assist-tool-grammar = تصحيح القواعد
assist-tool-rewrite = إعادة الصياغة
assist-tool-tone = ضبط النبرة
assist-tool-translate = ترجمة
assist-toolarg-tone = النبرة
assist-toolarg-translate = اللغة
assist-tone-neutral = محايدة
assist-tone-friendly = ودّية
assist-tone-formal = رسمية
assist-tone-concise = موجزة
assist-tone-assertive = حازمة
assist-busy = { $label }…
assist-composer-empty-error = اكتب شيئًا أولًا.
assist-composer-error = تعذّر على مساعد الذكاء الاصطناعي إكمال ذلك الآن.
assist-suggested-edit = تعديل مقترح
assist-apply-draft = تطبيق على المسودة
assist-discard = تجاهل

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = تصنيفات مقترحة
assist-apply-auto = التطبيق تلقائيًا
assist-apply-label = تطبيق { $label }
assist-remove-label = إزالة { $label }
assist-apply = تطبيق
assist-undo = تراجع

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = اضغط مطولًا للإملاء
assist-dictate-stop = إيقاف الإملاء
assist-dictate-hold-text = اضغط مطولًا للإملاء
assist-dictate-listening = ● جارٍ الاستماع…
assist-dictate-endpoint-note = يُحوَّل الصوت إلى نص بواسطة نقطة نهاية مساعد الذكاء الاصطناعي الخاصة بك.
assist-dictate-err = خطأ في الإملاء.
assist-transcribe-err = تعذّر تحويل الصوت إلى نص.
assist-mic-err = الميكروفون غير متاح.

# -- Semantic search --------------------------------------------------------
assist-semantic-label = البحث الدلالي
assist-semantic-note = يُرسل نص الاستعلام إلى نقطة نهاية مساعد الذكاء الاصطناعي الخاصة بك للترتيب حسب المعنى.

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = ما الذي يمكن أن يغادر هذا الجهاز
assist-left-1 = نص الرسالة المحددة (الموضوع + المتن) للقدرة المختارة
assist-left-2 = مضيف نقطة النهاية الذي أُرسل إليه
assist-left-3 = أبدًا: المحتوى المشفّر من طرف إلى طرف (مستبعد افتراضيًا)
assist-left-4 = أبدًا: المرفقات (مستبعدة افتراضيًا)
assist-left-5 = أبدًا: بيانات اعتمادك أو حساباتك الأخرى
assist-disclosure-off = مساعد الذكاء الاصطناعي معطّل. لا يغادر أي محتوى رسائل هذا الجهاز.
assist-disclosure-sentence = عند استخدام أداة من أدوات مساعد الذكاء الاصطناعي، يُمرَّر نص الرسالة المحددة إلى { $host }. { $excl } الإرسال ليس آليًا أبدًا — فأنت تؤكد دائمًا قبل أن يغادر أي شيء صندوق الصادر.
assist-disclosure-excl-one = لا يرسل أبدًا { $a }.
assist-disclosure-excl-two = لا يرسل أبدًا { $a } أو { $b }.
assist-disclosure-admin-allowed = سمح المسؤول بإرسال المحتوى المشفّر والمرفقات.
assist-excluded-e2ee = المحتوى المشفّر من طرف إلى طرف
assist-excluded-attachments = المرفقات

# -- Admin → AI assistant (deployment configuration) ------------------------------
assist-admin-title = مساعد الذكاء الاصطناعي
assist-admin-intro = يرسل مساعد الذكاء الاصطناعي نص الرسائل الذي يحدده المستخدمون إلى نقطة نهاية الذكاء الاصطناعي المُعدّة هنا. لا يمكنه إرسال البريد أو حذفه أو قبوله. يُحجب المحتوى المشفّر من طرف إلى طرف والمرفقات ما لم تسمح بها أدناه.
assist-admin-load-unreadable = تتعذّر قراءة إعدادات مساعد الذكاء الاصطناعي المخزّنة: مفتاح API الخاص بها لا يُفتح بمفتاح هذا الخادم (MW_SERVER_KEY). حُفظت بمفتاح آخر أو تم تغييرها. مساعد الذكاء الاصطناعي معطّل. املأ النموذج، بما في ذلك مفتاح API، واحفظه لاستبدالها.
assist-admin-load-error = تعذّر تحميل إعدادات مساعد الذكاء الاصطناعي. الحفظ معطّل حتى لا تُستبدل الإعدادات المخزّنة.
assist-admin-status-running = مساعد الذكاء الاصطناعي قيد التشغيل. تذهب الطلبات إلى { $host }.
assist-admin-status-running-pending = مساعد الذكاء الاصطناعي يعمل بالإعدادات التي بدأ بها الخادم. تسري التغييرات المحفوظة عند إعادة تشغيل الخادم.
assist-admin-status-off = مساعد الذكاء الاصطناعي معطّل. يُرفض كل طلب إليه.
assist-admin-status-pending = مساعد الذكاء الاصطناعي ليس قيد التشغيل. تسري الإعدادات المحفوظة عند إعادة تشغيل الخادم.
assist-admin-status-local-program-not-allowed = مساعد الذكاء الاصطناعي ليس قيد التشغيل. البرنامج المُعدّ { $program } غير مُدرج في MW_ASSIST_LOCAL_PROGRAMS على الخادم، لذا لم يشغّله الخادم.
assist-admin-status-no-endpoint = مساعد الذكاء الاصطناعي مفعّل لكنه ليس قيد التشغيل لعدم إعداد نقطة نهاية قابلة للاستخدام.
assist-admin-stop = إيقاف مساعد الذكاء الاصطناعي الآن
assist-admin-stop-note = يسري الإيقاف فورًا على جميع المستخدمين ويبقى ساريًا بعد إعادة التشغيل.
assist-admin-resume = تشغيل مساعد الذكاء الاصطناعي
assist-admin-resume-note = إذا بدأ الخادم ومساعد الذكاء الاصطناعي معطّل، فإن تشغيله يسري عند إعادة تشغيل الخادم.
assist-admin-kill-error = تعذّر تغيير حالة تشغيل مساعد الذكاء الاصطناعي.
assist-admin-endpoint = نقطة النهاية
assist-admin-endpoint-note = يتصل الخادم بنقطة النهاية هذه؛ ولا تتصل بها المتصفحات أبدًا. تسري التغييرات عند إعادة تشغيل الخادم.
assist-admin-kind = نوع نقطة النهاية
assist-admin-kind-none = لا شيء
assist-admin-kind-open-ai-compatible = متوافقة مع OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = برنامج محلي
assist-admin-local-program-enable = لكي يشغّل مساعد الذكاء الاصطناعي برنامجًا على هذا الخادم، عيّن MW_ASSIST_LOCAL_PROGRAMS إلى المسار المطلق للبرنامج حيث يُشغَّل الخادم (افصل بين المسارات المتعددة بفواصل)، ثم أعد تشغيل الخادم.
assist-admin-local-program-note = يشغّل الخادم هذا البرنامج لكل طلب إلى مساعد الذكاء الاصطناعي ويمرّر إليه الطلب عبر الإدخال القياسي، بما في ذلك نص الرسالة الذي حدده المستخدم. ويقرأ الرد من الإخراج القياسي للبرنامج. لا يمكن اختيار إلا البرامج المدرجة في MW_ASSIST_LOCAL_PROGRAMS على الخادم.
assist-admin-local-program = البرنامج
assist-admin-local-program-unlisted = { $program } (غير مُدرج)
assist-admin-local-program-unlisted-note = هذا البرنامج غير مُدرج في MW_ASSIST_LOCAL_PROGRAMS، لذا لا يشغّله الخادم ولن يقبله عند الحفظ. اختر برنامجًا مُدرجًا أو نوع نقطة نهاية آخر.
assist-admin-local-program-required = اختر البرنامج المراد تشغيله.
assist-admin-local-args = الوسائط، واحدة في كل سطر
assist-admin-local-args-note = يُمرَّر كل سطر إلى البرنامج كوسيطة واحدة. لا تُستخدم أي صدفة (shell).
assist-admin-base-url = عنوان URL الأساسي
assist-admin-base-url-required = أدخل عنوان URL الأساسي لنقطة النهاية، بدءًا بـ http:// أو https://.
assist-admin-api-key = مفتاح API
assist-admin-api-key-stored = يوجد مفتاح مخزّن. لا يعرضه الخادم مرة أخرى.
assist-admin-api-key-replace = استبدال مفتاح API
assist-admin-api-key-replace-placeholder = اتركه فارغًا للاحتفاظ بالمفتاح المخزّن
assist-admin-api-key-remove = إزالة المفتاح المخزّن
assist-admin-api-key-keep = الاحتفاظ بالمفتاح المخزّن
assist-admin-api-key-remove-pending = ستتم إزالة المفتاح المخزّن عند الحفظ.
assist-admin-chat-model = نموذج المحادثة
assist-admin-embed-model = نموذج التضمين
assist-admin-stt-model = نموذج تحويل الكلام إلى نص
assist-admin-model = النموذج
assist-admin-anthropic-version = إصدار API
assist-admin-max-tokens = الحد الأقصى لرموز الرد
assist-admin-default-placeholder = الإعداد الافتراضي للخادم
assist-admin-grants = القدرات
assist-admin-grants-note = لا يمكن استخدام إلا القدرات المحددة هنا.
assist-admin-cap-summarize = التلخيص
assist-admin-cap-draft = الصياغة وإعادة الصياغة
assist-admin-cap-grammar = القواعد
assist-admin-cap-dictation = الإملاء
assist-admin-cap-search-semantic = البحث الدلالي
assist-admin-cap-auto-tag = التصنيف التلقائي
assist-admin-cap-recap = الخلاصة
assist-admin-cap-assistant = محادثة مساعد الذكاء الاصطناعي
assist-admin-ceilings = حدود البيانات
assist-admin-ceilings-note = تنطبق هذه الحدود على كل طلب، أيًا كانت القدرة.
assist-admin-accounts = معرّفات الحسابات التي يجوز إرسال بريدها، واحد في كل سطر
assist-admin-accounts-note = إذا لم يُدرج أي حساب، فلن يُرسل أي محتوى بريد إلى نقطة النهاية.
assist-admin-folders = معرّفات المجلدات المقصور عليها الإرسال، واحد في كل سطر
assist-admin-folders-note = إذا لم يُدرج أي مجلد، فكل مجلدات الحسابات المدرجة مسموح بها.
assist-admin-allow-e2ee = السماح بإرسال المحتوى المشفّر من طرف إلى طرف
assist-admin-allow-attachments = السماح بإرسال المرفقات
assist-admin-save = حفظ الإعدادات
assist-admin-saved = تم الحفظ.
assist-admin-saved-pending = تم الحفظ. يسري التغيير عند إعادة تشغيل الخادم.
assist-admin-save-error = لم تُحفظ الإعدادات.
assist-admin-save-not-allowed = لم تُحفظ الإعدادات لأن الخادم لا يسمح بهذا البرنامج. لا يمكن استخدام إلا البرامج المدرجة في MW_ASSIST_LOCAL_PROGRAMS.
