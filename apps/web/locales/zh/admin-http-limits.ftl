# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-http-limits. Source: ../en/admin-http-limits.ftl

admin-http-limits-deployment = 当前生效：{ $connections } 个连接（{ $connectionsSource ->
        [console] 在控制台中设置
        [explicit] 由环境设置
       *[other] 自动
    }），{ $requests } 个同时请求（{ $requestsSource ->
        [console] 在控制台中设置
        [explicit] 由环境设置
       *[other] 自动
    }），每个连接 { $streams } 个 HTTP/2 流。达到上限的请求最多等待 { $queueWaitMs } 毫秒，最多 { $queueMax } 个同时等待。
admin-http-limits-warning = 警告：{ $text }
