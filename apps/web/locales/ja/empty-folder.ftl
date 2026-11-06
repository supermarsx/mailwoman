# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: empty-folder. Source: ../en/empty-folder.ftl


# -- Machine translation (2026-11): added to match en ---------------------
empty-folder-action = フォルダーを空にする…
empty-folder-title = 「{ $name }」を空にしますか？
empty-folder-body-trash = { $count ->
   *[other] { $name } 内の { $count } 件のメッセージすべてがゴミ箱に移動されます。ゴミ箱から復元できます。
}サブフォルダーとその中のメッセージには影響しません。
empty-folder-body-destroy-trash = { $count ->
   *[other] ゴミ箱内の { $count } 件のメッセージすべてが完全に削除されます。この操作は元に戻せません。
}サブフォルダーとその中のメッセージには影響しません。
empty-folder-body-destroy-junk = { $count ->
   *[other] { $name } 内の { $count } 件のメッセージすべてが完全に削除されます。この操作は元に戻せません。
}サブフォルダーとその中のメッセージには影響しません。
empty-folder-body-destroy-no-trash = このアカウントにはゴミ箱フォルダーがないため、{ $count ->
   *[other] { $name } 内の { $count } 件のメッセージすべてが完全に削除されます。
}この操作は元に戻せません。サブフォルダーとその中のメッセージには影響しません。
empty-folder-confirm-trash = ゴミ箱に移動
empty-folder-confirm-destroy = 完全に削除
empty-folder-cancel = キャンセル
empty-folder-already-empty = 「{ $name }」はすでに空です。
empty-folder-prepare-failed = 「{ $name }」の件数を数えられなかったため、何も変更されていません。接続を確認して、もう一度お試しください。
empty-folder-syncing = 「{ $name }」はまだ同期中のため、件数を数えられません。何も変更されていません。しばらくしてからもう一度お試しください。
empty-folder-running = すでに別のフォルダーを空にしています。別のフォルダーを始める前に、その通知から停止してください。
empty-folder-progress-trash = { $name } のメッセージをゴミ箱に移動中：{ $done } / { $total } 件。
empty-folder-progress-destroy = { $name } のメッセージを削除中：{ $done } / { $total } 件。
empty-folder-stop = 停止
empty-folder-stopping = 現在のバッチの後で停止します。これまでに { $total } 件中 { $done } 件が完了しました。
empty-folder-done-trash = { $done ->
   *[other] { $name } の { $done } 件のメッセージをゴミ箱に移動しました。
}
empty-folder-done-destroy = { $done ->
   *[other] { $name } の { $done } 件のメッセージを完全に削除しました。
}
empty-folder-stopped = 停止しました。{ $total } 件中 { $done } 件のメッセージを処理しました。{ $name } にはまだ { $remaining } 件あります。
empty-folder-refused = 停止しました。{ $total } 件中 { $done } 件のメッセージを処理しました。メールサーバーが { $refused } 件を拒否しました。{ $name } にはまだ { $remaining } 件あります。
empty-folder-failed = { $name } を空にする処理は、{ $done } 件のメッセージを処理した後、エラーで停止しました。最後のバッチが完了したかどうかは不明です。もう一度試す前にフォルダーを確認してください。
