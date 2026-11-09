# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: folder-dnd. Source: ../en/folder-dnd.ftl


# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-moved = 文件夹已移动：{ $name }
folder-dnd-moved-many = 已移动 { $count } 个文件夹
folder-dnd-undo = 撤销
folder-dnd-cancel = 取消
folder-dnd-handle = 拖动 { $name } 以移动
folder-dnd-locked-hint = 文件夹顺序已由管理员固定。
folder-dnd-forced-hint = 已由管理员置顶，无法移动或取消置顶。
folder-dnd-disabled-hint = 文件夹拖放已关闭。
folder-dnd-system-hint = 系统文件夹无法移动。
folder-dnd-pinned-group = 已置顶
folder-dnd-top-level = 移到顶层
folder-dnd-problem-disabled = 移动文件夹已关闭。
folder-dnd-problem-locked = 文件夹顺序已由管理员固定。
folder-dnd-problem-self = 文件夹无法移入自身。
folder-dnd-problem-descendant = 文件夹无法移入其自身的子文件夹。
folder-dnd-problem-system = 系统文件夹无法移到此处。
folder-dnd-problem-no-children = 该文件夹不能包含子文件夹。
folder-dnd-problem-nesting-off = 管理员不允许嵌套文件夹。
folder-dnd-problem-depth = 这样会使文件夹嵌套过深。
folder-dnd-problem-forced = 此文件夹已由管理员置顶。
folder-dnd-problem-unknown = 该文件夹已不存在。
folder-dnd-problem-noop = 文件夹已在该位置。
folder-dnd-problem-collision = 此处已有名为 { $name } 的文件夹。请先重命名。
folder-dnd-problem-server = 服务器未移动 { $name }。未作任何更改。
folder-dnd-announce-up = { $name } 已上移
folder-dnd-announce-down = { $name } 已下移
folder-dnd-announce-indent = { $name } 已移入上方的文件夹
folder-dnd-announce-outdent = { $name } 已上移一级
folder-dnd-nest-title = 将 { $name } 移到此处？
folder-dnd-nest-title-many = 将 { $count } 个文件夹移到此处？
folder-dnd-nest-body = 选择是更改邮件服务器上的文件夹，还是仅更改此应用中的显示方式。
folder-dnd-nest-legend = 移动方式
folder-dnd-nest-local = 仅在此应用中排列
folder-dnd-nest-local-help = 仅影响显示。文件夹在服务器和其他邮件应用中保持原位。
folder-dnd-nest-server = 在服务器上移动
folder-dnd-nest-server-help = 在所有使用此账户的邮件应用中更改该文件夹。
folder-dnd-nest-remember = 记住我的选择
folder-dnd-nest-confirm = 移动
folder-dnd-arranged-only = 仅在此应用中排列
folder-dnd-arranged-only-help = 它在服务器上的位置不同：{ $path }
folder-dnd-apply-server = 应用到服务器
folder-dnd-reset = 将顺序重置为默认
folder-dnd-reset-done = 文件夹顺序已重置
folder-dnd-move-to = 将 { $name } 移到…
folder-dnd-layout-link = 文件夹布局
folder-dnd-reorder-enable = 通过拖动重新排列文件夹
folder-dnd-nest-mode = 嵌套文件夹时
folder-dnd-nest-mode-ask = 询问我
folder-dnd-nest-mode-server = 在服务器上移动
folder-dnd-nest-mode-local = 仅在此应用中排列
folder-dnd-new-placement = 新文件夹显示位置
folder-dnd-new-placement-end = 在同级文件夹之后
folder-dnd-new-placement-alphabetical = 按字母顺序
folder-dnd-handles = 显示拖动手柄
folder-dnd-handles-hover = 悬停时
folder-dnd-handles-always = 始终
folder-dnd-arrange-title = 排列文件夹
folder-dnd-arrange-help = 拖动文件夹可重新排序，拖到另一个文件夹上可嵌套，或在选中的文件夹上使用 Alt 和方向键。
folder-dnd-admin-locked = 由管理员设置

# -- Machine translation (2026-11): added to match en ---------------------
folder-dnd-arrange-menu = 移到…
folder-star-add = 为文件夹加星标
folder-star-remove = 取消文件夹星标
folder-star-starred = 已加星标
folder-star-toggle = 为 { $name } 加星标
folder-star-only = 仅显示加星标的
folder-star-only-help = 仅显示加星标的文件夹及包含它们的文件夹。
folder-dnd-indent = 将 { $name } 移入上方的文件夹
folder-dnd-outdent = 将 { $name } 上移一级
folder-dnd-more-actions = 排列
folder-dnd-selected-move = 将所选项移到…
folder-dnd-locked-row = 已锁定
folder-dnd-problem-pin = 置顶的文件夹保持在其他文件夹上方。请先取消置顶再将其移到那里。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
folder-bar-empty = 空……

folder-bar-label = 对选定文件夹的操作

folder-bar-mark-read = 标记全部已读

folder-bar-pin = 销

folder-bar-star = 明星

folder-bar-unpin = 取消固定

folder-bar-unstar = 取消星标

folder-dnd-handle-hint = 拖动以移动该文件夹

folder-pinned-group = 已固定

folder-pinned-unpin-zone = 放到此处取消固定

settings-folders-bulk-delete-body = { $folders ->
       *[other] { $folders } 个文件夹（及其子文件夹）及其中的{ $messages } 条消息将被删除。此操作无法撤消。
    }

settings-folders-bulk-delete-confirm = { $count ->
       *[other] 删除 { $count } 个文件夹
    }

settings-folders-bulk-delete-done = { $count ->
       *[other] 已删除 { $count } 个文件夹
    }

settings-folders-bulk-delete-failed = 什么都没有被删除。未删除：{ $names }

settings-folders-bulk-delete-list = 要删除的文件夹

settings-folders-bulk-delete-none = 所选文件夹均不可删除（系统文件夹和您不能删除的文件夹将被跳过）。

settings-folders-bulk-delete-others = 也没有删除：{ $names }

settings-folders-bulk-delete-partial = 已删除 { $done } 文件夹。未删除：{ $names }

settings-folders-bulk-delete-partial-done = 已删除 { $done } 文件夹。

settings-folders-bulk-delete-refused-legalHold = { $count ->
       *[other] 未删除，因为该帐户处于合法保留状态（{ $count } 个文件夹）：{ $names }
    }

settings-folders-bulk-delete-refused-processingRestricted = { $count ->
       *[other] 未删除，因为帐户处理受到限制（{ $count } 个文件夹）：{ $names }
    }

settings-folders-bulk-delete-skipped = { $count } 选定的文件夹将被跳过，因为它们无法删除：{ $names }

settings-folders-bulk-delete-subfolders = 同时删除每个文件夹的子文件夹

settings-folders-bulk-delete-subfolders-off = 仅删除选定的文件夹。保留仍具有所选内容之外的子文件夹的文件夹。

settings-folders-bulk-delete-title = { $count ->
       *[other] 删除{ $count }个文件夹？
    }

settings-folders-bulk-empty-body = { $folders }文件夹中的{ $total }邮件将被删除：{ $toTrash }移至垃圾箱并可撤消，{ $permanent }将永久删除且无法恢复。子文件夹不会被触及。

settings-folders-bulk-empty-confirm = 清空文件夹

settings-folders-bulk-empty-counting = 计算每个文件夹中的邮件数...

settings-folders-bulk-empty-done = 清空 { $count } 个文件夹、{ $messages } 条消息

settings-folders-bulk-empty-fail-refused = 服务器拒绝了部分内容； { $remaining } 消息剩余

settings-folders-bulk-empty-fail-stopped = 停止了； { $remaining } 消息剩余

settings-folders-bulk-empty-fail-uncertain = 最后一批结果未知；检查文件夹

settings-folders-bulk-empty-list = 要清空的文件夹

settings-folders-bulk-empty-partial = 清空{ $done }文件夹； { $failed }失败。查看列表。

settings-folders-bulk-empty-permanent = { $count } 消息被永久删除

settings-folders-bulk-empty-progress = 已完成 { $done } 个文件夹（共 { $of }），已处理 { $messages } 条消息。现在：{ $name }

settings-folders-bulk-empty-skip-count-failed = 无法计算，跳过

settings-folders-bulk-empty-skip-empty = 已为空，已跳过

settings-folders-bulk-empty-skip-permanent-denied = 将被永久删除，管理员已关闭，跳过

settings-folders-bulk-empty-skip-search = 已保存的搜索，已跳过

settings-folders-bulk-empty-skip-unsupported = 该帐号无法清空，已跳过

settings-folders-bulk-empty-title = { $count ->
       *[other] 清空 { $count } 个文件夹？
    }

settings-folders-bulk-empty-to-trash = { $count } 邮件移至垃圾箱

settings-folders-bulk-empty-undo-label = 清空 { $count } 文件夹（{ $messages } 消息）

settings-folders-delete-selected = 删除...

settings-folders-select-user = 选择所有用户文件夹

settings-folders-subscribe-failed = 无法更改订阅。

settings-folders-subscribe-partial = 更改了{ $done }文件夹； { $failed }无法更改。

settings-folders-subscribe-selected = 订阅

settings-folders-subscribed = { $count ->
       *[other] 订阅了 { $count } 个文件夹
    }

settings-folders-unsubscribe-selected = 退订

settings-folders-unsubscribed = { $count ->
       *[other] 取消订阅 { $count } 个文件夹
    }
