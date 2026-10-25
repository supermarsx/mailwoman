# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: remote-images. Source: ../en/remote-images.ftl

remote-bar-label = 外部コンテンツ

# -- Blocked state: count summary -------------------------------------------
remote-blocked-count = { $count ->
   *[other] { $count } 件の外部画像をブロックしました
}
remote-blocked-trackers = { $blocked ->
   *[other] { $blocked } 件の外部画像のうち
}、{ $count ->
   *[other] { $count } 件のトラッカーをブロックしました
}
remote-blocked-hosts = ブロックしたホスト

# -- Grant actions (deny-by-default; each loads exactly what it names) --------
remote-grant-once = 画像を読み込む
remote-grant-sender = { $sender } からは常に読み込む
remote-grant-domain = { $domain } からは常に読み込む
remote-grant-all = すべての外部画像を常に読み込む

remote-grant-done-single = このメッセージの外部画像を読み込みました。
remote-grant-done-per-sender = この送信者からの外部画像を読み込むようになります。
remote-grant-done-per-domain = このドメインからの外部画像を読み込むようになります。
remote-grant-done-all = すべてのメールで外部画像を読み込むようになります。

# -- Allowed state: revoke ---------------------------------------------------
remote-allowed = このメッセージの外部画像を読み込んでいます。
remote-revoke = オフにする
remote-revoke-done = 外部画像をオフにしました。次に開いたときは再びブロックされます。
