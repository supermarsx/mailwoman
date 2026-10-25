# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = 仪表板
admin-shell-group-overview = 概览
admin-shell-group-people = 人员
admin-shell-group-security = 安全
admin-shell-group-configuration = 配置
admin-shell-group-operations = 运维
admin-shell-menu = 菜单
admin-shell-close-menu = 关闭菜单
admin-shell-search = 搜索
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = 已登录为 { $name }
admin-shell-language-label = 语言
admin-shell-breadcrumb = 导航路径
admin-shell-unknown-section = 此部分不存在，或你的角色不包含此部分。
admin-shell-back-to-dashboard = 返回仪表板
admin-shell-cap-missing = 你的角色不包含 { $capability }。

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = 正在加载…
admin-shell-retry = 重试
admin-shell-cancel = 取消
admin-shell-save = 保存
admin-shell-not-available = 不可用

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = 离开此页面？
admin-shell-leave-body = 此页面上的更改尚未保存。离开将丢弃这些更改。
admin-shell-leave-confirm = 离开并丢弃更改

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = 输入 { $name } 以确认

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = 搜索管理控制台
admin-shell-palette-label = 部分、用户、域名或设置
admin-shell-palette-results = 结果
admin-shell-palette-none = 没有匹配项。
admin-shell-palette-searching = 正在搜索用户、域名和设置…
admin-shell-palette-sections-only = 此服务器尚不能搜索用户、域名和设置，因此只列出各部分。
admin-shell-palette-error = 搜索用户、域名和设置失败；只列出各部分。
admin-shell-palette-kind-section = 部分
admin-shell-palette-kind-user = 用户
admin-shell-palette-kind-domain = 域名
admin-shell-palette-kind-setting = 设置

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = 筛选
admin-shell-table-actions = 操作
admin-shell-table-error = 无法加载列表。
admin-shell-table-paging = { $table } 的分页
admin-shell-table-prev = 上一页
admin-shell-table-next = 下一页

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }：{ $value }
admin-shell-chart-summary = 最新 { $latest }，最高 { $max }
admin-shell-chart-col-time = 时间
admin-shell-chart-col-item = 项目
admin-shell-chart-col-value = 值

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = 设置来源
admin-shell-pinned-sr = 此值来自服务器的环境变量，无法在此更改。
admin-shell-lock-text = 对用户锁定
admin-shell-lock-setting = { $setting }
admin-shell-restore = 恢复先前的值
admin-shell-restore-note = 由 { $actor } 于 { $time } 更改。先前的值：{ $value }
admin-shell-restore-done = 已恢复先前的值。

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = 步骤
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = 第 { $n } 步，共 { $total } 步：{ $title }
admin-shell-wizard-review = 检查
admin-shell-wizard-back = 上一步
admin-shell-wizard-next = 下一步
admin-shell-wizard-saved = 已保存。

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = 无法连接到服务器。请检查连接后重试。
admin-shell-err-session = 你的管理员会话已结束。请重新登录。
admin-shell-err-forbidden = 你的角色不允许执行此操作。请联系具有相应角色的管理员。
admin-shell-err-out-of-scope = 此对象属于你的角色未涵盖的域名。
admin-shell-err-not-found = 它已不存在。请重新加载列表。
admin-shell-err-conflict = 其他人在此期间更改了它。请重新加载后重试。
admin-shell-err-validation = 服务器拒绝了某个值。请检查各字段后重试。
admin-shell-err-rate-limited = 请求过多。请等待一分钟后重试。
admin-shell-err-server = 服务器返回错误（{ $status }）。请重试；如果再次出现，请检查服务器日志。
admin-shell-err-upstream-dns = 无法解析主机名。请检查主机名的拼写。
admin-shell-err-upstream-connect = 该主机和端口上没有响应。请检查端口以及中间的防火墙。
admin-shell-err-upstream-tls = 服务器有响应，但 TLS 握手失败。请检查端口和连接安全设置。
admin-shell-err-upstream-auth = 服务器拒绝了凭据。请检查用户名和密码。

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = 登录尝试次数过多。请等待几分钟后重试。
admin-shell-totp-form = 管理员登录，第二步
admin-shell-totp-note = 输入为 { $name } 设置的身份验证器应用中的 6 位验证码。
admin-shell-totp-code = 验证码
admin-shell-totp-verify = 验证
admin-shell-totp-start-over = 重新开始
admin-shell-totp-invalid = 该验证码无效。请输入身份验证器应用中的当前验证码。
admin-shell-totp-expired = 登录耗时过长。请重新开始并输入密码。
