# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: send-delay. Source: ../en/send-delay.ftl


# -- Machine translation (2026-11): added to match en ---------------------
send-delay-title = 送信の取り消し
send-delay-help = 送信を押すと、メッセージはこの時間だけ送信トレイで待機します。それまでは取り戻して編集を続けられます。メッセージはサーバーが保持するため、ブラウザーを閉じても早く送信されたり失われたりすることはありません。
send-delay-label = 送信済みメッセージを保留する時間
send-delay-off = オフ（すぐに送信）
send-delay-seconds = { $n } 秒
send-delay-minutes = { $n ->
   *[other] { $n } 分
}
send-delay-custom = カスタム…
send-delay-custom-label = カスタムの遅延時間（秒）
send-delay-custom-help = 0 から { $max } までの整数の秒数。
send-delay-capped = 管理者によって { $max } までに制限されています。
send-delay-disabled = 送信済みメッセージの取り戻しは管理者によって無効にされています。メッセージはすぐに送信されます。
send-delay-invalid = 0 から { $max } までの整数の秒数を入力してください。
send-later-button = 後で送信
send-later-menu = 後で送信のオプション
send-later-preset-hour = 1 時間後
send-later-preset-today = 今日の後ほど、17:00
send-later-preset-tomorrow = 明日の朝、09:00
send-later-preset-monday = 月曜日の朝、09:00
send-later-custom = 日付と時刻を選択…
send-later-custom-label = 送信日時
send-later-zone = タイムゾーン：{ $zone }
send-later-schedule = 送信を予約
send-later-cancel = キャンセル
send-later-err-invalid = 有効な日付と時刻を入力してください。
send-later-err-past = 未来の時刻を選んでください。
send-later-err-horizon = 先すぎます。メッセージは最大 { $days } 日先まで予約できます。
send-later-err-off = 予約送信は管理者によって無効にされています。
send-queued = メッセージを待機中です。送信まで
send-queued-undo = 元に戻す
send-queued-now = 今すぐ送信
send-queued-recalled = メッセージを取り戻しました。下書きに戻っています。
send-queued-too-late = 取り戻せません。メッセージはすでに送信中です。
send-queued-sent-now = 送信中です。
send-queued-scheduled = { $when }（{ $zone }）に送信を予約しました。
outbox-scheduled-badge = 予約済み
outbox-sends-in = { $time }後に送信
outbox-sends-at = 送信：{ $when }
outbox-recall-edit = 取り戻して編集
outbox-recall-too-late = すでに送信中のため、取り戻せません。
outbox-failed-drafts = 送信できなかったため、下書きに保存されています。
outbox-queued-count = { $count ->
   *[other] { $count } 件のメッセージが送信待ち
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
outbox-chip-next = 次の送信: { $when }

outbox-chip-title = 送信を待機しているメッセージ

outbox-chip-view-all = 送信トレイを開く

outbox-delete = 削除

outbox-delete-confirm = メッセージを削除しますか?

outbox-delete-failed = メッセージを削除できませんでした。 { $error }

outbox-deleted = キューに入れられたメッセージが削除されました。

outbox-edit-time = 送信時刻を編集する

outbox-edit-time-conflict = 送信時間が変更されたか、メッセージはすでに送信されました。送信トレイが再ロードされました。

outbox-edit-time-current = 現在 { $when }

outbox-edit-time-refused = 送信時刻を変更できませんでした。 { $error }

outbox-edit-time-save = 送信時間を節約する

outbox-failed-toast = メッセージを送信できませんでした。 { $error }

outbox-failed-uncertain-toast = メッセージは配信された場合もあれば、配信されなかった場合もあります。再度送信する前に「送信済み」にチェックを入れてください。

outbox-open = 開く

outbox-reader-actions = キューに入れられたメッセージのアクション

outbox-reader-cc = Cc: { $recipients }

outbox-reader-label = キューに入れられたメッセージ

outbox-reader-loading = メッセージを読み込んでいます…

outbox-reader-to = 宛先: { $recipients }

outbox-reader-unavailable = メッセージを読み込めませんでした。

outbox-recall-to-drafts = 下書きをリコールする

outbox-send-again = 再送信

outbox-send-again-failed = メッセージを再度キューに入れることができませんでした。 { $error }

outbox-send-again-queued = メッセージは再送信するためにキューに入れられます。

outbox-sent-copy-folder = { $folder }に保存されます

outbox-sent-copy-missing = 送信済みに保存されます（選択したフォルダーは使用できません）

outbox-sent-copy-skip = コピーは保存されません

outbox-status-attempts = { $count ->
       *[other] これまでに { $count } 回の試行
    }

outbox-status-canceled = キャンセルされました

outbox-status-failed = 送信されませんでした

outbox-status-held = 解放するまで保持される

outbox-status-last-error = 最後のエラー: { $error }

outbox-status-next-attempt = 次の試行 { $when }

outbox-status-queued = キューに入れられました

outbox-status-retrying = 再試行中

outbox-status-scheduled = 予定されている

outbox-status-sending = 送信中

outbox-status-sends-at-in = { $when } を送信します ({ $time } 内)

outbox-status-sent = 送信済み

outbox-status-uncertain = 配送不明
