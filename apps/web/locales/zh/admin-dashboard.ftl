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
admin-dash-h-migrations = 迁移
admin-dash-h-engine = 邮件同步
admin-dash-h-outbox = 发件箱
admin-dash-h-index = 搜索索引
admin-dash-h-sandbox = 渲染沙箱
admin-dash-h-plugins = 插件运行时

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = 存储
admin-dash-db-size = 数据库文件
admin-dash-wal-size = 预写日志
admin-dash-index-size = 搜索索引
admin-dash-volume-free = 数据卷上的可用空间
admin-dash-volume-of = 共 { $total }
admin-dash-na-wal = 此数据库不使用预写日志文件，或服务器未报告此项。
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

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = 国家/地区策略已启用,但未加载可用的国家/地区数据库,因此其涵盖的登录将被拒绝。
admin-dash-warn-geoip-stale = 国家/地区策略已启用,但国家/地区数据库已过期。
admin-dash-h-status-ok = 正常
admin-dash-h-status-warn = 警告
admin-dash-h-status-fail = 失败
admin-dash-h-status-disabled = 已停用
admin-dash-h-checked = 检查时间:{ $time }
admin-dash-h-refresh = 重新检查
admin-dash-h-refresh-failed = 无法重新执行健康检查(状态 { $status })。显示的是上一次的结果。
admin-dash-h-refresh-network = 无法重新执行健康检查:服务器没有响应。显示的是上一次的结果。
admin-dash-h-enable = 如何开启
admin-dash-h-probe-time = 探测耗时 { $time }
admin-dash-h-outbox-probe-time = 状态汇总耗时 { $time };提交握手耗时单独显示。
admin-dash-h-history = 最近 { $n } 次检查
admin-dash-h-open-diagnostics = 诊断
admin-dash-h-open-logs = 日志
admin-dash-h-open-plugins = 插件
admin-dash-h-open-maintenance = 维护
admin-dash-h-not-tracked = 未跟踪
admin-dash-h-unlimited = 无限制
admin-dash-h-m-none = 无
admin-dash-h-m-ago = { $age }前
admin-dash-h-m-ping = 往返时间
admin-dash-h-m-pool = 连接池
admin-dash-h-m-pool-value = 使用中 { $inUse },空闲 { $idle }(已打开 { $size })
admin-dash-h-m-engine = 数据库服务器
admin-dash-h-m-location = 位置
admin-dash-h-m-db-size = 数据库大小
admin-dash-h-m-wal-size = 预写日志
admin-dash-h-m-journal = 日志模式
admin-dash-h-m-slow = 慢查询(最近一小时)
admin-dash-h-m-applied = 已应用
admin-dash-h-m-applied-value = 此版本共 { $expected } 个,已应用 { $applied } 个
admin-dash-h-m-latest = 最新已应用
admin-dash-h-m-pending = 尚未应用
admin-dash-h-m-failed = 已开始但未完成
admin-dash-h-m-unknown = 来自更新的版本
admin-dash-h-m-accounts = 邮箱
admin-dash-h-m-accounts-value = 共注册 { $total } 个:同步中 { $syncing },空闲 { $idle },失败 { $erroring }
admin-dash-h-m-last-sync = 自上次完成的同步以来
admin-dash-h-m-last-sync-value = 中位数 { $p50 },最长 { $max }
admin-dash-h-m-never-synced = 服务器启动后尚未同步
admin-dash-h-m-queue = 正在运行的同步
admin-dash-h-m-errors-hour = 同步错误(最近一小时)
admin-dash-h-m-by-kind = 按后端划分的连接
admin-dash-h-m-kind-value = { $kind }:{ $accounts } 个邮箱,{ $watch } 个监视连接
admin-dash-h-m-recent-errors = 最近的错误
admin-dash-h-m-queued = 等待发送
admin-dash-h-m-queued-value = 待发送 { $queued },重试中 { $deferred },发送中 { $sending }
admin-dash-h-m-oldest = 等待最久
admin-dash-h-m-scheduled = 已定时或已搁置
admin-dash-h-m-scheduled-value = 已定时 { $scheduled },已搁置 { $held }
admin-dash-h-m-failed-total = 失败(仍在发件箱中)
admin-dash-h-m-failed-total-value = 失败 { $failed },投递结果未知 { $uncertain }
admin-dash-h-m-sends-hour = 发送(最近一小时)
admin-dash-h-m-sends-hour-value = 已发送 { $sent },已延后 { $deferred },失败 { $failed }
admin-dash-h-m-last-send = 上次发送
admin-dash-h-m-last-send-value = { $outcome },{ $age }前
admin-dash-h-m-last-send-none = 服务器启动后尚无发送
admin-dash-h-outcome-sent = 已发送
admin-dash-h-outcome-deferred = 已延后
admin-dash-h-outcome-failed = 失败
admin-dash-h-outcome-uncertain = 投递结果未知
admin-dash-h-m-submission = 提交服务器
admin-dash-h-m-submission-value = { $checked } 个中 { $reachable } 个已响应(仅握手,不登录;最慢 { $slowest }),{ $age }前检查
admin-dash-h-m-submission-none = 未配置提交服务器。
admin-dash-h-m-submission-pending = 正在检查连接。
admin-dash-h-m-submission-unmeasured = { $count } 个服务器需要经过身份验证的提供商检查;未发送任何凭据。
admin-dash-h-m-documents = 文档
admin-dash-h-m-documents-value = 已存储的 { $messages } 封邮件中的 { $documents } 封
admin-dash-h-m-index-pending = 尚未建立索引
admin-dash-h-m-index-size = 磁盘大小
admin-dash-h-m-index-memory = 内存中
admin-dash-h-m-last-commit = 上次提交
admin-dash-h-m-rebuild = 重建
admin-dash-h-m-rebuild-running = 运行中:{ $pct }({ $total } 中的 { $done })
admin-dash-h-m-rebuild-idle = 未运行
admin-dash-h-m-version = 工作进程版本
admin-dash-h-m-worker-missing = 没有工作进程二进制文件
admin-dash-h-m-render-ping = Ping(启动、jail、应答)
admin-dash-h-m-render-jobs = 当前正在运行的任务
admin-dash-h-m-render-failures = 问题(最近一小时)
admin-dash-h-m-render-failures-value = 失败 { $failed },超时 { $timeouts },繁忙时拒绝 { $busy }
admin-dash-h-m-jail = 内核 jail
admin-dash-h-m-jail-required = 必需
admin-dash-h-m-jail-optional = 此平台不需要
admin-dash-h-m-jail-degraded = 已降级:{ $why }
admin-dash-h-m-plugins = 插件
admin-dash-h-m-plugins-value = 已安装 { $installed },已启用 { $enabled },已加载 { $loaded }
admin-dash-h-m-plugin-list = 已启用的插件
admin-dash-h-m-plugin-loaded = 已加载
admin-dash-h-m-plugin-not-loaded = 未加载({ $reason })
admin-dash-h-m-plugin-limits = 内存 { $memory },期限 { $deadline },燃料 { $fuel }
admin-dash-h-m-plugin-calls = 调用失败(最近一小时)
admin-dash-h-m-plugin-calls-value = 陷阱 { $traps },触发限制 { $limits },其他错误 { $errors }
admin-dash-h-m-host-ping = 插件宿主锁
admin-dash-h-t-ping = 超过 { $warn } 警告,超过 { $fail } 失败
admin-dash-h-t-oldest = 邮件等待超过 { $warn } 时警告,超过 { $fail } 时失败
admin-dash-h-t-pending = 未建立索引的邮件超过 { $n } 封时警告
admin-dash-h-t-render = 超过 { $warn } 警告;在 { $timeout } 内无应答时失败
admin-dash-h-t-sync = 当 { $pct } 或更多邮箱同步失败时判为失败
admin-dash-h-chart-db = 数据库往返时间
admin-dash-h-chart-render = 渲染工作进程 Ping
admin-dash-h-chart-outbox = 等待发送的邮件
admin-dash-h-chart-search = 未建立索引的邮件
admin-dash-na-index-proxy = 代理模式:此服务器不保留搜索索引。
admin-dash-na-index-memory = 搜索索引保存在内存中,因此没有磁盘大小。
admin-dash-customize = 自定义仪表板
admin-dash-limitations = 数据可用性
