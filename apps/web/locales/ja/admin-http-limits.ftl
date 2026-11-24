# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = 現在の値: 接続 { $connections } 件 ({ $connectionsSource ->
        [console] コンソールで設定
        [explicit] 環境で設定
       *[other] Auto
    })、同時リクエスト { $requests } 件 ({ $requestsSource ->
        [console] コンソールで設定
        [explicit] 環境で設定
       *[other] Auto
    })、接続あたりの HTTP/2 ストリーム { $streams } 個。上限に達したリクエストは最大 { $queueWaitMs } ms 待機し、同時に待機できるのは最大 { $queueMax } 件です。
admin-http-limits-warning = 警告: { $text }
