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
mail-inbox-filter = 受信トレイのフィルター
mail-inbox-focused = 優先
mail-inbox-other = その他

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
mail-encrypted-note = このメッセージはエンドツーエンドで暗号化されています。読むには、このデバイスでロックを解除してください。
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
mail-compose-options = 送信オプション
mail-compose-request-receipt = 開封確認を要求する
mail-compose-request-receipt-hint = 受信者のメールソフトは、受信者に確認する、確認せずに送信する、または要求を無視する場合があります。
mail-compose-track-opens = このメッセージが開かれた日時を記録する
mail-compose-track-opens-hint = このサーバーから読み込まれる 1×1 の画像がメッセージに追加されます。多くのメールソフトは画像をブロックしたり事前に読み込んだりするため、開封が記録されてもメッセージが読まれた証明にはならず、記録がなくても読まれていない証明にはなりません。受信者には通知されません。
mail-compose-image-type = { $name } は本文に配置できません。PNG、JPEG、GIF、WebP のいずれかの画像を使ってください。
mail-send-not-signed-in = サインインしていません。
mail-send-no-draft-folder = このアカウントには下書きを保存するフォルダーがありません。
mail-send-no-recipient = 受信者を 1 人以上追加してください。
mail-send-bad-recipient = { $count ->
   *[other] メールアドレスではありません：{ $addresses }
}
mail-send-later-past = 送信日時が未来ではありません。何も送信されていません。
mail-send-tracking-failed = 現在は開封を記録できないため、何も送信されていません。「このメッセージが開かれた日時を記録する」をオフにすると、記録なしで送信できます。
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
mail-compose-rt-image = 画像を挿入

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
mail-toast-pinned = 「{ $target }」をピン留めしました
mail-toast-unpinned = 「{ $target }」のピン留めを外しました
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
mail-toast-tracking-not-bound = メッセージは送信されましたが、開封は記録されません。
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
mail-feature-off = 設定または管理者によりオフになっています
mail-send-disabled = 送信は設定または管理者によりオフになっています。下書きは引き続き保存できます。

# -- Machine translation (2026-11): added to match en ---------------------
mail-nav-events = イベント
mail-events-upcoming = 今後のみ
mail-events-filter = イベントフィルター
mail-loading-more = 古いメッセージを読み込み中…
mail-event-state-invitation = 招待
mail-event-state-updated = 更新済み
mail-event-state-cancelled = キャンセル済み
mail-event-state-reply = 返信
mail-event-state-counter = 新しい時間を提案
mail-event-state-event = イベント
mail-event-hint = { $state }、{ $when }
mail-row-actions-expand = その他の操作を表示
mail-remote-moved = このメッセージは別のデバイスまたはアプリで { $folder } に移動されました。開いたコピーを引き続き読んでいます。
mail-remote-moved-unknown = このメッセージは別のデバイスまたはアプリで移動されました。開いたコピーを引き続き読んでいます。
mail-remote-deleted = このメッセージは別のデバイスまたはアプリで削除されました。閉じるまではこのコピーを読み続けられます。
mail-remote-dismiss = 閉じる
mail-reply-draft = 返信を作成中
mail-reply-draft-open = 下書きを開く
mail-draft-load-failed = 下書きを開けませんでした。もう一度お試しください。
mail-reply-images-omitted =
    { $count ->
       *[other] { $count } 個の画像を返信に含められませんでした。
    }
mail-export-forbidden = 管理者がこのアカウントのメッセージのエクスポートを無効にしています。
mail-export-open-failed = エクスポートのオプションを開けませんでした。ワークスペースのタブを1つ閉じてからもう一度お試しください。
mail-headers-title = 元のメッセージヘッダー
mail-headers-action-help = 配信や認証の詳細を含む、元のメッセージヘッダーを表示します。
mail-headers-help = メッセージの元のヘッダーです。折り返し行はそのまま保持され、ここにはヘッダーのみが表示されます。
mail-headers-loading = メッセージヘッダーを読み込み中…
mail-headers-failed = メッセージヘッダーを読み込めませんでした。接続を確認してもう一度お試しください。
mail-headers-copy = ヘッダーをコピー
mail-headers-copied = ヘッダーをコピーしました。
mail-headers-copy-failed = ヘッダーをコピーできませんでした。テキストを選択して手動でコピーしてください。
mail-attachment-failed = この添付ファイルをダウンロードできませんでした。接続が利用できるときにもう一度お試しください。
mail-compose-rt-rule = 水平線
mail-compose-signature-new = 新規メッセージ
mail-compose-signature-replies = 返信
mail-editor-body-size = { $size } KiB
mail-editor-body-size-label = 本文のサイズ
mail-editor-body-size-help = HTML マークアップと埋め込み画像を含む、現在の本文の UTF-8 でのサイズです。メッセージヘッダーと個別の添付ファイルは含まれません。
mail-outbox-load-failed = 送信トレイを確認できませんでした。接続を確認してから「更新」をお試しください。
mail-outbox-unavailable = 送信トレイを利用できません。「更新」でもう一度お試しください。
mail-maxsec-view = 表示：{ $mode }
mail-maxsec-menu = 表示モード
mail-toast-tab-pinned = タブ「{ $target }」を固定しました
mail-toast-tab-unpinned = タブ「{ $target }」の固定を解除しました
mail-workspace-message = メッセージ
mail-opened-announcement = 開きました：{ $subject }
mail-workspace-layout = ワークスペースのレイアウト
mail-workspace-tabs = タブ
mail-workspace-side-by-side = 左右に並べる
mail-workspace-stack = 上下に重ねる
mail-workspace-refresh = フォルダーを更新
mail-workspace-load-error = このメッセージを読み込めませんでした。タブを閉じてもう一度お試しください。
mail-resize-folders = フォルダーサイドバーのサイズを変更
mail-resize-list = メッセージ一覧のサイズを変更
mail-resize-tiles = ワークスペースのペインのサイズを変更
mail-show-folders = フォルダーサイドバーを表示
mail-hide-folders = フォルダーサイドバーを非表示
mail-compact-folders = フォルダーアイコンをコンパクトにする
mail-toggle-list = メッセージ一覧の表示/非表示
mail-toggle-reader = 閲覧ペインの表示/非表示
mail-folder-counts = 未読 { $unread } 件、全 { $total } 件
mail-print = 印刷 / PDF として保存
mail-go-to-top = 先頭へ移動
mail-print-help = このメッセージを印刷するか、印刷ダイアログで「PDF として保存」を選択します。外部画像は引き続きブロックされます。
mail-file-details = ファイルの詳細とフィンガープリント
mail-file-type = ファイルの種類
mail-file-extension = 拡張子
mail-file-detected = 検出されたシグネチャ
mail-file-unknown = 認識できるシグネチャがありません
mail-file-size = サイズ
mail-file-size-bytes = { $count } バイト
mail-file-hash-unavailable = フィンガープリントを利用できません（セキュアなブラウザーと 64 MiB までのファイルが必要です）。
mail-file-fingerprint-help = フィンガープリントはこのバイト列そのものを識別します。ファイルの安全性や作成者を証明するものではありません。
mail-file-warning-executable = このファイルはプログラムを実行したり、システム設定を変更したりする可能性があります。提供元を信頼できる場合のみダウンロードしてください。
mail-file-warning-macro = この文書にはマクロが含まれている可能性があります。送信者と目的を確認するまでマクロを有効にしないでください。
mail-file-warning-disguisedName = ファイル名が本来の拡張子を偽装している可能性があります。ダウンロードする前にファイルの詳細を確認してください。
mail-file-warning-typeMismatch = 検出されたファイルのシグネチャが宣言された種類と一致しません。自動プレビューはブロックされました。
mail-file-warning-activeDocument = この形式にはアクティブなコンテンツが含まれる可能性があります。プレビューは分離されており、ダウンロードしたコピーを開くと動作が異なる場合があります。
mail-file-preview-blocked = このファイルの種類、または種類の不一致のため、自動プレビューは無効です。
mail-quote-header-from = 差出人：
mail-quote-header-sent = 送信日時：
mail-quote-header-to = 宛先：
mail-quote-header-cc = Cc：
mail-quote-header-subject = 件名：
mail-spam-score = 迷惑メール分析
mail-spam-score-loading = 迷惑メールのヘッダーを確認中…
mail-spam-score-unavailable = 迷惑メールスコアを利用できません
mail-spam-score-unverified = メッセージヘッダーによる報告であり、独自の検証はされていません。送信者が指定したヘッダーは偽造される可能性があります。
mail-spam-score-positive = ヘッダーは迷惑メールと報告しています。
mail-spam-score-negative = ヘッダーは迷惑メールではないと報告しています。
mail-spam-score-unknown = このスコアには判定が付属していません。
mail-external-recipients = 外部の宛先
mail-external-count = 信頼済みドメイン外の宛先が { $count } 件あります
mail-external-help = 機密情報を送信する前に、宛先、Cc、Bcc を確認してください。信頼の判定は、選択した送信元 ID と有効なドメインポリシーに基づきます。
mail-external-reviewed = これらの外部宛先を確認しました。
mail-external-confirm-required = 送信する前に、外部の宛先を確認して承認してください。
mail-quick-reply = クイック返信
mail-quick-reply-help = 宛先があらかじめ入力されたコンパクトな返信タブを表示します。既定ではオフです。通常の安全確認と送信オプションは引き続き適用されます。
mail-quick-reply-expand = 宛先と件名を編集
mail-attachment-policy-loading = 添付ファイルのポリシーを確認中…
mail-attachment-policy-unavailable = 添付ファイルのポリシーを確認できませんでした。このファイルを開く前にもう一度お試しください。
mail-attachment-policy-blocked = 管理者がこの添付ファイル、またはその元のダウンロードをブロックしています。
mail-attachment-policy-warning = 管理者はこの添付ファイルの取り扱いに注意するよう推奨しています。提供元を信頼できる場合のみ続行してください。
mail-attachment-policy-acknowledge = 理解しました。続行します
mail-attachment-policy-no-preview = 管理者により添付ファイルのプレビューは無効です。元のファイルのダウンロードは引き続き利用できる場合があります。
mail-time-sent = 送信
mail-time-received = 受信
mail-time-unknown = 未提供
mail-time-comparison = { $zone } の時刻
mail-time-privacy = 比較するタイムゾーンはあなたが選択したものです。メッセージのタイムスタンプから誰かの所在地が特定されることはありません。
mail-time-event-comparison = イベント時刻の比較
mail-time-ambiguous = この現地時刻は、時計が切り替わる際に2回現れます。可能性のある2つの時点を両方表示しています。保存する前にイベントの意図したオフセットを確認してください。
mail-bulk-add-label = ラベルを追加
mail-select-messages = メッセージを選択
mail-bulk-remove-label = ラベルを削除
mail-mark-spam = 迷惑メールとしてマーク
mail-bulk-forward = 選択したメッセージを添付ファイルとして転送
mail-bulk-download = 選択したメッセージを mbox でダウンロード
mail-bulk-forward-subject = 転送メッセージ（{ $count } 件）
mail-bulk-transfer-failed = 選択したメッセージを準備できませんでした。合計 32 MiB 以下で最大 100 件までのメッセージを選択し、アカウントのエクスポート権限を確認してください。
mail-toast-labels-added = { $count } 件のメッセージにラベル { $label } を追加しました
mail-toast-labels-removed = { $count } 件のメッセージからラベル { $label } を削除しました
mail-action-help-reply = 新しい作成タブで送信者に返信します。
mail-action-help-reply-all = 新しい作成タブで送信者とその他の宛先に返信します。
mail-action-help-forward = このメッセージを確認し、選択した宛先に転送します。
mail-action-help-pin = このメッセージを固定して簡単にアクセスできるようにするか、固定を解除します。
mail-action-help-read = このメッセージの既読状態を変更します。変更は元に戻せます。
mail-action-help-flag = フォローアップ用のフラグを設定または解除します。
mail-action-help-archive = このメッセージを設定済みのアーカイブフォルダーに移動します。
mail-action-help-delete = このメッセージをゴミ箱フォルダーに移動します。移動は元に戻せます。
mail-action-help-spam = このメッセージを迷惑メールとしてマークし、設定済みの迷惑メールフォルダーに移動します。
mail-action-help-export = エクスポートページを開き、選択した形式でこのメッセージをダウンロードします。
mail-action-help-sweep = 適用する前に、この送信者の他のメッセージに対する操作を確認します。
mail-empty-trash = ゴミ箱を空にする
mail-trash-help = 現在ゴミ箱にあるメッセージを確認し、完全に削除します。この操作は元に戻せません。
mail-trash-preparing = ゴミ箱内のメッセージのスナップショットを準備中…
mail-trash-prepare-failed = ゴミ箱のプレビューを準備できませんでした。接続を確認してもう一度開いてください。メッセージは削除されていません。
mail-trash-confirm = ゴミ箱から { $count } 件のメッセージを完全に削除しますか？この操作は元に戻せません。このプレビュー以降に届いたメッセージは対象外で、ゴミ箱から移動されたメッセージはスキップされます。
mail-trash-limit = この操作の対象は最大 { $count } 件です。ゴミ箱にはまだメッセージが残っています。削除するには、後で別の操作を行ってください。
mail-trash-permanent = 完全に削除
mail-trash-stop = 現在のリクエストの後で停止
mail-trash-progress = { $deleted } 件を削除しました。ゴミ箱に存在しない { $skipped } 件をスキップしました。このプレビューには { $remaining } 件が残っています。
mail-trash-stopped = 削除を停止しました。削除確認済み：{ $deleted }、スキップ：{ $skipped }、残り：{ $remaining }、結果不明：{ $uncertain }。別の操作を開始する前にゴミ箱を確認してください。
mail-links-title = メッセージのリンク
mail-links-review = リンクを確認（{ $count }）
mail-links-help = メッセージのリンクは、ここで宛先を確認した後でのみ開きます。宛先は自動的には取得されません。
mail-links-inert = メッセージ上部の「リンクを確認」を使って、この宛先を確認して開いてください。
mail-links-limit = サポートされている重複のない宛先は最初の 100 件のみ表示されます。それ以外のリンクはメッセージ内で無効のままです。
mail-links-destination = 開く宛先
mail-links-external = この宛先はメッセージの送信者によるものです。開くと Mailwoman を離れます。続行する前にアドレスを確認してください。
mail-links-open = 新しいタブで開く
mail-links-compose = 新しいメッセージを確認
mail-links-warning-http = このアドレスは暗号化されていない HTTP を使用しています。
mail-links-warning-address = この宛先はドメイン名ではなく IP アドレスを使用しています。
mail-links-warning-international = このドメインには国際化された名前が含まれています。ASCII での綴りを慎重に確認してください。
mail-links-warning-mismatch = 表示されている Web アドレスと実際の宛先のホストが異なります。
mail-links-warning-direction = リンクのテキストに、アドレスを偽装する可能性のある方向変更文字が含まれています。
mail-outbox-uncertain = 配信状況が不明です
mail-outbox-uncertain-note = 送信は開始されましたが、配信を確認できませんでした。このメッセージは自動的には再送信されません。再度送信する前に、送信済みを確認するか受信者に確認してください。
mail-conversation-title = スレッドを表示
mail-conversation-help = このアカウントで利用可能なスレッドのメッセージをフォルダー横断で閲覧します。メッセージを開いても、選択中のフォルダーは変わりません。
mail-conversation-count = スレッドのメッセージ { $total } 件中 { $loaded } 件を表示しています。
mail-conversation-other-folder = 別のフォルダー
mail-conversation-loading = スレッドを読み込み中…
mail-conversation-missing = { $count } 件のメッセージは利用できなくなりました。
mail-conversation-error = スレッドを読み込めませんでした。もう一度お試しください。
mail-conversation-more = さらにメッセージを読み込む
mail-conversation-limit = このビューは { $count } 件のメッセージに制限されています。
mail-compose-request-delivery = 配信通知を要求
mail-compose-request-delivery-hint = 受信側のメールサーバーに、配信、失敗、遅延の報告を依頼します。誰かがメッセージを読んだことを要求または証明するものではありません。返送される報告にはヘッダーが含まれ、本文全体は含まれません。
mail-compose-delivery-unavailable = このアカウントでは、配信通知は利用できないか、管理者により無効になっています。明示的に要求した通知が黙って削除されることはありません。
mail-compose-account-changed = この作成内容は別のサインインに属しています。正しいアカウントでサインインし、下書きを開き直してください。
mail-context-open-message = メッセージを開く
mail-context-open-folder = フォルダーを開く
mail-context-download = メッセージをダウンロード
mail-context-refresh = フォルダーを更新
mail-context-folder-settings = フォルダー設定
mail-context-failed = 操作を完了できませんでした。もう一度お試しください。
mail-sharing-unavailable = フォルダーの共有は利用できないか、管理する権限がなくなりました。
mail-trash-cancelling = 現在のリクエストの後で停止します。削除確認済み：{ $deleted }、スキップ：{ $skipped }、残り：{ $remaining }。すでに受け付けられた削除は元に戻せません。
mail-trash-already-running = ゴミ箱の削除はすでに実行中です。進行状況の通知から、以降のバッチを停止できます。
mail-context-search-sender = この送信者を検索
mail-context-search-subject = 同じ件名を検索
mail-drag-too-many = 一度にドラッグできるのは最大 { $count } 件です。移動する前に、選択するメッセージを減らしてください。
mail-subtab-reorder-help = ドラッグしてタブを並べ替えるか、アクティブなタブで Ctrl+Shift+左矢印または右矢印を押します。
mail-compose-ribbon = エディターのリボン
mail-compose-ribbon-format = テキストの書式
mail-compose-ribbon-insert = 挿入
mail-compose-font = フォント
mail-compose-font-size = フォントサイズ
mail-compose-font-inherit = 既定
mail-compose-undo = 元に戻す
mail-compose-redo = やり直す
mail-compose-undo-tip = 元に戻す（{ $shortcut }）
mail-compose-redo-tip = やり直す（{ $shortcut }）
mail-workspace-limit = 別のタブを開く前に、タブを1つ閉じてください。保持できるタブは最大 32 個です。
mail-compose-private-storage-unavailable = このセッションではローカルの下書き復元を利用できません。メッセージを送信またはコピーするまで、このエディターを開いたままにしてください。
mail-offline-storage-unavailable = このセッションでストレージの権限が確認されるまで、オフラインでの変更は利用できません。再接続して、もう一度サインインしてください。
mail-draft-status-idle = アカウントにはまだ保存されていません
mail-draft-status-pending = 未保存の変更があります
mail-draft-status-saving = 下書きに保存中…
mail-draft-status-saved = アカウントの下書きに保存しました
mail-draft-status-error = 下書きは保存されませんでした。ローカルの復元用コピーは引き続き利用できます。
mail-draft-status-uncertain = 保存の確認が取れていません。別のコピーを作成する前に下書きを確認してください。自動保存は一時停止しています。
mail-draft-status-conflict = この下書きは別のデバイスで変更されました。続行する前に、アカウント上のコピーを開き直してください。
mail-draft-status-unsupported = このアカウントは現在、下書きフォルダーへの保存を確認できません。
mail-draft-protected = 暗号化または復号された引用内容は平文では保存されません。以前のアカウントの下書きは、明示的に破棄するまで残ります。
mail-draft-save-now = 下書きに保存
mail-draft-save-failed = この下書きをアカウントに保存できませんでした。閉じる前に作成画面を確認してください。
mail-draft-uncertain = 下書きは保存された可能性があります。自動再試行は一時停止しています。下書きフォルダーを確認してください。
mail-draft-conflict = 別のデバイスがこの下書きを変更しました。あなたの編集内容はそのコピーを上書きしていません。
mail-draft-cleanup-pending = 新しい下書きは保存されましたが、以前のコピーを削除できませんでした。もう一度保存する前に下書きを確認してください。
mail-draft-account-list = アカウントの下書き
mail-draft-loading = アカウントの下書きを読み込み中…
mail-draft-close-recovery = 閉じてローカルの復元のみ保持
mail-draft-discard-account = 破棄
mail-draft-discard-pending = 下書きの削除を確認できませんでした。アカウント上の記録は残っています。下書きフォルダーを確認してから、もう一度「破棄」をお試しください。
mail-draft-limit = このアカウントにはすでに 100 件の下書きが保存されています。新しい下書きを保存する前に、不要なアカウントの下書きを破棄してください。
mail-draft-quota-help = アカウントの下書きは最大 100 件まで保持されます。破棄すると、確認済みのサーバー上のコピーを削除してから枠が解放されます。
mail-draft-discard-confirm = 下書きの削除を確認
mail-draft-discard-confirm-detail = アカウントの下書きから「{ $subject }」を完全に削除しますか？代わりに作成画面を閉じると、下書きは保持されます。
mail-draft-discard-title = この下書きを破棄しますか？
mail-draft-keep-editing = 編集を続ける
mail-draft-discard-action = 破棄
mail-draft-close-title = 未保存の変更を破棄しますか？
mail-draft-close-protected = このメッセージには暗号化または復号された内容が含まれているため、下書きとして保存されません。閉じるとこれらの変更は破棄されます。以前のアカウントの下書きがある場合は、下書きに残ります。
mail-draft-close-unstored = このセッションでは下書きを保存できないため、このメッセージを閉じると破棄されます。
mail-workspace-view-mode = ビューの配置
mail-workspace-tabs-help = アクティブなビューを1つ表示し、他のタブは開いたままにします。
mail-workspace-side-by-side-help = 選択した2つのビューを左右に並べて表示します。タブをどちらかのビューにドラッグすると置き換えられます。
mail-workspace-stack-help = 選択した2つのビューを上下に並べて表示します。タブをどちらかのビューにドラッグすると置き換えられます。
mail-workspace-masonry-side-by-side = マソンリー（左右）
mail-workspace-masonry-side-by-side-help = 開いているすべてのビューを、ずらした列に並べて表示します。小さい画面ではアクティブなタブを表示します。
mail-workspace-masonry-stack = マソンリー（上下）
mail-workspace-masonry-stack-help = 開いているすべてのビューを、ずらした行に並べて表示します。小さい画面ではアクティブなタブを表示します。
mail-workspace-view-first = 1つ目のビュー
mail-workspace-view-second = 2つ目のビュー
mail-workspace-view-empty = 開いているタブをここにドラッグして、このビューを選択します。
mail-popup-blocked = このタブを切り離すには、Mailwoman のポップアップウィンドウを許可してください。タブはここに残ります。
mail-popup-unavailable = 切り離したウィンドウを読み込めませんでした。タブと未保存の変更はメインウィンドウに残っています。アプリを更新してからもう一度お試しください。
mail-popup-reattach = メインウィンドウに戻す
mail-popup-action-reattached = この操作を完了するため、メインウィンドウに戻りました。
mail-popup-detached = このタブは別のウィンドウで開いています。そのウィンドウを閉じると、ここに戻ります。
mail-compose-from-custom = 別の承認済みアドレス…
mail-compose-from-alias = このメッセージの差出人アドレス
mail-compose-from-alias-help = 送信を許可されているエイリアスを使用します。この下書きにのみ適用され、送信前に承認が確認されます。
mail-compose-from-refused = このアドレスは、このアカウントからの送信が承認されていません。
mail-compose-from-unavailable = この差出人アドレスを確認できませんでした。メッセージは送信されていません。
mail-compose-from-invalid = 差出人に有効なメールアドレスを1つ入力してください。
mail-compose-from-toggle = このメッセージの差出人の表示/非表示
mail-compose-ribbon-security = セキュリティ
mail-compose-ribbon-delivery = 配信
mail-compose-ribbon-recipients = 宛先
mail-compose-ribbon-message-type = メッセージの種類
mail-compose-recall-help = 取り消し待ちまたは送信予約中のメッセージをキャンセルします。すでに配信されたメッセージは取り消せません。
mail-compose-recall-loading = キューにあるメッセージを確認中…
mail-compose-recall-empty = キャンセルできるキュー内のメッセージはありません。

# -- Machine translation (2026-11): added to match en ---------------------
mail-compose-recall-failed = キューの状態を確認できませんでした。このパネルを閉じて開き直して更新してから、別の操作を行ってください。
mail-compose-importance = 重要度
mail-compose-importance-high = 高
mail-compose-importance-normal = 標準
mail-compose-importance-low = 低
mail-insert-account-title = アカウントの項目を挿入
mail-insert-account-help = このアカウントのカレンダーのエクスポート、連絡先の名刺、または元のメッセージを添付します。
mail-insert-account-close = アカウント項目の選択を閉じる
mail-insert-account-kind = 項目の種類
mail-insert-account-event = カレンダーのイベント (.ics)
mail-insert-account-calendar = カレンダー (.ics)
mail-insert-account-contact = 連絡先の名刺 (.vcf)
mail-insert-account-message = 元のメッセージ (.eml)
mail-insert-account-search = 表示中の項目を絞り込む
mail-insert-account-selection = 項目を選択
mail-insert-account-scope = 現在のアカウントの項目のみが表示されます。元のメッセージにはサーバーのエクスポートポリシーが適用されます。
mail-insert-account-limited = 最大 100 件の項目を表示しています。絞り込みはこの一覧のみが対象です。
mail-insert-account-unavailable = 利用できるアカウントのエクスポートがありません。
mail-insert-account-empty = 一致する項目がありません。
mail-insert-account-attach = 選択した項目を添付
mail-insert-account-loading = アカウントの項目を読み込み中…
mail-insert-account-error = 項目を添付できませんでした。アクセス権とエクスポートポリシーを確認してからもう一度お試しください。大きなカレンダーは、イベントを個別にエクスポートする必要がある場合があります。
mail-compose-html-source = HTML ソース
mail-editor-clear-format = 書式をクリア
mail-compose-format-html = HTML
mail-compose-html-visual = ビジュアルエディター
mail-compose-html-help = HTML を直接編集します。サポートされているメールの書式のみが保存・送信され、スクリプト、外部画像、安全でないマークアップは削除されます。
mail-compose-html-processing = 安全な HTML を準備中…
mail-compose-html-failed = HTML を準備できませんでした。保存または送信する前に、編集するかもう一度お試しください。
mail-compose-check-names = 名前を確認
mail-compose-check-names-help = 読み込み済みの連絡先から完全一致する名前を解決し、アドレスの形式を確認します。メールボックスへの配信は検証されません。
mail-compose-names-unresolved = 解決できない、または曖昧な次の名前について、完全なメールアドレスを選択してください：{ $names }
mail-compose-names-checked = 宛先の形式を確認しました。{ $count } 件の連絡先名を解決しました。メールボックスへの配信は検証されていません。
mail-compose-ribbon-table = 表
mail-compose-group-clipboard = クリップボード
mail-compose-group-paragraph = 段落
mail-compose-group-editing = 編集
mail-compose-group-view = 表示
mail-compose-dark-background = ダーク背景
mail-compose-dark-background-help = 作成中、メッセージをダークな背景で表示します。変わるのは画面表示だけで、メッセージは背景色なしで送信されます。
mail-compose-group-assist = 執筆アシスタント
mail-editor-counts = エディターの統計
mail-editor-words = { $count ->
   *[other] { $count } 語
}
mail-editor-paragraphs = { $count ->
   *[other] { $count } 段落
}
mail-editor-characters = { $count ->
   *[other] { $count } 文字
}
mail-editor-selection-required = この操作を使う前に、適切なテキストまたは表のセルを選択してください。
mail-editor-clipboard-unavailable = クリップボードにアクセスできませんでした。ブラウザーのコピー、切り取り、貼り付けのショートカットを使用してください。
mail-editor-selection-changed = 選択範囲が変更されました。対象のテキストを選択してもう一度お試しください。
mail-editor-clipboard = クリップボードと書式
mail-editor-copy = コピー
mail-editor-cut = 切り取り
mail-editor-paste = 貼り付け
mail-editor-copy-format = 書式をコピー
mail-editor-apply-format = 書式を適用
mail-editor-style = 段落スタイル
mail-editor-text-color = フォントの色
mail-editor-clear-color = 既定の色
mail-editor-highlight = 蛍光ペン
mail-editor-clear-highlight = 蛍光ペンをクリア
mail-editor-paragraph = 段落のオプション
mail-editor-alignment = 配置
mail-editor-line-height = 行の高さ
mail-editor-space-before = 段落前の間隔
mail-editor-space-after = 段落後の間隔
mail-editor-indent = インデント
mail-editor-shading = 段落の網かけ
mail-editor-clear-shading = 網かけをクリア
mail-editor-case = 大文字と小文字の変更
mail-editor-sort = 選択した段落を並べ替え
mail-editor-sort-alpha = 五十音・アルファベット順
mail-editor-sort-numeric = 数値順
mail-editor-table-rows = 表の行数
mail-editor-table-columns = 表の列数
mail-editor-table-insert = 表を挿入
mail-editor-table-limits = 1 つの表につき最大 20 行、10 列、200 セルまで、表は最大 10 個までです。入れ子の表はサポートされていません。
mail-editor-table-row-before = 上に行を挿入
mail-editor-table-row-after = 下に行を挿入
mail-editor-table-row-delete = 行を削除
mail-editor-table-column-before = 左に列を挿入
mail-editor-table-column-after = 右に列を挿入
mail-editor-table-column-delete = 列を削除
mail-editor-table-merge = セルを結合
mail-editor-table-split = セルを分割
mail-editor-table-header = 見出し行を切り替え
mail-editor-table-delete = 表を削除
mail-editor-table-shading = セルの網かけ
mail-editor-find-replace = 検索と置換
mail-editor-find = 検索するテキスト
mail-editor-replacement = 置換後のテキスト
mail-editor-match-case = 大文字と小文字を区別
mail-editor-previous = 前の一致
mail-editor-next = 次の一致
mail-editor-replace = 選択した一致を置換
mail-editor-replace-all = すべての一致を置換
mail-editor-matches = { $count } 件が一致（最大 1,000 件）
mail-editor-select-all = すべて選択
mail-editor-clear-selection = 選択を解除
mail-editor-history = 元に戻す履歴
mail-editor-undo-groups = 直近 { $count } 件の変更グループを元に戻す
mail-editor-zoom = エディターのズーム
mail-editor-markers = 段落記号
mail-editor-style-normal = 標準
mail-editor-style-no-spacing = 行間詰め
mail-editor-style-title = タイトル
mail-editor-style-subtitle = サブタイトル
mail-editor-style-heading1 = 見出し 1
mail-editor-style-heading2 = 見出し 2
mail-editor-style-heading3 = 見出し 3
mail-editor-style-heading4 = 見出し 4
mail-editor-style-heading5 = 見出し 5
mail-editor-style-heading6 = 見出し 6
mail-editor-align-left = 左揃え
mail-editor-align-center = 中央揃え
mail-editor-align-right = 右揃え
mail-editor-align-justify = 両端揃え
mail-editor-case-lower = 小文字
mail-editor-case-upper = 大文字
mail-editor-case-title = 各単語の先頭を大文字
mail-editor-case-sentence = 文頭のみ大文字
mail-editor-action-history = { $action }の履歴
mail-editor-history-empty = 利用できる操作がありません
mail-editor-redo-groups = { $count } 件の変更をやり直す
mail-compose-identities-failed = 送信元 ID を読み込めませんでした。この下書きを閉じて開き直し、もう一度お試しください。
mail-folders-loading = フォルダーを読み込み中…
mail-initial-sync-error = メッセージをまだ同期できませんでした。
mail-initial-sync-error-help = メールサーバーとの接続が中断されました。もう一度お試しください。問題が続く場合は、管理者に接続ログの確認を依頼してください。
mail-links-trusted = 信頼済みドメイン：確認なしで開きます
mail-links-trust-domain = { $domain } を信頼する
mail-links-trust-failed = 信頼するドメインを保存できませんでした。もう一度お試しください。
mail-focus-always-focused = 常に「重要」に移動
mail-focus-always-other = 常に「その他」に移動
mail-focus-moved-focused = { $sender } からのメッセージは今後「重要」に入ります。
mail-focus-moved-other = { $sender } からのメッセージは今後「その他」に入ります。
mail-focus-save-failed = この送信者の「重要」受信トレイの設定を保存できませんでした。もう一度お試しください。
mail-context-search-domain = このドメインからのメッセージを検索
mail-loading-mailbox = { $name } を読み込み中…
mail-user-folders = ユーザーフォルダー
mail-context-hide-folder = サイドバーから非表示
mail-folder-fallback = フォルダー
mail-toast-moving = { $count ->
   *[other] { $count } 件のメッセージを { $folder } に移動中…
    }
mail-toast-move-queueing = { $count ->
   *[other] { $count } 件のメッセージを { $folder } への移動キューに追加中…
    }
mail-toast-move-queued = { $count ->
   *[other] { $count } 件のメッセージを { $folder } への移動キューに追加しました
    }
mail-toast-move-partial = { $total } 件中 { $count } 件のメッセージを { $folder } に移動しました。{ $failed } 件は移動できませんでした
mail-send-copy-folder-unavailable = 選択した送信済みコピーのフォルダーを利用できません。送信する前に別のフォルダーを選択してください。
mail-send-copy-failed = メッセージは受け付けられましたが、送信済みコピーを保存できませんでした。再送信しないでください。
mail-compose-attach-public-key = 自分の公開鍵または証明書を添付
mail-compose-include-hashes = 添付ファイルの SHA-256 ハッシュを含める
mail-compose-hashes-pending = 添付ファイルのハッシュを計算中です。準備ができたら送信してください。
mail-compose-hashes-failed = 添付ファイルのバイト列を検証できませんでした。ハッシュをオフにするか、再度オンにしてやり直してください。
mail-compose-crypto-pending = 暗号化を更新中です。送信する前にお待ちください。
mail-compose-public-key-unavailable = 送信元アドレスに一致する現在の公開鍵または証明書がありません。
mail-compose-sent-copy = 送信済みメッセージの保存先
mail-compose-sent-copy-default = アカウントの既定
folder-create = フォルダーを作成
folder-name = フォルダー名
folder-parent = 親フォルダー
folder-top-level = 最上位
folder-created = フォルダーを作成しました。
folder-create-unavailable = このアカウントまたは親フォルダーではフォルダーを作成できません。
folder-create-invalid = 階層の区切り文字や制御文字を含まない、有効で一意なフォルダー名を指定してください。
folder-create-failed = フォルダーを作成できませんでした。更新してもう一度お試しください。
folder-cleanup = フォルダーを整理…
folder-cleanup-title = { $name } を整理
folder-cleanup-help = 指定した期間より古いメッセージを探します。削除する前に件数を確認してください。サブフォルダーは対象外です。
folder-cleanup-age = 次より古い
folder-age-days = 日
folder-age-weeks = 週間
folder-age-months = か月
folder-age-years = 年
folder-cleanup-preview = 整理をプレビュー
folder-cleanup-confirm = 一致するメッセージを削除
folder-cleanup-permanent = 一致する { $count } 件のメッセージを完全に削除します。この操作は元に戻せません。
folder-cleanup-trash = 一致する { $count } 件のメッセージをゴミ箱に移動します。
folder-cleanup-before = { $date } より前に受信
folder-cleanup-done = { $count } 件のメッセージを整理しました。
mail-from = 差出人
mail-subject = 件名
mail-date = 日付
mail-cancel = キャンセル
mail-bulk-selected = { $count } 件のスレッドを選択中
mail-bulk-select-folder = { $folder } 内をすべて選択
mail-list-order = メッセージの並び順

# -- Machine translation (2026-11): added to match en ---------------------
mail-list-order-newest = 新しい順
mail-list-order-oldest = 古い順
mail-list-order-sender = 差出人
mail-list-order-subject = 件名
mail-list-order-error = メッセージの並び順を保存できませんでした。
mail-bulk-selection-load-failed = フォルダー全体を選択できませんでした。接続を確認してもう一度お試しください。
mail-privacy-bar = メッセージのプライバシー
mail-privacy-trackers-allow = このメッセージのトラッカーを許可
mail-privacy-trackers-block = トラッカーを再びブロック
mail-privacy-images-loading = { $total } 件中 { $loaded } 件の画像を表示 · { $pending } 件を読み込み中
mail-privacy-images-failed = { $total } 件中 { $loaded } 件の画像を表示 · { $failed } 件が失敗
mail-privacy-images-loaded = { $total } 件中 { $loaded } 件の画像を表示
mail-privacy-images-loading-failed = { $total } 件中 { $loaded } 件の画像を表示 · { $pending } 件を読み込み中 · { $failed } 件が失敗
mail-reader-folder-title = フォルダー：{ $folder }
mail-reader-move-to = 移動先…
mail-action-help-move = このメッセージを別のフォルダーに移動します。
folder-ops-new-subfolder = 新しいサブフォルダー
folder-ops-mark-read = すべて既読にする
folder-ops-this-folder = このフォルダー
folder-ops-with-subfolders = サブフォルダーを含める
folder-ops-no-unread = 未読のメッセージはありません。
folder-ops-read-unavailable = このフォルダー内のメッセージは変更できません。
folder-ops-more = その他
folder-ops-system-folder = システムフォルダーはこの方法では変更できません。
folder-ops-sort-subfolders = サブフォルダーの並べ替え基準
folder-ops-sort-folders = フォルダーの並べ替え基準
folder-ops-sort-name = 名前
folder-ops-sort-unread = 未読数
folder-ops-sort-total = 合計数
folder-ops-sort-date = 最終更新
folder-ops-sort-custom = カスタム順
folder-ops-sort-asc = 昇順
folder-ops-sort-desc = 降順
folder-ops-delete-with-subfolders = フォルダーとサブフォルダーを削除
folder-ops-delete-unavailable = このフォルダーは削除できません。
folder-ops-delete-tree-body = { $folders } 個のフォルダーと、その中の { $count } 件のメッセージを完全に削除します。この操作は元に戻せません。
folder-ops-copy-all-menu = すべての項目のコピー先…
folder-ops-move-all-menu = すべての項目の移動先…
folder-ops-move-folder-menu = フォルダーの移動先…
folder-ops-move-up = 上へ移動
folder-ops-move-down = 下へ移動
folder-ops-move-unavailable = このフォルダーまたはその項目は移動できません。
folder-ops-move-duplicate = 移動先に同じ名前のフォルダーがすでにあります。
folder-ops-move-folder-title = { $name } を移動
folder-ops-move-folder-body = フォルダーはサブフォルダーとメッセージごと移動します。入れるフォルダーを選ぶか、最上位を選択してください。
folder-ops-move-here = ここに移動
folder-ops-moved-folder = { $name } を移動しました。
folder-ops-choose-folder = 移動先のフォルダーを選択してください。
folder-ops-destination = 移動先フォルダー
folder-ops-include-subfolders = { $count } 個のサブフォルダーのメッセージも含める
folder-ops-copy-all-title = { $name } のすべての項目をコピー
folder-ops-move-all-title = { $name } のすべての項目を移動
folder-ops-copy-all-body = { $count } 件のメッセージを移動先のフォルダーに追加します。元のメッセージはそのまま残ります。
folder-ops-move-all-body = { $count } 件のメッセージを移動先のフォルダーに移動します。あとで元に戻せます。
folder-ops-copy-all = 項目をコピー
folder-ops-move-all = 項目を移動
folder-ops-undo = 元に戻す
folder-ops-stop = 停止
folder-ops-marking = { $name } を既読にしています…
folder-ops-marking-progress = { $name } を既読にしています：{ $done } / { $total }
folder-ops-marked = { $name } の { $count } 件のメッセージを既読にしました。
folder-ops-copying = { $name } をコピーしています…
folder-ops-copying-progress = { $name } をコピーしています：{ $done } / { $total }
folder-ops-copied = { $name } から { $count } 件のメッセージをコピーしました。
folder-ops-moving = { $name } を移動しています…
folder-ops-moving-progress = { $name } を移動しています：{ $done } / { $total }
folder-ops-moved = { $name } から { $count } 件のメッセージを移動しました。
folder-ops-stopped = { $total } 件中 { $count } 件のメッセージを処理した時点で { $name } を停止しました。
folder-ops-partial = { $name } の { $total } 件中 { $count } 件のメッセージを処理しました。{ $refused } 件は拒否されました。
folder-ops-failed = { $name } を完了できませんでした。更新してもう一度お試しください。
folder-ops-error-forbidden = この変更を行う権限がありません。
folder-ops-error-exists = 同じ名前のフォルダーがすでに存在します。
folder-ops-error-mailboxHasEmail = フォルダーにまだメッセージがあります。
folder-ops-error-notFound = フォルダーは存在しなくなりました。
folder-ops-error-failed = 変更を行えませんでした。更新してもう一度お試しください。
folder-ops-expand = { $name } のサブフォルダーを表示
folder-ops-collapse = { $name } のサブフォルダーを非表示
mail-context-reply = 返信
mail-context-reply-all = 全員に返信
mail-context-forward = 転送
mail-context-quick-print = クイック印刷
mail-context-copy = コピー
mail-context-copy-to-folder = メッセージをフォルダーにコピー…
mail-context-copy-sender = 差出人のアドレスをコピー
mail-context-copy-subject = 件名をコピー
mail-context-copy-link = メッセージのリンクをコピー
mail-context-copied = コピーしました。
mail-context-copy-failed = クリップボードにコピーできませんでした。
mail-context-print-failed = メッセージを印刷できませんでした。
mail-reply-images-title = 元のメッセージの画像
mail-reply-images-ask = 元のメッセージの画像をこの返信に含めますか？
mail-reply-images-include = 画像を含める
mail-reply-images-not-now = 今はしない
mail-reply-images-trust = { $domain } を常に信頼する
mail-reply-images-loading = 画像を追加中…
mail-reply-images-cancelled = 読み込みをキャンセルしました。画像は追加されていません。
mail-reply-images-failed = 画像を追加できませんでした。
mail-reply-images-retry = もう一度読み込む
mail-image-cancel = キャンセル
mail-compose-from-why-match = メッセージが { $address } 宛てに送信されたため、{ $identity } から送信します。
mail-compose-from-why-match-plus = メッセージが { $address } のプラスアドレス宛てに送信されたため、{ $identity } から送信します。
mail-compose-from-why-ambiguous = { $address } に一致する ID が複数あります。{ $identity } から送信します。別の ID を選択することもできます：
mail-compose-from-why-default = 元の宛先に一致する ID がないため、既定の ID である { $identity } から送信します。
mail-compose-from-why-forced-default = 設定に従い、既定の ID である { $identity } から送信します。
mail-compose-from-why-keep = { $identity } から送信します。
mail-compose-from-change = 変更
reader-attachments-reason-encrypted = このメッセージは暗号化されています。添付ファイルはロック解除後に表示されます。
reader-attachments-reason-list-missing = このメッセージの添付ファイル一覧を読み込めなかったため、添付ファイルバーは非表示です。
reader-attachments-reason-inline-only = このメッセージの添付ファイル（{ $count } 件）は、メッセージ内に表示されるインライン画像です。
reader-attachments-reason-unavailable-parts = { $count ->
   *[other] このメッセージの添付ファイル { $count } 件にはダウンロード可能な内容がないため、添付ファイルバーは非表示です。
  }
reader-attachments-reason-policy-blocked = 添付ファイルのポリシーにより、{ $count ->
   *[other] これらの添付ファイルはダウンロードできません。
  }
reader-attachments-reason-policy-unavailable = 添付ファイルのポリシーを確認できなかったため、添付ファイルはまだ開けません。
mail-drafts-popup-count = { $count ->
   *[other] 下書き { $count } 件
  }
mail-drafts-popup-search = 件名または宛先で検索
mail-drafts-popup-tab-local = ローカルの下書き
mail-drafts-popup-local-hint = ローカルの下書きはこのブラウザーにのみ保存されます。
mail-drafts-popup-badge-account = アカウント
mail-drafts-popup-badge-local = ローカル
mail-drafts-popup-current = 現在開いています
mail-drafts-popup-select = 選択
mail-drafts-popup-select-all = すべて選択
mail-drafts-popup-clear = クリア
mail-drafts-popup-done = 完了
mail-drafts-popup-selected = { $count } 件選択中
mail-drafts-popup-select-row = { $subject } を選択
mail-drafts-popup-open = 開く
mail-drafts-popup-discard = 破棄
mail-drafts-popup-discard-selected = 選択項目を破棄
mail-drafts-popup-more = { $subject } のその他の操作
mail-drafts-popup-retry = 再試行
mail-drafts-popup-load-failed = アカウントの下書きを読み込めませんでした。
mail-drafts-popup-empty-account = アカウントの下書きはありません
mail-drafts-popup-no-match = 「{ $query }」に一致する下書きはありません
mail-drafts-popup-discarding = 破棄しています…
mail-drafts-popup-row-failed = 破棄できませんでした。アカウント上のコピーは残っています。
mail-drafts-popup-confirm-title = { $count ->
   *[other] { $count } 件の下書きを破棄しますか？
  }
mail-drafts-popup-confirm-account = アカウントの下書きは、まずアカウントの下書きフォルダーから削除され、その削除が確認された後で初めて枠が解放されます。
mail-drafts-popup-confirm-local = ローカルの下書きはこのブラウザーから削除されます。
mail-drafts-popup-announce-done = { $count ->
   *[other] { $count } 件の下書きを破棄しました
  }
mail-drafts-popup-announce-failed = { $done } 件を破棄しました。{ $failed } 件は破棄できませんでした
mail-check-title = 送信前の確認
mail-check-intro = 次の点をもう一度確認してください：
mail-check-invalid = 無効と思われる宛先アドレスがあります：{ $list }
mail-check-subject = 件名が空です。
mail-check-attachment = メッセージで添付ファイルに触れていますが、何も添付されていません。
mail-check-external = 信頼済みドメイン外の宛先：{ $list }
mail-check-send-anyway = このまま送信
mail-check-review = 確認する
mail-check-remember = これらについては今後確認しない
mail-check-denied-invalid = 設定により送信できません。無効な宛先アドレスを修正してください：{ $list }
mail-check-denied-subject = 設定により送信できません。件名を追加してください。
mail-check-denied-attachment = 設定により送信できません。言及しているファイルを添付するか、文面を修正してください。
mail-check-denied-external = 設定により送信できません。信頼済みドメイン外：{ $list }
mail-alerts-label = メッセージの問題
mail-alert-invalid = 無効なアドレス：{ $address }
mail-alert-subject = 件名なし
mail-recipient-chip-remove = { $address } を削除
mail-recipient-chips = { $field } の宛先
mail-recipient-invalid-title = 有効なメールアドレスではありません
mail-recipient-chip-name = { $name }、{ $address }。Enter で連絡先の詳細、Backspace または F2 で編集、Delete で削除
mail-recipient-chip-name-invalid = { $address }、有効なアドレスではありません。Enter で編集、Delete で削除
mail-recipient-added = { $name } を追加しました
mail-recipient-removed = { $name } を削除しました
mail-recipient-removed-many = { $count ->
   *[other] { $count } 件の宛先を削除しました
}
mail-recipient-editing = { $name } を編集中
mail-recipient-all-selected = { $count ->
   *[other] { $count } 件の宛先を選択中
}
mail-external-line = 信頼済みドメイン外が { $count } 件：{ $domains }
mail-external-dismiss = 閉じる
mail-status-label = 作成画面の状態

# -- Machine translation (2026-11): added to match en ---------------------
mail-status-more = ほか { $count } 件のお知らせ
mail-status-more-label = その他のお知らせ
mail-status-no-recipients = 宛先なし
mail-status-no-attachment = 添付なし
mail-status-external = 外部：{ $count }
mail-status-external-reviewed = 外部の宛先を確認済み
mail-status-external-review = 確認済みにする
mail-status-from-match = 差出人：{ $identity }
mail-status-from-match-plus = 差出人：{ $identity }
mail-status-from-ambiguous = 差出人：{ $identity }（複数が一致）
mail-status-from-default = 差出人：既定の ID
mail-status-from-forced-default = 差出人：既定の ID
mail-status-from-keep = 差出人：{ $identity }
mail-status-held = { $count } 件がキュー内
mail-status-storage = ローカル復元なし
mail-status-protected = 平文では保存されません
mail-status-saving = 保存中…
mail-status-saved = { $time } に保存しました
mail-status-saved-full = アカウントの下書きに保存しました · { $time }
mail-status-not-saved = 下書きは未保存
mail-status-retry = 再試行
mail-status-conflict = 別のデバイスで変更されました
mail-status-uncertain = 保存が未確認
mail-status-close-recovery = 閉じて復元用を保持
mail-status-unsupported = 下書きの保存が未確認
mail-compose-header = メッセージヘッダー
mail-header-label = メッセージヘッダー
mail-header-collapse = メッセージヘッダーを折りたたむ
mail-header-expand = メッセージヘッダーを展開：{ $summary }
mail-header-from-chip = 差出人 { $address }。差出人を変更
mail-compose-group-timing = 送信タイミング
mail-compose-group-priority = 優先度
mail-compose-group-receipts = 開封確認とトラッキング
mail-compose-group-keys-files = 鍵とファイル
mail-compose-group-sent-copy = 送信済みコピー
mail-editor-style-quote = 引用
mail-editor-style-code = コード
mail-compose-ribbon-table-layout = 表のレイアウト
mail-compose-ribbon-picture = 画像
mail-editor-table-group-rows = 行
mail-editor-table-group-columns = 列
mail-editor-table-group-cells = セル
mail-editor-table-row-height = 行の高さ
mail-editor-table-column-width = 列の幅
mail-editor-table-size-auto = 自動
mail-editor-table-padding = セルの余白
mail-editor-table-padding-compact = 狭い
mail-editor-table-padding-normal = 標準
mail-editor-table-padding-roomy = 広い
mail-editor-table-align-left = セルを左揃え
mail-editor-table-align-center = セルを中央揃え
mail-editor-table-align-right = セルを右揃え
mail-picture-group-size = サイズ
mail-picture-group-arrange = 配置
mail-picture-group-text = テキスト
mail-picture-group-style = スタイル
mail-picture-group-edit = 編集
mail-picture-width = 幅
mail-picture-height = 高さ
mail-picture-lock = 縦横比を維持
mail-picture-reset-size = 元のサイズ
mail-picture-align-inline = 行内
mail-picture-align-left = 左揃え
mail-picture-align-center = 中央
mail-picture-align-right = 右揃え
mail-picture-alt = 代替テキスト
mail-picture-title = タイトル
mail-picture-border = 枠線
mail-picture-radius = 角
mail-picture-none = なし
mail-picture-round = 丸
mail-picture-rotate-ccw = 左に回転
mail-picture-rotate-cw = 右に回転
mail-picture-flip-h = 左右反転
mail-picture-flip-v = 上下反転
mail-picture-crop = トリミング
mail-picture-crop-left = 左（%）
mail-picture-crop-top = 上（%）
mail-picture-crop-right = 右（%）
mail-picture-crop-bottom = 下（%）
mail-picture-crop-apply = トリミング
mail-picture-replace = 置換
mail-picture-remove = 削除
mail-picture-failed-no-image = 先に画像を選択してください。
mail-picture-failed-no-source = この画像には編集できるローカルのコピーがありません。代わりに置換してください。
mail-picture-failed-failed = 画像を変更できませんでした。
mail-context-download-attachments = 添付ファイルをダウンロード
mail-context-download-named = { $name } をダウンロード
mail-context-download-all = すべて ZIP でダウンロード
mail-context-loading-attachments = 添付ファイルを読み込み中…
mail-context-attachments-failed = 添付ファイルを読み込めませんでした。
mail-context-searching = メッセージを探しています…
mail-context-go-reply = 返信へ移動
mail-context-go-forward = 転送へ移動
mail-context-reply-not-found = 返信が見つかりません
mail-context-forward-not-found = 転送が見つかりません
mail-no-recipients = （宛先なし）
mail-attachment-downloaded = { $name } をダウンロードしました
mail-attachment-download-failed = ダウンロードに失敗しました。
mail-attachments-zip-done = { $count } 件の添付ファイルを ZIP でダウンロードしました。
mail-attachments-zip-capped = ZIP には最初の { $count } 件の添付ファイルが含まれます。上限は { $max } ファイルまたは { $size } です。
mail-focus-not-found = { $folder } にメッセージが見つかりません
mail-focus-go-to = { $folder } へ移動
mail-nav-views-queues = ビューとキュー
mail-badge-unread = { $unread ->
   *[other] 未読メッセージ { $unread } 件
  }
mail-badge-total = { $total ->
   *[other] このフォルダーに { $total } 件のメッセージ
  }
mail-badge-both = { $unread ->
   *[other] 未読メッセージ { $unread } 件
  }（このフォルダー内の { $total ->
   *[other] { $total } 件中
  }）
mail-badge-drafts = { $total ->
   *[other] 下書き：{ $total } 件のメッセージ
  }
mail-badge-compact-hint = 受信トレイ以外のフォルダーには小さいバッジを使用します。「設定」の「フォルダーとサイドバー」で変更できます。
mail-badge-outbox = { $count ->
   *[other] 送信トレイ：送信待ちのメッセージ { $count } 件
  }
mail-color-menu = メッセージの色
mail-color-button = メッセージの色：{ $mode }
mail-color-use-default = 自分の既定を使う（{ $mode }）
mail-color-make-default = これを自分の既定にする
mail-color-locked = 管理者によりロックされています
mail-color-confirm-title = メッセージの既定の色を変更しますか？
mail-color-confirm-body = メッセージ単位で変更しない限り、すべてのメッセージで「{ $mode }」が使用されます。
mail-color-confirm = 既定にする
mail-color-default-failed = メッセージの既定の色を保存できませんでした。
