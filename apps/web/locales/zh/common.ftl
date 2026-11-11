# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = 确定
common-cancel = 取消
common-close = 关闭
common-save = 保存
common-delete = 删除
common-remove = 移除
common-edit = 编辑
common-add = 添加
common-back = 返回
common-next = 下一步
common-done = 完成
common-apply = 应用
common-confirm = 确认
common-retry = 重试
# Desktop and mobile apps only: under the error shown when the server the app
# was built or provisioned for does not answer at start. The user cannot change
# the server; an administrator can.
common-contact-administrator = 如果此问题持续出现，请联系管理员。
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = 此应用尚未设置服务器。请联系管理员。
common-copy = 复制
common-search = 搜索
common-more = 更多
common-yes = 是
common-no = 否

# -- Generic states ----------------------------------------------------------
common-loading = 正在加载…
common-saving = 正在保存…
common-empty = 这里还没有内容
common-error = 出现问题
common-error-network = 无法连接到服务器。请检查网络连接后重试。
common-offline = 你已离线

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = 正在连接…
common-conn-degraded = 实时更新已暂停 — 正在重新连接
common-conn-offline = 你已离线 — 重新连接后将同步更改
common-conn-auth-expired = 会话已过期 — 请重新登录
common-conn-reconnect = 重新连接
common-conn-sign-in-again = 重新登录
common-conn-reconnected = 已重新连接

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = 新邮件
common-notify-no-subject = （无主题）

# -- Generic labels ----------------------------------------------------------
common-required = 必填
common-optional = 可选

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = 打开文件夹和应用
common-nav-close = 关闭文件夹和应用

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = 附件
common-attach-search = 搜索附件
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = 按类型筛选
common-attach-loading = 正在加载附件…
common-attach-empty = 没有匹配的附件。
common-attach-cat-all = 全部
common-attach-cat-image = 图片
common-attach-cat-pdf = PDF
common-attach-cat-text = 文本
common-attach-cat-audio = 音频
common-attach-cat-video = 视频
common-attach-cat-other = 其他

# -- Password fields (components/PasswordInput.tsx): the button at the end of
# every password field, on both sign-in screens and in settings. It is a toggle;
# its name stays "Show password" and screen readers announce pressed or not
# pressed. The hide text is the pointer tooltip while the password is shown.
common-password-show = 显示密码
common-password-hide = 隐藏密码

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
# and the title of the pinned mail tab (screens/Mailbox.tsx).
common-module-mail = 邮件
common-module-calendar = 日历
common-module-tasks = 任务
common-module-notes = 笔记
common-module-contacts = 联系人
common-module-keys = 密钥

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
# thumbnail strip under a message. The MIME type itself is shown untranslated.
common-viewer-loading = 正在加载查看器…
common-viewer-loading-audio = 正在加载音频…
common-viewer-loading-image = 正在加载图片…
common-viewer-loading-video = 正在加载视频…
common-viewer-rendering-pdf = 正在渲染 PDF…
common-viewer-pdf-failed = 无法渲染此 PDF。
common-viewer-no-preview = 此文件类型不支持内嵌预览。
common-viewer-unknown-type = 未知类型
common-viewer-download = 下载
common-viewer-attachments = 附件

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = 正在重试…
common-waiting-online = 正在等待网络连接…
common-failure-offline = 你已离线。请重新连接后重试。
common-failure-network = 无法连接到服务器。
common-failure-server = 服务器未能及时响应。
common-failure-rate = 短时间内请求过多。请稍候再试。
common-failure-conflict = 此内容刚在其他位置被同时修改。请刷新后重试。
common-failure-account = 当前登录的账户已更改。请刷新页面。
common-failure-linked = 打开关联邮箱期间无法使用此功能。
common-failure-refused = 服务器拒绝了此更改。
common-new-version = Mailwoman 有新版本可用。关闭后将在一小时后再次提醒。
common-new-version-reload = 立即刷新
common-toast-close = 关闭通知
common-toast-seconds = { $seconds } 秒
common-attach-refresh = 刷新附件
common-attach-open-message = 打开包含此附件的邮件
common-attach-page-filter-note = 筛选仅作用于当前页。使用“下一页”可浏览带附件的更早邮件。
common-attach-pages = 附件分页
common-attach-previous = 上一页
common-attach-next = 下一页
common-attach-page = 第 { $page } 页
common-attach-column-name = 文件
common-attach-column-type = 类型
common-attach-column-size = 大小
common-attach-column-sender = 发件人
common-attach-column-subject = 主题
common-attach-column-date = 接收时间
common-pdf-preview = PDF 预览
common-pdf-link-title = 要打开此 PDF 中的链接吗？
common-pdf-link-help = 将在 Mailwoman 之外的新标签页中打开目标地址。
common-pdf-link-open = 打开链接
common-open-event = 打开日程
common-message-preview-too-large = 邮件附件预览限制为 4 MiB。你仍可下载原始附件。
common-message-preview-failed = 无法预览此邮件附件。文件可能不受支持或已损坏，也可能是预览服务不可用。你仍可下载原始文件。
common-message-preview-attachments = 此邮件内的文件
common-message-preview-download = 下载文件
common-message-preview-size-bytes = { $count } 字节
common-attach-cat-message = 邮件 / Outlook 附件
common-error-retry-help = 此部分暂时不可用。请重试以重新加载。
common-error-network-help = 请检查网络连接后重试。服务器可能暂时不可用。
common-error-page = 无法加载此页面
common-error-page-help = 某个页面资源下载失败。请重试。如果问题持续，请重新加载应用以获取最新版本。
common-reload-app = 重新加载应用
common-new-version-reloading = 有新版本可用，正在重新加载…
common-attach-account-changed = 加载附件期间账户已更改。请为当前账户重新打开附件。
common-attach-search-account = 搜索整个账户
common-attach-scanning = 正在搜索… 已检查 { $pages } 页邮件 · 找到 { $count } 个文件
common-attach-scan-cancelled = 搜索已取消 · 在 { $pages } 页中找到 { $count } 个文件。结果不完整。
common-attach-scan-incomplete = 搜索被中断 · 找到 { $count } 个文件。请重试以搜索整个账户。
common-attach-scan-limited = 仅显示前 1,000 个匹配的文件。请缩小筛选范围以找到更具体的文件。
common-attach-scan-complete = 搜索完成 · 在 { $pages } 页邮件中找到 { $count } 个文件
common-attach-browse-pages = 浏览分页
common-attach-cancel-search = 取消搜索
multiselect-selected = 已选：{ $label }
multiselect-remove = 移除 { $label }
multiselect-no-match = 没有匹配的选项
common-notification-permission-title = 允许通知吗？
common-notification-permission-help = 在此设备上接收新邮件通知。你可以随时在“设置”中更改。
common-notification-permission-allow = 允许通知
common-notification-permission-later = 暂不
common-notification-permission-error = 无法启用通知。请检查权限和通知设置后重试。
common-notifications-enabled = 此设备已启用通知。

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
   *[other] 已选 { $count } 项
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = 服务器繁忙，请等待几秒后重试。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
combobox-empty = 没有匹配项

combobox-toggle = 显示{ $label }的选项

common-attach-cat-archive = 档案

common-attach-cat-document = 文件

common-attach-cat-spreadsheet = 电子表格

common-conn-reconnecting = 正在重新连接...

common-viewer-pdf-details = 技术细节

common-viewer-pdf-details-help = 报告问题时请包括这些行。它们不包含文档内容。

timezone-group-other = 其他

timezone-use-browser = 使用浏览器时区 ({ $zone })

common-context-actions = { $name } 的操作
common-open = 打开
common-new = 新建项目
common-refresh = 刷新
common-duplicate = 创建副本
common-copy-fingerprint = 复制指纹
common-export-public-key = 导出公钥或证书
common-context-show = 显示
common-context-show-only = 仅显示此日历
common-context-color = 颜色
common-context-import = 导入
common-context-export = 导出
common-context-pinned = 已固定项目
common-context-unpin = 取消固定
common-context-my-day = 添加到我的一天
common-context-remove-my-day = 从我的一天移除
