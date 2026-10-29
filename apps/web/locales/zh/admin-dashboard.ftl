# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = 刷新
admin-dash-unavailable = 仪表板数据尚不可用。
admin-dash-unavailable-detail = 此服务器不提供仪表板数据（GET /admin/dashboard 返回 404）。该功能将随后续的服务器更新提供。其他部分照常工作。
admin-dash-error-network = 无法加载仪表板：服务器没有响应。请重试。
admin-dash-error-status = 无法加载仪表板：服务器返回 { $status }。请重试；如果再次出现，请检查服务器日志。

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = 需要关注
admin-dash-warn-fix = 打开可修复此问题的部分
admin-dash-warn-blocked-accounts =
    { $count ->
       *[other] { $count } 个账户因其域名没有配置而无法登录。
    }
admin-dash-warn-no-trusted-proxy = IP 规则已启用，但未配置受信任的代理，因此所有客户端都可能显示为来自代理的地址。
admin-dash-warn-allowlist-bypass = 某个环境变量关闭了 IP 允许列表。
admin-dash-warn-metrics-token = 指标端点未设置令牌。
admin-dash-warn-pending-migrations =
    { $count ->
       *[other] 有 { $count } 个数据库迁移尚未应用。请重启服务器以应用。
    }
admin-dash-warn-cert-expiring =
    { $count ->
       *[other] 有 { $count } 个上游证书即将过期（根据上次连接测试）。
    }
admin-dash-warn-unknown = 服务器警告 { $code }（{ $count }）

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title =
    { $count ->
       *[other] { $count } 个账户被阻止：没有域名配置
    }
admin-dash-blocked-note = 这些账户在此次更新前曾登录过，但其域名在控制台中没有配置，因此现在无法登录。配置该域名后即可重新登录。
admin-dash-blocked-chart = 各域名被阻止的账户
admin-dash-blocked-row =
    { $count ->
       *[other] { $count } 个账户
    }
admin-dash-blocked-seen = （上次使用的邮件主机 { $host }）
admin-dash-configure-domain = 配置 { $domain }
admin-dash-blocked-no-access = 你的角色无法配置域名。

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist =
    { $open ->
        [0] 设置清单：没有未完成项
       *[other] 设置清单：{ $open } 项未完成
    }
admin-dash-check-done = 已完成
admin-dash-check-open = 未完成
admin-dash-check-optional = （可选）
admin-dash-check-go = 打开
admin-dash-check-hide = 隐藏
admin-dash-check-unhide = 重新显示
admin-dash-check-hidden =
    { $count ->
       *[other] 有 { $count } 项已隐藏。隐藏不会将其标记为已完成。
    }
admin-dash-check-show-hidden = 显示已隐藏的项
admin-dash-check-hide-hidden = 不显示已隐藏的项
admin-dash-check-domain-exists = 添加邮件域名
admin-dash-check-domain-upstream = 为域名指定邮件服务器，或设置默认上游服务器
admin-dash-check-test-connection = 运行一次成功的连接测试（最近 7 天内）
admin-dash-check-preferred-domain = 选择首选域名，使用户无需输入域名即可登录
admin-dash-check-second-admin = 创建第二个管理员账户，或关闭环境变量登录
admin-dash-check-trusted-proxies = 配置此服务器前面的受信任代理
admin-dash-check-security-policy = 保存安全策略
admin-dash-check-metrics-token = 为指标端点设置令牌
admin-dash-check-branding = 检查品牌设置

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = 账户和会话
admin-dash-users = 已开通的用户
admin-dash-signed-in-24h = 已登录的账户，最近 24 小时
admin-dash-signed-in-7d = 已登录的账户，最近 7 天
admin-dash-domains = 域名
admin-dash-domains-detail = { $upstream } 个已有邮件服务器，{ $disabled } 个已停用
admin-dash-sessions = 活动的网页邮箱会话
admin-dash-admin-sessions = 活动的管理员会话
admin-dash-security = 登录安全
admin-dash-failed-webmail = 失败的网页邮箱登录，最近 24 小时
admin-dash-failed-admin = 失败的管理员登录，最近 24 小时
admin-dash-active-blocks = 被阻止的地址
admin-dash-allowlists = IP 允许列表
admin-dash-allowlists-value = 管理员 { $admin }，网页邮箱 { $webmail }
admin-dash-on = 开
admin-dash-off = 关
admin-dash-trusted-proxies = 受信任的代理
admin-dash-configured = 已配置
admin-dash-not-configured = 未配置
admin-dash-untrusted-forwarded =
    { $count ->
       *[other] 24 小时内有 { $count } 个请求带有来自不受信任对端的转发标头
    }

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = 流量，最近 { $mins } 分钟
admin-dash-traffic-note = 由此服务器进程自启动以来统计。不包括其他进程或副本。
admin-dash-traffic-empty = 此时间段内没有统计到请求。
admin-dash-requests = 每分钟请求数
admin-dash-errors = 每分钟服务器错误（5xx）
admin-dash-p95 = 响应时间，第 95 百分位

# -- Health ----------------------------------------------------------------------------
admin-dash-health = 运行状况
admin-dash-h-ok = 正常
admin-dash-h-attention = 需要关注
admin-dash-h-db = 数据库
admin-dash-h-db-ok = 响应用时 { $ms }
admin-dash-h-migrations = 迁移
admin-dash-h-migrations-ok = 已是最新（已应用 { $n } 个）
admin-dash-h-migrations-behind = 已应用 { $applied } / { $expected } 个
admin-dash-h-engine = 邮件同步
admin-dash-h-engine-value = 已注册 { $accounts } 个账户，其中 { $errors } 个出现同步错误
admin-dash-h-outbox = 发件箱
admin-dash-h-outbox-value = { $pending } 封等待中（最早 { $oldest }），24 小时内 { $failed } 封失败
admin-dash-h-outbox-none = 无
admin-dash-h-index = 搜索索引
admin-dash-h-index-value = { $n } 个文档
admin-dash-h-index-rebuilding = 正在重建（目前已有 { $n } 个文档）
admin-dash-h-sandbox = 渲染沙箱
admin-dash-h-sandbox-ok = 可用：不受信任的文档在单独的进程中渲染。
admin-dash-h-sandbox-missing = 缺失：不受信任的文档在服务器进程内渲染。
admin-dash-h-sandbox-required = 缺失，而此服务器要求使用它：不受信任的文档将无法渲染。
admin-dash-h-plugins = 插件运行时
admin-dash-h-plugins-ok = 可用
admin-dash-h-plugins-off = 已关闭
admin-dash-na-proxy = 此服务器以代理模式运行，自身不同步邮件。
admin-dash-na-unreported = 服务器未报告此项。

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = 存储
admin-dash-db-size = 数据库文件
admin-dash-wal-size = 预写日志
admin-dash-index-size = 搜索索引
admin-dash-volume-free = 数据卷上的可用空间
admin-dash-volume-of = 共 { $total }
admin-dash-na-wal = 此数据库不使用预写日志文件，或服务器未报告此项。
admin-dash-na-index = 此服务器没有搜索索引。
admin-dash-na-volume = 服务器无法在此平台上读取可用空间。

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = 最近的管理员操作
admin-dash-audit-empty = 尚未记录任何管理员操作。
admin-dash-audit-when = 时间
admin-dash-audit-actor = 操作者
admin-dash-audit-action = 操作
admin-dash-audit-target = 对象
admin-dash-audit-all = 打开完整的审计日志
admin-dash-server = 服务器
admin-dash-version = 版本
admin-dash-build = 构建
admin-dash-mode = 模式
admin-dash-mode-engine = 引擎：此服务器同步并存储邮件
admin-dash-mode-proxy = 代理：此服务器将请求转发给邮件服务器
admin-dash-uptime = 已运行
admin-dash-base-path = 访问路径
admin-dash-base-root = 站点根目录
admin-dash-duration-dh = { $d } 天 { $h } 小时
admin-dash-duration-hm = { $h } 小时 { $m } 分钟
admin-dash-duration-m = { $m } 分钟
admin-dash-duration-s = { $s } 秒
