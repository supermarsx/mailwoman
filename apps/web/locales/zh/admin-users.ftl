# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = 用户。点击列标题可对列表排序。
admin-userlist-search = 搜索用户
admin-userlist-search-placeholder = 姓名、地址、别名或域名
admin-userlist-count = { $count ->
   *[other] { $count } 位用户
}
admin-userlist-count-filtered = { $count ->
   *[other] { $total } 位用户中有 { $count } 位匹配
}
admin-userlist-range = 第 { $from }–{ $to } 位,共 { $total } 位
admin-userlist-loading = 正在加载用户…
admin-userlist-retry = 重试
admin-userlist-empty-filtered = 没有用户符合此搜索和筛选条件。
admin-userlist-compact = 紧凑行距
admin-userlist-columns = 列
admin-userlist-columns-legend = 显示列
admin-userlist-actions = 操作
admin-userlist-paging = 用户分页
admin-userlist-page-size = 每页行数
admin-userlist-prev = 上一页
admin-userlist-next = 下一页
admin-userlist-sort-by = 排序方式
admin-userlist-sort-asc = { $column },升序
admin-userlist-sort-desc = { $column },降序
admin-userlist-col-user = 用户
admin-userlist-col-domain = 域名
admin-userlist-col-status = 状态
admin-userlist-col-2fa = 双重验证
admin-userlist-col-role = 控制台角色
admin-userlist-col-last-signin = 上次登录
admin-userlist-col-last-seen = 上次活动
admin-userlist-col-created = 创建时间
admin-userlist-col-quota = 配额
admin-userlist-col-profile = 功能配置文件
admin-userlist-col-linked = 已关联邮箱
admin-userlist-state-active = 活跃
admin-userlist-state-pending = 从未登录
admin-userlist-state-blocked = 已阻止
admin-userlist-state-disabled = 已停用
admin-userlist-state-deleted = 已删除
admin-userlist-2fa-passkey = 通行密钥
admin-userlist-2fa-totp = 身份验证器应用
admin-userlist-2fa-none = 无
admin-userlist-signin-never = 从未
admin-userlist-signin-at = { $relative }({ $exact })
admin-userlist-seen-never = 无活动记录
admin-userlist-seen-at = 上次活动 { $relative }({ $exact })
admin-userlist-quota-unlimited = 无限制
admin-userlist-profile-default = 默认
admin-userlist-profile-via-user = 已分配给此用户
admin-userlist-profile-via-group = 来自群组
admin-userlist-profile-via-domain = 来自域名
admin-userlist-more = 更多操作
admin-userlist-more-for = { $account } 的更多操作
admin-userlist-select = 选择 { $account }
admin-userlist-select-page = 选择此页上的所有用户
admin-userlist-filters = 筛选
admin-userlist-add-filter = 添加筛选条件
admin-userlist-clear-all = 全部清除
admin-userlist-chip-edit = 编辑筛选条件 { $label }
admin-userlist-chip-remove = 移除筛选条件 { $label }
admin-userlist-filter-domain = 域名
admin-userlist-filter-status = 状态
admin-userlist-filter-tfa = 双重验证
admin-userlist-filter-role = 控制台角色
admin-userlist-filter-profile = 功能配置文件
admin-userlist-filter-signin = 上次登录
admin-userlist-filter-created = 创建时间
admin-userlist-filter-no-choices = 暂无可选项。
admin-userlist-filter-remove = 移除筛选条件
admin-userlist-apply = 应用
admin-userlist-cancel = 取消
admin-userlist-role-any = 任意控制台角色
admin-userlist-role-none = 无控制台角色
admin-userlist-profile-none = 无配置文件(默认)
admin-userlist-signin-older = { $days ->
   *[other] 超过 { $days } 天前
}
admin-userlist-signin-within = { $days ->
   *[other] 最近 { $days } 天内
}
admin-userlist-signin-custom = 多于以下天数之前:
admin-userlist-signin-days = 天数
admin-userlist-days-invalid = 请输入 1 到 36500 之间的整数天数。
admin-userlist-created-from = 起始
admin-userlist-created-to = 截止
admin-userlist-created-between = { $from } 至 { $to }
admin-userlist-created-since = 自 { $date } 起
admin-userlist-created-until = 至 { $date }
admin-userlist-dates-invalid = 开始日期晚于结束日期。
admin-userlist-views = 已保存的视图
admin-userlist-view-save = 保存此视图…
admin-userlist-view-name = 视图名称
admin-userlist-view-save-confirm = 保存视图
admin-userlist-view-delete = 删除视图“{ $name }”
admin-userlist-bulk = 对所选用户的操作
admin-userlist-selected = { $count ->
   *[other] 已选择 { $count } 位
}
admin-userlist-clear-selection = 清除选择
admin-userlist-block = 阻止登录
admin-userlist-unblock = 解除登录阻止
admin-userlist-sign-out = 在所有位置退出登录
admin-userlist-delete = 删除本地访问权限
admin-userlist-bulk-profile = 分配功能配置文件
admin-userlist-bulk-profile-clear = 移除用户自己的配置文件
admin-userlist-confirm-skipped = { $count ->
   *[other] 所选的 { $count } 位用户将被跳过:该操作不适用于他们。
}
admin-userlist-confirm-block = { $count ->
   *[other] 阻止 { $count } 位用户登录?
}
admin-userlist-confirm-block-one = 阻止 { $account } 登录?
admin-userlist-confirm-block-body = 新的登录将被拒绝,当前的 Mailwoman 会话将结束。上游邮箱不会改变。
admin-userlist-confirm-block-action = 阻止登录
admin-userlist-confirm-unblock = { $count ->
   *[other] 解除 { $count } 位用户的登录阻止?
}
admin-userlist-confirm-unblock-one = 解除 { $account } 的登录阻止?
admin-userlist-confirm-unblock-body = 本地登录阻止将被移除。其他安全限制仍然有效。
admin-userlist-confirm-unblock-action = 解除登录阻止
admin-userlist-confirm-signout = { $count ->
   *[other] 让 { $count } 位用户在所有位置退出登录?
}
admin-userlist-confirm-signout-one = 让 { $account } 在所有位置退出登录?
admin-userlist-confirm-signout-body = 这些用户所有已打开的 Mailwoman 会话都将结束。他们可以重新登录。
admin-userlist-confirm-signout-action = 退出登录
admin-userlist-confirm-delete = { $count ->
   *[other] 删除 { $count } 位用户的本地访问权限?
}
admin-userlist-confirm-delete-one = 删除 { $account } 的本地访问权限?
admin-userlist-confirm-delete-body = 已删除的访问记录会被保留,因此不会自动重新注册该用户。Mailwoman 设置、历史记录和上游邮箱仍会保留。请输入地址以确认。
admin-userlist-confirm-delete-action = 删除本地访问权限
admin-userlist-confirm-profile = { $count ->
   *[other] 更改 { $count } 位用户的功能配置文件?
}
admin-userlist-confirm-profile-one = 更改 { $account } 的功能配置文件?
admin-userlist-confirm-profile-body = 配置文件会直接分配给每位用户,并从其下一次请求起生效。支持原生、代理和 SSO 邮箱身份。
admin-users-profile-help = 选择“默认”可移除此用户的直接分配,并使用从其群组或域名继承的配置文件。
admin-users-profile-ambiguous = 此用户有多个登录身份。请在“功能配置文件”中选择目标身份。
admin-users-profile-unavailable = 没有可用的已验证邮箱身份。此用户必须先登录,才能分配配置文件。
admin-userlist-confirm-profile-action = 更改配置文件
admin-userlist-bulk-done-block = { $count ->
   *[other] 已阻止 { $count } 位用户登录。
}
admin-userlist-bulk-done-unblock = { $count ->
   *[other] 已解除 { $count } 位用户的登录阻止。
}
admin-userlist-bulk-done-signout = { $count ->
   *[other] 已让 { $count } 位用户退出登录。
}
admin-userlist-bulk-done-delete = { $count ->
   *[other] 已删除 { $count } 位用户的本地访问权限。
}
admin-userlist-bulk-done-profile = { $count ->
   *[other] 已更改 { $count } 位用户的功能配置文件。
}
admin-userlist-bulk-failed = 已完成 { $done } 项,失败 { $failed } 项:{ $reason }

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
admin-user-compliance-due = 到期{ $date }

admin-user-compliance-empty = 此帐户没有数据请求。

admin-user-compliance-help = 对此帐户地址提出的请求。在数据请求中打开一个以对其进行操作。

admin-user-compliance-open-all = 打开所有数据请求

admin-user-compliance-title = 此帐户的数据请求

admin-user-never = 从来没有

admin-user-overview-last-signin = 上次登录

admin-user-overview-status = 状态

admin-user-status-active = 活跃

admin-user-status-deleted = 已删除

admin-user-status-deleting = 正在删除

admin-user-status-disabled = 被阻止

admin-user-tab-access = 访问和删除

admin-user-tab-compliance = 合规性

admin-user-tab-overview = 概述

admin-user-tab-profile = 特征简介

admin-user-tab-security = 安全性

admin-user-tab-sessions = 会议

admin-user-tabs-label = 账户部分
