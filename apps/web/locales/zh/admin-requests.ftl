# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-requests. Source: ../en/admin-requests.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-history-not-found = 每个时段的未找到（404）
admin-history-nf-total = 此期间的未找到（404）
admin-history-nf-share = 占请求的 { $share }
admin-history-nf-not-tracked-window = 此期间未跟踪 404。
admin-history-nf-not-tracked-before = { $time } 之前未跟踪 404。更早的时段显示为不可用，而不是零。
admin-history-nf-paths-title = 最常见的未找到路径
admin-history-nf-paths-help = 每小时最多 { $limit } 条路径，从 { $from } 到 { $to }，向外取整到整小时。查询字符串会被移除，电子邮件地址、数字、ID、令牌和长值会被替换为 {"{"}token{"}"} 之类的占位符。应用自身预期的未命中（例如浏览器图标请求）不计入。
admin-history-nf-paths-approximate = 到达的不同路径多于保留的数量，因此部分计数是一个范围，而非精确数字。
admin-history-nf-paths-empty = 此期间没有未找到的响应。
admin-history-nf-paths-error = 无法加载未找到的路径。请刷新后重试。
admin-history-nf-paths-method = 方法
admin-history-nf-paths-path = 路径
admin-history-nf-paths-count = 数量
admin-history-nf-paths-range = { $min } 至 { $max }
admin-history-nf-paths-client = 来自应用自身的客户端
admin-history-nf-paths-client-help = 网页应用、其 service worker 或桌面和移动应用请求的路径：构建文件、worker 脚本、API、JMAP、DAV 和控制台路由。这里出现未命中通常意味着客户端已过期或某个路由有误。
admin-history-nf-paths-external = 其他流量
admin-history-nf-paths-external-help = 应用从不请求的路径，例如扫描器探测 PHP 文件、点文件或其他软件，以及来自其他地方的链接。
admin-history-nf-paths-all = 路径
admin-history-nf-paths-all-help = 此服务器无法区分客户端路径和其他流量。
