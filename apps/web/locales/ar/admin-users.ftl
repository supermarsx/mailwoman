# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = المستخدمون. تتيح عناوين الأعمدة ترتيب القائمة.
admin-userlist-search = البحث عن مستخدمين
admin-userlist-search-placeholder = الاسم أو العنوان أو الاسم المستعار أو النطاق
admin-userlist-count = { $count ->
    [zero] لا مستخدمين
    [one] مستخدم واحد
    [two] مستخدمان
    [few] { $count } مستخدمين
    [many] { $count } مستخدمًا
   *[other] { $count } مستخدم
}
admin-userlist-count-filtered = { $count ->
    [zero] لا يطابق أي مستخدم من أصل { $total }
    [one] يطابق مستخدم واحد من أصل { $total }
    [two] يطابق مستخدمان من أصل { $total }
    [few] يطابق { $count } مستخدمين من أصل { $total }
    [many] يطابق { $count } مستخدمًا من أصل { $total }
   *[other] يطابق { $count } مستخدم من أصل { $total }
}
admin-userlist-range = { $from }–{ $to } من { $total }
admin-userlist-loading = جارٍ تحميل المستخدمين…
admin-userlist-retry = إعادة المحاولة
admin-userlist-empty-filtered = لا يطابق أي مستخدم هذا البحث وهذه المرشحات.
admin-userlist-compact = صفوف مدمجة
admin-userlist-columns = الأعمدة
admin-userlist-columns-legend = إظهار الأعمدة
admin-userlist-actions = الإجراءات
admin-userlist-paging = صفحات المستخدمين
admin-userlist-page-size = الصفوف في الصفحة
admin-userlist-prev = السابق
admin-userlist-next = التالي
admin-userlist-sort-by = الترتيب حسب
admin-userlist-sort-asc = { $column }، تصاعديًا
admin-userlist-sort-desc = { $column }، تنازليًا
admin-userlist-col-user = المستخدم
admin-userlist-col-domain = النطاق
admin-userlist-col-status = الحالة
admin-userlist-col-2fa = المصادقة الثنائية
admin-userlist-col-role = دور وحدة التحكم
admin-userlist-col-last-signin = آخر تسجيل دخول
admin-userlist-col-last-seen = آخر ظهور
admin-userlist-col-created = تاريخ الإنشاء
admin-userlist-col-quota = الحصة
admin-userlist-col-profile = ملف الميزات
admin-userlist-col-linked = صناديق البريد المرتبطة
admin-userlist-state-active = نشط
admin-userlist-state-pending = لم يسجّل الدخول قط
admin-userlist-state-blocked = محظور
admin-userlist-state-disabled = معطّل
admin-userlist-state-deleted = محذوف
admin-userlist-2fa-passkey = مفتاح مرور
admin-userlist-2fa-totp = تطبيق المصادقة
admin-userlist-2fa-none = لا شيء
admin-userlist-signin-never = أبدًا
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = لا يوجد نشاط مسجَّل
admin-userlist-seen-at = آخر ظهور { $relative } ({ $exact })
admin-userlist-quota-unlimited = غير محدودة
admin-userlist-profile-default = الافتراضي
admin-userlist-profile-via-user = معيَّن لهذا المستخدم
admin-userlist-profile-via-group = من مجموعة
admin-userlist-profile-via-domain = من النطاق
admin-userlist-more = مزيد من الإجراءات
admin-userlist-more-for = مزيد من الإجراءات لـ { $account }
admin-userlist-select = تحديد { $account }
admin-userlist-select-page = تحديد كل مستخدمي هذه الصفحة
admin-userlist-filters = المرشحات
admin-userlist-add-filter = إضافة مرشح
admin-userlist-clear-all = مسح الكل
admin-userlist-chip-edit = تعديل المرشح { $label }
admin-userlist-chip-remove = إزالة المرشح { $label }
admin-userlist-filter-domain = النطاق
admin-userlist-filter-status = الحالة
admin-userlist-filter-tfa = المصادقة الثنائية
admin-userlist-filter-role = دور وحدة التحكم
admin-userlist-filter-profile = ملف الميزات
admin-userlist-filter-signin = آخر تسجيل دخول
admin-userlist-filter-created = تاريخ الإنشاء
admin-userlist-filter-no-choices = لا توجد خيارات بعد.
admin-userlist-filter-remove = إزالة المرشح
admin-userlist-apply = تطبيق
admin-userlist-cancel = إلغاء
admin-userlist-role-any = أي دور في وحدة التحكم
admin-userlist-role-none = بلا دور في وحدة التحكم
admin-userlist-profile-none = بلا ملف (افتراضي)
admin-userlist-signin-older = { $days ->
    [zero] قبل أكثر من { $days } يوم
    [one] قبل أكثر من يوم
    [two] قبل أكثر من يومين
    [few] قبل أكثر من { $days } أيام
    [many] قبل أكثر من { $days } يومًا
   *[other] قبل أكثر من { $days } يوم
}
admin-userlist-signin-within = { $days ->
    [zero] خلال آخر { $days } يوم
    [one] خلال آخر يوم
    [two] خلال آخر يومين
    [few] خلال آخر { $days } أيام
    [many] خلال آخر { $days } يومًا
   *[other] خلال آخر { $days } يوم
}
admin-userlist-signin-custom = قبل أكثر من هذا العدد من الأيام:
admin-userlist-signin-days = الأيام
admin-userlist-days-invalid = أدخل عددًا صحيحًا من الأيام بين 1 و36500.
admin-userlist-created-from = من
admin-userlist-created-to = إلى
admin-userlist-created-between = من { $from } إلى { $to }
admin-userlist-created-since = منذ { $date }
admin-userlist-created-until = حتى { $date }
admin-userlist-dates-invalid = تاريخ البداية بعد تاريخ النهاية.
admin-userlist-views = العروض المحفوظة
admin-userlist-view-save = حفظ هذا العرض…
admin-userlist-view-name = اسم العرض
admin-userlist-view-save-confirm = حفظ العرض
admin-userlist-view-delete = حذف العرض "{ $name }"
admin-userlist-bulk = إجراءات على المستخدمين المحدَّدين
admin-userlist-selected = { $count ->
    [zero] لا شيء محدد
    [one] محدد مستخدم واحد
    [two] محدد مستخدمان
    [few] محدد { $count } مستخدمين
    [many] محدد { $count } مستخدمًا
   *[other] محدد { $count } مستخدم
}
admin-userlist-clear-selection = مسح التحديد
admin-userlist-block = حظر تسجيل الدخول
admin-userlist-unblock = إلغاء حظر تسجيل الدخول
admin-userlist-sign-out = تسجيل الخروج من كل مكان
admin-userlist-delete = حذف الوصول المحلي
admin-userlist-bulk-profile = تعيين ملف ميزات
admin-userlist-bulk-profile-clear = إزالة ملف المستخدم الخاص
admin-userlist-confirm-skipped = { $count ->
    [zero] لا مستخدمين محدَّدين متخطّين.
    [one] يُتخطّى مستخدم محدَّد واحد: الإجراء لا ينطبق عليه.
    [two] يُتخطّى مستخدمان محدَّدان: الإجراء لا ينطبق عليهما.
    [few] يُتخطّى { $count } مستخدمين محدَّدين: الإجراء لا ينطبق عليهم.
    [many] يُتخطّى { $count } مستخدمًا محدَّدًا: الإجراء لا ينطبق عليهم.
   *[other] يُتخطّى { $count } مستخدم محدَّد: الإجراء لا ينطبق عليهم.
}
admin-userlist-confirm-block = { $count ->
    [zero] حظر تسجيل الدخول لـ { $count } مستخدم؟
    [one] حظر تسجيل الدخول لمستخدم واحد؟
    [two] حظر تسجيل الدخول لمستخدمَين؟
    [few] حظر تسجيل الدخول لـ { $count } مستخدمين؟
    [many] حظر تسجيل الدخول لـ { $count } مستخدمًا؟
   *[other] حظر تسجيل الدخول لـ { $count } مستخدم؟
}
admin-userlist-confirm-block-one = حظر تسجيل الدخول لـ { $account }؟
admin-userlist-confirm-block-body = تُرفض عمليات تسجيل الدخول الجديدة وتنتهي جلسات Mailwoman الحالية. لا يتغير صندوق البريد الأصلي.
admin-userlist-confirm-block-action = حظر تسجيل الدخول
admin-userlist-confirm-unblock = { $count ->
    [zero] إلغاء حظر تسجيل الدخول لـ { $count } مستخدم؟
    [one] إلغاء حظر تسجيل الدخول لمستخدم واحد؟
    [two] إلغاء حظر تسجيل الدخول لمستخدمَين؟
    [few] إلغاء حظر تسجيل الدخول لـ { $count } مستخدمين؟
    [many] إلغاء حظر تسجيل الدخول لـ { $count } مستخدمًا؟
   *[other] إلغاء حظر تسجيل الدخول لـ { $count } مستخدم؟
}
admin-userlist-confirm-unblock-one = إلغاء حظر تسجيل الدخول لـ { $account }؟
admin-userlist-confirm-unblock-body = يُزال حظر تسجيل الدخول المحلي. تبقى القيود الأمنية الأخرى سارية.
admin-userlist-confirm-unblock-action = إلغاء حظر تسجيل الدخول
admin-userlist-confirm-signout = { $count ->
    [zero] تسجيل خروج { $count } مستخدم من كل مكان؟
    [one] تسجيل خروج مستخدم واحد من كل مكان؟
    [two] تسجيل خروج مستخدمَين من كل مكان؟
    [few] تسجيل خروج { $count } مستخدمين من كل مكان؟
    [many] تسجيل خروج { $count } مستخدمًا من كل مكان؟
   *[other] تسجيل خروج { $count } مستخدم من كل مكان؟
}
admin-userlist-confirm-signout-one = تسجيل خروج { $account } من كل مكان؟
admin-userlist-confirm-signout-body = تنتهي كل جلسات Mailwoman المفتوحة لهؤلاء المستخدمين. يمكنهم تسجيل الدخول مجددًا.
admin-userlist-confirm-signout-action = تسجيل الخروج
admin-userlist-confirm-delete = { $count ->
    [zero] حذف الوصول المحلي لـ { $count } مستخدم؟
    [one] حذف الوصول المحلي لمستخدم واحد؟
    [two] حذف الوصول المحلي لمستخدمَين؟
    [few] حذف الوصول المحلي لـ { $count } مستخدمين؟
    [many] حذف الوصول المحلي لـ { $count } مستخدمًا؟
   *[other] حذف الوصول المحلي لـ { $count } مستخدم؟
}
admin-userlist-confirm-delete-one = حذف الوصول المحلي لـ { $account }؟
admin-userlist-confirm-delete-body = يُحتفظ بسجل الوصول المحذوف حتى لا يُسجَّل المستخدم تلقائيًا من جديد. تبقى إعدادات Mailwoman والسجل وصندوق البريد الأصلي محفوظة. اكتب العنوان للتأكيد.
admin-userlist-confirm-delete-action = حذف الوصول المحلي
admin-userlist-confirm-profile = { $count ->
    [zero] تغيير ملف الميزات لـ { $count } مستخدم؟
    [one] تغيير ملف الميزات لمستخدم واحد؟
    [two] تغيير ملف الميزات لمستخدمَين؟
    [few] تغيير ملف الميزات لـ { $count } مستخدمين؟
    [many] تغيير ملف الميزات لـ { $count } مستخدمًا؟
   *[other] تغيير ملف الميزات لـ { $count } مستخدم؟
}
admin-userlist-confirm-profile-one = تغيير ملف الميزات لـ { $account }؟
admin-userlist-confirm-profile-body = يُعيَّن الملف لكل مستخدم مباشرةً ويسري من طلبه التالي. تُدعم هويات صناديق البريد الأصلية والوكيل وSSO.
admin-users-profile-help = اختر "الافتراضي" لإزالة التعيين المباشر لهذا المستخدم واستخدام الملفات الموروثة من مجموعاته أو نطاقه.
admin-users-profile-ambiguous = لهذا المستخدم عدة هويات لتسجيل الدخول. اختر الهوية المقصودة في ملفات الميزات.
admin-users-profile-unavailable = لا توجد هوية صندوق بريد متحقق منها. يجب أن يسجّل هذا المستخدم الدخول قبل تعيين ملف.
admin-userlist-confirm-profile-action = تغيير الملف
admin-userlist-bulk-done-block = { $count ->
    [zero] لم يُحظر تسجيل الدخول لأي مستخدم.
    [one] حُظر تسجيل الدخول لمستخدم واحد.
    [two] حُظر تسجيل الدخول لمستخدمَين.
    [few] حُظر تسجيل الدخول لـ { $count } مستخدمين.
    [many] حُظر تسجيل الدخول لـ { $count } مستخدمًا.
   *[other] حُظر تسجيل الدخول لـ { $count } مستخدم.
}
admin-userlist-bulk-done-unblock = { $count ->
    [zero] لم يُلغَ حظر تسجيل الدخول لأي مستخدم.
    [one] أُلغي حظر تسجيل الدخول لمستخدم واحد.
    [two] أُلغي حظر تسجيل الدخول لمستخدمَين.
    [few] أُلغي حظر تسجيل الدخول لـ { $count } مستخدمين.
    [many] أُلغي حظر تسجيل الدخول لـ { $count } مستخدمًا.
   *[other] أُلغي حظر تسجيل الدخول لـ { $count } مستخدم.
}
admin-userlist-bulk-done-signout = { $count ->
    [zero] لم يُسجَّل خروج أي مستخدم.
    [one] سُجّل خروج مستخدم واحد.
    [two] سُجّل خروج مستخدمَين.
    [few] سُجّل خروج { $count } مستخدمين.
    [many] سُجّل خروج { $count } مستخدمًا.
   *[other] سُجّل خروج { $count } مستخدم.
}
admin-userlist-bulk-done-delete = { $count ->
    [zero] لم يُحذف الوصول المحلي لأي مستخدم.
    [one] حُذف الوصول المحلي لمستخدم واحد.
    [two] حُذف الوصول المحلي لمستخدمَين.
    [few] حُذف الوصول المحلي لـ { $count } مستخدمين.
    [many] حُذف الوصول المحلي لـ { $count } مستخدمًا.
   *[other] حُذف الوصول المحلي لـ { $count } مستخدم.
}
admin-userlist-bulk-done-profile = { $count ->
    [zero] لم يتغير ملف الميزات لأي مستخدم.
    [one] تغيّر ملف الميزات لمستخدم واحد.
    [two] تغيّر ملف الميزات لمستخدمَين.
    [few] تغيّر ملف الميزات لـ { $count } مستخدمين.
    [many] تغيّر ملف الميزات لـ { $count } مستخدمًا.
   *[other] تغيّر ملف الميزات لـ { $count } مستخدم.
}
admin-userlist-bulk-failed = اكتمل { $done }، وفشل { $failed }: { $reason }

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-user-compliance-due = مستحق { $date }

admin-user-compliance-empty = لا توجد طلبات بيانات لهذا الحساب.

admin-user-compliance-help = الطلبات المقدمة لعناوين هذا الحساب. افتح واحدة في طلبات البيانات للتعامل معها.

admin-user-compliance-open-all = فتح كافة طلبات البيانات

admin-user-compliance-title = طلبات البيانات لهذا الحساب

admin-user-never = أبدا

admin-user-overview-last-signin = آخر تسجيل دخول

admin-user-overview-status = الحالة

admin-user-status-active = نشط

admin-user-status-deleted = تم الحذف

admin-user-status-deleting = حذف

admin-user-status-disabled = محظور

admin-user-tab-access = الوصول والإزالة

admin-user-tab-compliance = الامتثال

admin-user-tab-overview = نظرة عامة

admin-user-tab-profile = ملف تعريف الميزة

admin-user-tab-security = الأمن

admin-user-tab-sessions = الجلسات

admin-user-tabs-label = أقسام الحساب
