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
