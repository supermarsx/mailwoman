# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = 外部コンテンツ

# -- Machine translation (2026-11): added to match en ---------------------
remote-change-failed = リモート画像の権限を更新できませんでした。{ $reason }
remote-banner-blocked = リモート画像をブロックしました
remote-banner-blocked-policy = プライバシー設定によりリモート画像をブロックしました
remote-banner-shown = 画像を表示しました
remote-banner-proxied = 画像をサーバー経由で読み込みました
remote-banner-direct = 画像を送信者のサーバーから読み込みました
remote-banner-trackers = { $count ->
   *[other] { $count } 件のトラッカーを削除しました
}
remote-banner-trackers-limited = 少なくとも { $count } 件のトラッカーを削除しました
remote-banner-show = 画像を表示
remote-banner-show-help = このメッセージの画像をサーバー経由で読み込みます。送信者に IP アドレスは見えません
remote-banner-more = その他の画像オプション
remote-banner-always-sender = { $sender } からの画像を常に表示
remote-banner-always-domain = { $domain } からの画像を常に表示
remote-banner-always-all = すべてのメールで画像を常に表示
remote-banner-settings = プライバシー設定
remote-banner-undo = 元に戻す
remote-banner-undo-help-single = このメッセージの画像を再びブロックする
remote-banner-undo-help-per-sender = { $sender } からの画像を自動的に表示しないようにする
remote-banner-undo-help-per-domain = { $domain } からの画像を自動的に表示しないようにする
remote-banner-undo-help-all = すべてのメールで画像を自動的に表示しないようにする
remote-banner-blocked-help = 他のサーバーに保存されている画像は、このメッセージをいつどこで開いたかを送信者に知らせることがあります。表示を選択するまで、ブロックされたままです。
remote-banner-help-block = プライバシー設定によって、すべてのリモート画像がブロックされます。プライバシー設定で変更できます。
remote-banner-help-ask = 画像はこのサーバー経由で読み込まれ、送信者から IP アドレスを隠します。
remote-banner-help-proxy = 画像はこのサーバー経由で読み込まれ、送信者から IP アドレスを隠します。
remote-banner-help-direct = 画像は送信者のサーバーから直接読み込まれ、IP アドレスが相手に見えます。
remote-banner-hosts = ブロックされたホスト：{ $hosts }
remote-banner-trackers-help = トラッキング画像は、メッセージが表示される前に削除されます。ダウンロード後に初めてサーバーがトラッカーと認識した画像は、一定期間のみカウントされます。
remote-images-toast-loading = 画像を読み込み中…
remote-images-toast-done = 画像を読み込みました
remote-images-toast-failed = { $count ->
   *[other] { $count } 件の画像を読み込めませんでした
}
remote-images-toast-retry = 再試行
remote-images-toast-progress = 画像を読み込み中… { $loaded }/{ $total }
remote-images-toast-retrying-progress = 画像を読み込み中… { $loaded }/{ $total } · 再試行 { $attempt }/{ $max }
remote-images-toast-cancel = キャンセル
remote-images-toast-cancelled = 画像の読み込みをキャンセルしました
remote-images-toast-load-again = もう一度読み込む
remote-banner-hosts-manage = 画像のソース…
image-hosts-title = ソース別の画像
image-hosts-help = ソースごとに画像の扱いを選択します。「常に表示しない」は「常に表示」より優先され、すべてのメッセージに適用されます。
image-hosts-none = このメッセージにはリモート画像がありません。
image-hosts-for = { $host } からの画像
image-hosts-allow = 常に表示
image-hosts-ask = 確認する
image-hosts-block = 常に表示しない
image-hosts-failed = 画像ソースの設定を保存できませんでした。
image-tip-blocked = 画像をブロックしました。「画像を表示」を選ぶと読み込まれます。
image-tip-loading = 画像を読み込み中…
image-tip-error = 画像を利用できません
