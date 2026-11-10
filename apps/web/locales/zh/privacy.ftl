# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: privacy. Source: ../en/privacy.ftl


# -- Machine translation (2026-11): added to match en ---------------------
privacy-title = 隐私
privacy-help = 远程图片可能暴露你何时打开了邮件。请选择它们何时加载,以及是否由服务器为你获取。
privacy-remote-mode = 远程内容
privacy-mode-block = 阻止远程内容
privacy-mode-ask = 通过服务器加载前先询问
privacy-mode-proxy = 通过服务器加载
privacy-mode-direct = 直接从发件人的服务器加载
privacy-trackers = 阻止跟踪图片和信标
privacy-clean-links = 移除链接中的跟踪参数
privacy-direct-unavailable = 直接加载需要管理员授权。这会向图片主机暴露你的网络地址。
privacy-direct-native = 此原生应用不支持直接加载。图片可通过服务器加载。
privacy-inherit = 使用继承的设置
privacy-content-failed = 部分图片无法授权。这些图片被阻止,但邮件仍可阅读。
privacy-frame-failed = 隔离的图片查看器无法加载。请重新加载邮件后再试。
privacy-download-file = 通过服务器下载
privacy-file-failed = 无法通过服务器下载此文件。
privacy-admin-title = 隐私与远程内容
privacy-admin-help = 通过设置策略来设定默认值、锁定和限制。诊断计数仅涵盖此服务器进程。
privacy-cache-entries = 已缓存的图片
privacy-cache-bytes = 缓存大小(字节)
privacy-replaced-trackers = 已替换的跟踪像素
privacy-edit-setting = 编辑策略
privacy-outbound-title = 出站连接
privacy-fail-closed = 已配置的路由失败时会阻止请求,绝不会回退为直接连接。
privacy-linked-files = 此邮件中链接的文件
privacy-allow-direct = 允许直接加载图片
privacy-max-bytes = 图片大小上限(字节)
privacy-cache-limit = 缓存大小上限(字节)
privacy-rate = 每分钟请求数
privacy-on = 开
privacy-off = 关
privacy-locked = 已被管理员策略锁定
privacy-process-stats = 自此服务器进程启动以来
privacy-refresh = 刷新
privacy-traffic-class = 流量类别
privacy-allowed = 已允许
privacy-refused = 被策略拒绝
privacy-failed = 失败
privacy-policy-values = 远程内容策略
privacy-cache-summary = 缓存与跟踪保护
privacy-traffic-summary = 出站流量处理
privacy-metric = 指标
privacy-value = 值
privacy-total-duration = 总时间
privacy-no-traffic = 自此进程启动以来,未记录任何出站流量。
privacy-analytics-title = 可选的使用计数
privacy-analytics-help = 默认关闭。如果管理员启用了收集器,你可以共享固定的屏幕名称,例如“邮件”或“日历”。绝不会发送邮件、电子邮件地址、搜索内容、路由详情和浏览器标识符。这些是事件计数,不是独立访客统计。
privacy-analytics-consent = 允许此账户提供可选的屏幕使用计数
privacy-analytics-signal = 你的浏览器隐私信号会阻止分析,即使已保存同意设置。
privacy-load-time-title = 加载时间
privacy-load-time-help = 帮助管理员了解 Mailwoman 用起来有多快。只会以匿名汇总的方式报告毫秒数,以及数据来自缓存还是网络。不包含文件夹、邮件、发件人或主题。
privacy-load-time-folder = 测量文件夹打开所需的时间
privacy-load-time-folder-help = 默认开启。报告从选择文件夹到显示其邮件列表的时间。
privacy-load-time-message = 测量邮件打开所需的时间
privacy-load-time-message-help = 默认开启。报告从打开邮件到显示其邮件头和正文的时间。
privacy-analytics-save-error = 无法同步同意设置。在更改保存之前,此浏览器会暂停新的分析请求。
privacy-analytics-saving = 正在保存同意设置…
privacy-back-policies = 返回隐私策略
privacy-image-toast = 远程图片加载时显示通知
privacy-image-downloads = 手动下载远程图片
privacy-image-download-help = 如果图片未显示,可通过此服务器下载经过检查的副本。服务器会验证它属于此邮件,并仍会应用其隐私和安全规则。此处不支持客户端解密的图片。
privacy-image-download-ready = 图片已就绪,下载已开始。
privacy-image-download-failed = 无法下载图片。它可能不可用,或被你的隐私策略阻止。
privacy-strip-mailer-headers = 移除外发邮件的邮件程序头
privacy-strip-mailer-help = 原生 SMTP 和 EWS 发送会移除顶层的客户端指纹头。邮件正文、附带的邮件和报告内容保持不变。这不包括上游 JMAP 代理发送,也不包括中继之后添加的头。覆盖了被移除头的预签名邮件,需要先禁用此设置才能发送。
privacy-frame-compatible = 兼容查看器:出于隐私考虑,直接从其他站点加载的图片仍被阻止。支持内嵌图片和通过服务器加载的图片。
privacy-analytics-tag-custom = 自定义分析脚本
privacy-analytics-tag-title = 浏览器分析:{ $provider }
privacy-analytics-tag-help = 管理员还提供了 { $provider },它会在你浏览器中的此页面里运行。与上面的计数不同,它的脚本可以看到你在此应用中打开的页面,包括屏幕上显示的邮件和地址、你的浏览器和 IP 地址,并且可能设置自己的标识符。服务提供方可以在不再次征求你同意的情况下更新其脚本。除非你在此开启,否则保持关闭;关闭后会重新加载页面且不带该脚本,所有标签页均如此。
privacy-analytics-tag-consent = 允许 { $provider } 为此账户在此应用中运行
privacy-analytics-tag-changed = 自你上次同意以来,分析服务提供方或此服务器加载它的方式已更改。在你再次同意之前,它将保持关闭。
privacy-trusted-links-title = 受信任的链接域名
privacy-trusted-links-help = 指向这些域名的链接会直接从“查看链接”中打开,无需目标确认步骤。其他链接(包括外观相似的域名)仍会被检查。使用纯 HTTP 或 IP 地址的链接始终会被检查。
privacy-trusted-links-domains = 域名
privacy-trusted-links-invalid = 最多输入 100 个域名,每个至少包含两级标签,且不含协议、路径、端口或通配符。未被接受:{ $entries }
privacy-trusted-links-save = 保存受信任的域名
privacy-trusted-links-subdomains = 同时信任子域名(信任 example.com 时包括 mail.example.com)
privacy-trusted-links-subdomains-help = 关:仅信任确切的域名。开:任何以 .example.com 结尾的名称也受信任。
privacy-linked-files-count = { $count ->
   *[other] 此邮件中链接了 { $count } 个文件
}
privacy-linked-links = 此邮件中的链接
privacy-linked-files-show = 显示文件
privacy-linked-links-show = 下载链接
privacy-linked-files-help = 下载通过此服务器进行,因此文件主机看不到你的地址。文件大小只有在下载后才能知道。
privacy-linked-links-help = 此邮件中的链接可以通过此服务器作为文件下载,因此其主机看不到你的地址。
privacy-linked-file-item = 从 { $host } 下载 { $name }
privacy-linked-other-item = 下载 { $link }
privacy-links-enable = 启用链接
privacy-links-options = 链接选项
privacy-links-enable-once = 仅启用一次
privacy-links-trust-sender = 始终允许此发件人的链接
privacy-links-revoke-sender = 不再始终允许此发件人的链接
privacy-links-trust-failed = 无法保存链接偏好设置。
privacy-trusted-link-senders-title = 受信任的链接发件人
privacy-trusted-link-senders-help = 对这些确切的发件人地址自动允许邮件中的链接。链接目标仍会经过验证。
privacy-trusted-link-senders-list = 发件人地址
privacy-trusted-link-senders-list-help = 每行一个完整的电子邮件地址。@ 之前的部分按精确匹配(包括大小写)。删除地址即可撤销信任。
privacy-trusted-link-senders-save = 保存受信任的发件人
privacy-trusted-link-senders-remove = 移除 { $sender }
privacy-trusted-link-senders-invalid = 请输入有效的单个电子邮件地址:{ $entries }
privacy-links-disable = 禁用链接
privacy-links-enable-help = 允许此邮件中的链接在新标签页中打开。你可以再次禁用它们。
privacy-links-loading = 正在准备链接…
privacy-body-blocked = 出于安全考虑,已阻止邮件 HTML
privacy-body-blocked-detail = 此正文包含可能不安全的内容,或无法检查。活动内容仍被阻止。原始邮件保持不变。
privacy-body-show-sanitized = 显示已净化的 HTML
privacy-body-hide-sanitized = 显示安全文本
privacy-body-sanitized-shown = 正在显示已净化的 HTML
privacy-unsafe-html = 可能不安全的邮件 HTML
privacy-unsafe-html-ask = 显示已净化的 HTML 前先询问
privacy-unsafe-html-strict = 始终显示安全文本
privacy-unsafe-html-sanitized = 始终显示已净化的 HTML
privacy-unsafe-html-help = 已净化的 HTML 会移除活动内容。这些邮件的远程资源、链接和自动回执仍被阻止。原始邮件保持不变。
privacy-original-html-allow = 允许按邮件查看原始 HTML
privacy-original-html-help = 为每封邮件提供明确的“原始 HTML”视图。脚本和表单仍被禁用,图片和链接仍遵循你的隐私控制。默认仍为已净化的 HTML。
privacy-original-html = 原始 HTML
privacy-original-html-show = 显示原始 HTML
privacy-original-html-hide = 返回已净化的视图
privacy-original-html-active = 正在显示原始 HTML — 脚本已禁用,资源受控
privacy-original-html-offer = 仅对这封邮件保留其原始版式。
privacy-tracker-choice = 允许按邮件选择是否加载跟踪器
privacy-tracker-choice-help = 提供一个明确的操作,可为单封邮件加载被阻止的跟踪器。在你选择之前,跟踪器保持被阻止,管理员也可以强制要求保护。
privacy-image-retry-attempts = 图片自动重试次数
privacy-image-retry-help = 仅在当前图片加载完成后,对暂时性的图片失败进行重试。设为零则禁用自动重试;你仍可手动重试。管理员锁定优先。
privacy-image-retry-delay = 图片重试延迟(秒)
privacy-concurrency-title = 自适应图片容量
privacy-concurrency-mode-auto = 自动
privacy-concurrency-mode-conservative = 低资源
privacy-concurrency-mode-balanced = 均衡
privacy-concurrency-mode-throughput = 高吞吐
privacy-concurrency-mode-manual = 手动
privacy-concurrency-apply = 应用配置
privacy-concurrency-saved = 图片并发配置已保存。
privacy-concurrency-unavailable = 不可用
privacy-concurrency-sizing = CPU 核心:{ $cpu } · 内存限制:{ $limit } · 可用内存:{ $available }
privacy-concurrency-running = 当前配置:{ $mode } · 自适应调整:{ $state }
privacy-concurrency-cap-perMessage = 每封邮件
privacy-concurrency-cap-perAccount = 每个账户
privacy-concurrency-cap-global = 全局网络
privacy-concurrency-cap-pipeline = 处理管线
privacy-concurrency-cap-decoders = 解码器
privacy-concurrency-effective-ceiling = 有效 { $effective } / 上限 { $ceiling }
privacy-concurrency-message-bound = 每封邮件的值是实例级上限。每封邮件最多使用该上限或其已批准的图片数量,取较小者。
privacy-concurrency-active = 活动:{ $network } 个网络 · { $pipeline } 条管线 · { $decoders } 个解码器 · { $queued } 个排队中
privacy-image-rate-readout = 每账户获取额度({ $mode }):突发 { $burst }(已配置 { $configuredBurst }),每分钟补充 { $perMin }(已配置 { $configuredPerMin })
privacy-image-cache-readout = 图片缓存({ $mode }):最多 { $bytes } 和 { $entries } 个条目,生存期 { $ttl } 秒,命中率 { $hit }
privacy-image-hit-unknown = 查询次数尚不足
privacy-tab-overview = 概览
privacy-tab-remote = 远程内容与跟踪器
privacy-tab-image-proxy = 图片代理
privacy-tab-fetch-limits = 获取限制
privacy-tab-traffic = 流量
privacy-tab-cookies = Cookie
privacy-tab-denied = 你无权访问此标签页。
privacy-tab-remote-help = 远程内容、跟踪保护和链接清理的默认值、锁定和按域名的值。每项设置单独保存。
privacy-settings-denied = 读取设置需要 settings.read 权限,而你没有该权限。其他标签页上的统计信息仍然可用。
privacy-overview-status = 状态
privacy-overview-figures = 关键数据
privacy-status-ok = 图片代理正在报告数据。
privacy-status-unavailable = 无法读取图片代理统计信息。
privacy-history-title = 图片代理随时间的变化
privacy-history-worker-title = 工作进程历史
privacy-history-cache-title = 缓存与限制
privacy-history-traffic-title = 出站流量
privacy-history-empty = 此时间范围内尚无样本。
privacy-history-unavailable = 此服务器无法提供历史记录。
privacy-history-summary = { $series }:最新 { $latest },最高 { $max }。
privacy-history-range = 时间范围
privacy-history-range-15m = 15 分钟
privacy-history-range-1h = 1 小时
privacy-history-range-6h = 6 小时
privacy-history-range-24h = 24 小时
privacy-history-updated = 更新于 { $time }
privacy-history-not-updated = 尚未更新
privacy-history-memory-note = 服务器每 { $interval } 秒保留一个样本,仅在内存中保存 24 小时。重启后历史记录为空,且不会在服务器进程之间共享。
privacy-chart-requests = 已提供的图片
privacy-chart-requests-unit = 每秒请求数
privacy-chart-bytes = 已提供的数据
privacy-chart-bytes-unit = 每秒字节数
privacy-chart-hit = 缓存命中率
privacy-chart-hit-unit = 占查询的百分比
privacy-chart-ram = 内存
privacy-chart-disk = 磁盘
privacy-chart-queue = 进行中和等待中的获取
privacy-chart-queue-unit = 数量
privacy-chart-active = 进行中
privacy-chart-queued = 等待中
privacy-chart-busy = 繁忙应答
privacy-chart-busy-unit = 每个间隔的应答数
privacy-chart-trackers = 已替换的跟踪像素
privacy-chart-trackers-unit = 每个间隔的数量
privacy-chart-size = 缓存大小
privacy-chart-size-unit = 占用字节数
privacy-chart-entries = 缓存条目
privacy-chart-entries-unit = 数量
privacy-chart-wait = 速率限制等待
privacy-chart-wait-unit = 每个等待请求的平均毫秒数
privacy-chart-rate = 速率限制预算
privacy-chart-rate-unit = 突发额度和活动账户
privacy-chart-burst = 突发额度
privacy-chart-accounts = 活动账户
privacy-chart-allowed = 已允许的请求
privacy-chart-refused = 被拒绝和失败的请求
privacy-chart-traffic-unit = 每个间隔的数量
privacy-image-group-rate = 请求速率
privacy-image-group-cache = 缓存
privacy-image-group-ram = 内存层
privacy-image-group-disk = 磁盘层
privacy-image-group-tiering = 分层与维护

# -- Machine translation (2026-11): added to match en ---------------------
privacy-trusted-links-domains-help = 每行一个域名,例如 example.com。匹配是精确的:example.com 不会信任 examples.com 或 example.com.other.net。国际化域名请使用 ASCII 形式(xn--)。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-error-reports-help = 当 Mailwoman 的一部分出现故障时，浏览器可以向管理员发送简短的技术报告，以便进行修复。它保存错误类型、应用程序中发生错误的位置、应用程序版本和浏览器类型。它从不保存消息内容、地址、附件名称或您键入的内容。

privacy-error-reports-send = 从此浏览器发送错误报告

privacy-error-reports-send-help = 默认开启。关闭以将这些报告保留在您的设备上。错误面板仍然显示您可以复制的参考。

privacy-error-reports-title = 错误报告

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-links-revoke-domain = 停止始终允许来自该发件人域的链接

privacy-links-trust-domain = 始终允许来自此发件人域的链接

privacy-trusted-link-sender-domains = 发件人域

privacy-trusted-link-sender-domains-help = 允许来自这些确切发件人域的邮件中经过审阅的链接。每行一个域；子域是分开的。这不信任目标网站或加载图像。空列表要求提供每条消息。

privacy-trusted-link-sender-domains-save = 保存受信任的发件人域

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
privacy-feedback-image-host = 图像主机首选项已保存。

privacy-feedback-images-off = 图片权限已删除。

privacy-feedback-images-on = 已保存图像权限。现在可以加载允许的图像。

privacy-feedback-link-domain-off = 已删除{ $address }的域链接信任。

privacy-feedback-link-domain-on = 来自域{ $address }的消息中将允许链接。

privacy-feedback-link-sender-off = 删除了{ $address }的发件人链接信任。

privacy-feedback-link-sender-on = 来自{ $address }的消息中允许存在链接。

privacy-feedback-links-off = 此消息的链接已禁用。

privacy-feedback-links-on = 为此消息启用链接。
