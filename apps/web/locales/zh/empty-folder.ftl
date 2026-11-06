# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: zh · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = 清空文件夹…
empty-folder-title = 清空“{ $name }”？
empty-folder-body-trash = { $count ->
   *[other] { $name } 中的全部 { $count } 封邮件将被移到废纸篓，你可以在那里恢复它们。
}子文件夹及其中的邮件不受影响。
empty-folder-body-destroy-trash = { $count ->
   *[other] 废纸篓中的全部 { $count } 封邮件将被永久删除。此操作无法撤销。
}子文件夹及其中的邮件不受影响。
empty-folder-body-destroy-junk = { $count ->
   *[other] { $name } 中的全部 { $count } 封邮件将被永久删除。此操作无法撤销。
}子文件夹及其中的邮件不受影响。
empty-folder-body-destroy-no-trash = 此账户没有废纸篓文件夹，因此{ $count ->
   *[other] { $name } 中的全部 { $count } 封邮件将被永久删除。
}此操作无法撤销。子文件夹及其中的邮件不受影响。
empty-folder-confirm-trash = 移到废纸篓
empty-folder-confirm-destroy = 永久删除
empty-folder-cancel = 取消
empty-folder-already-empty = “{ $name }”已经是空的。
empty-folder-prepare-failed = 无法统计“{ $name }”中的邮件数量，因此未做任何更改。请检查网络连接后重试。
empty-folder-syncing = “{ $name }”仍在同步，暂时无法统计。未做任何更改，请稍后再试。
empty-folder-running = 已有一个文件夹正在清空。请先在其通知中停止，再开始另一个。
empty-folder-progress-trash = 正在将 { $name } 中的邮件移到废纸篓：{ $done } / { $total }。
empty-folder-progress-destroy = 正在删除 { $name } 中的邮件：{ $done } / { $total }。
empty-folder-stop = 停止
empty-folder-stopping = 将在当前批次完成后停止。目前已完成 { $done } / { $total }。
empty-folder-done-trash = { $done ->
   *[other] 已将 { $name } 中的 { $done } 封邮件移到废纸篓。
}
empty-folder-done-destroy = { $done ->
   *[other] 已永久删除 { $name } 中的 { $done } 封邮件。
}
empty-folder-stopped = 已停止。已处理 { $done } / { $total } 封邮件；{ $name } 中仍有 { $remaining } 封。
empty-folder-refused = 已停止。已处理 { $done } / { $total } 封邮件；邮件服务器拒绝了 { $refused } 封。{ $name } 中仍有 { $remaining } 封。
empty-folder-failed = 清空 { $name } 时在处理 { $done } 封邮件后因错误而停止。最后一批可能已完成，也可能未完成；请先检查该文件夹再重试。
