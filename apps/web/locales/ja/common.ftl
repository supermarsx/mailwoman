# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: common. Source: ../en/common.ftl

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = キャンセル
common-close = 閉じる
common-save = 保存
common-delete = 削除
common-remove = 削除
common-edit = 編集
common-add = 追加
common-back = 戻る
common-next = 次へ
common-done = 完了
common-apply = 適用
common-confirm = 確認
common-retry = 再試行
common-contact-administrator = この問題が続く場合は、管理者に問い合わせてください。
common-shell-not-configured = このアプリはサーバー用に設定されていません。管理者に問い合わせてください。
common-copy = コピー
common-search = 検索
common-more = その他
common-yes = はい
common-no = いいえ

# -- Generic states ----------------------------------------------------------
common-loading = 読み込んでいます…
common-saving = 保存しています…
common-empty = まだ何もありません
common-error = 問題が発生しました
common-error-network = サーバーに接続できません。接続を確認して、もう一度お試しください。
common-offline = オフラインです

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
common-conn-connecting = 接続しています…
common-conn-degraded = リアルタイム更新を一時停止中 — 再接続しています
common-conn-offline = オフラインです — 再接続すると変更が同期されます
common-conn-auth-expired = セッションの有効期限が切れました — もう一度サインインしてください
common-conn-reconnect = 再接続
common-conn-sign-in-again = もう一度サインイン
common-conn-reconnected = 再接続しました

# -- New-mail notification (App.tsx). The native desktop or mobile notification
common-notify-new-message = 新しいメッセージ
common-notify-no-subject = (件名なし)

# -- Generic labels ----------------------------------------------------------
common-required = 必須
common-optional = 任意

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
common-nav-open = フォルダーとアプリを開く
common-nav-close = フォルダーとアプリを閉じる

# -- Attachments (global, cross-account view; rides the entry bundle so the
common-attach-title = 添付ファイル
common-attach-search = 添付ファイルを検索
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = 種類で絞り込み
common-attach-loading = 添付ファイルを読み込んでいます…
common-attach-empty = 一致する添付ファイルはありません。
common-attach-cat-all = すべて
common-attach-cat-image = 画像
common-attach-cat-pdf = PDF
common-attach-cat-text = テキスト
common-attach-cat-audio = 音声
common-attach-cat-video = 動画
common-attach-cat-other = その他

# -- Password fields (components/PasswordInput.tsx): the button at the end of
common-password-show = パスワードを表示
common-password-hide = パスワードを隠す

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
common-module-mail = メール
common-module-calendar = カレンダー
common-module-tasks = タスク
common-module-notes = メモ
common-module-contacts = 連絡先
common-module-keys = 鍵

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
common-viewer-loading = ビューアーを読み込んでいます…
common-viewer-loading-audio = 音声を読み込んでいます…
common-viewer-loading-image = 画像を読み込んでいます…
common-viewer-loading-video = 動画を読み込んでいます…
common-viewer-rendering-pdf = PDF を描画しています…
common-viewer-pdf-failed = この PDF を描画できませんでした。
common-viewer-no-preview = この種類のファイルはプレビューできません。
common-viewer-unknown-type = 不明な種類
common-viewer-download = ダウンロード
common-viewer-attachments = 添付ファイル

# -- Machine translation (2026-11): added to match en ---------------------
common-retrying = 再試行しています…
common-waiting-online = 接続を待機しています…
common-failure-offline = オフラインです。再接続してから再試行してください。
common-failure-network = サーバーに接続できませんでした。
common-failure-server = サーバーが時間内に応答しませんでした。
common-failure-rate = 短時間にリクエストが多すぎます。しばらく待ってから再試行してください。
common-failure-conflict = 同時に別の場所で変更されました。再読み込みしてから再試行してください。
common-failure-account = サインイン中のアカウントが変更されました。ページを再読み込みしてください。
common-failure-linked = リンクされたメールボックスを開いている間は利用できません。
common-failure-refused = サーバーが変更を拒否しました。
common-new-version = Mailwoman の新しいバージョンがあります。閉じると1時間後に再度お知らせします。
common-new-version-reload = 今すぐ再読み込み
common-toast-close = 通知を閉じる
common-toast-seconds = { $seconds } 秒
common-attach-refresh = 添付ファイルを更新
common-attach-open-message = この添付ファイルを含むメッセージを開く
common-attach-page-filter-note = フィルターはこのページに適用されます。「次へ」で、添付ファイル付きの古いメッセージを表示できます。
common-attach-pages = 添付ファイルのページ
common-attach-previous = 前へ
common-attach-next = 次へ
common-attach-page = { $page } ページ
common-attach-column-name = ファイル
common-attach-column-type = 種類
common-attach-column-size = サイズ
common-attach-column-sender = 差出人
common-attach-column-subject = 件名
common-attach-column-date = 受信日時
common-pdf-preview = PDF プレビュー
common-pdf-link-title = この PDF のリンクを開きますか？
common-pdf-link-help = リンク先は Mailwoman の外の新しいタブで開きます。
common-pdf-link-open = リンクを開く
common-open-event = 予定を開く
common-message-preview-too-large = メッセージ添付のプレビューは 4 MiB までです。元の添付ファイルはダウンロードできます。
common-message-preview-failed = このメッセージ添付をプレビューできませんでした。ファイルが未対応か破損しているか、プレビュー処理が利用できない可能性があります。元のファイルはダウンロードできます。
common-message-preview-attachments = このメッセージ内のファイル
common-message-preview-download = ファイルをダウンロード
common-message-preview-size-bytes = { $count } バイト
common-attach-cat-message = メッセージ / Outlook の添付ファイル
common-error-retry-help = このセクションは一時的に利用できません。もう一度お試しください。
common-error-network-help = 接続を確認してから、もう一度お試しください。サーバーが一時的に利用できない可能性があります。
common-error-page = このページを読み込めませんでした
common-error-page-help = ページのリソースをダウンロードできませんでした。もう一度お試しください。問題が続く場合は、アプリケーションを再読み込みして最新バージョンを取得してください。
common-reload-app = アプリケーションを再読み込み
common-new-version-reloading = 新しいバージョンがあります。再読み込みしています…
common-attach-account-changed = 添付ファイルの読み込み中にアカウントが変更されました。現在のアカウントで添付ファイルを開き直してください。
common-attach-search-account = アカウント全体を検索
common-attach-scanning = 検索しています… メッセージ { $pages } ページを確認済み · { $count } 件のファイルが見つかりました
common-attach-scan-cancelled = 検索をキャンセルしました · { $pages } ページで { $count } 件のファイルが見つかりました。結果は不完全です。
common-attach-scan-incomplete = 検索が中断されました · { $count } 件のファイルが見つかりました。アカウント全体を検索するには再試行してください。
common-attach-scan-limited = 一致する最初の 1,000 件のファイルを表示しています。フィルターを絞り込むと、目的のファイルを見つけやすくなります。
common-attach-scan-complete = 検索が完了しました · メッセージ { $pages } ページで { $count } 件のファイルが見つかりました
common-attach-browse-pages = ページを閲覧
common-attach-cancel-search = 検索をキャンセル
multiselect-selected = 選択済み: { $label }
multiselect-remove = { $label } を削除
multiselect-no-match = 一致する選択肢がありません
common-notification-permission-title = 通知を許可しますか？
common-notification-permission-help = このデバイスで新着メッセージの通知を受け取ります。設定でいつでも変更できます。
common-notification-permission-allow = 通知を許可
common-notification-permission-later = 今はしない
common-notification-permission-error = 通知を有効にできませんでした。権限と通知の設定を確認してから、もう一度お試しください。
common-notifications-enabled = このデバイスで通知が有効になりました。

# -- Machine translation (2026-11): added to match en ---------------------
multiselect-count = { $count ->
   *[other] { $count } 件選択中
}

# -- Machine translation (2026-11): added to match en ---------------------
common-rate-limited = サーバーが混み合っています。数秒待ってからもう一度お試しください。

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
combobox-empty = 一致しません

combobox-toggle = { $label } のオプションを表示

common-attach-cat-archive = アーカイブ

common-attach-cat-document = 書類

common-attach-cat-spreadsheet = スプレッドシート

common-conn-reconnecting = 再接続中…

common-viewer-pdf-details = 技術的な詳細

common-viewer-pdf-details-help = 問題を報告するときは、次の行を含めてください。文書コンテンツは含まれていません。

timezone-group-other = その他

timezone-use-browser = ブラウザのタイムゾーン ({ $zone }) を使用する

common-context-actions = { $name } の操作
common-open = 開く
common-new = 新しい項目
common-refresh = 更新
common-duplicate = 複製
common-copy-fingerprint = フィンガープリントをコピー
common-export-public-key = 公開鍵または証明書をエクスポート
common-context-show = 表示
common-context-show-only = このカレンダーのみ表示
common-context-color = 色
common-context-import = インポート
common-context-export = エクスポート
common-context-pinned = ピン留めした項目
common-context-unpin = ピン留めを解除
common-context-my-day = 今日の予定に追加
common-context-remove-my-day = 今日の予定から削除
common-notify-new-count = 新着メッセージ: { $count }
common-network-internal = 内部ネットワーク
common-module-notifications = 通知
