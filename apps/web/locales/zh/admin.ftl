# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Mailwoman 管理
admin-nav = 管理部分
admin-sign-out = 退出登录
admin-nav-domains = 域名
admin-nav-users = 用户
admin-nav-integrations = 集成
admin-nav-observability = 可观测性
admin-nav-plugins = 插件
admin-nav-assist = AI 助手
admin-nav-sso = 单点登录
admin-nav-servermeta = 服务器元数据
admin-nav-rethread = 维护
admin-nav-2fa = 要求双重身份验证

# Shared admin actions / states
admin-delete = 删除
admin-revoke = 撤销
admin-remove = 移除
admin-saved = 已保存。

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = 管理员登录
admin-login-note = 此面板在单独的管理员会话中运行。会话在 30 分钟无活动后结束，最迟在登录 12 小时后结束。
admin-login-session-ended = 你的管理员会话已结束。请重新登录。
admin-login-username = 管理员用户名
admin-login-password = 密码
admin-login-sign-in = 登录
admin-login-signing-in = 正在登录…
admin-login-invalid = 管理员凭据无效
admin-login-unreachable = 无法连接到服务器

# -- Domains -----------------------------------------------------------------
admin-domains-title = 域名
admin-domains-load-error = 无法加载域名
admin-domains-save-error = 无法保存域名
admin-domains-delete-error = 无法删除域名
admin-domains-add = 添加域名
admin-domains-name = 域名名称
admin-domains-name-placeholder = example.com
admin-domains-note = 域名按名称注册。当某条规则适用于单个域名时，“要求双重身份验证”页面会提供已注册的名称供选择。注册域名不会路由或过滤邮件。
admin-domains-save = 保存域名
admin-domains-empty = 还没有域名。
admin-domains-delete-for = 删除 { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = 用户
admin-users-load-error = 无法加载用户
admin-users-provision-error = 无法开通用户
admin-users-flag-error = 无法更新标志
admin-users-revoke-error = 无法撤销会话
admin-users-provision = 开通用户
admin-users-username = 用户名
admin-users-domain = 域名
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = 配额字节数（0 = 不限）
admin-users-quota-msgs = 配额邮件数（0 = 不限）
admin-users-empty = 还没有用户。
admin-users-col-account = 账户
admin-users-col-quota = 配额（字节/邮件数）
admin-users-col-zeroaccess = 零访问
admin-users-col-flags = 标志
admin-users-col-sessions = 会话
admin-users-zeroaccess-for = { $account } 的零访问
admin-users-zeroaccess-help = 仅作记录。零访问存储由用户在自己的设置中使用自己的密钥开启和关闭。勾选此框不会加密该账户已存储的邮件，取消勾选也不会解密。
admin-users-disable-for = 停用 { $account }
admin-users-disabled = 已停用
admin-users-disabled-help = 阻止登录，并结束此账户所有已打开的会话。其 API 密钥和令牌会被拒绝。这不会停用邮件服务器上的邮箱：在你也在邮件服务器上停用它之前，其他邮件客户端仍可访问它。
admin-users-force-change-for = 强制 { $account } 更改密码
admin-users-force-change = 强制更改
admin-users-force-change-help = 用户在下次登录时以及在已打开的会话中会停留在更改密码页面，更改成功之前无法访问邮件。这需要可用的密码更改后端（MW_PASSWD_BACKEND）。如果没有，更改无法成功，用户会一直停留在该页面，直到你取消勾选此框。
admin-users-revoke-for = 撤销 { $account } 的会话

# -- Integrations ------------------------------------------------------------
admin-integrations-title = 集成
admin-integrations-load-error = 无法加载集成
admin-integrations-revoke-error = 无法撤销密钥
admin-integrations-ldap = LDAP / GAL 目录
admin-integrations-nextcloud = Nextcloud 桥接
admin-integrations-active = 活动
admin-integrations-configured = 已配置
admin-integrations-not-configured = 未配置
admin-integrations-unknown = 状态未知
admin-integrations-config-note = “已配置”表示此部署具有该集成的设置：数据库中已启用的 LDAP 目录条目，或三个 MW_NEXTCLOUD_* 环境变量。两者都在服务器启动时读取。这并不表示已联系过远程服务或该服务可以访问。
admin-integrations-webhooks = Webhook
admin-integrations-webhooks-empty = 未注册 Webhook。
admin-integrations-keys = API 和 MCP 密钥
admin-integrations-keys-empty = 未颁发密钥。
admin-integrations-col-prefix = 前缀
admin-integrations-col-account = 账户
admin-integrations-col-scopes = 范围
admin-integrations-col-status = 状态
admin-integrations-status-revoked = 已撤销
admin-integrations-status-active = 有效
admin-integrations-revoke-key = 撤销密钥 { $prefix }
admin-integrations-col-unattended = 无人值守发送
admin-integrations-unattended-not-requested = 未申请
admin-integrations-unattended-requested = 已申请，未批准
admin-integrations-unattended-approved = 已批准
admin-integrations-unattended-approve = 批准
admin-integrations-unattended-withdraw = 撤回
admin-integrations-unattended-approve-key = 批准密钥 { $prefix } 的无人值守发送
admin-integrations-unattended-withdraw-key = 撤回对密钥 { $prefix } 无人值守发送的批准
admin-integrations-unattended-approve-title = 批准此密钥的无人值守发送？
admin-integrations-unattended-withdraw-title = 撤回对此密钥的批准？
admin-integrations-unattended-owner = 所有者：{ $account }
admin-integrations-unattended-key = 密钥：{ $prefix }
admin-integrations-unattended-scope = 权限：{ $permissions }。MCP 工具：{ $tools }。
admin-integrations-unattended-scope-unreadable = 无法读取此密钥的范围。
admin-integrations-scope-read = 读取
admin-integrations-scope-send = 发送
admin-integrations-scope-delete = 删除
admin-integrations-scope-mail = 邮件
admin-integrations-scope-pim = 日历、任务、笔记和联系人
admin-integrations-scope-none = 无
admin-integrations-unattended-approve-meaning = 通过 MCP 使用此密钥发送的邮件将直接发出，无需有人放行。
admin-integrations-unattended-withdraw-meaning = 在再次批准此密钥之前，通过 MCP 使用此密钥的发送会被拒绝。邮件不会发出，也不会放入所有者的发件箱。
admin-integrations-unattended-approve-effect = 批准立即生效，从该密钥的下一次发送开始。无需重启。
admin-integrations-unattended-withdraw-effect = 撤回立即生效：从下一次发送起，该密钥不再进行无人值守发送。无需重启。
admin-integrations-unattended-cancel = 取消
admin-integrations-unattended-saving = 正在保存…
admin-integrations-unattended-approved-done = 已批准密钥 { $prefix } 的无人值守发送。现已生效。
admin-integrations-unattended-withdrawn-done = 已撤回对密钥 { $prefix } 的批准。此密钥不再进行无人值守发送。
admin-integrations-unattended-error-gone = 密钥 { $prefix } 已不存在或已被撤销。未做任何更改。
admin-integrations-unattended-error-conflict = 密钥 { $prefix } 已被撤销，或其所有者未申请无人值守发送。未批准任何内容。
admin-integrations-unattended-error = 无法更改密钥 { $prefix } 的批准状态。

# -- Observability -----------------------------------------------------------
admin-obs-title = 可观测性
admin-obs-load-error = 无法加载可观测性数据
admin-obs-export-error = 无法导出审计日志
admin-obs-ban-add-error = 无法添加封禁
admin-obs-unban-error = 无法移除封禁
admin-obs-telemetry-note = 日志和遥测不在此处设置。日志过滤器（MW_LOG）、OTLP 收集器（MW_OTLP_ENDPOINT）和指标端点（MW_METRICS_TOKEN）在服务器启动时从其环境变量中读取。
admin-obs-audit = 审计日志
admin-obs-export = 导出 JSONL
admin-obs-audit-empty = 没有审计条目。
admin-obs-col-time = 时间
admin-obs-col-actor = 操作者
admin-obs-col-action = 操作
admin-obs-col-target = 对象
admin-obs-bans = 登录监控 / 封禁列表
admin-obs-bans-note = 此列表仅作记录。Mailwoman 不会拒绝来自列表中地址的连接，条目对登录没有影响。每次管理员登录失败都会在服务器日志中写入一行，供 fail2ban jail 处理；同一地址失败五次后，该地址会被添加到此处。封锁由该 jail 或你的防火墙执行。
admin-obs-ban-add = 添加封禁
admin-obs-ban-ip = IP 地址
admin-obs-ban-reason = 原因
admin-obs-ban-btn = 封禁 IP
admin-obs-bans-empty = 没有有效的封禁。
admin-obs-unban-for = 解除封禁 { $ip }
admin-obs-unban-btn = 解除封禁

# -- Plugins (§22) -----------------------------------------------------------
# NB: the unsigned-plugin banner copy is a FROZEN, exported const (UNSIGNED_BANNER
# in Plugins/index.tsx) referenced by tests and the security model — not localised.
admin-plugins-title = 插件
admin-plugins-intro = 引擎插件在按权限控制的 WebAssembly 沙箱中运行。插件需先获批准才能启用，并且只应授予其所需的权限。
admin-plugins-empty = 没有已注册的插件。
admin-plugins-signed = 已签名
admin-plugins-unsigned = 未签名
admin-plugins-approved = 已批准
admin-plugins-enabled = 已启用
admin-plugins-approve = 批准
admin-plugins-enable = 启用
admin-plugins-disable = 停用
admin-plugins-net = 网络：{ $hosts }
admin-plugins-limits = 限制：{ $memory } MiB · { $deadline } 毫秒
admin-plugins-limits-fuel = 限制：{ $memory } MiB · { $deadline } 毫秒 · { $fuel } fuel
admin-plugins-allow-unsigned-for = 允许未签名插件 { $name }
admin-plugins-allow-unsigned = 允许运行此未签名插件
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
# The trust surface for loading non-first-party components. An operator drops a
# <id>.wasm into the third-party plugin directory; the server computes its SHA-256
# and shows it here. An admin approves that exact digest to let it load — nothing
# else does. Copy is factual: this is a security action, neither alarmist nor
# reassuring-marketing.
admin-allowlist-title = 第三方插件允许列表
admin-allowlist-intro = 第三方（非官方）组件只有在管理员批准其确切的 SHA-256 摘要后才会加载。下方的摘要由服务器根据磁盘上组件的字节计算；批准即固定这些确切的字节。官方组件在构建时已固定，不在此处管理。
admin-allowlist-load-error = 无法加载插件允许列表。
admin-allowlist-present-heading = 磁盘上的组件
admin-allowlist-present-empty = 没有第三方组件。将组件放入第三方插件目录后，它会显示在此处。
admin-allowlist-digest-label = 计算出的 SHA-256
admin-allowlist-status-approved = 已批准
admin-allowlist-status-pending = 未批准
admin-allowlist-status-firstparty = 官方
# A component approved by digest without a signature is expected — a neutral note,
# not a warning. The digest pin is what authorizes loading.
admin-allowlist-unsigned-note = 已通过摘要固定准入。此组件没有签名；批准信任的是显示其摘要的确切字节，这是以此方式批准的组件的预期状态。
# High-power capabilities are refused to third-party plugins at grant time by the
# server, regardless of admin action. Surfaced so an admin is not surprised by a
# rejected grant.
admin-allowlist-highpower-note = 高权限能力（{ $caps }）不能授予第三方插件。无论是否批准，服务器都会拒绝；它们仅供官方组件使用。
admin-allowlist-firstparty-note = 此 ID 与某个官方组件相同。官方固定始终优先，因此此 ID 不能作为第三方组件获批。
admin-allowlist-approve = 批准摘要
admin-allowlist-approve-for = 批准 { $id } 的摘要
admin-allowlist-revoke = 撤销
admin-allowlist-revoke-for = 撤销 { $id } 的固定
admin-allowlist-uninstall = 卸载
admin-allowlist-uninstall-for = 卸载 { $id }
admin-allowlist-pins-heading = 已批准和已撤销的固定
admin-allowlist-pins-empty = 没有固定记录。
admin-allowlist-pin-approved-by = 由 { $by } 于 { $at } 批准
admin-allowlist-pin-revoked = 已撤销
admin-allowlist-cancel = 取消

# Approve confirmation (shows the exact digest being trusted).
admin-allowlist-approve-title = 批准加载此组件？
admin-allowlist-approve-detail = 批准将固定 SHA-256 显示在下方的确切字节。批准后，此 ID 只会加载与该摘要匹配的字节；组件的任何改动都会产生不同的摘要，在重新批准之前不会加载。批准本身不授予任何权限。
admin-allowlist-approve-confirm = 批准摘要

# Revoke confirmation.
admin-allowlist-revoke-title = 撤销此固定？
admin-allowlist-revoke-detail = 撤销会移除对此摘要的批准并停用该插件。它在下次加载时生效；在此之前，已在运行的实例不会被停止。
admin-allowlist-revoke-confirm = 撤销固定

# Uninstall confirmation (clear about what it deletes).
admin-allowlist-uninstall-title = 卸载此插件？
admin-allowlist-uninstall-detail = 卸载会删除该插件为所有账户存储的键值数据，移除其允许列表固定，并停用它。磁盘上的组件文件不会被删除；之后可以重新批准。
admin-allowlist-uninstall-confirm = 卸载插件

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = 单点登录
admin-sso-intro = 配置 OIDC 和 SAML 2.0 登录后端。已启用的后端会在邮箱登录页面上显示为“使用…登录”按钮，范围可以是整个部署或单个域名。
admin-sso-add = 添加登录后端
admin-sso-edit = 编辑
admin-sso-create = 添加后端
admin-sso-update = 保存更改
admin-sso-cancel = 取消
admin-sso-empty = 未配置 SSO 后端。
admin-sso-load-error = 无法加载 SSO 后端。
admin-sso-save-error = 无法保存后端。
admin-sso-delete-error = 无法删除后端。

# Common fields
admin-sso-id = 后端 ID
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = 显示名称
admin-sso-display-name-placeholder = 使用 Acme SSO 登录
admin-sso-kind = 协议
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = 范围
admin-sso-scope-deployment = 整个部署
admin-sso-scope-domain = 单个域名
admin-sso-domain = 域名
admin-sso-domain-placeholder = example.org
admin-sso-enabled = 已启用
admin-sso-first-login = 首次登录策略
admin-sso-policy-allowlist = 允许列表（拒绝未知用户）
admin-sso-policy-autocreate = 首次登录时自动创建账户

# OIDC fields
admin-sso-issuer = 颁发者 URL
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = 客户端 ID
admin-sso-client-secret = 客户端密钥
admin-sso-secret-unchanged = 留空以保留已存储的密钥
admin-sso-redirect = 重定向 URL
admin-sso-scopes = 范围
admin-sso-metadata = SP 元数据

# SAML fields
admin-sso-sp-entity-id = SP 实体 ID
admin-sso-acs-url = ACS URL
admin-sso-idp-metadata-url = IdP 元数据 URL
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = IdP 元数据 XML
admin-sso-idp-metadata-xml-placeholder = 粘贴 IdP 元数据 XML，或使用上方的 URL
admin-sso-idp-sso-url = IdP SSO URL
admin-sso-idp-slo-url = IdP 退出登录（SLO）URL
admin-sso-idp-certs = IdP 签名证书（PEM）
admin-sso-idp-certs-placeholder = 每个块一个 PEM 证书，块之间用空行分隔
admin-sso-nameid-format = NameID 格式
admin-sso-want-signed = 要求断言已签名
admin-sso-want-encrypted = 要求断言已加密

# Claim map
admin-sso-claims = 声明映射
admin-sso-claim-email = 电子邮件声明
admin-sso-claim-username = 用户名声明
admin-sso-claim-display = 显示名称声明
admin-sso-claim-groups = 组声明

# List row
admin-sso-badge-enabled = 已启用
admin-sso-badge-disabled = 已停用
admin-sso-enable = 启用
admin-sso-disable = 停用
admin-sso-delete = 删除
# `name` is the backend's admin-set display name.
admin-sso-enable-for = 启用 { $name }
admin-sso-disable-for = 停用 { $name }
admin-sso-delete-for = 删除 { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
# The editor body (entry list, add form) reuses the servermeta.ftl catalog; these
# ids cover only the admin wrapper (account picker + framing).
admin-servermeta-title = 服务器元数据
admin-servermeta-intro = 查看和编辑已开通账户的 RFC 5464 服务器注解。更改会直接写入邮件服务器，由邮件服务器决定该账户是否可以设置它们。
admin-servermeta-account = 账户
admin-servermeta-select-option = 选择账户…
admin-servermeta-select-prompt = 选择一个账户以查看和编辑其服务器注解。
admin-servermeta-load-error = 无法加载账户列表。
admin-servermeta-no-accounts = 没有已开通的账户。

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
# Keys are disjoint from admin-servermeta-* (E4) — additive. This drives the
# admin-gated POST /admin/maintenance/rethread; the action is non-destructive by
# default and never fires without the explicit confirmation below.
admin-rethread-title = 重建邮箱会话
admin-rethread-intro = 对已开通账户的已存储邮件重新运行会话归组（JWZ），并重新生成其会话分组。这是一次性维护操作，不会自动运行。
admin-rethread-account = 账户
admin-rethread-select-option = 选择账户…
admin-rethread-no-accounts = 没有已开通的账户。
admin-rethread-load-error = 无法加载账户列表。
admin-rethread-run = 重建邮箱会话
admin-rethread-confirm-title = 重建此邮箱的会话？
admin-rethread-confirm-warning = 重建会话会重新生成此账户的会话分组。现有会话可能会合并或拆分，用户可能会看到会话被重新分组。
admin-rethread-confirm-detail = 这是一次性维护操作，现在运行一次；可以安全地重复运行，不会删除任何邮件。
admin-rethread-confirm = 重建邮箱会话
admin-rethread-running = 正在重建会话…
admin-rethread-cancel = 取消
admin-rethread-summary = 已将 { $accounts } 个账户中的 { $messages } 封邮件归入 { $threads } 个会话；{ $reassigned } 封邮件被移到了其他会话。
admin-rethread-error = 重建会话操作失败。如果服务器拒绝了请求，会话分组不会被更改；请检查服务器日志后重试。

# -- Search index: status and rebuild (t28-e13) --------------------------------
# GET /admin/maintenance/search-index and POST /admin/maintenance/reindex.
admin-searchindex-title = 搜索索引
admin-searchindex-intro = 搜索读取根据已存储邮件构建的索引。当索引与已存储邮件不一致时，服务器会在启动时重建它。零访问账户的邮件不会被索引，因此搜索在这些账户中找不到任何内容。
admin-searchindex-count = 已存储的 { $messages } 封邮件中有 { $documents } 封已编入索引。
admin-searchindex-on-disk = 索引保存在磁盘上。
admin-searchindex-in-memory = 索引保存在内存中，每次服务器启动时都会重建。
admin-searchindex-progress = 正在重建：已完成 { $done } / { $total } 封邮件。
admin-searchindex-run = 重建搜索索引
admin-searchindex-running = 正在重建…
admin-searchindex-summary = 已读取 { $accounts } 个账户中已存储的 { $messages } 封邮件：{ $indexed } 封已编入索引，{ $removed } 封已从索引中移除，{ $failed } 封无法读取。{ $zeroAccess } 个零访问账户未编入索引。
admin-searchindex-busy = 已有重建正在运行。请等待其完成。
admin-searchindex-error = 重建失败。索引保留原有内容；请检查服务器日志后重试。
admin-searchindex-status-error = 无法读取搜索索引状态。
admin-searchindex-unavailable = 此服务器以代理模式运行，没有搜索索引。

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
# The require-2FA policy (global / per-domain). Any user may enrol a factor on
# their own; this panel governs where a second factor is REQUIRED. A required but
# not-yet-enrolled account is prompted to enrol on its next sign-in.
admin-2fa-title = 要求双重身份验证
admin-2fa-intro = 要求登录时使用第二因素（通行密钥或身份验证器应用）。任何用户都可以自行注册因素；在此处设为必需后，范围内的账户必须在下次登录时注册。
admin-2fa-load-error = 无法加载双重身份验证策略。
admin-2fa-save-error = 无法保存双重身份验证策略。
admin-2fa-global = 对整个部署要求双重身份验证
admin-2fa-global-label = 对整个部署要求双重身份验证
admin-2fa-global-note = 开启后，每个账户都必须有第二因素。下方的按域名规则也可以只对某个域名要求，而不在所有地方要求。
admin-2fa-domains-heading = 按域名的要求
admin-2fa-domains-empty = 未设置按域名的要求。
admin-2fa-col-domain = 域名
admin-2fa-col-require = 必需
admin-2fa-require-for = 对 { $domain } 要求双重身份验证
admin-2fa-add-domain = 域名
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = 对此域名要求双重身份验证
admin-2fa-add-require-label = 对此域名要求双重身份验证
admin-2fa-add-save = 添加域名规则

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = 出站
admin-egress-heading = 出站代理路由
admin-egress-intro = 将服务器的出站请求通过 HTTP CONNECT 或 SOCKS5 代理转发。Mailwoman 仍执行自己的地址策略 — 从不让代理解析名称。
admin-egress-empty = 未配置路由。出站请求直接发出。
admin-egress-none-active = 没有正在使用的路由。出站请求直接发出。已保存的路由只有在你选择“使用此路由”后才会承载流量。
admin-egress-in-use = 出站请求使用路由“{ $id }”。
admin-egress-deactivate = 停止使用代理
admin-egress-activate = 使用此路由
admin-egress-activate-for = 使用路由 { $id }
admin-egress-activate-error = 无法切换到该路由
admin-egress-deactivate-error = 无法停止使用该路由
admin-egress-col-state = 状态
admin-egress-state-active = 使用中
admin-egress-state-inactive = 未使用
admin-egress-test-scope = 只能测试正在使用的路由。测试会通过该路由获取一个固定 URL，并报告进行到哪一步；不会更改任何内容。
admin-egress-probe-url = 已获取：{ $url }
admin-egress-load-error = 无法加载出站路由
admin-egress-save-error = 无法保存路由
admin-egress-delete-error = 无法删除路由
admin-egress-invalid = 需要 ID、主机和大于零的端口。
admin-egress-username-required = 设置密码需要用户名；没有用户名，该路由将无法以任何身份进行认证。
admin-egress-col-id = ID
admin-egress-col-route = 路由
admin-egress-col-username = 用户名
admin-egress-col-credentials = 密码
admin-egress-col-plaintext = 明文
admin-egress-col-actions = 操作
admin-egress-cred-set = 已设置
admin-egress-cred-none = 无
admin-egress-plaintext-on = 允许
admin-egress-plaintext-off = 拒绝
admin-egress-add-heading = 添加路由
admin-egress-edit-heading = 编辑路由
admin-egress-id = ID
admin-egress-scheme = 协议
admin-egress-host = 主机
admin-egress-port = 端口
admin-egress-username = 用户名
admin-egress-password = 密码
admin-egress-password-unchanged = 留空以保留已存储的密码
admin-egress-allow-plaintext = 允许明文 http 源站通过此路由
admin-egress-allow-plaintext-note = 默认关闭。https 源站可防止代理读取其承载的流量；允许 http 意味着代理可以看到这些流量。
admin-egress-test = 测试
admin-egress-testing = 正在测试…
admin-egress-test-failed = 无法运行测试，因此无法得知此路由的任何情况。
admin-egress-outcome-connected = 已连接
admin-egress-outcome-auth-rejected = 认证被拒绝
admin-egress-outcome-refused-by-policy = 被策略拒绝
admin-egress-outcome-dns-failed = 名称无法解析
admin-egress-outcome-unreachable = 无法访问
admin-egress-outcome-origin-tls-failed = 源站 TLS 失败
admin-egress-outcome-route-invalid = 路由无效
admin-egress-outcome-unknown = 无法识别的结果
admin-egress-stage-dns = 在名称解析时停止
admin-egress-stage-connect = 在连接代理时停止
admin-egress-stage-tunnel = 在建立隧道时停止
admin-egress-stage-origin = 在源站处停止
admin-egress-stage-unknown = 在无法识别的阶段停止
admin-egress-proxied-yes = 已经过代理
admin-egress-proxied-no = 未经过代理
