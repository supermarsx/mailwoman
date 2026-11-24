# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ar · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = السارية الآن: { $connections } اتصال ({ $connectionsSource ->
        [console] معيّن في وحدة التحكم
        [explicit] معيّن من البيئة
       *[other] تلقائي
    })، و{ $requests } طلب في وقت واحد ({ $requestsSource ->
        [console] معيّن في وحدة التحكم
        [explicit] معيّن من البيئة
       *[other] تلقائي
    })، و{ $streams } تدفق HTTP/2 لكل اتصال. ينتظر الطلب عند بلوغ الحد حتى { $queueWaitMs } مللي ثانية، مع { $queueMax } طلب منتظر على الأكثر.
admin-http-limits-warning = تحذير: { $text }
