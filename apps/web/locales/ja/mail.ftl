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
