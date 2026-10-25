# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: settings. Source: ../en/settings.ftl

settings-title = 設定
settings-appearance = 外観
settings-close = 設定を閉じる

settings-theme = テーマ
settings-density = 表示密度
settings-accent = アクセントカラー
settings-font = UI フォント
settings-layout = レイアウト

settings-density-compact = コンパクト
settings-density-cozy = 標準
settings-density-relaxed = ゆったり

settings-font-default = 既定
settings-font-system = システム
settings-font-serif = セリフ
settings-font-mono = 等幅

settings-layout-default = 既定
settings-layout-ribbon = リボン

settings-cancel = キャンセル
settings-edit = 編集
settings-delete = 削除
settings-save = 保存
settings-saved = 保存しました

settings-2fa-title = 2 段階認証
settings-2fa-intro = 2 つ目の要素を追加すると、サインイン時にパスワード以外の確認も必要になります。
settings-2fa-policy-required = 組織の方針により、このアカウントでは 2 つ目の要素が必要です。
settings-2fa-enabled = 有効
settings-2fa-recovery-title = リカバリーコード
settings-2fa-recovery-once = 今すぐ保存してください。表示されるのは一度だけで、後から取得することはできません。各コードは 1 回だけ使えます。
settings-2fa-recovery-ack = これらのコードを保存しました
settings-2fa-totp-title = 認証アプリ
settings-2fa-totp-enrol = 認証アプリを設定
settings-2fa-totp-scan = このシークレットを認証アプリに追加し、表示されたコードを入力してください。
settings-2fa-totp-uri-link = 認証アプリで開く
settings-2fa-totp-code-label = 認証コード
settings-2fa-totp-confirm = 確認
settings-2fa-totp-disable = 認証アプリを削除
settings-2fa-passkey-title = パスキー
settings-2fa-passkey-remove = 削除
settings-2fa-passkey-unsupported = このブラウザーはパスキーに対応していません。
settings-2fa-passkey-label-placeholder = パスキーの名前（任意）
settings-2fa-passkey-add = パスキーを追加
settings-2fa-recovery-remaining = 残りのリカバリーコード：{ $count ->
       *[other] { $count } 件
    }
settings-2fa-recovery-regenerate = 新しいリカバリーコードを生成
settings-2fa-recovery-code-label = リカバリーコード
settings-2fa-error-generic = 処理できませんでした。もう一度お試しください。
settings-2fa-error-code = コードを確認できませんでした。
settings-2fa-error-passkey = パスキーを登録できませんでした。

settings-2fa-challenge-title = 2 段階認証
settings-2fa-challenge-intro = サインインを完了するには、2 つ目の要素で確認してください。
settings-2fa-challenge-failed = 確認できませんでした。もう一度お試しください。
settings-2fa-challenge-use-passkey = パスキーを使用
settings-2fa-challenge-method = 方法を選択
settings-2fa-challenge-totp-tab = 認証コード
settings-2fa-challenge-recovery-tab = リカバリーコード
settings-2fa-challenge-verify = 確認

settings-sessions-title = アクティブなセッション
settings-sessions-intro = 現在このアカウントにサインインしているセッション。
settings-sessions-last-seen = 最終アクティブ：{ $when }
settings-sessions-revoke = サインアウト
settings-sessions-current = このセッション
settings-sessions-revoke-others = 他の { $count ->
       *[other] { $count } 件のセッション
    }からサインアウト
settings-sessions-error = セッションを変更できませんでした。

settings-sig-title = 署名
settings-sig-intro = 送信するメッセージの末尾に追加されるテキストです。1 つの署名を既定にできます。
settings-sig-default = 既定
settings-sig-new = 新しい署名
settings-sig-name-label = 名前
settings-sig-body-label = 署名
settings-sig-default-label = 既定として使用
settings-sig-error-name = 署名に名前を付けてください。
settings-sig-error-generic = 署名を保存できませんでした。

settings-ident-title = 差出人情報
settings-ident-intro = 送信時に使う名前とアドレスです。差出人情報ごとに別の署名を使えます。
settings-ident-new = 新しい差出人情報
settings-ident-name-label = 表示名
settings-ident-email-label = メールアドレス
settings-ident-replyto-label = 返信先アドレス（任意）
settings-ident-signature-label = 既定の署名
settings-ident-signature-none = なし
settings-ident-error-fields = 名前と有効なメールアドレスを入力してください。
settings-ident-error-generic = 差出人情報を保存できませんでした。

settings-notif-title = 通知
settings-notif-intro = どの新着メッセージを通知するか、いつ通知を止めるかを選択します。
settings-notif-enabled-label = 新着メールの通知を表示
settings-notif-quiet-title = おやすみ時間
settings-notif-quiet-enabled-label = 指定した時間は通知をミュート
settings-notif-quiet-start = 開始
settings-notif-quiet-end = 終了
settings-notif-rules-title = ルール
settings-notif-rule-match-placeholder = 送信者、メールボックス、件名に含まれる語…
settings-notif-rule-action-label = 動作
settings-notif-rule-notify = 通知
settings-notif-rule-mute = ミュート
settings-notif-rule-add = ルールを追加
settings-notif-error = 通知設定を保存できませんでした。

settings-search-title = 保存した検索
settings-search-intro = 保存した検索を、メールボックス一覧にフォルダーとして表示します。
settings-search-empty = 保存した検索はまだありません。
settings-search-as-folder = { $name } をフォルダーとして表示
settings-search-as-folder-label = フォルダーとして表示
settings-search-error = 保存した検索を変更できませんでした。

settings-kbd-title = キーボードショートカット
settings-kbd-intro = ショートカットのセットを選択します。変更はこのデバイスに適用されます。
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-off = オフ（ショートカットなし）
settings-kbd-action-compose = 作成
settings-kbd-action-archive = アーカイブ
settings-kbd-action-reply = 返信
settings-kbd-action-next = 次のメッセージ
settings-kbd-action-previous = 前のメッセージ
settings-kbd-action-search = 検索

settings-offline-title = オフラインキャッシュ
settings-offline-intro = オフラインで使うためにこのデバイスに保持するメールの量と、容量の回収方法を設定します。
settings-offline-budget-label = キャッシュ容量（MB）
settings-offline-retention-label = 保持期間（日）
settings-offline-strategy-label = 容量の上限に達したとき
settings-offline-lru = 最近使っていないものから削除
settings-offline-oldest = 古いものから削除
settings-offline-manual = 手動で消去したときのみ
settings-offline-purge = オフラインキャッシュを今すぐ消去

settings-dir-title = UI の方向
settings-dir-intro = 言語に従うか、方向を固定します。右から左にするとレイアウトが反転します。
settings-dir-auto = 自動
settings-dir-ltr = 左から右
settings-dir-rtl = 右から左
settings-dir-preview-title = プレビュー
settings-dir-preview-body = 方向が右から左のとき、このテキストは反転します。
