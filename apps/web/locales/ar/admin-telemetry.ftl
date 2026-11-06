# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-telemetry. Source: ../en/admin-telemetry.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-telemetry-title = قياسات التشغيل عن بُعد
admin-telemetry-help = صدّر تتبعات ومقاييس التشغيل إلى مجمِّع متوافق مع OpenTelemetry، مثل OpenTelemetry Collector المستضاف ذاتيًا. التصدير متوقف افتراضيًا. تسري التغييرات المحفوظة بعد إعادة تشغيل الخادم.
admin-telemetry-back = العودة إلى قابلية الملاحظة
admin-telemetry-saved = تم حفظ إعدادات القياس عن بُعد. أعد تشغيل الخادم لتطبيق التغييرات.
admin-telemetry-restart = تختلف الإعدادات المحفوظة عن الخادم العامل. يلزم إعادة التشغيل.
admin-telemetry-reload = إعادة تحميل الإعدادات
admin-telemetry-save = حفظ إعدادات القياس عن بُعد
admin-telemetry-collector = المجمِّع والإشارات
admin-telemetry-collector-help = لا يستطيع اختيار هذه الوجهة على مستوى الخادم سوى المسؤول. استخدم مجمِّعًا خاصًا أو HTTPS؛ فـ HTTP ينقل القياسات دون تشفير.
admin-telemetry-enabled = تفعيل تصدير OTLP
admin-telemetry-traces = تصدير التتبعات
admin-telemetry-metrics = تصدير المقاييس
admin-telemetry-endpoint = نقطة نهاية OTLP gRPC
admin-telemetry-endpoint-help = عنوان مجمِّع HTTP أو HTTPS مطلق، وعادةً على المنفذ 4317. لا تضمّن بيانات اعتماد أو استعلامًا أو جزءًا. هذه نقطة نهاية gRPC، وليست منفذ OTLP HTTP رقم 4318.
admin-telemetry-serviceName = اسم الخدمة
admin-telemetry-serviceName-help = تسمية ثابتة للخدمة تُستخدم لتعريف نشر Mailwoman هذا في مجمِّعك. لا تستخدم عنوان بريد إلكتروني أو معلومات شخصية أخرى.
admin-telemetry-environment = البيئة
admin-telemetry-environment-help = تسمية نشر اختيارية مثل production أو staging. تُرسل مع القياسات المصدَّرة.
admin-telemetry-pinned = الحقول المعطّلة تتحكم فيها متغيرات بيئة النشر. غيّر تلك المتغيرات لتجاوز قيمها.
admin-telemetry-limits = حدود أخذ العينات والتصدير
admin-telemetry-limits-help = تحدّ من الذاكرة وحركة المرور إلى المجمِّع. لا يمكن أن يتجاوز حجم الدفعة حجم الطابور. قد تُسقط الطوابير الممتلئة نطاقات (spans) بدلًا من حجب طلبات البريد.
admin-telemetry-sampleRatio = نسبة أخذ عينات التتبع
admin-telemetry-sampleRatio-help = جزء من التتبعات الجذرية الجديدة الذي يُحتفظ به، من 0 إلى 1. تتبع النطاقات الفرعية قرار أخذ العينات للأصل.
admin-telemetry-exportTimeoutMs = مهلة التصدير (مللي ثانية)
admin-telemetry-exportTimeoutMs-help = أقصى مدة لطلب تصدير إلى المجمِّع، من 100 إلى 30000 مللي ثانية.
admin-telemetry-metricsIntervalSecs = فاصل المقاييس (ثوانٍ)
admin-telemetry-metricsIntervalSecs-help = الوقت بين عمليات تصدير المقاييس، من 10 إلى 3600 ثانية.
admin-telemetry-spanQueueSize = حجم طابور النطاقات
admin-telemetry-spanQueueSize-help = أقصى عدد من النطاقات المنتظرة للتصدير، من 128 إلى 32768.
admin-telemetry-spanBatchSize = حجم دفعة النطاقات
admin-telemetry-spanBatchSize-help = أقصى عدد من النطاقات في تصدير واحد، من 1 إلى 4096 وبما لا يزيد على حجم الطابور.
admin-telemetry-spanExportDelayMs = تأخير دفعة النطاقات (مللي ثانية)
admin-telemetry-spanExportDelayMs-help = الانتظار المعتاد بين دفعات النطاقات، من 100 إلى 60000 مللي ثانية.
admin-telemetry-prometheus-help = نقطة نهاية مصادَق عليها منفصلة لأداة كشط متوافقة مع Prometheus. يتطلب الوصول دائمًا رمز حامل MW_METRICS_TOKEN الخاص بالنشر؛ ولا تُعاد قيمته إلى هذه الصفحة أبدًا.
admin-telemetry-prometheusEnabled = تفعيل كشط Prometheus
admin-telemetry-token-missing = لم يُضبط أي رمز كشط. تبقى نقطة نهاية المقاييس غير متاحة إلى أن يضبط المشغّل MW_METRICS_TOKEN.
