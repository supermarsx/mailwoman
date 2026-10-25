# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = 关闭

# -- Page states (UX11) ---------------------------------------------------------
# $reason — a sentence saying why the data is not there.
admin-ux-state-unavailable = 不可用 — { $reason }
admin-ux-unavailable-default = 此服务器尚不提供此功能。
# $code — the server's error code or "HTTP <status>".
admin-ux-state-code = 错误代码：{ $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = 打开
admin-ux-list-back = 返回列表

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = 没有未保存的更改
admin-ux-form-changes =
    { $count ->
       *[other] { $count } 项未保存的更改
    }
admin-ux-form-discard = 丢弃更改
admin-ux-form-saving = 正在保存…
admin-ux-form-saved =
    { $count ->
       *[other] 已保存 { $count } 项更改。
    }
admin-ux-form-fix =
    { $count ->
       *[other] 未保存任何内容。请修正下方标出的 { $count } 个字段。
    }
admin-ux-learn-more = 了解详情
# Added for screen readers after "Learn more". $label — the field label.
admin-ux-learn-more-field = 关于“{ $label }”
admin-ux-propagation-hot = 在 30 秒内对所有服务器进程生效。
admin-ux-propagation-restart = 在服务器重启后生效。
admin-ux-secret-set = 已存储一个值。输入新值即可替换。
admin-ux-secret-unset = 未存储任何值。

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = 请输入一个值。
admin-ux-err-number = 请输入数字。
admin-ux-err-whole = 请输入整数。
# $min, $max — the bounds the server declares for this field.
admin-ux-err-min = 请输入不小于 { $min } 的值。
admin-ux-err-max = 请输入不大于 { $max } 的值。
admin-ux-err-max-length = 最多使用 { $max } 个字符。
admin-ux-err-url = 请输入以 https:// 或 http:// 开头的完整地址。
admin-ux-err-email = 请输入电子邮件地址，例如 name@example.org。
admin-ux-err-pattern = 此值的格式不符合要求。
admin-ux-err-option = 请从列出的选项中选择一项。
admin-ux-err-out-of-range = 服务器拒绝了此值，因为它超出了允许的范围。
admin-ux-err-too-long = 服务器拒绝了此值，因为它太长。
admin-ux-err-invalid = 服务器拒绝了此值。请检查后再次保存。
admin-ux-err-pinned = 此值由服务器环境设置，无法在此更改。
admin-ux-err-locked = 此值已在更大的范围内锁定，无法在此更改。

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = 默认
admin-ux-source-stored = 已存储
# $name — the environment variable, e.g. MW_LOG_LEVEL.
admin-ux-source-env-pin = 由 { $name } 设置
admin-ux-source-env-fallback = 回退自 { $name }
admin-ux-source-build = 构建时设置
# $scope — "global" or a domain name.
admin-ux-source-locked = 已被 { $scope } 锁定
admin-ux-source-read-only = 只读
admin-ux-source-restart = 需要重启
admin-ux-source-env-pin-reason = 服务器环境通过 { $name } 设置此值。请在那里更改，然后重启服务器。
admin-ux-source-build-reason = 此值在构建服务器时即已固定。
admin-ux-source-locked-reason = { $scope } 范围的锁定固定了此值。要更改它，请在那里解除锁定。
admin-ux-source-read-only-reason = 控制台显示此值，但无法更改它。文档中说明了它的设置位置。

# -- History (UX8) ------------------------------------------------------------
# $who — an admin name; $time — date and time in the reader's locale.
admin-ux-history-changed-by-at = 最后由 { $who } 于 { $time } 更改
admin-ux-history-changed-by = 最后由 { $who } 更改
admin-ux-history-changed-at = 最后更改于 { $time }
admin-ux-history-never = 未通过控制台更改
admin-ux-history-link = 历史记录

# -- Revert to default (UX7) --------------------------------------------------
# $value — the default as shown, e.g. "587".
admin-ux-revert-default-is = 默认值：{ $value }
admin-ux-revert = 恢复默认值
# Added for screen readers after "Revert to default". $label — the field label.
admin-ux-revert-field = （{ $label }）
admin-ux-revert-done = 已恢复为默认值（{ $value }）。

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = 在你编辑期间，{ $who } 于 { $time } 更改了此项。
admin-ux-stale-by = 在你编辑期间，{ $who } 更改了此项。
admin-ux-stale-unknown = 在你编辑期间，其他人更改了此项。
admin-ux-stale-choice = 重新加载以查看对方的版本并放弃你的编辑，或者保留你的编辑并覆盖对方的版本保存。
admin-ux-stale-reload = 重新加载
admin-ux-stale-keep = 保留我的

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = 正在检查此更改的影响范围…
admin-ux-impact-heading = 此更改影响
# $label — what is affected (e.g. "Users refused"); $count — how many.
admin-ux-impact-count = { $label }：{ $count }
admin-ux-impact-more =
    { $count ->
       *[other] 另有 { $count } 项
    }
admin-ux-impact-none = 其他内容均无变化。
admin-ux-impact-unavailable = 影响预览不可用 — 服务器未报告此更改影响的内容。

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = 筛选
admin-ux-stream-pause = 暂停
admin-ux-stream-resume = 继续
admin-ux-stream-clear = 清除
admin-ux-stream-announce = 朗读新条目
admin-ux-stream-connecting = 正在连接…
admin-ux-stream-live = 实时
admin-ux-stream-ended = 已结束
admin-ux-stream-lost = 连接已断开
admin-ux-stream-reconnect = 重新连接
admin-ux-stream-paused =
    { $count ->
        [0] 已暂停
       *[other] 已暂停 — { $count } 个新条目暂未显示
    }
admin-ux-stream-no-match = 没有与筛选条件匹配的条目。

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = 字段

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = 外观
admin-ux-appearance-title = 控制台外观
admin-ux-appearance-note = 这些选择仅更改此浏览器中的管理控制台。
admin-ux-appearance-following = 在你于此处做出选择之前，控制台会沿用此浏览器的网页邮箱主题。这些选择仅更改此浏览器中的管理控制台。
admin-ux-appearance-mode = 模式
admin-ux-appearance-mode-light = 浅色
admin-ux-appearance-mode-dark = 深色
admin-ux-appearance-mode-system = 跟随系统
admin-ux-appearance-theme = 主题
admin-ux-appearance-accent = 强调色
admin-ux-appearance-light = 浅色
admin-ux-appearance-dark = 深色
admin-ux-appearance-reset = 使用网页邮箱主题
