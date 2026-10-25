# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: apikeys. Source: ../en/apikeys.ftl

# -- ApiKeys panel -----------------------------------------------------------
apikeys-panel-label = API 密钥
apikeys-heading = API 密钥
apikeys-intro = 为脚本和集成创建限定范围的密钥。每个密钥以哈希形式存储，只显示一次，并可单独撤销。请只授予够用的最小范围。

apikeys-label = 标签
apikeys-label-aria = 密钥标签
apikeys-label-placeholder = 例如：备份脚本

apikeys-create = 创建密钥
apikeys-saved = 我已保存

apikeys-reveal-warning = 请立即复制此密钥 — 它只显示一次，之后无法再次获取。
apikeys-copy = 复制
apikeys-copy-aria = 将密钥复制到剪贴板
apikeys-copied = 已复制到剪贴板

apikeys-error-need-label = 请为密钥设置标签，以便日后识别
apikeys-error-create = 无法创建密钥
apikeys-error-revoke = 无法撤销密钥

# -- Existing-key list -------------------------------------------------------
apikeys-existing-label = 现有密钥
apikeys-existing = 现有密钥
apikeys-none = 还没有密钥。
apikeys-created = 创建于 { $date }
apikeys-last-used = 上次使用于 { $date }
apikeys-never-used = 从未使用
apikeys-revoked-at = 撤销于 { $date }
apikeys-revoke = 撤销
apikeys-status-active = 有效
apikeys-status-revoked = 已撤销

# -- Scope builder -----------------------------------------------------------
apikeys-scope-label = 范围
apikeys-capabilities = 权限
apikeys-cap-read = 读取
apikeys-cap-send = 发送
apikeys-cap-delete = 删除
apikeys-surface = 适用范围
apikeys-surface-mail-label = 邮件
apikeys-surface-pim-label = PIM（日历 / 任务 / 笔记 / 联系人）
apikeys-accounts = 账户
apikeys-all-accounts = 所有账户
apikeys-folders = 文件夹
apikeys-all-folders = 所有文件夹
apikeys-folder-ids = 文件夹 ID（以逗号分隔）
apikeys-folder-subset-aria = 文件夹子集
apikeys-constraints = 限制条件
apikeys-expiry = 过期时间（RFC 3339，留空 = 永不过期）
apikeys-expiry-aria = 过期时间
apikeys-expiry-placeholder = 2026-12-31T00:00:00Z
apikeys-rate-limit = 速率限制（每分钟请求数，留空 = 不限制）
apikeys-rate-limit-aria = 速率限制
apikeys-ip-allowlist = IP 允许列表（CIDR/IP，以逗号分隔；留空 = 任意）
apikeys-ip-allowlist-aria = IP 允许列表
apikeys-ip-placeholder = 203.0.113.0/24, 198.51.100.7

# -- Scope summary (one-line human review) -----------------------------------
apikeys-verb-read = 读取
apikeys-verb-send = 发送
apikeys-verb-delete = 删除
apikeys-summary-mail = 邮件
apikeys-summary-pim = PIM
apikeys-summary-no-verbs = 无操作
apikeys-summary-nothing = 无
apikeys-summary-none = 无
apikeys-summary-verbs-on = 对 { $surfaces } 的 { $verbs } 权限
apikeys-summary-all-accounts = 所有账户
apikeys-summary-accounts = 账户：{ $ids }
apikeys-summary-all-folders = 所有文件夹
apikeys-summary-folders = 文件夹：{ $ids }
apikeys-summary-expires = 过期时间 { $date }
apikeys-summary-rate = 每分钟 { $n } 次请求
apikeys-summary-ips = IP：{ $ips }
apikeys-summary-mcp-tools = MCP 工具：{ $tools }
apikeys-summary-unattended = 无人值守发送

# -- MCP keys ----------------------------------------------------------------
apikeys-mcp-panel-label = MCP 密钥
apikeys-mcp-heading = MCP 密钥
apikeys-mcp-intro = 通过 Model Context Protocol 向 AI 代理只授予你选定的工具。返回给代理的邮件正文会被标记为不受信任的输入。每个工具需单独授予。
apikeys-mcp-label-aria = MCP 密钥标签
apikeys-mcp-label-placeholder = 例如：助理代理
apikeys-mcp-create = 创建 MCP 密钥
apikeys-mcp-reveal-warning = 请立即复制此密钥 — 它只显示一次。
apikeys-mcp-error-need-label = 请为 MCP 密钥设置标签
apikeys-mcp-error-need-tool = 请至少授予一个工具
apikeys-mcp-error-create = 无法创建 MCP 密钥

apikeys-tools = 工具
apikeys-tools-group = MCP 工具
apikeys-outbox-suffix = （需经发件箱确认）

apikeys-unattended-aria = 无人值守发送
apikeys-unattended-allow = 允许无人值守发送（绕过发件箱 — 需要管理员会签）
apikeys-unattended-pending = 在管理员会签此密钥之前，无人值守发送保持未启用状态。
apikeys-unattended-disclosure = 默认情况下，获准的发送操作会进入你的发件箱，等待你在应用中确认 — 自动化程序无法自行发送。无人值守发送会【移除】这一人工确认步骤，使此密钥无需确认即可发送邮件。此外还需要管理员对该密钥进行会签。只应将其授予你完全信任的自动化程序；无人值守发送密钥一旦泄露，他人即可在没有任何提示的情况下以你的名义发送邮件。

# -- MCP tool grants (label + description shown on each checkbox) -------------
apikeys-tool-mail-search-label = 搜索邮件
apikeys-tool-mail-search-desc = 搜索邮件。结果会被标记为不受信任的输入。
apikeys-tool-mail-read-label = 读取邮件
apikeys-tool-mail-read-desc = 读取邮件正文。正文会被标记为不受信任的输入。
apikeys-tool-folders-list-label = 列出文件夹
apikeys-tool-folders-list-desc = 列出邮箱文件夹。
apikeys-tool-drafts-create-label = 创建草稿
apikeys-tool-drafts-create-desc = 创建草稿邮件（绝不会自动发送）。
apikeys-tool-mail-send-label = 发送邮件
apikeys-tool-mail-send-desc = 将邮件放入发件箱队列，等待在应用中确认。
apikeys-tool-calendar-read-label = 读取日历
apikeys-tool-calendar-read-desc = 读取日历日程。
apikeys-tool-calendar-propose-label = 提议日程
apikeys-tool-calendar-propose-desc = 提议（而非直接创建）日历日程。
apikeys-tool-tasks-read-label = 读取任务
apikeys-tool-tasks-read-desc = 读取任务。
apikeys-tool-tasks-write-label = 写入任务
apikeys-tool-tasks-write-desc = 创建或更新任务。
apikeys-tool-contacts-read-label = 读取联系人
apikeys-tool-contacts-read-desc = 读取联系人。
apikeys-surface-scripts-label = 管理 Sieve 脚本
