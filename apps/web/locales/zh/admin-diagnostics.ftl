# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = 诊断页面
admin-diagx-tab-overview = 概览
admin-diagx-tab-server = 服务器
admin-diagx-tab-connectivity = 连通性
admin-diagx-tab-backends = 邮件后端
admin-diagx-tab-posture = 安全状况
admin-diagx-tab-tools = 工具
admin-diagx-tab-history = 历史记录
admin-diagx-tab-report = 报告
admin-diagx-tab-server-intro = 检查此服务器自身的各个部分：数据库、其迁移和服务器密钥。它们随服务器检查一起运行。
admin-diagx-tab-connectivity-intro = 此服务器能否通过其公开名称访问、能否连接到已保存的后端，以及是否让专用地址无法被访问。下方的域名工具可查询 DNS 记录、测试 TLS 并发现某个域名的设置。
admin-diagx-tab-backends-intro = 对每个已保存的邮件、提交、规则和 DAV 端点进行连接和安全握手。不发送凭据，也不投递邮件。
admin-diagx-tab-posture-intro = 保护登录和会话的设置，以及 fail2ban 如何看待请求的地址。
admin-diagx-tab-tools-intro = 检查服务的其他位置，以及适用于诊断运行的限制。
admin-diagx-tab-history-intro = 已保存的运行，最新的在前。打开某次运行可查看其结果以及相对上一次运行的变化。
admin-diagx-status-ok = 正常
admin-diagx-status-warn = 警告
admin-diagx-status-fail = 失败
admin-diagx-status-skipped = 已跳过
admin-diagx-status-not-run = 未运行
admin-diagx-status-running = 运行中
admin-diagx-run = 运行
admin-diagx-run-again = 再次运行
admin-diagx-running = 正在运行…
admin-diagx-run-server-note = 再次运行服务器检查。该次运行的所有卡片都会更新。
admin-diagx-run-backends-note = 再次检查所有已保存的后端。所有后端卡片都会更新。
admin-diagx-how-to-fix = 如何修复
admin-diagx-details = 详情
admin-diagx-copy-details = 复制详情
admin-diagx-copy-value = 复制{ $label }
admin-diagx-copied = 已复制。
admin-diagx-copy-failed = 浏览器不允许复制。请选中文本后手动复制。
admin-diagx-not-run-yet = 尚未运行。
admin-diagx-not-reported = 上次服务器运行未报告此检查；它不适用于此部署。
admin-diagx-posture-met = 已满足
admin-diagx-posture-not-met = 未满足
admin-diagx-loading = 正在加载…
admin-diagx-none = 无
admin-diagx-yes = 是
admin-diagx-no = 否
admin-diagx-open = 打开
admin-diagx-row-result = 结果
admin-diagx-row-duration = 耗时
admin-diagx-row-checked = 检查时间
admin-diagx-row-id = 检查 ID
admin-diagx-row-domain = 域名
admin-diagx-row-backend = 后端
admin-diagx-row-protocol = 协议
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = 能力
admin-diagx-row-credentials = 已发送的凭据
admin-diagx-row-source = 发现于
admin-diagx-row-oauth = 需要 OAuth
admin-diagx-overview-intro = 按分组显示每项检查的最新结果。“全部运行”会先运行服务器检查，然后检查每个已保存的后端。
admin-diagx-run-all = 运行所有检查
admin-diagx-run-all-step = 第 { $step } 步，共 { $steps } 步：{ $suite ->
    [backends] 已保存的后端
   *[server] 服务器检查
}。
admin-diagx-run-all-done = 所有检查已完成。结果已保存到历史记录。
admin-diagx-backends-progress = 已检查 { $completed } / { $total } 个后端。
admin-diagx-counts = { $ok } 项正常 · { $warn } 项警告 · { $fail } 项失败 · { $notRun } 项未运行
admin-diagx-attention = 需要关注
admin-diagx-attention-none = 没有检查在其最新运行中报告警告或失败。
admin-diagx-never-run = 尚未运行任何检查。运行所有检查以获得首个结果。
admin-diagx-last-server = 上次服务器检查
admin-diagx-last-backends = 上次后端检查
admin-diagx-never = 从未
admin-diagx-open-health = 服务健康状况
admin-diagx-check-database-connectivity = 数据库连接
admin-diagx-check-database-connectivity-purpose = 数据库能响应简单的查询。
admin-diagx-check-database-connectivity-fix = 检查数据库是否在运行，以及 MW_DB_PATH 或 MW_DATABASE_URL 是否指向它。服务器日志会指出连接错误。
admin-diagx-check-database-write = 数据库写入
admin-diagx-check-database-write-purpose = 可以写入一条临时行，然后将其回滚。
admin-diagx-check-database-write-fix = 检查可用磁盘空间，以及数据库文件或用户是否可写。只读卷或磁盘已满会导致无法写入。
admin-diagx-check-database-migrations = 数据库迁移
admin-diagx-check-database-migrations-purpose = 数据库包含此服务器版本所需的所有迁移。
admin-diagx-check-database-migrations-fix = 重启服务器以应用待处理的迁移，并在其日志中查看是否有迁移失败。请勿在较新的数据库上运行较旧的服务器。
admin-diagx-check-server-key-roundtrip = 服务器密钥
admin-diagx-check-server-key-roundtrip-purpose = 服务器密钥可以封装并打开一个测试值。
admin-diagx-check-server-key-roundtrip-fix = 检查 MW_SERVER_KEY（或密钥文件）是否就是创建此数据库时使用的密钥。已存储的机密无法用另一个密钥打开。
admin-diagx-check-other-purpose = 此服务器报告的一项检查。
admin-diagx-check-other-fix = 打开“详情”查看原始结果，并检查运行时刻的服务器日志。
admin-diagx-check-network-public-dns = 公开名称
admin-diagx-check-network-public-dns-purpose = MW_PUBLIC_URL 中的主机名可以从此服务器解析。
admin-diagx-check-network-public-dns-fix = 检查公开名称的 DNS 记录，或将 MW_PUBLIC_URL 设置为用户访问此服务器所用的地址。
admin-diagx-check-network-public-port = 公开端口
admin-diagx-check-network-public-port-purpose = 公开 URL 的端口接受来自此服务器的连接。
admin-diagx-check-network-public-port-fix = 检查防火墙、反向代理和端口转发。某些路由器无法连接自己的公开地址（NAT 回环）；也请从网络外部进行测试。
admin-diagx-check-network-backend-ports = 后端端口
admin-diagx-check-network-backend-ports-purpose = 每个已保存后端的主机和端口都接受直接连接。
admin-diagx-check-network-backend-ports-fix = 检查后端主机是否在线，以及防火墙是否允许此服务器访问它们。“邮件后端”会显示哪个端点的握手失败。
admin-diagx-check-egress-private-policy = 专用地址保护
admin-diagx-check-egress-private-policy-purpose = 默认情况下，到环回地址和专用地址的出站连接会被拒绝。
admin-diagx-check-egress-private-policy-fix = 查看“隐私”下的出站设置。只对需要的具名后端允许专用地址。
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = 请求通过 HTTPS 到达此服务器。
admin-diagx-check-security-https-fix = 通过 HTTPS 提供控制台，或在由反向代理终止 TLS 时，将 MW_PUBLIC_URL 设置为 https 并设置 MW_TRUSTED_PROXIES。
admin-diagx-check-security-secure-cookies = 安全 Cookie
admin-diagx-check-security-secure-cookies-purpose = 会话 Cookie 仅通过 HTTPS 发送。
admin-diagx-check-security-secure-cookies-fix = 当服务器通过 HTTPS 访问后，将 MW_COOKIE_SECURE 设置为 true。
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = 浏览器被告知对此站点使用 HTTPS。
admin-diagx-check-security-hsts-fix = 在安全设置中开启 Strict-Transport-Security，或由反向代理发送。
admin-diagx-check-security-csrf = CSRF 保护
admin-diagx-check-security-csrf-purpose = 更改数据的请求必须携带 CSRF 令牌。
admin-diagx-check-security-csrf-fix = 将 MW_CSRF_STRICT 设置为 true。
admin-diagx-check-security-admin-factors = 管理员第二重验证
admin-diagx-check-security-admin-factors-purpose = 每位管理员都有第二重登录验证。
admin-diagx-check-security-admin-factors-fix = 请没有第二重验证的管理员在“我的偏好设置”中添加通行密钥或身份验证器应用，或在安全中心中强制要求。
admin-diagx-check-security-signin-pacing = 登录重试延迟
admin-diagx-check-security-signin-pacing-purpose = 登录失败后会有短暂等待。
admin-diagx-check-security-signin-pacing-fix = 在安全策略中开启登录节奏控制。
admin-diagx-check-security-credential-locks = 凭据锁定
admin-diagx-check-security-credential-locks-purpose = 多次登录失败会将该凭据锁定一段时间。
admin-diagx-check-security-credential-locks-fix = 在安全中心中开启自动锁定。
admin-diagx-check-security-geography = 登录地理数据
admin-diagx-check-security-geography-purpose = 登录地理策略拥有所需的国家/地区数据。
admin-diagx-check-security-geography-fix = 安装或更新国家/地区数据库，或关闭地理策略。
admin-diagx-check-security-render-isolation = 隔离的邮件渲染
admin-diagx-check-security-render-isolation-purpose = 邮件在独立的隔离工作进程中渲染。
admin-diagx-check-security-render-isolation-fix = 检查 MW_RENDER_BIN 是否指向 mw-render 程序且其能启动。仪表板上的服务健康状况会显示该工作进程的状态。
admin-diagx-check-security-deployment = 部署设置
admin-diagx-check-security-deployment-purpose = 可以读取部署安全设置。
admin-diagx-check-security-deployment-fix = 检查数据库连接，然后打开部署安全评估。
admin-diagx-check-security-plaintext-backends = 通过未加密连接传输的密码
admin-diagx-check-security-plaintext-backends-purpose = 没有已保存的后端接受不带 TLS 的密码。
admin-diagx-check-security-plaintext-backends-fix = 编辑域名的连接并选择 TLS 或 STARTTLS。仅在你控制的专用网络中允许明文连接。
admin-diagx-check-fail2ban-client-address = fail2ban 客户端地址
admin-diagx-check-fail2ban-client-address-purpose = fail2ban 日志行中会记录的地址就是真实客户端的地址。
admin-diagx-check-fail2ban-client-address-fix = 在反向代理之后，请将 MW_TRUSTED_PROXIES 设置为代理的地址，否则 fail2ban 会封禁代理而不是客户端。在服务器本机上打开控制台时，出现环回地址是正常的。
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = { $protocol } 端点：连接和安全握手，不登录。
admin-diagx-backend-other-purpose = 来自后端运行的一条说明。
admin-diagx-backend-fix = 检查为此后端保存的主机名、端口和 TLS 模式，DNS 是否能解析，以及证书对该主机名是否有效。然后再次运行。
admin-diagx-backend-config-fix = 打开域名的连接并重新保存；无法读取已保存的后端列表。
admin-diagx-backend-results = 后端检查
admin-diagx-backends-not-run = 尚未检查已保存的后端。
admin-diagx-backends-run-at = 上次检查于 { $date }：{ $count } 项结果。
admin-diagx-failover-title = 故障转移
admin-diagx-failover-purpose = 当首选邮件后端无法连接时，域名是否可以切换到另一个邮件后端。
admin-diagx-failover-state = 故障转移{ $state ->
    [on] 开启
   *[off] 关闭
} · { $backends } 个邮件后端{ $enabled ->
    [no] {" "}· 域名已停用
   *[yes] {""}
}
admin-diagx-no-domains = 没有已保存的域名。
admin-diagx-domain-tools = 域名工具
admin-diagx-domain-tools-purpose = 针对已保存域名的一次性检查。它们不会登录或发送邮件。
admin-diagx-domain = 域名
admin-diagx-tool-dns = DNS 记录
admin-diagx-tool-dns-purpose = 查询域名的 MX、SPF、DMARC 和服务（SRV）记录。
admin-diagx-tool-dns-fix = 在你的 DNS 提供商处添加缺少的记录。没有 MX，域名将无法收到邮件；没有 SPF 和 DMARC，其他服务器可能会拒绝其邮件。
admin-diagx-srv-found = 找到 { $found } / { $total } 条服务记录
admin-diagx-tool-tls = TLS 和握手
admin-diagx-tool-tls-purpose = 连接域名的每个已保存端点，并报告 TLS 版本、证书到期时间和耗时。
admin-diagx-tool-tls-fix = 对于无法连接的端点，请检查主机、端口和防火墙。请更新 14 天内到期的证书。
admin-diagx-probe-none = 该域名没有可测试的已保存端点。
admin-diagx-probe-line = { $host } · 可连接：{ $reachable ->
    [yes] 是
   *[no] 否
} · TLS { $tls } · 证书{ $days ->
    [none] 未报告
   *[other] 将在 { $days } 天后到期
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
admin-diagx-tool-detect = 发现设置
admin-diagx-tool-detect-purpose = 查询域名已发布的邮件设置（autoconfig、autodiscover、SRV）。不会保存任何内容。
admin-diagx-tool-detect-fix = 为该域名发布 autoconfig 或 SRV 记录，或在“域名”下手动输入设置。
admin-diagx-detect-name = 要查询的域名
admin-diagx-tool-health = 服务健康状况
admin-diagx-tool-health-purpose = 仪表板上的邮件同步、发件箱、搜索索引、渲染工作进程和插件。
admin-diagx-tool-deployment-purpose = 此服务器可观察到的所有部署安全设置，以及控制它们的设置。
admin-diagx-tool-ssh = SSH 隧道测试
admin-diagx-tool-ssh-purpose = 测试用于访问其他网络中后端的 SSH 隧道。
admin-diagx-tool-geo = 登录地理
admin-diagx-tool-geo-purpose = 登录的来源地，以及地理策略所依据的国家/地区数据库。
admin-diagx-catalogue-purpose = 每个诊断套件涵盖的内容，以及每次运行遵守的限制。
admin-diagx-suite-unsupported = 不可用：{ $reason }
admin-diagx-suite-server = 服务器检查
admin-diagx-suite-backends = 已保存的后端
admin-diagx-run-title = { $suite ->
    [backends] 已保存的后端
   *[server] 服务器检查
}，{ $date }
admin-diagx-changes = 变化
admin-diagx-changes-first = 没有可供比较的此类较早运行。
admin-diagx-changes-none = 自 { $date } 的运行以来没有变化。
admin-diagx-changes-since = 自 { $date } 的运行以来的变化：
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = 故障排查报告
admin-diagx-report-purpose = 供支持人员使用的 JSON 报告：运行时标志、最新结果和你选择的运行。绝不包含密码、令牌、Cookie、密钥、原始配置和邮件。
admin-diagx-report-run = 要包含的运行
admin-diagx-report-preview = 预览
admin-crumb-server = 服务器
admin-crumb-connectivity = 连通性
admin-crumb-posture = 安全状况
admin-crumb-tools = 工具
admin-crumb-history = 历史记录
admin-crumb-report = 报告
admin-diagx-table-actions = 操作
admin-diagx-check-pdfjs-viewer-assets = PDF 查看器文件
admin-diagx-check-pdfjs-viewer-assets-purpose = PDF 查看器的文件齐全、类型正确，并与此服务器构建一致。
admin-diagx-check-pdfjs-viewer-assets-fix = 重新部署 Web 构建，或从 MW_WEB_DIR 中删除 pdfjs 文件夹，以使用服务器内置的副本。如果此项为失败而非警告，请重新构建服务器镜像。
