# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = المحتوى الخارجي

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
    [zero] لم تُحظر أي صورة خارجية
    [one] حُظرت صورة خارجية واحدة
    [two] حُظرت صورتان خارجيتان
    [few] حُظرت { $count } صور خارجية
    [many] حُظرت { $count } صورة خارجية
   *[other] حُظرت { $count } صورة خارجية
}
remote-blocked-trackers = { $count ->
    [zero] لم يُحظر أي متتبّع
    [one] حُظر متتبّع واحد
    [two] حُظر متتبّعان
    [few] حُظرت { $count } متتبّعات
    [many] حُظر { $count } متتبّعًا
   *[other] حُظر { $count } متتبّع
} من أصل { $blocked ->
    [zero] { $blocked } صورة خارجية
    [one] صورة خارجية واحدة
    [two] صورتين خارجيتين
    [few] { $blocked } صور خارجية
    [many] { $blocked } صورة خارجية
   *[other] { $blocked } صورة خارجية
}
remote-blocked-hosts = المضيفون المحظورون

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = تحميل الصور
remote-grant-sender = التحميل دائمًا من { $sender }
remote-grant-domain = التحميل دائمًا من { $domain }
remote-grant-all = تحميل جميع الصور الخارجية دائمًا

remote-grant-done-single = حُمّلت الصور الخارجية لهذه الرسالة.
remote-grant-done-per-sender = ستُحمَّل الصور الخارجية من هذا المرسل.
remote-grant-done-per-domain = ستُحمَّل الصور الخارجية من هذا النطاق.
remote-grant-done-all = ستُحمَّل الصور الخارجية لكل البريد.

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = يجري تحميل الصور الخارجية لهذه الرسالة.
remote-revoke = إيقاف
remote-revoke-done = أُوقفت الصور الخارجية. ستُحظر مجددًا عند الفتح التالي.
