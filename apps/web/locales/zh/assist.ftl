# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: assist. Source: ../en/assist.ftl

# -- AI assistant chat panel ------------------------------------------------------
assist-panel-label = AI 助手
assist-heading = AI 助手
assist-transcript-label = AI 助手对话
assist-proposed-action = 建议的操作
assist-outbox-note = 此操作会将一封邮件放入你的发件箱。在你于发件箱中确认之前，不会发送任何内容。
assist-review-outbox = 在发件箱中检查
assist-review = 检查
assist-empty = 让 AI 助手进行摘要、起草或查找内容。它可以建议操作，但始终由你亲自确认。
assist-error = AI 助手暂时无法响应。
assist-input-label = 向 AI 助手发送消息
assist-input-placeholder = 向 AI 助手提问…
assist-thinking = 正在思考…
assist-ask = 提问

# -- Composer tools ---------------------------------------------------------
assist-composer-label = 写作帮助
assist-composer-toolbar = 撰写工具
assist-tool-grammar = 修正语法
assist-tool-rewrite = 改写
assist-tool-tone = 调整语气
assist-tool-translate = 翻译
assist-toolarg-tone = 语气
assist-toolarg-translate = 语言
assist-tone-neutral = 中性
assist-tone-friendly = 友好
assist-tone-formal = 正式
assist-tone-concise = 简洁
assist-tone-assertive = 坚定
assist-busy = { $label }…
assist-composer-empty-error = 请先写点内容。
assist-composer-error = AI 助手暂时无法完成此操作。
assist-suggested-edit = 建议的修改
assist-apply-draft = 应用到草稿
assist-discard = 放弃

# -- Auto-tag ---------------------------------------------------------------
assist-autotag-label = 建议的标签
assist-apply-auto = 自动应用
assist-apply-label = 应用 { $label }
assist-remove-label = 移除 { $label }
assist-apply = 应用
assist-undo = 撤销

# -- Dictation --------------------------------------------------------------
assist-dictate-hold = 按住以听写
assist-dictate-stop = 停止听写
assist-dictate-hold-text = 按住以听写
assist-dictate-listening = ● 正在聆听…
assist-dictate-endpoint-note = 音频由你的 AI 助手端点转写。
assist-dictate-err = 听写出错。
assist-transcribe-err = 无法转写音频。
assist-mic-err = 麦克风不可用。

# -- Semantic search --------------------------------------------------------
assist-semantic-label = 语义搜索
assist-semantic-note = 查询文本会发送到你的 AI 助手端点，以按含义排序。

# -- "What left the device" disclosure --------------------------------------
assist-disclosure-summary = 哪些内容可能离开此设备
assist-left-1 = 为所选功能选定的邮件文本（主题 + 正文）
assist-left-2 = 内容被发送到的端点主机
assist-left-3 = 绝不发送：端到端加密的内容（默认排除）
assist-left-4 = 绝不发送：附件（默认排除）
assist-left-5 = 绝不发送：你的凭据或其他账户
assist-disclosure-off = AI 助手已关闭。没有任何邮件内容会离开此设备。
assist-disclosure-sentence = 使用 AI 助手工具时，所选的邮件文本会通过代理发送到 { $host }。{ $excl } 发送绝不会自动进行 — 任何内容离开你的发件箱之前，始终需要你确认。
assist-disclosure-excl-one = 它绝不发送{ $a }。
assist-disclosure-excl-two = 它绝不发送{ $a }或{ $b }。
assist-disclosure-admin-allowed = 你的管理员已允许发送加密内容和附件。
assist-excluded-e2ee = 端到端加密的内容
assist-excluded-attachments = 附件

# -- Admin → AI assistant (deployment configuration) ------------------------------
# Wording here states what the server does. "Stop" takes effect on the running
# server; every other change is read when the server starts.
assist-admin-title = AI 助手
assist-admin-intro = AI 助手会将用户选定的邮件文本发送到此处配置的 AI 端点。它不能发送、删除或接收邮件。除非你在下方允许，否则端到端加密的内容和附件不会被发送。
assist-admin-load-unreadable = 无法读取已存储的 AI 助手配置：其 API 密钥无法用此服务器的密钥（MW_SERVER_KEY）解开。它是在另一个密钥下保存的，或已被更改。AI 助手已关闭。请填写表单（包括 API 密钥）并保存以替换它。
assist-admin-load-error = 无法加载 AI 助手配置。已禁用保存，以免覆盖已存储的配置。
assist-admin-status-running = AI 助手正在运行。请求发送到 { $host }。
assist-admin-status-running-pending = AI 助手正在使用服务器启动时的配置运行。已保存的更改将在服务器重启后生效。
assist-admin-status-off = AI 助手已关闭。所有 AI 助手请求都会被拒绝。
assist-admin-status-pending = AI 助手未在运行。已保存的配置将在服务器重启后生效。
assist-admin-status-local-program-not-allowed = AI 助手未在运行。配置的程序 { $program } 未列在服务器的 MW_ASSIST_LOCAL_PROGRAMS 中，因此服务器没有启动它。
assist-admin-status-no-endpoint = AI 助手已开启但未在运行，因为没有配置可用的端点。
assist-admin-stop = 立即停止 AI 助手
assist-admin-stop-note = 停止会立即对所有用户生效，并在重启后保持。
assist-admin-resume = 开启 AI 助手
assist-admin-resume-note = 如果服务器启动时 AI 助手处于关闭状态，开启操作将在服务器重启后生效。
assist-admin-kill-error = 无法更改 AI 助手的开启状态。
assist-admin-endpoint = 端点
assist-admin-endpoint-note = 由服务器连接此端点，浏览器从不连接。更改将在服务器重启后生效。
assist-admin-kind = 端点类型
assist-admin-kind-none = 无
assist-admin-kind-open-ai-compatible = 兼容 OpenAI
assist-admin-kind-anthropic = Anthropic
assist-admin-kind-local-process = 本地程序
assist-admin-local-program-enable = 若要让 AI 助手在此服务器上运行程序，请在启动服务器的环境中将 MW_ASSIST_LOCAL_PROGRAMS 设置为该程序的绝对路径（多个路径用逗号分隔），然后重启服务器。
assist-admin-local-program-note = 服务器会为每个 AI 助手请求启动此程序，并通过标准输入将请求传给它，其中包括用户选定的邮件文本。服务器从程序的标准输出读取回复。只能选择服务器上 MW_ASSIST_LOCAL_PROGRAMS 中列出的程序。
assist-admin-local-program = 程序
assist-admin-local-program-unlisted = { $program }（未列出）
assist-admin-local-program-unlisted-note = 此程序未列在 MW_ASSIST_LOCAL_PROGRAMS 中，因此服务器不会运行它，保存时也不会接受它。请选择已列出的程序或其他端点类型。
assist-admin-local-program-required = 请选择要运行的程序。
assist-admin-local-args = 参数，每行一个
assist-admin-local-args-note = 每一行作为一个参数传给程序。不经过 shell。
assist-admin-base-url = 基础 URL
assist-admin-base-url-required = 请输入端点的基础 URL，以 http:// 或 https:// 开头。
assist-admin-api-key = API 密钥
assist-admin-api-key-stored = 已存储一个密钥。服务器不会再次显示它。
assist-admin-api-key-replace = 替换 API 密钥
assist-admin-api-key-replace-placeholder = 留空以保留已存储的密钥
assist-admin-api-key-remove = 移除已存储的密钥
assist-admin-api-key-keep = 保留已存储的密钥
assist-admin-api-key-remove-pending = 保存时将移除已存储的密钥。
assist-admin-chat-model = 对话模型
assist-admin-embed-model = 嵌入模型
assist-admin-stt-model = 语音转文字模型
assist-admin-model = 模型
assist-admin-anthropic-version = API 版本
assist-admin-max-tokens = 回复的最大 token 数
assist-admin-default-placeholder = 服务器默认值
assist-admin-grants = 功能
assist-admin-grants-note = 只能使用此处勾选的功能。
assist-admin-cap-summarize = 摘要
assist-admin-cap-draft = 起草和改写
assist-admin-cap-grammar = 语法
assist-admin-cap-dictation = 听写
assist-admin-cap-search-semantic = 语义搜索
assist-admin-cap-auto-tag = 自动标记
assist-admin-cap-recap = 回顾
assist-admin-cap-assistant = AI 助手对话
assist-admin-ceilings = 数据限制
assist-admin-ceilings-note = 这些限制适用于每个请求，无论使用哪项功能。
assist-admin-accounts = 允许发送其邮件的账户 ID，每行一个
assist-admin-accounts-note = 如果未列出任何账户，则不会向端点发送任何邮件内容。
assist-admin-folders = 限定发送范围的文件夹 ID，每行一个
assist-admin-folders-note = 如果未列出任何文件夹，则允许所列账户的所有文件夹。
assist-admin-allow-e2ee = 允许发送端到端加密的内容
assist-admin-allow-attachments = 允许发送附件
assist-admin-save = 保存配置
assist-admin-saved = 已保存。
assist-admin-saved-pending = 已保存。更改将在服务器重启后生效。
assist-admin-save-error = 配置未保存。
assist-admin-save-not-allowed = 配置未保存，因为服务器不允许此程序。只能使用 MW_ASSIST_LOCAL_PROGRAMS 中列出的程序。
