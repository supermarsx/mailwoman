# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: mail. Source: ../en/mail.ftl

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = 設定
mail-compose = 作成
mail-nav-mailboxes = メールボックス
mail-nav-attachments = 添付ファイル
mail-nav-outbox = 送信トレイ
mail-nav-sharing = フォルダーを共有
mail-nav-apps = アプリ
mail-logout = ログアウト
mail-offline = オフライン
mail-module-loading = { $module } を読み込み中…

# -- Search ------------------------------------------------------------------
mail-search = 検索
mail-search-label = メールを検索
mail-search-placeholder = メールを検索 — from:alice subject:invoice larger:1mb
mail-search-clear = クリア

# -- Message list ------------------------------------------------------------
mail-list-label = メッセージ
mail-loading = メッセージを読み込み中…
mail-empty = メッセージはありません
mail-unknown-sender = （不明な送信者）
mail-no-subject = （件名なし）
mail-pinned = ピン留め
mail-has-attachment = 添付ファイルあり
mail-answered = 返信済み
mail-forwarded = 転送済み
mail-unread = 未読
mail-flagged = フラグ付き
mail-selected = 選択済み
mail-row-position = メッセージ { $pos } / { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
   *[other] { $count } 件のメッセージ
}
mail-thread-expand = スレッドを展開（{ $count ->
   *[other] { $count } 件のメッセージ
}）
mail-thread-collapse = スレッドを折りたたむ（{ $count ->
   *[other] { $count } 件のメッセージ
}）

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = 表示オプション
mail-reading-pane = 閲覧ウィンドウ
mail-reading-pane-right = 右
mail-reading-pane-bottom = 下
mail-reading-pane-off = オフ

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = 選択したメッセージ
mail-bulk-count = { $count } 件を選択
mail-mark-read = 既読にする
mail-mark-unread = 未読にする
mail-flag-message = フラグを付ける
mail-unflag-message = フラグを外す
mail-move-to = フォルダーに移動
mail-move-to-placeholder = 移動先…
mail-select-all = すべて選択
mail-clear-selection = 選択を解除

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-focused-enable = 優先受信トレイ
mail-inbox-filter = 受信トレイのフィルター
mail-inbox-focused = 優先
mail-inbox-other = その他
mail-inbox-turn-off = オフにする

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = 開いているタブ
mail-subtab-pin = { $title } をピン留め
mail-subtab-unpin = { $title } のピン留めを外す
mail-subtab-tearoff = { $title } を新しいウィンドウで開く
mail-subtab-close = { $title } を閉じる

# -- Per-row actions ---------------------------------------------------------
mail-pin = ピン留め
mail-unpin = ピン留めを外す
mail-snooze = スヌーズ
mail-snooze-menu = スヌーズの期限
mail-snooze-later = 今日の後ほど
mail-snooze-tomorrow = 明日
mail-snooze-next-week = 来週
mail-unsnooze = スヌーズを解除
mail-label = ラベル
mail-labels-menu = ラベル
mail-flag = フォローアップのフラグ
mail-clear-flag = フォローアップを解除
mail-archive = アーカイブ
mail-delete = 削除
mail-more-actions = その他の操作

# -- Reader ------------------------------------------------------------------
mail-reader-label = メッセージ
mail-reader-empty = 読むメッセージを選択してください
mail-back = 戻る
mail-reader-from = 差出人：{ $addr }
mail-reader-to = 宛先：{ $addr }
mail-reader-cc = Cc：{ $addr }
mail-reader-actions = メッセージの操作
mail-reply = 返信
mail-reply-all = 全員に返信
mail-forward = 転送
mail-reply-failed = メッセージを準備できませんでした。もう一度お試しください。
mail-reply-unthreaded = 元のメッセージの ID を読み取れませんでした。この返信は元のメッセージとスレッドにまとめられません。
mail-spam = スパム
mail-export = エクスポート
mail-sweep-sender = 送信者のメールを一掃
mail-message-body = メッセージ本文
mail-no-content = 内容がありません
mail-sanitizing = 無害化中…
mail-attachments = 添付ファイル
mail-attachment-unnamed = （名前なし）
mail-attachment-open = 添付ファイル { $name }
mail-attachment-close = 添付ファイルを閉じる
mail-attachment-loading = 添付ファイルを読み込み中…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = 暗号化されたメッセージ
mail-encrypted-note = 🔒 このメッセージはエンドツーエンドで暗号化されています。読むには、このデバイスでロックを解除してください。
mail-key-passphrase = 鍵のパスフレーズ
mail-decrypt = 復号
mail-decrypting = 復号中…
mail-decrypt-no-key = このメッセージを復号できる秘密鍵がありません。
mail-decrypt-failed = 復号に失敗しました

# -- Reply and forward: text written into the outgoing message --------------
mail-quote-attribution = { $date }、{ $name } さんは書きました：
mail-quote-attribution-undated = { $name } さんは書きました：
mail-forward-heading = ---------- 転送メッセージ ----------
mail-forward-from = 差出人：{ $addr }
mail-forward-date = 日時：{ $date }
mail-forward-subject = 件名：{ $subject }
mail-forward-to = 宛先：{ $addr }
mail-forward-cc = Cc：{ $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = メッセージを作成
mail-compose-title = 新規メッセージ
mail-compose-title-reply = 返信
mail-compose-title-reply-all = 全員に返信
mail-compose-title-forward = 転送
mail-compose-close = 閉じる
mail-compose-from = 差出人
mail-compose-from-default = 既定
mail-compose-to = 宛先
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = 連絡先の候補
mail-compose-cc = Cc
mail-compose-bcc = Bcc
mail-compose-show-cc-bcc = Cc または Bcc を追加
mail-compose-bcc-encrypted = 暗号化されたメッセージにはすべての受信者の鍵が記載されるため、Bcc の受信者が他の受信者に見えてしまいます。Bcc の受信者を削除するか、暗号化をオフにしてください。
mail-compose-plaintext-confirm-title = 復号したテキストを暗号化せずに送信しますか？
mail-compose-plaintext-confirm-body = このメッセージは暗号化されたメッセージのテキストを引用しています。暗号化する設定になっていないため、引用したテキストは保護されずに送信されます。
mail-compose-plaintext-confirm-send = 暗号化せずに送信
mail-compose-plaintext-confirm-cancel = 戻る
mail-compose-subject = 件名
mail-compose-body = 本文
mail-compose-attachments = 添付ファイル
mail-compose-remove-attachment = { $name } を削除
mail-compose-attach-nextcloud = Nextcloud から添付
mail-compose-close-nextcloud = Nextcloud を閉じる
mail-compose-attach-file = ファイルを添付
mail-compose-uploading = アップロード中…
mail-compose-upload-unavailable = 現在ファイルをアップロードできません。
mail-compose-upload-failed = { $name } をアップロードできませんでした。
mail-compose-upload-too-large = { $name } は { $size } MB です。アップロードできる最大サイズは { $max } MB です。
mail-compose-send-later = 後で送信
mail-compose-cancel = キャンセル
mail-compose-send = 送信
mail-compose-schedule = 予約
mail-compose-sending = 送信中…
mail-compose-encrypted-subject = 暗号化されたメッセージ
mail-compose-dlp-blocked = 情報漏えい防止ルールにより送信がブロックされています（上の警告を参照）。
mail-compose-send-failed = 送信に失敗しました
mail-compose-send-later-past = 未来の送信日時を選ぶか、欄を空にしてすぐに送信してください。
mail-send-not-signed-in = サインインしていません。
mail-send-no-draft-folder = このアカウントには下書きを保存するフォルダーがありません。
mail-send-no-recipient = 受信者を 1 人以上追加してください。
mail-send-bad-recipient = { $count ->
   *[other] メールアドレスではありません：{ $addresses }
}
mail-send-later-past = 送信日時が未来ではありません。何も送信されていません。
mail-send-draft-refused = サーバーがこのメッセージを拒否しました：{ $reason }。何も送信されていません。
mail-send-submission-refused = サーバーがこのメッセージの送信を拒否しました：{ $reason }。何も送信されていません。
mail-compose-sign-unlock-title = 署名鍵のロックを解除
mail-compose-sign-unlock-note = このメッセージに署名するには、鍵のパスフレーズを入力してください。ロック解除はこの作成画面でのみ有効です。
mail-compose-sign-unlock = ロック解除
mail-compose-sign-unlocking = ロック解除中…
mail-compose-sign-no-key = このデバイスには署名鍵がありません。
mail-compose-sign-unlock-failed = 署名鍵のロックを解除できませんでした。
mail-compose-sign-unlock-required = この署名付きメッセージを送信するには、署名鍵のロックを解除してください。

mail-compose-format-plain = プレーンテキスト
mail-compose-format-rich = リッチテキスト
mail-compose-rt-toolbar = 書式
mail-compose-rt-bold = 太字
mail-compose-rt-italic = 斜体
mail-compose-rt-underline = 下線
mail-compose-rt-strike = 取り消し線
mail-compose-rt-bullet = 箇条書き
mail-compose-rt-ordered = 番号付きリスト
mail-compose-rt-quote = 引用
mail-compose-rt-link = リンク
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = 適用

mail-compose-signature = 署名
mail-compose-signature-none = 署名を追加…

mail-compose-recall = 取り消し
mail-compose-recall-action = 取り消し
mail-compose-recall-holding = 保留中、送信待ち
mail-compose-recall-scheduled = { $when } に予約済み

mail-compose-drafts = 下書き
mail-compose-drafts-empty = 保存された下書きはありません
mail-compose-drafts-delete = 下書きを削除
mail-compose-drafts-no-recipient = （受信者なし）

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = 送信トレイ
mail-outbox-refresh = 更新
mail-outbox-empty = 送信待ちのものはありません
mail-outbox-send-now = 今すぐ送信
mail-outbox-cancel = キャンセル
mail-outbox-scheduled = 予約済み
mail-outbox-holding = まもなく送信
mail-outbox-sent = 送信済み
mail-outbox-canceled = キャンセル済み
mail-outbox-held = 保留中
mail-outbox-failed = 未送信
mail-outbox-release = 解放
mail-outbox-discard = 破棄
mail-outbox-held-note = 保留中。解放するまで送信されません。
mail-outbox-held-note-api-key = 保留中 — API キー { $name } が作成。解放するまで送信されません。
mail-outbox-held-note-oauth = 保留中 — 接続済みアプリ { $name } が作成。解放するまで送信されません。
mail-outbox-origin-api-key = API キー { $name } が作成
mail-outbox-origin-oauth = 接続済みアプリ { $name } が作成
mail-outbox-to = 宛先：{ $recipients }
mail-outbox-no-subject = （件名なし）
mail-outbox-attempts = { $count } 回試行しました。最後のエラー：{ $error }
mail-outbox-error = エラー：{ $error }
mail-outbox-toast-canceled = 送信をキャンセルしました
mail-outbox-toast-discarded = 破棄しました。メッセージは送信されていません。
mail-outbox-toast-sent = 送信しました
mail-outbox-toast-retrying = まだ送信されていません：{ $error }。再試行します。
mail-outbox-toast-failed = 送信されませんでした：{ $error }
mail-outbox-toast-not-released = 送信できませんでした：{ $error }
mail-outbox-toast-not-canceled = キャンセルできませんでした：{ $error }

# -- Network status and the offline queue ------------------------------------
mail-net-lost = 接続が切れました — 再試行中…
mail-net-back = オンラインに戻りました
mail-queue-sent = { $count ->
   *[other] キューにあった { $count } 件の変更を送信しました
}
mail-queue-failed = { $count ->
   *[other] キューにあった { $count } 件の変更が失敗しました
}
mail-queue-notice-label = 適用できなかった変更
mail-queue-notice-title = { $count ->
   *[other] オフライン中に行った { $count } 件の変更を適用できませんでした
}
mail-queue-kind-send = 「{ $subject }」を送信
mail-queue-kind-send-untitled = 件名のないメッセージを送信
mail-queue-kind-move = メッセージを移動
mail-queue-kind-flag = ラベルまたはフラグを変更
mail-queue-kind-draft = 下書きを保存
mail-queue-kind-pim = カレンダー、タスク、メモ、連絡先の変更
mail-queue-reason = 理由：{ $reason }
mail-queue-reason-unknown = サーバーは理由を示しませんでした。
mail-queue-retry = 再試行
mail-queue-discard = 破棄
mail-queue-retry-item = 再試行：{ $what }
mail-queue-discard-item = 破棄：{ $what }
mail-queue-discarded = 破棄しました。変更は適用されていません。

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = メッセージを一掃
mail-sweep-title = { $sender } のメールを一掃
mail-sweep-all = この送信者のメールをすべて削除
mail-sweep-keep-latest = 最新のものを残して残りを削除
mail-sweep-older-than = N 日より古いものを削除
mail-sweep-block = すべて削除してこの送信者をブロック
mail-sweep-days = 日数
mail-sweep-count = { $count ->
   *[other] { $count } 件のメッセージがゴミ箱に移動されます
}
mail-sweep-cancel = キャンセル
mail-sweep-run = { $count } 件を一掃
mail-sweeping = 一掃中…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = 閉じる
mail-undo-action = 元に戻す
mail-undo-send-cancel = キャンセル
mail-undo-failed = 元に戻せませんでした

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = ラベルを追加しました
mail-toast-label-removed = ラベルを削除しました
mail-toast-pinned = ピン留めしました
mail-toast-unpinned = ピン留めを外しました
mail-toast-snoozed = スヌーズしました
mail-toast-follow-up-set = フォローアップを設定しました
mail-toast-follow-up-cleared = フォローアップを解除しました
mail-toast-archived = アーカイブしました
mail-toast-trashed = ゴミ箱に移動しました
mail-toast-marked-spam = スパムとしてマークしました
mail-toast-moved-to = { $folder } に移動しました
mail-toast-moved = フォルダーに移動しました
mail-toast-archived-count = { $count } 件のメッセージをアーカイブしました
mail-toast-trashed-count = { $count } 件のメッセージをゴミ箱に移動しました
mail-toast-marked-spam-count = { $count } 件のメッセージをスパムとしてマークしました
mail-toast-moved-count = { $count } 件のメッセージを移動しました
mail-toast-moved-to-count = { $count } 件のメッセージを { $folder } に移動しました
mail-toast-marked-read = { $count ->
   *[other] { $count } 件のメッセージを既読にしました
}
mail-toast-marked-unread = { $count ->
   *[other] { $count } 件のメッセージを未読にしました
}
mail-toast-flagged = { $count ->
   *[other] { $count } 件のメッセージにフラグを付けました
}
mail-toast-unflagged = { $count ->
   *[other] { $count } 件のメッセージからフラグを外しました
}
mail-toast-move-failed = メッセージを移動できませんでした。何も移動されていません。
mail-toast-update-failed = メッセージを変更できませんでした。何も変更されていません。
mail-toast-update-refused = { $count ->
   *[other] サーバーが { $count } 件のメッセージの変更を拒否しました
}
mail-toast-no-archive-folder = アーカイブフォルダーがありません
mail-toast-no-trash-folder = ゴミ箱フォルダーがありません
mail-toast-no-spam-folder = スパムフォルダーがありません
mail-toast-sweep-nothing = 一掃するものはありません
mail-toast-swept = { $count ->
   *[other] { $count } 件のメッセージを一掃しました
}
mail-toast-load-more-failed = これ以上メッセージを読み込めませんでした
mail-toast-export-nothing = エクスポートするものはありません
mail-toast-exported = .eml をエクスポートしました
mail-toast-export-failed = エクスポートに失敗しました
mail-toast-send-queued = キューに追加しました — オンラインに戻ると送信されます
mail-toast-send-scheduled = 送信を予約しました
mail-toast-sent = メッセージを送信しました
mail-toast-send-canceled = 送信をキャンセルしました

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = リボン
mail-ribbon-tab-home = ホーム
mail-ribbon-tab-view = 表示
mail-ribbon-tab-folder = フォルダー
mail-ribbon-expand = 展開
mail-ribbon-collapse = 折りたたむ
mail-ribbon-group-new = 新規
mail-ribbon-compose = 作成
mail-ribbon-group-session = セッション
mail-ribbon-logout = ログアウト
mail-ribbon-group-theme = テーマ
mail-ribbon-group-density = 表示密度
mail-ribbon-group-settings = 設定
mail-ribbon-more = その他…
mail-ribbon-group-folders = フォルダー
mail-density-compact = コンパクト
mail-density-cozy = 標準
mail-density-relaxed = ゆったり

# -- Message security level: the three-position switch in the reader toolbar
mail-maxsec-label = メッセージのセキュリティレベル
mail-maxsec-full = 完全
mail-maxsec-full-hint = 画像とスタイルを含む、無害化した HTML。
mail-maxsec-no-media = メディアなし
mail-maxsec-no-media-hint = 画像とメディアをすべて取り除いた、無害化した HTML。
mail-maxsec-plain-text = プレーンテキスト
mail-maxsec-plain-text-hint = HTML を表示しません — メッセージはプレーンテキストのみで表示されます。
