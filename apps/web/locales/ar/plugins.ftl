# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = تحذير من إضافة واجهة غير موقّعة
plugins-unsigned-title = توجد إضافات غير موقّعة قيد التشغيل
plugins-unsigned-text = قُبلت هذه الإضافات دون توقيع تم التحقق منه. تعمل في بيئة معزولة، لكن لم يُتحقق من شيفرتها مقابل مفتاح موثوق.

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = إضافة: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = تعمل إضافات المحرك في بيئة WebAssembly معزولة. لا تعمل الإضافة إلا بعد تسجيلها والموافقة عليها ومنحها قدرة واحدة على الأقل وتفعيلها. ولا يمكنها استخدام إلا القدرات الممنوحة هنا.
plugins-admin-unsigned-banner = تم تحميل إضافة من جهة خارجية بلا توقيع. تعمل لأن مسؤولًا سمح بها؛ ويتحقق الخادم من ملفها مقابل البصمة الموافق عليها فقط.
plugins-admin-load-error = تعذّر تحميل الإضافات.
plugins-admin-register-heading = تسجيل إضافة
plugins-admin-register-component = المكوّن
plugins-admin-register-third-party = مكوّن من جهة خارجية
plugins-admin-register-id = معرّف الإضافة
plugins-admin-register-id-hint = يقرأ الخادم المكوّن من الملف المسمّى بهذا المعرّف في دليل إضافات الجهات الخارجية. وافق أولًا على بصمة ذلك الملف في قائمة السماح أدناه.
plugins-admin-register-name = الاسم
plugins-admin-register-version = الإصدار
plugins-admin-register-capabilities = القدرات التي تعلنها الإضافة
plugins-admin-register-hosts = المضيفون الذين يمكن للإضافة الاتصال بهم
plugins-admin-register-hosts-hint-first-party = أسماء مضيفين مفصولة بفواصل. اتركه فارغًا لاستخدام المضيفين الافتراضيين للمكوّن.
plugins-admin-register-hosts-hint-third-party = أسماء مضيفين مفصولة بفواصل. اتركه فارغًا لعدم السماح بالوصول إلى الشبكة.
plugins-admin-register-submit = تسجيل
plugins-admin-trust-first-party = أساسية، البصمة مضمّنة في البناء
plugins-admin-trust-third-party = جهة خارجية، البصمة موافق عليها
plugins-admin-status-loaded = محمّلة
plugins-admin-status-not-loaded = غير محمّلة
plugins-admin-status-restart = تلزم إعادة التشغيل
plugins-admin-restart-to-load = تلزم إعادة التشغيل لتحميل هذه الإضافة.
plugins-admin-restart-to-apply = تلزم إعادة التشغيل لتطبيق هذه التغييرات. لن تُخدم بعض حسابات هذا الجسر حتى ذلك الحين.
plugins-admin-reason-not-approved = غير محمّلة: لم يوافق أي مسؤول على هذه الإضافة.
plugins-admin-reason-disabled = غير محمّلة: الإضافة غير مفعّلة.
plugins-admin-reason-unsigned-not-allowed = غير محمّلة: الإضافة بلا توقيع ولم يُسمح لها بالعمل بدونه.
plugins-admin-reason-no-grant = غير محمّلة: لم تُمنح أي قدرة تحتاجها.
plugins-admin-reason-component-unavailable = غير محمّلة: لم يجتز أي ملف مكوّن التحقق من البصمة.
plugins-admin-reason-load-failed = غير محمّلة: تعذّر على الخادم تحميل المكوّن. راجع سجل الخادم.
plugins-admin-reason-proxy-mode = غير محمّلة: يعمل هذا الخادم في وضع الوكيل ولا يعالج البريد بنفسه.
plugins-admin-reason-no-account-binding = غير محمّلة: لا يوجد حساب مرتبط بهذا الجسر.
plugins-admin-reason-no-host-caller = غير محمّلة: إصدار الخادم هذا لا يستدعي الخطافات التي توفرها هذه الإضافة.
plugins-admin-reason-another-classifier-active = غير محمّلة: مصنّف بريد عشوائي آخر محمّل. يعمل مصنّف واحد فقط في كل مرة.
plugins-admin-running-with = تعمل بالقدرات: { $capabilities }
plugins-admin-hosts = المضيفون: { $hosts }
plugins-admin-hosts-none = المضيفون: لا يوجد
plugins-admin-grants-heading = القدرات الممنوحة
plugins-admin-grants-hint = لا يمكن للإضافة استخدام إلا القدرات المحددة هنا. الحفظ يستبدل المنح السابق.
plugins-admin-grant-for = منح { $capability } لـ { $name }
plugins-admin-grants-save = حفظ المنح
plugins-admin-endpoint = عنوان المصنّف
plugins-admin-endpoint-for = عنوان المصنّف لـ { $name }
plugins-admin-endpoint-hint = المكان الذي تصل منه الإضافة إلى خدمتها. يجب أن يكون المضيف أيضًا في قائمة مضيفي الإضافة. اتركه فارغًا لاستخدام القيمة الافتراضية للمكوّن.
plugins-admin-endpoint-save = حفظ العنوان
plugins-admin-test = اختبار المصنّف
plugins-admin-test-verdict = نتيجة الاختبار: { $verdict }
plugins-admin-uninstall = إلغاء التثبيت
plugins-admin-uninstall-confirm = تأكيد إلغاء التثبيت
plugins-admin-uninstall-note = إلغاء التثبيت يزيل التسجيل والمنح والإعدادات المخزّنة.
plugins-admin-uninstall-detail = إلغاء التثبيت يحذف البيانات المخزّنة للإضافة لكل الحسابات، وتثبيتاتها في قائمة السماح، وتسجيلها ومنحها. لا يُحذف ملف المكوّن على القرص.
plugins-admin-revoke-detail = الإبطال يزيل الموافقة على هذه البصمة ويعطّل الإضافة. إذا كانت الإضافة قيد التشغيل، فتتوقف فورًا.
plugins-admin-error-already-registered = هذه الإضافة مسجّلة بالفعل. ألغِ تثبيتها قبل تسجيلها مرة أخرى.
plugins-admin-error-component-unavailable = لا يوجد ملف مكوّن لهذه الإضافة يطابق البصمة المضمّنة في الخادم.
plugins-admin-error-digest-not-approved = لم يُعثر على ملف مكوّن ببصمة موافق عليها لهذا المعرّف. وافق أولًا على بصمة الملف في قائمة السماح.
plugins-admin-error-not-approved = وافق على الإضافة قبل تفعيلها.
plugins-admin-error-unsigned-not-allowed = هذه الإضافة بلا توقيع. اسمح لها بالعمل دون توقيع قبل تفعيلها.
plugins-admin-error-not-loaded = الإضافة غير محمّلة، لذا لا يمكن اختبارها.
plugins-admin-error-classifier-error = فشل استدعاء المصنّف: { $detail }
plugins-admin-error-other = رفض الخادم الطلب: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = إضافات الواجهة
plugins-admin-ui-title = إضافات الواجهة
plugins-admin-ui-intro = تضيف إضافات الواجهة عناصر تحكم إلى عميل الويب. تعمل كل منها في إطار معزول ولا يمكنها استخدام إلا القدرات الممنوحة هنا. تُعرض الإضافة على المستخدمين المسجّلين بمجرد الموافقة عليها وتفعيلها.
plugins-admin-ui-empty = لا توجد إضافات واجهة مسجّلة.
plugins-admin-ui-load-error = تعذّر تحميل إضافات الواجهة.
plugins-admin-ui-register-heading = تسجيل إضافة واجهة
plugins-admin-ui-manifest = البيان (JSON)
plugins-admin-ui-bundle = ملف الحزمة
plugins-admin-ui-bundle-hint = مطلوب للإضافة الموقّعة: يُتحقق من التوقيع مقابل هذا الملف.
plugins-admin-ui-allow-unsigned = السماح بتسجيل هذه الإضافة دون توقيع
plugins-admin-ui-register = تسجيل
plugins-admin-ui-manifest-invalid = البيان ليس JSON صالحًا.
plugins-admin-ui-refused = رفض الخادم الطلب: { $detail }
plugins-admin-ui-approve = الموافقة والتفعيل
plugins-admin-ui-extension-points = نقاط التوسعة: { $points }
plugins-admin-ui-grants-heading = منح قدرة
plugins-admin-ui-grants-note = لا يُبلغ الخادم عن القدرات الممنوحة بالفعل. منح قدرة مرة أخرى يستبدل منحها السابق.
plugins-admin-ui-grant = منح { $capability }
plugins-admin-ui-grant-for = منح { $capability } لـ { $name }
plugins-admin-ui-grant-hosts = المضيفون لـ { $capability } في { $name }
plugins-admin-ui-grant-hosts-hint = أسماء المضيفين التي يمكن للإضافة الجلب منها، مفصولة بفواصل.
plugins-admin-ui-granted = تم منح { $capability }.
plugins-admin-ui-delete = حذف
plugins-admin-ui-delete-confirm = تأكيد الحذف
