# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: mail-dsn. Source: ../en/mail-dsn.ftl


# -- Machine translation (2026-11): added to match en ---------------------
dsn-title = 投递状态报告
dsn-unverified = 此报告描述发送方的投递尝试，其内容未经独立验证。已投递并不代表邮件已被阅读。
dsn-reporting-mta = 报告服务器：
dsn-original-recipient = 原始收件人：
dsn-remote-mta = 远程服务器：
dsn-last-attempt = 最后一次尝试：
dsn-retry-until = 发送方计划重试至：
dsn-action-failed = 报告失败
dsn-action-delayed = 报告延迟
dsn-action-delivered = 报告已投递
dsn-action-relayed = 报告已转发
dsn-action-expanded = 报告列表已展开

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
dsn-advice-0 = 向您的电子邮件提供商寻求帮助时，请使用服务器诊断。

dsn-advice-1 = 检查发件人和收件人地址。如有必要，请确认收件人的当前地址。

dsn-advice-2 = 如果邮箱问题仍然存在，请通过其他方式联系收件人。

dsn-advice-3 = 如果问题仍然存在，请与接收邮件系统的管理员共享此报告。

dsn-advice-4 = 如果问题仍然存在，请让您的邮件管理员检查连接、DNS 和转发路由。

dsn-advice-5 = 与您的电子邮件提供商共享诊断结果，以便他们可以检查SMTP兼容性和限制。

dsn-advice-6 = 根据收件人的要求检查邮件格式、国际地址和附件。

dsn-advice-7 = 与您的邮件管理员共享诊断信息，以确定确切的安全或身份验证规则。

dsn-advice-success = 检查上面报告的传递操作，了解消息是否已传递、中继或扩展到列表。

dsn-category-0 = 服务器提供了无法识别的或特定于提供商的详细代码。下面的诊断可以解释确切的原因。

dsn-category-1 = 这是一个与地址相关的代码。精确的细节是特定于提供商的或不被认可的。

dsn-category-2 = 这是收件人邮箱代码。精确的细节是特定于提供商的或不被认可的。

dsn-category-3 = 这是接收邮件系统代码。精确的细节是特定于提供商的或不被认可的。

dsn-category-4 = 这是网络或路由代码。精确的细节是特定于提供商的或不被认可的。

dsn-category-5 = 这是邮件协议代码。精确的细节是特定于提供商的或不被认可的。

dsn-category-6 = 这是消息内容或格式代码。精确的细节是特定于提供商的或不被认可的。

dsn-category-7 = 这是安全或策略代码。精确的细节是提供商特定的或不被认可的；它本身并不能确定该邮件是垃圾邮件。

dsn-class-2 = 成功：服务器报告肯定的传送步骤。这并不能证明该消息已被阅读。

dsn-class-4 = 暂时失败：稍后尝试可能会成功。发送者仍可重试延迟报告；失败报告意味着尝试已经结束。

dsn-class-5 = 永久失败：不改变地重新发送相同的消息不太可能起作用。再次发送之前更正原因。

dsn-meaning-title = 这段代码的含义

dsn-next-step = 该怎么办

dsn-provider-barracuda = 梭鱼

dsn-provider-detected = 基于服务器诊断或主机名的解释：

dsn-provider-exchange = 微软交换/微软365

dsn-provider-fortimail = 富蒂邮件

dsn-provider-gmail = Gmail/Google Workspace

dsn-provider-help = 提供商文档

dsn-provider-mimecast = 模仿秀

dsn-provider-proofpoint = 证明点

dsn-provider-relay = SMTP继电器

dsn-provider-yahoo = 雅虎/美国在线

dsn-server-diagnostic = 服务器诊断

dsn-status-0-0 = 服务器没有给出更具体的原因。

dsn-status-1-0 = 电子邮件地址有问题。

dsn-status-1-1 = 收件人邮箱不存在。与收件人确认地址。

dsn-status-1-10 = 收件人域明确声明不接受电子邮件（null MX）。使用另一个地址。

dsn-status-1-2 = 收件人域无法接收此邮件。检查@后面的部分。

dsn-status-1-3 = 收件人地址的格式无效。检查是否有拼写错误或多余空格。

dsn-status-1-4 = 服务器无法识别该地址的唯一收件人。

dsn-status-1-5 = 收件人地址被视为有效。

dsn-status-1-6 = 该邮箱已移动，没有可用的转发地址。询问新地址。

dsn-status-1-7 = 发件人地址的格式无效。检查帐户的发送地址。

dsn-status-1-8 = 发件人域名无效或无法收到退回邮件。

dsn-status-1-9 = 该消息被传递到无法提供进一步传送报告的系统。

dsn-status-2-0 = 收件人邮箱存在未指定的问题。

dsn-status-2-1 = 收件人邮箱已禁用或不接受邮件。通过其他方式联系收件人。

dsn-status-2-2 = 收件人邮箱已满。收件人需要释放空间或增加配额。

dsn-status-2-3 = 邮件超出了收件人的大小限制。减少附件或使用共享链接。

dsn-status-2-4 = 服务器无法解析邮件列表的成员。联系列表所有者。

dsn-status-3-0 = 接收邮件系统报告了未指定的问题。

dsn-status-3-1 = 接收邮件服务器的存储空间不足。其管理员需要释放空间。

dsn-status-3-2 = 接收服务器不接受邮件，例如在维护或过载期间。

dsn-status-3-3 = 接收系统不支持该消息所需的功能。

dsn-status-3-4 = 消息超出了接收服务器的大小限制。减少附件或使用共享链接。

dsn-status-3-5 = 邮件服务器配置问题导致无法投递。请邮件管理员进行调查。

dsn-status-3-6 = 服务器接受了该消息，但更改了其请求的传送优先级。

dsn-status-4-0 = 报告了网络或邮件路由问题，但没有提供更多详细信息。

dsn-status-4-1 = 接收服务器没有应答连接尝试。

dsn-status-4-2 = 连接已建立，但在邮件传输完成之前中断。

dsn-status-4-3 = DNS 或目录查找失败。邮件管理员可能需要检查名称解析。

dsn-status-4-4 = 服务器找不到通往收件人邮件系统的路由。

dsn-status-4-5 = 邮件交通拥堵导致无法投递。

dsn-status-4-6 = 检测到转发循环。请管理员检查转发和路由规则。

dsn-status-4-7 = 交货期限已过。在重新发送之前检查根本问题的诊断。

dsn-status-5-0 = 邮件服务器无法完成其传递协议交换。

dsn-status-5-1 = 邮件服务器拒绝了不受支持的命令或以错误顺序发送的命令。

dsn-status-5-2 = 邮件服务器无法理解命令的语法。

dsn-status-5-3 = 该消息的收件人过多。发送给较小的团体。

dsn-status-5-4 = 邮件服务器拒绝了命令的参数。

dsn-status-5-5 = 服务器使用不兼容版本的邮件协议。

dsn-status-5-6 = 身份验证响应对于服务器来说太长。请邮件管理员检查身份验证设置。

dsn-status-6-0 = 服务器报告消息内容或格式存在未指定的问题。

dsn-status-6-1 = 接收系统不支持该消息的内容类型。

dsn-status-6-10 = 此过时的代码意味着需要国际 (UTF-8) 回复，但不受支持，就像代码 x.6.8 一样。

dsn-status-6-2 = 传送需要进行发件人或服务器策略禁止的内容转换。

dsn-status-6-3 = 传送需要服务器无法执行的内容转换。

dsn-status-6-4 = 内容转换丢失了一些信息。检查原始消息和服务器诊断。

dsn-status-6-5 = 服务器在转换消息内容时失败。

dsn-status-6-6 = 服务器无法从另一个系统检索消息内容。

dsn-status-6-7 = 此发件人或收件人不支持国际电子邮件地址。询问替代地址。

dsn-status-6-8 = 服务器需要发送系统无法接受的国际（UTF-8）回复。

dsn-status-6-9 = 该邮件包含无法传递给至少一个收件人的国际 (UTF-8) 标头。

dsn-status-7-0 = 安全规则或邮件策略阻止了正常投递。检查服务器诊断以了解确切的规则。

dsn-status-7-1 = 服务器拒绝发送许可，可能是因为过滤或中继限制。请邮件管理员检查诊断。

dsn-status-7-10 = 服务器在身份验证之前需要加密连接或更强的保护。

dsn-status-7-11 = 此身份验证方法需要加密的SMTP连接（TLS）。

dsn-status-7-12 = 该帐户需要身份验证方法或密码迁移。联系帐户提供商。

dsn-status-7-13 = 发送帐户已被禁用。请联系其管理员以恢复访问权限。

dsn-status-7-14 = 服务器需要配置信任关系来检索消息。联系其管理员。

dsn-status-7-15 = 消息优先级低于接收服务器当前的最低优先级。

dsn-status-7-16 = 该消息对于其请求的优先级而言太大。减小其大小或询问管理员。

dsn-status-7-17 = 自要求日期以来，收件人邮箱已更改所有权。再次发送前请确认收件人。

dsn-status-7-18 = 自要求日期以来，收件人域的所有权已更改。再次发送前请确认目的地。

dsn-status-7-19 = 服务器无法验证收件人地址的持续所有权。询问管理员有关收件人有效性检查的信息。

dsn-status-7-2 = 不允许发件人在此邮件列表中发帖。联系列表所有者。

dsn-status-7-20 = 没有DKIM签名通过验证。请发件人的邮件管理员检查签名和DNS。

dsn-status-7-21 = DKIM签名已通过，但接收服务器的策略不接受它。

dsn-status-7-22 = 没有接受的DKIM签名与发件人地址匹配。请邮件管理员检查签名域对齐情况。

dsn-status-7-23 = SPF 失败：发送服务器未获得发件人域的 SPF 策略授权。

dsn-status-7-24 = 服务器在检查SPF时遇到错误。请邮件管理员检查SPF记录和DNS。

dsn-status-7-25 = 发送服务器未通过接收方的反向-DNS 检查。其管理员需要检查IP地址的DNS记录。

dsn-status-7-26 = 多次消息身份验证检查失败。该诊断可能会识别 SPF、DKIM 或 DMARC 策略问题。

dsn-status-7-27 = 发件人域声明不接受返回电子邮件 (null MX)，而收件人会拒绝。

dsn-status-7-28 = 接收系统检测到可能存在大量滥用信息。

dsn-status-7-29 = ARC 验证失败，因此接收方无法接受消息记录的身份验证链。

dsn-status-7-3 = 传送需要服务器无法执行的安全格式转换。

dsn-status-7-30 = 该消息需要REQUIRETLS，但下一个邮件服务器不支持它。请管理员检查安全交付支持。

dsn-status-7-4 = 交付系统不支持所需的安全功能。

dsn-status-7-5 = 服务器无法验证或解密消息，例如因为密钥丢失或无效。

dsn-status-7-6 = 服务器不支持消息所需的加密算法。

dsn-status-7-7 = 完整性检查表明消息已更改或损坏。

dsn-status-7-8 = 发送服务器拒绝登录凭据。检查帐户的身份验证设置。

dsn-status-7-9 = 身份验证方法比服务器允许的更弱。需要更强的方法。

dsn-vendor-advice-account = 要求发送管理员调查可能的帐户滥用、保护帐户并请求取消限制。

dsn-vendor-advice-admin = 与组织的邮件管理员共享此诊断信息以查看匹配的配置或策略。

dsn-vendor-advice-auth = 检查帐户支持的SMTP身份验证方法和授权。通过提供商的官方帐户页面完成任何所需的登录。

dsn-vendor-advice-authentication = 要求发件人管理员使用准确的诊断来检查 SPF、DKIM、DMARC 和域对齐。

dsn-vendor-advice-connector = 要求管理员检查中继授权、连接器IP/证书匹配、接受域和实际邮件路由。

dsn-vendor-advice-content = 与收件人管理员一起查看诊断和网关日志，以确定被阻止的内容和适用的策略。

dsn-vendor-advice-dns = 请发送邮件管理员或IP提供商更正DNS、反转DNS以及SMTP服务器身份。

dsn-vendor-advice-domain = 要求 Microsoft 365 管理员注册并验证发送域并查看租户的发送限制。

dsn-vendor-advice-format = 要求发送应用程序或邮件管理员更正邮件生成，然后创建新邮件。

dsn-vendor-advice-rate = 减少发送量并等待配额或限制被清除。在假设此尝试仍在排队之前，请检查报告的操作。

dsn-vendor-advice-recipient-admin = 另一种方式联系收件人或群组所有者，并要求其管理员查看相关权限或策略。

dsn-vendor-advice-reputation = 要求发送管理员调查不需要的流量和声誉，然后遵循提供商的官方审核或除名流程。

dsn-vendor-advice-route = 请邮件管理员检查目的地、转发规则、邮件服务和防火墙/网络路径。

dsn-vendor-advice-tls = 要求邮件管理员修复TLS、证书或DNS安全记录，以便所需的安全路由能够成功。

dsn-vendor-barracuda-reputation = 梭子鱼的信誉服务列出或阻止发送IP地址。

dsn-vendor-certificate-chain = 无法根据所需的信任或 MTA-STS 策略验证远程 TLS 证书。

dsn-vendor-certificate-expired = 目标邮件服务器提供了过期的 TLS 证书。

dsn-vendor-certificate-host = 目标TLS证书与所需的邮件服务器主机名不匹配。

dsn-vendor-connection-refused = 目标服务器拒绝网络连接。其邮件服务、防火墙或网络路由需要检查。

dsn-vendor-content-security = 网关根据其安全过滤规则拒绝了邮件内容或附件。请查阅其诊断或日志以了解特定项目。

dsn-vendor-credentials = SMTP 服务拒绝了提供的登录凭据。

dsn-vendor-dane = 目标的 TLS 证书或 TLSA 配置不满足 DANE 验证。

dsn-vendor-dkim = 接收方不接受消息的DKIM身份验证或签名配置。

dsn-vendor-dmarc = 接收方的DMARC策略不满足。检查SPF或DKIM身份验证并与可见的发件人域保持一致。

dsn-vendor-dnssec = 无法验证目标域的 DNSSEC 记录。

dsn-vendor-exchange-auth = 应用程序或设备未针对 Microsoft 365 SMTP 提交进行正确身份验证。

dsn-vendor-exchange-connector = Microsoft 无法通过配置的租户连接器授权中继。其证书、发送 IP 或接受的域可能不再匹配。

dsn-vendor-exchange-domain-blocked = Microsoft 在检测到可疑的垃圾邮件活动后限制了此发送域。

dsn-vendor-exchange-forward-relay = 下游系统拒绝转发或中继的邮件。身份验证可能受到转发或非 Microsoft 网关的影响。

dsn-vendor-exchange-forwarding = Microsoft 365 策略阻止自动转发到外部收件人。组织管理员必须审查转发策略。

dsn-vendor-exchange-generic = 交易所报告称普遍拒绝交货。诊断可识别实际问题，其中可能涉及消息格式或附件策略。

dsn-vendor-exchange-group-allow = 发件人不在该 Exchange 组的允许发件人列表中。

dsn-vendor-exchange-group-internal = 此 Exchange 通讯组拒绝其组织外部的发件人，除非其所有者或管理员允许。

dsn-vendor-exchange-internal-only = 此 Exchange 收件人接受来自其自己组织的经过身份验证的发件人；该消息被视为外部消息或未经身份验证的消息。

dsn-vendor-exchange-ipv6 = 目的地不接受超过IPv6的交货。发送管理员需要检查路线或使用可接受的交通工具。

dsn-vendor-exchange-journaling = 该组织正在尝试将本地消息记录到 Microsoft 365 中，而无需进行所需的存档配置。

dsn-vendor-exchange-onmicrosoft-limit = 该组织已达到其 onmicrosoft.com 发送域的外部收件人限制。定期发送应使用经过验证的自定义域。

dsn-vendor-exchange-public-folder = 此 Exchange 公用文件夹不接受此外部或未经身份验证的发件人。

dsn-vendor-exchange-receive-rate = 此 Exchange 收件人已达到所有发件人的每小时接收限制。

dsn-vendor-exchange-recipient = Exchange 无法解析其目录中的此收件人。在创建新消息之前，请检查地址并删除所有过时的自动完成条目。

dsn-vendor-exchange-recipient-access = Exchange 拒绝访问此收件人或目标域。检查收件人地址和组织的接受域配置。

dsn-vendor-exchange-recipient-rate = Exchange 在每小时窗口内收到了太多从该发件人发往该收件人的邮件。

dsn-vendor-exchange-sender-blocked = Microsoft 在检测到可疑垃圾邮件或滥用行为后限制了此发送帐户。这可能需要管理员来保护和解锁帐户。

dsn-vendor-exchange-temporary-restriction = Microsoft 在评估可疑活动时暂时限制发送 IP 或流量。

dsn-vendor-exchange-tenant-blocked = Microsoft 限制了该组织的发送，因为其流量被认为是可疑的。

dsn-vendor-exchange-tenant-blocklist = 收件人地址或域被发件人组织的租户允许/阻止列表阻止。一个被阻止的目的地可以拒绝整个消息。

dsn-vendor-exchange-tenant-limit = Microsoft 365 组织已达到外部收件人的滚动每日限制。

dsn-vendor-exchange-unregistered = Microsoft 拒绝从未在租户中正确注册和验证的域发送邮件。

dsn-vendor-expired = 发送服务在其传送时间窗口到期后放弃。检查导致延迟的连接或路由故障的诊断。

dsn-vendor-fortimail-relay = FortiMail 根据其访问控制或中继策略拒绝了此传送。网关管理员需要检查匹配的接收规则。

dsn-vendor-gmail-app-password = Google 需要应用程序密码或受支持的帐户授权方法才能进行此SMTP 登录。

dsn-vendor-gmail-auth-method = Google 拒绝了不受支持的身份验证方法，例如已停用的 XOAUTH 方法。

dsn-vendor-gmail-authentication = Google 不接受发件人的身份验证。诊断可区分缺失的SPF/DKIM、SPF硬故障和DMARC拒绝。

dsn-vendor-gmail-browser-login = Google 要求在允许此SMTP 登录之前在浏览器中进行帐户登录或安全检查。

dsn-vendor-gmail-custom-policy = Google Workspace 管理员的自定义邮件规则拒绝了该邮件。这并不一定意味着 Google 的垃圾邮件过滤器拒绝了它。

dsn-vendor-gmail-dmarc-record = Google 需要 DMARC 记录以及针对该发送域声明的策略。

dsn-vendor-gmail-rate = Google 应用了发送或中继限制，或者检测到异常的消息速率。诊断会识别受影响的帐户、IP或域。

dsn-vendor-gmail-relay = Google Workspace 未授权发送 IP、域名或帐号用于其 SMTP 中继服务。

dsn-vendor-gmail-reputation = Google 的过滤拒绝了该邮件，因为它认为该邮件是未经请求的或发送的IP/域信誉太低。

dsn-vendor-gmail-spf-record = Google 认为发件人域的 SPF 配置无效或可疑。

dsn-vendor-greylist = 网关使用灰名单暂时推迟了此交付。正确配置的发送服务器通常会稍后重试。

dsn-vendor-headers = 接收服务拒绝无效或丢失的消息标头。发送应用程序需要生成格式正确的邮件。

dsn-vendor-helo = 发送服务器使用了不可接受的 HELO/EHLO 身份。它的问候语必须正确识别发送服务器。

dsn-vendor-ip-block = 接收服务由于其声誉或阻止列表条目而拒绝发送IP。

dsn-vendor-mimecast-block = Mimecast 根据阻止发件人或管理策略拒绝了发件人或域。

dsn-vendor-mimecast-content = Mimecast 内容检查策略拒绝了该消息。

dsn-vendor-mimecast-spoof = Mimecast 的反欺骗策略拒绝发件人身份。

dsn-vendor-mta-sts-host = 域的 MTA-STS 策略不允许目标邮件服务器。

dsn-vendor-proofpoint-block = Proofpoint 在检测到垃圾邮件或恶意活动后阻止了发送 IP 或域。

dsn-vendor-proofpoint-throttle = Proofpoint 暂时推迟了此发送IP，因为它的发送模式被认为是不寻常的。

dsn-vendor-ptr = 发送的IP缺少可接受的反向DNS，或其PTR名称无法解析回相同的IP。

dsn-vendor-recipient-ip-block = 接收组织明确阻止发送 IP 地址。其管理员控制该本地块。

dsn-vendor-relay = 邮件服务器不会将此消息转发到目的地。检查SMTP身份验证、可信发送 IP、连接器和接受域。

dsn-vendor-routing-loop = 消息进入转发或路由循环并超出允许的跳数。

dsn-vendor-sending-limit = 发送帐户已超出消息或收件人配额。重复立即重新发送不会消除限制。

dsn-vendor-spf = 接收方不接受发送方 IP 和信封发送方域的 SPF 身份验证。

dsn-vendor-starttls = 所需的加密SMTP连接无法建立或不受下一个服务器支持。

dsn-vendor-yahoo-deferred = 雅虎通常会在评估数量、声誉或收件人投诉时暂时推迟发送此消息。检查诊断结果以确定原因。

dsn-vendor-yahoo-policy = 雅虎根据其邮件政策拒绝了该邮件。可能涉及身份验证、DMARC对齐或消息内容；诊断给出了具体原因。
