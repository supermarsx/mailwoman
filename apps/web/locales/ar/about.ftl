# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: about. Source: ../en/about.ftl

about-tp-label = مكتبات الجهات الخارجية وتراخيصها
about-tp-search = البحث بالاسم أو الإصدار أو الترخيص
about-tp-filter = التصفية حسب الترخيص
about-tp-filter-all = الكل ({ $n })
about-tp-filter-licence = { $licence } ({ $n })
about-tp-showing =
    { $count ->
        [0] لا توجد إدخالات مطابقة.
        [zero] لا توجد إدخالات مطابقة.
        [one] عرض { $shown } من أصل { $total } إدخال.
        [two] عرض { $shown } من أصل { $total } إدخال.
        [few] عرض { $shown } من أصل { $total } إدخالات.
        [many] عرض { $shown } من أصل { $total } إدخالًا.
       *[other] عرض { $shown } من أصل { $total } إدخال.
    }
about-tp-empty = لا يوجد ما يطابق هذا البحث وهذا الترخيص.
about-tp-clear = مسح البحث والتصفية
about-tp-more = عرض المزيد (المتبقي { $n })

about-tp-kind-crate = حزمة Rust (crate)
about-tp-kind-npm = حزمة npm
about-tp-kind-font = خط
about-tp-kind-icons = أيقونات
about-tp-kind-image = صور
about-tp-kind-os = حزمة الصورة

about-tp-group-count = { $title } ({ $n })
about-tp-group-rust = حزم Rust في الخادم وتطبيق الويب والإضافات
about-tp-group-shells = حزم Rust في تطبيقَي سطح المكتب والجوال فقط
about-tp-group-web = حزم npm في تطبيق الويب
about-tp-group-assets = الخطوط والأيقونات والصور
about-tp-group-image = ملفات صورة الخادم
about-tp-group-build = أدوات البناء: تُشغَّل أثناء البناء ولا تُشحن

about-tp-licence = الترخيص:
about-tp-licence-none = غير مُعلَن
about-tp-licence-in-file = انظر ملف حقوق النشر
about-tp-used-in = مستخدم في: { $parts }
about-tp-artifact-server = الخادم
about-tp-artifact-web = تطبيق الويب
about-tp-artifact-plugins = الإضافات
about-tp-artifact-desktop = تطبيق سطح المكتب
about-tp-artifact-mobile = تطبيق الجوال
about-tp-note-bundled = مضمَّن مع تطبيق الويب.
about-tp-note-fonts-pull = غير مضمَّن: يُنزّله مشغّل الخادم باستخدام `mailwoman fonts pull`.
about-tp-note-first-party = صُنع لـ Mailwoman، بموجب ترخيص المشروع نفسه.
about-tp-note-static-libc = مربوط بالملفات التنفيذية للخادم في صورة FROM-scratch.
about-tp-note-alpine-ca = حزمة شهادات CA في صورة FROM-scratch، مأخوذة من Alpine Linux.
about-tp-in-images = في: { $images }
about-tp-image-runtime = صورة التشغيل distroless
about-tp-image-static = صورة FROM-scratch

about-tp-repository = الشيفرة المصدرية
about-tp-homepage = الصفحة الرئيسية
about-tp-new-tab = لـ { $name } (يُفتح في علامة تبويب جديدة)

about-tp-text-toggle =
    { $count ->
        [zero] نصوص الترخيص ({ $count })
        [one] نص الترخيص
        [two] نصّا الترخيص ({ $count })
        [few] نصوص الترخيص ({ $count })
        [many] نصوص الترخيص ({ $count })
       *[other] نصوص الترخيص ({ $count })
    }
about-tp-text-open-for = نص الترخيص لـ { $name }
about-tp-dialog-title = نص الترخيص: { $name } { $version }
about-tp-close = إغلاق
about-tp-text-loading = جارٍ تحميل نص الترخيص…
about-tp-text-error = تعذّر تحميل نصوص التراخيص.
about-tp-retry = إعادة المحاولة
about-tp-text-missing = هذا النص مفقود من القائمة. أعد توليده باستخدام scripts/about/gen-third-party.mjs.
about-tp-text-label = نص ترخيص { $name }: { $file }
about-tp-standard = ترخيص { $licence }
about-tp-standard-note = (نص قياسي، لا يوجد ملف في الحزمة)
about-tp-no-text = لا يتوفر نص ترخيص لهذا الإدخال.
