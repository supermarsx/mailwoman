# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = 入站访问
admin-inbound-help = 分别限制管理控制台和网页邮箱的可信来源 IP。两项策略默认均为禁用。这些规则不会更改受信任的代理、API 范围或出站访问。
admin-inbound-unscoped = 此部署范围的策略需要具有不受范围限制的安全访问权限的管理员。
admin-inbound-admin = 管理控制台和 API
admin-inbound-admin-help = 包括登录和此编辑器。现有的管理员认证不会绕过网络策略。
admin-inbound-webmail = 网页邮箱和账户 API
admin-inbound-webmail-help = 涵盖邮件、DAV、签名内容、SSO、OIDC 和 API 路由。已升级的现有连接保留其握手时的决定。
admin-inbound-enabled = 将此界面限制在允许列表内
admin-inbound-cidrs = 救援 IP 和 CIDR
admin-inbound-cidrs-help = 每行一个字面 IP 或 CIDR，最多 128 个。启用时至少需要一个。请包含一个可用于恢复的可信来源。
admin-inbound-hostnames = 允许的 DNS 主机名
admin-inbound-hostnames-help = 每行一个小写 ASCII DNS 名称。国际化名称请使用 IDNA ASCII 形式。不要使用 URL、端口、通配符、IP 字面量或末尾的点。名称仅授予其当前解析出的地址。
admin-inbound-dns = DNS 解析限制
admin-inbound-dns-help = 新的、已过期的和失败的名称不授予任何访问权限。成功的应答遵循其 DNS TTL 和所配置的缓存上限。请求绝不会等待 DNS 解析。
admin-inbound-servers = 递归 DNS 服务器
admin-inbound-servers-help = 留空则使用系统解析器，或输入最多四个 IP:端口套接字，每行一个（例如 10.0.0.53:53 或 [fd00::53]:53）。显式指定的服务器会取代系统解析器，且没有公共回退。解析器的信任由你负责；不声称提供独立的 DNSSEC 验证。
admin-inbound-dns-timeoutMs = 查询超时（毫秒）
admin-inbound-dns-cacheSeconds = 最长缓存时间（秒）
admin-inbound-dns-maxHosts = 不同主机名的最大数量
admin-inbound-dns-maxAddresses = 每个主机名的最大地址数
admin-inbound-dns-concurrency = 并发查询数
admin-inbound-bounds = 允许范围：{ $min }–{ $max }。
admin-inbound-lockout = 此策略可能会立即阻止你自己的访问，包括此编辑器。请确认救援 IP 涵盖了由所配置的受信任代理确定的你的实际来源。没有自动的回环或已登录管理员绕过机制。要从锁定中恢复，需要在部署端还原；普通登录无法绕过它。
admin-inbound-confirm-title = 应用入站访问策略？
admin-inbound-confirm-word = APPLY
admin-inbound-save = 应用已确认的策略
admin-inbound-review = 查看更改
admin-inbound-refresh = 刷新已保存的策略和 DNS 状态
admin-inbound-reset = 将草稿重置为已加载的策略
admin-inbound-saved = 服务器已接受入站访问策略。
admin-inbound-conflict = 你在编辑期间，已保存的策略发生了变化。请重置草稿并刷新以加载其当前版本，然后再继续编辑。
admin-inbound-no-revision = 服务器没有返回策略版本。请先刷新再保存。
admin-inbound-status = 当前的 DNS 授权
admin-inbound-status-help = 这是上次状态刷新时的快照，不是倒计时，也不是连通性探测。请刷新以查看当前授权。在原有过期时间之前，错误可能与仍然有效的先前应答同时存在。
admin-inbound-refreshing = 正在进行有限的 DNS 刷新。
admin-inbound-idle = 上次读取状态时没有正在运行的 DNS 刷新。
admin-inbound-no-hosts = 已启用的策略中没有生效的主机名。
admin-inbound-hostname = 主机名
admin-inbound-state = 访问授权
admin-inbound-addresses = 可用地址
admin-inbound-validity = 上次刷新时的有效期
admin-inbound-error = DNS 结果代码
admin-inbound-usable = 可用
admin-inbound-unavailable = 没有当前授权
admin-inbound-seconds = { $seconds } 秒
admin-inbound-on = 已启用
admin-inbound-off = 已禁用
admin-inbound-invalid-dns = DNS 限制或解析器数量超出允许范围。
admin-inbound-rescue-required = 每个已启用的界面都至少需要一个救援 IP 或 CIDR。
admin-inbound-too-many = 该策略超过了所配置的主机名或字面地址上限。
admin-inbound-invalid-host = 请使用小写 ASCII DNS 名称，不含 URL、端口、通配符、IP 字面量或末尾的点。
