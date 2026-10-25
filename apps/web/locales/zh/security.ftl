# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: security. Source: ../en/security.ftl

# -- Security panel: sections + empty states --------------------------------
security-panel-region = 邮件安全详情
security-section-auth = 身份验证
security-section-received = 投递路径
security-section-signature = 签名
security-section-attachments = 附件
security-section-warnings = 警告
security-section-controls = 发件人控制
security-received-empty = 没有可用的 Received 链。
security-attachments-empty = 没有附件。
security-hop-unknown = 未知
security-unknown = 未知

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = 通过
security-auth-fail = 失败
security-auth-none = 不存在
security-auth-neutral = 中立
security-auth-temperror = 临时错误
security-auth-permerror = 永久错误

# Expert alignment detail field labels + values.
security-detail-domain = 域名
security-detail-selector = 选择器
security-detail-policy = 策略
security-detail-alignment = 对齐
security-detail-chain-length = 链长度
security-aligned = 已对齐
security-not-aligned = 未对齐

# -- Signature / certificate ------------------------------------------------
security-sig-verified = 签名已验证
security-sig-unverified-key = 已签名——签名者密钥未验证
security-sig-invalid = 签名无效
security-sig-none = 未签名
security-chain-trusted = 受信任的证书链
security-chain-untrusted = 不受信任的证书链
security-chain-expired = 证书链已过期
security-chain-unknown = 证书链未知
security-revocation-good = 未吊销
security-revocation-revoked = 密钥已吊销
security-revocation-unknown = 吊销状态未知
security-key-changed = 签名者密钥自上次见到后已更改
security-fact-signer-key = 签名者密钥
security-fact-algorithm = 算法
security-fact-key-created = 密钥创建时间
security-fact-key-expires = 密钥到期时间
security-fact-chain = 证书链
security-fact-revocation = 吊销状态
security-fact-key-change = 密钥变更

# -- Attachment risk --------------------------------------------------------
security-attach-none = 无已知风险
security-attach-macro = 包含宏
security-attach-executable = 可执行文件
security-attach-encrypted-archive = 加密压缩包
security-attach-double-extension = 双重文件扩展名
security-attach-mismatch = 类型不匹配（声明为 { $declared }，检测为 { $detected }）

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = Reply-To 地址与发件人不同
security-anomaly-envelopeFromDivergence = 信封发件人与 From 地址不同
security-anomaly-messageIdDomainAnomaly = Message-ID 域名与发件人不匹配
security-anomaly-dateSkew = 发送日期似乎有偏差
security-anomaly-punycodeSender = 发件人使用 punycode（可能是仿冒域名）

# -- Sender controls --------------------------------------------------------
security-control-block = 阻止发件人
security-control-silence = 静音发件人
security-control-ignore-conversation = 忽略会话
security-control-report-phishing = 举报网络钓鱼
security-control-report-junk = 举报垃圾邮件
security-control-done-block = 已阻止发件人
security-control-done-silence = 已静音发件人
security-control-done-ignore-conversation = 已忽略会话
security-control-done-report-phishing = 已举报为网络钓鱼
security-control-done-report-junk = 已举报为垃圾邮件

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = 零访问存储
security-za-on = 开
security-za-off = 关
security-za-protects = 零访问会加密你存储的邮件、笔记和个人信息管理（PIM）数据，使托管服务器只保存它无法读取的密文。它针对好奇的运营者或存储主机遭到入侵的情况，保护你存储数据的内容：任何读取数据库、磁盘文件或被盗备份的人只能看到 XChaCha20-Poly1305 密文，无法恢复邮件正文、主题、附件、笔记文本或搜索索引。

security-za-server-sees-title = 服务器仍能看到的内容
security-za-sees-1 = 密文数据块——即加密的数据行本身。
security-za-sees-2 = 不透明的行 ID——用于存储和获取数据行的标识符。
security-za-sees-3 = 大小——每段密文的长度（大致对应邮件/附件的大小）。
security-za-sees-4 = 时间戳——数据行写入和更新的时间。
security-za-sees-5 = 代理 IMAP/SMTP 所需的信封路由元数据——服务器仍会代表你连接到上游邮件服务商，因此收发邮件所需的连接元数据和路由信封会经过服务器。

security-za-active-server-caveat = 零访问保护的是静态存储的数据，防范好奇的或遭到入侵的主机。一个代理你实时 IMAP/SMTP 流量的完全恶意的主动服务器是更强大的对手，零访问无法防御它：这样的服务器位于你与邮件服务商之间的实时连接上，无论存储副本如何加密，它都可以在邮件流经时观察或篡改邮件。如果你担心的是“我不希望运营（或入侵）存储主机的人读取我存储的邮件”，请选择零访问。它不能防御篡改实时邮件路径的运营者。

security-za-no-search-claim = 这里没有服务器端可搜索加密，也不作此声明。搜索完全在客户端上针对本地解密的内容进行；服务器从不持有你明文的可搜索形式。

security-za-recovery-tradeoff = 你的密钥仅在你的设备上派生和保存——服务器从不接收明文密钥。如果你同时丢失了密码短语（或通行密钥）、所有已配对的设备以及恢复短语，加密数据将无法恢复。请离线保存恢复短语，并将其视同你账户的主密钥。

security-za-setup-key = 设置密钥
security-za-key-source = 密钥来源
security-za-passphrase = 密码短语
security-za-passkey = 通行密钥（无密码）
security-za-passkey-unavailable = 此浏览器不支持通行密钥
security-za-passphrase-label = 零访问密码短语
security-za-enable = 启用零访问
security-za-recovery-title = 恢复短语
security-za-recovery-heading = 恢复短语——请立即离线保存
security-za-recovery-note = 这是唯一的副本。任何持有它的人都能读取你的数据；没有它（也没有已配对的设备），你的数据将无法恢复。
security-za-disable-section = 停用零访问
security-za-disable-heading = 关闭零访问
security-za-disable-note = 新数据将再次以未加密方式存储。现有的加密数据只有在你仍能派生密钥时才可读取。
security-za-disable-btn = 停用零访问
security-za-err-passphrase-len = 请使用至少 8 个字符的密码短语
security-za-err-enable = 无法启用零访问
security-za-err-disable = 无法停用零访问

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = 设备配对
security-pair-heading = 配对设备
security-pair-intro = 将密钥转移到另一台设备，而无需通过服务器发送——服务器只转发一个不透明的密封信封。在信任此次配对之前，请比对两个屏幕上的六个词。
security-pair-role = 配对角色
security-pair-new-role = 这是新设备
security-pair-existing-role = 从此处配对另一台设备
security-pair-show-qr = 显示配对二维码
security-pair-scan = 在现有设备上扫描此码
security-pair-qr-label = 设备配对二维码
security-pair-qr-default = 配对二维码
security-pair-copy-code = 或复制此配对码
security-pair-paste-envelope = 粘贴来自另一台设备的密封信封
security-pair-envelope-placeholder = 信封…
security-pair-envelope-label = 密封信封
security-pair-complete = 完成配对
security-pair-code-from-new = 来自新设备的配对码
security-pair-code-placeholder = 配对码…
security-pair-code-label = 配对码
security-pair-seal = 为该设备密封我的密钥
security-pair-envelope-out = 密封信封——请将其粘贴到新设备上
security-pair-compare = 在两台设备上比对这些词
security-pair-sas-label = 短认证字符串
security-pair-match = 这些词一致
security-pair-abort = 不一致——中止
security-pair-confirmed = 配对已确认。此通道已通过认证。
security-pair-err-start = 无法开始配对
security-pair-err-complete = 无法完成配对
security-pair-err-unlock = 请先解锁零访问，再配对另一台设备
security-pair-err-seal = 无法密封根密钥
