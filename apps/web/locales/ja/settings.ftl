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
settings-mail-features = メール機能
settings-mail-feature-allowSend = メッセージの送信を許可する
settings-mail-feature-allowSend-help = オフにすると、送信とそのショートカットが無効になります。下書きは引き続き保存できます。
settings-mail-feature-allowReply = 返信と全員に返信を許可する
settings-mail-feature-allowReply-help = オフにすると、返信と全員に返信の操作およびそのショートカットは表示されません。
settings-mail-feature-allowForward = 転送を許可する
settings-mail-feature-allowForward-help = オフにすると、転送の操作とそのショートカットは表示されません。
settings-mail-feature-allowDownload = メッセージと添付ファイルのダウンロードを許可する
settings-mail-feature-allowDownload-help = オフにすると、メッセージと添付ファイルのダウンロードは表示されません。
settings-mail-feature-allowPrint = 印刷を許可する
settings-mail-feature-allowPrint-help = オフにすると、印刷の操作は表示されません。
settings-mail-feature-showSpam = スパムフォルダーを表示する
settings-mail-feature-showSpam-help = オフにすると、スパムフォルダーとスパムとしてマークされたメッセージがフォルダー一覧と検索結果で非表示になります。何も削除されません。
settings-mail-feature-showAuthStatus = 送信者認証の結果を表示する
settings-mail-feature-showAuthStatus-help = リーダーにメッセージの SPF、DKIM、DMARC の結果を表示します。
settings-mail-feature-closeOnDelete = 削除したメッセージを閉じる
settings-mail-feature-closeOnDelete-help = 開いているメッセージが削除またはゴミ箱に移動されたとき、そのメッセージのタブまたはウィンドウを閉じます。

# -- Machine translation (2026-11): added to match en ---------------------
settings-breadcrumb = 設定のパス
settings-crumb-loading = 名前を読み込み中
settings-crumb-new = 新規
settings-crumb-edit = 編集
settings-badge-color = 未読バッジの色
settings-badge-color-accent = アクセントカラーと同じ
settings-badge-color-accent-lighter = アクセントカラーより明るく
settings-badge-color-accent-darker = アクセントカラーより暗く
settings-badge-color-option = { $color }のバッジ
settings-appearance-theme-card = テーマと切り替え
settings-appearance-theme-card-help = テーマを選び、システムまたは時間帯に合わせるかどうかを設定します。
settings-appearance-colours-card = 色
settings-appearance-colours-card-help = 操作や選択に使うアクセントカラーと、未読カウンターの色です。
settings-appearance-text-card = 文字とレイアウト
settings-appearance-text-card-help = インターフェースの密度、使用するフォント、メールのレイアウトを設定します。
settings-sig-create-variant = バリエーションを作成
settings-sig-error-existing = この名前の署名はすでに存在します。別の名前を選択してください。
settings-sig-save-failed = 署名を保存できませんでした。{ $reason }
settings-ident-signature-new = 新規メッセージ
settings-ident-signature-account = アカウントの署名を使用
settings-ident-signature-replies = 返信
settings-ident-signature-inherit = 新規メッセージの署名を使用
settings-ident-signature-unavailable = 利用できない署名: { $name }
settings-ident-signature-help = 「署名」で署名のバリエーションを作成し、ここで割り当てます。同じバリエーションを両方に使用できます。
settings-ident-save-failed = 差出人情報を保存できませんでした。{ $reason }
settings-search-save-failed = 保存済み検索を保存できませんでした。{ $reason }
settings-kbd-preset-title = ショートカットのプリセット
settings-editor-discard-title = 未保存の変更を破棄しますか?
settings-editor-discard-detail = 変更はまだ保存されていません。編集を続けるか、この下書きを破棄してエディターを閉じてください。
settings-editor-keep = 編集を続ける
settings-editor-discard = 変更を破棄
settings-editor-missing = この項目はすでに存在しません。一覧に戻って別の項目を選択してください。
settings-editor-back = 一覧に戻る
settings-sig-edit-title = 署名を編集
settings-ident-edit-title = 差出人情報を編集
settings-sections = 設定のセクション
settings-category-account = アカウント
settings-category-mail = メール
settings-category-interface = インターフェース
settings-category-integrations = 連携
settings-category-about = 情報
settings-nav-preferences = キーボードと表示
settings-nav-rules = メールルール
settings-nav-access = API アクセス
settings-nav-metadata = サーバー情報
settings-folders-title = フォルダーとサイドバー
settings-folders-help = フォルダーを他より上に固定し、アイコンを選び、ドラッグまたは移動ボタンで各グループ内の順序を調整します。これらの設定はこのアカウントと同期され、メッセージの移動やフォルダー名の変更は行われません。
settings-folders-pin = { $name }を固定
settings-folders-icon = { $name }のアイコン
settings-folders-automatic = 自動のフォルダーの役割
settings-folders-drag = { $name }をドラッグして並べ替え
settings-folders-up = { $name }を上へ移動
settings-folders-down = { $name }を下へ移動
settings-folders-sync-idle = フォルダーの設定を同期するにはログインしてください。
settings-folders-sync-loading = アカウントの設定を読み込み中…
settings-folders-sync-saving = アカウントの設定を保存中…
settings-folders-sync-synced = 設定をこのアカウントと同期しました。
settings-folders-sync-error = 設定を同期できませんでした。変更はこのセッションに保持され、接続が回復したときに再試行されます。
settings-folder-icon-folder = フォルダー
settings-folder-icon-star = スター
settings-folder-icon-flag = フラグ
settings-folder-icon-tag = タグ
settings-folder-icon-clock = 時計
settings-folder-icon-calendar = カレンダー
settings-folder-icon-file = ドキュメント
settings-folder-icon-image = 画像
settings-folder-icon-lock = 鍵
settings-folder-icon-users = ユーザー
settings-folder-icon-archive = アーカイブ
settings-folder-icon-mail = 封筒
settings-folder-icon-attachment = 添付ファイル
settings-folder-icon-check = チェックマーク
settings-folder-icon-alert = 警告
settings-search-new = 新しい検索フォルダー
settings-search-editor = 検索フォルダー
settings-search-editor-help = メッセージはすべての条件に一致する必要があります。条件を空にすると無視されます。検索フォルダーは、一致するメッセージを移動やコピーをせずに表示します。
settings-search-name = フォルダー名
settings-search-name-required = この検索フォルダーの名前を入力してください。
settings-search-invalid = この保存済み検索には無効なフィルターがあり、安全に編集できません。
settings-search-saved = 検索フォルダー「{ $name }」を保存しました。
settings-search-field-text = すべてのメッセージ本文を検索
settings-search-field-from = 送信者に含む
settings-search-field-to = 宛先に含む
settings-search-field-subject = 件名に含む
settings-search-field-body = メッセージ本文に含む
settings-search-field-filename = 添付ファイル名に含む
settings-search-field-inMailbox = メールボックス識別子で絞り込む(任意)
settings-search-field-after = 受信日時が次より後(UTC の日付と時刻)
settings-search-field-before = 受信日時が次より前(UTC の日付と時刻)
settings-search-attachment = 添付ファイル
settings-search-any = すべてのメッセージ
settings-search-with-attachments = 添付ファイルあり
settings-search-without-attachments = 添付ファイルなし
settings-search-unread = 未読メッセージのみ
settings-search-flagged = フラグ付きメッセージのみ
settings-sessions-search = アカウント、デバイス、IP を検索
settings-sessions-account = アカウント
settings-sessions-device = デバイスとクライアント
settings-sessions-location = IP と最後に確認された国
settings-sessions-location-unavailable = 国を取得できません
settings-sessions-activity = アクティビティ
settings-sessions-actions = 操作
settings-sessions-revoke-device = { $device }からログアウト
settings-font-inter = Inter
settings-font-manrope = Manrope
settings-font-roboto = Roboto
settings-font-open-sans = Open Sans
settings-font-jost = Jost
settings-compose-title = 作成と返信
settings-quote-style = 返信と転送のヘッダー形式
settings-quote-classic = 従来の引用返信
settings-quote-outlook = Outlook 形式のヘッダー
settings-quote-help = Outlook 形式では、リッチテキストとプレーンテキストの両方で、元のメッセージの上に差出人、送信日時、宛先、Cc、件名を追加します。既存の下書きは変更されません。
settings-external-warning = 外部受信者への警告
settings-external-off = オフ
settings-external-warn = 警告を表示
settings-external-confirm = 送信前に確認を必須にする
settings-external-help = 宛先、Cc、Bcc を、選択した差出人情報のドメインと信頼するドメインの一覧と照合します。この作成画面からの送信を保護するもので、サーバーの配信ルールではありません。
settings-trusted-domains = 信頼する受信者のドメイン
settings-trusted-domains-help = 完全一致のドメインを 1 行に 1 つ、最大 100 件入力します。サブドメインは別々に記載する必要があります。送信する差出人情報自身のドメインは常に信頼されます。URL、メールアドレス、ワイルドカードは使用できません。
settings-trusted-domains-invalid = URL、メールアドレス、ワイルドカードを含まない有効なドメイン名を、最大 100 件まで入力してください。
settings-trusted-domains-save = 信頼するドメインを保存
settings-shortcuts-enabled = アプリのショートカットを有効にする
settings-shortcuts-action = 操作
settings-shortcuts-binding = キー割り当て
settings-shortcuts-edit = ショートカットを編集
settings-shortcuts-edit-action = ショートカット「{ $action }」を編集
settings-shortcuts-help = 選択したプリセットに従う場合は inherit、ショートカットなしの場合は disabled、または Ctrl+Shift+K のようなキーの組み合わせを入力します。macOS では Ctrl は Cmd を意味します。競合する割り当ては無効になります。入力欄とダイアログでは通常のキーが維持されます。ブラウザーが予約している組み合わせは使用できない場合があります。
settings-shortcuts-invalid = inherit、disabled、または Ctrl、Alt、Shift の修飾キーを任意で付けた有効な 1 つのキーを入力してください。
settings-shortcuts-conflict = 2 つの操作が同じショートカットを使用しています。どちらかの割り当てを変更してください。競合するショートカットは動作しません。
settings-shortcuts-discard = 未保存のショートカットの変更を破棄しますか?
settings-shortcuts-save = ショートカットを保存
settings-shortcuts-back = ショートカットに戻る
settings-shortcuts-off = プリセットが「オフ」の場合、カスタム割り当てを含むすべてのショートカットが無効になります。これらの割り当てを使うには、環境設定で別のプリセットを選択してください。
settings-kbd-action-replyAll = 全員に返信
settings-kbd-action-forward = 転送
settings-kbd-action-refresh = メッセージを更新
settings-kbd-action-settings = 設定を開く
settings-kbd-action-trash = ゴミ箱に移動
settings-kbd-action-markRead = 既読にする
settings-kbd-action-markUnread = 未読にする
settings-kbd-action-spam = 迷惑メールにする
settings-kbd-action-zoomIn = メッセージを拡大
settings-kbd-action-zoomOut = メッセージを縮小
settings-kbd-action-zoomReset = メッセージの拡大率をリセット
settings-kbd-action-renameFolder = フォルダー名を変更
settings-kbd-action-openInTab = メッセージを専用のタブで開く
settings-notif-device-title = このデバイスでの許可
settings-notif-device-help = 通知の設定はアカウントに紐づきます。ブラウザーの許可はこのデバイスに属し、Mailwoman は、許可を選択したときにのみ確認します。
settings-notif-ask-permission = 新しいデバイスで通知の許可を求める
settings-notif-ask-permission-help = このデバイスで通知が許可も拒否もされていない場合に、許可の確認を表示します。既定で有効です。この設定はすぐに保存されます。
settings-notif-permission-default = ブラウザーの許可はまだ要求されていません。
settings-notif-permission-granted = このデバイスでブラウザー通知が許可されています。
settings-notif-permission-denied = 通知はブロックされています。許可するには、ブラウザーの設定でこのサイトの権限を変更してください。
settings-notif-permission-unsupported = このブラウザーでは、ここでシステム通知を利用できません。
settings-notif-permission-request = このデバイスで通知を許可
settings-notif-permission-error = ブラウザーが通知の許可を要求できませんでした。
settings-notif-timezone = 通知を控える時間帯のタイムゾーン(IANA)
settings-notif-timezone-device = このデバイスのタイムゾーンを使用
settings-permissions-background-title = このブラウザーでのバックグラウンド通知
settings-permissions-background-help = 先に通知の設定を保存し、ブラウザー通知を許可してください。このブラウザーを登録すると、アプリを閉じていてもアカウントのアクティビティを受け取れます。
settings-permissions-push-unsupported = このブラウザーではバックグラウンドの Web Push を利用できません。
settings-permissions-push-registered = このブラウザーにはプッシュ購読があります。現在のアカウントに紐づけるには登録してください。
settings-permissions-push-unregistered = このブラウザーにはプッシュ購読がありません。
settings-permissions-push-register = バックグラウンド通知を登録
settings-permissions-push-remove = ブラウザーの購読を削除
settings-permissions-push-policy = 管理者はバックグラウンドのプッシュ通知を許可していません。
settings-permissions-storage-title = ブラウザーのストレージ
settings-permissions-storage-help = 永続ストレージは、ブラウザーが許可した場合に、このサイトのローカルデータが自動的に削除されるのを防ぎます。これはデバイス内のみで、サーバーの保持期間は変わりません。ブラウザーの設定でサイトデータを消去できますが、未送信のオフライン変更が失われる場合があります。
settings-permissions-storage-unsupported = このブラウザーでは永続ストレージのリクエストを利用できません。
settings-permissions-storage-persistent = ブラウザーがこのサイトに永続ストレージを許可しています。
settings-permissions-storage-evictable = ブラウザーがこのサイトのローカルデータを自動的に削除する場合があります。
settings-permissions-storage-usage = サイトの現在のストレージ使用量: { $mib } MiB。
settings-permissions-storage-request = 永続ストレージを要求
settings-permissions-storage-policy = 管理者は永続ストレージのリクエストを許可していません。
settings-permissions-error = ブラウザーの操作を完了できませんでした。保存済みのアカウント設定と、このサイトのブラウザー権限を確認してください。
settings-notif-background-rules = コンテンツルールは Mailwoman を開いている間に評価されます。コンテンツルールが設定されている間は、バックグラウンドのシステム通知は抑制されます。不透明なバックグラウンドの起動には、メッセージの詳細が含まれないためです。
settings-time-title = タイムゾーン
settings-time-help = メッセージの時刻の表示方法と、スケジュール調整用の比較タイムゾーン(任意)を選択します。場所がメールアドレスから推測されることはありません。
settings-time-displayZone = 表示タイムゾーン
settings-time-displayZone-help = このデバイスの場合は local、または Europe/Lisbon のような IANA 名を使用します。
settings-time-comparisonZone = 比較タイムゾーン
settings-time-comparisonZone-help = America/New_York のような IANA 名を入力します。比較を非表示にする場合は空のままにします。
settings-time-helpers = タイムゾーンの補助情報を表示
settings-time-save = タイムゾーンを保存
settings-time-invalid = 有効な IANA タイムゾーンを入力してください。local は表示タイムゾーンにのみ使用でき、比較タイムゾーンは空のままにできます。
settings-time-discard = 未保存のタイムゾーンの変更を破棄しますか?
settings-folders-name = フォルダー
settings-folders-actions = 操作
settings-folders-edit = { $name }の外観を編集
settings-folders-icon-help = 50 種類のテーマアイコンから 1 つ選ぶか、フォルダーの用途に基づく自動選択のままにします。この設定はアカウントに紐づき、メールボックス名は変更されません。
settings-folders-save = フォルダーの外観を保存
settings-folders-back = フォルダーに戻る
settings-folders-discard = 未保存のフォルダーの外観の変更を破棄しますか?
settings-folder-labels = フォルダーのラベル
settings-folder-labels-help = アカウントのラベルを最大 32 個使って、このフォルダーを整理します。フォルダー内のメッセージにラベルを付けたり、移動したりすることはありません。ラベル名と色は「ラベル」で管理します。
settings-folders-leave-pending = これらの変更はまだ同期待ちです。このエディターを閉じますか?
settings-folders-retry = 保存を再試行
settings-folder-role-title = 特別なフォルダー
settings-folder-role-help = メールの各役割に使う実際のフォルダーを選択します。「自動」はサーバーの検出を使用します。「管理者の既定値を使用」は、あなたの上書き設定を削除します。移動先が競合している、または存在しない場合は、別のフォルダーを選ぶのではなく、その役割が無効になります。

# -- Machine translation (2026-11): added to match en ---------------------
cache-clear = キャッシュされたメールを今すぐ消去
settings-compose-window = 新規メッセージを開く場所
settings-compose-window-tab = ワークスペースのタブ
settings-compose-window-new = 新しいウィンドウ
settings-compose-window-help = ブラウザーが新しいウィンドウをブロックした場合は、作成画面がワークスペースのタブで開きます。
settings-compose-read-receipt = 常に開封確認を要求
settings-compose-delivery-receipt = 常に配信確認を要求
settings-compose-reply-copy = 返信のコピーを元のフォルダーに保存
settings-compose-receipts-help = 確認の通知はあくまで要求です。受信者またはそのメールサーバーが拒否または無視する場合があります。
settings-compose-forward = 既定の転送形式
settings-compose-forward-inline = メッセージ本文をインラインで転送
settings-compose-forward-attachment = 元のメッセージを .eml 添付ファイルとして転送
settings-compose-saving = 作成の設定を保存中…
settings-compose-saved = 作成の設定を保存しました
settings-ident-organization = 組織
settings-ident-auto-bcc = 自動的に Bcc に追加
settings-ident-auto-bcc-help = この差出人情報から送信するときに、最大 20 件のアドレスを Bcc に追加します。アドレスはカンマで区切ります。
settings-ident-pgp-key = OpenPGP 鍵
settings-ident-pgp-none = 既定の鍵を使用
settings-ident-pgp-unavailable = 保存済みの鍵を利用できません — 別の鍵を選択してください
settings-ident-pgp-help = 既存の OpenPGP 署名鍵から 1 つ選択します。秘密鍵のデータは鍵ボールトに残ります。
settings-ident-error-optional = Reply-To と自動 Bcc のアドレス、組織、選択した鍵を確認してください。
settings-compose-open-tracking = 既定でメッセージの開封を記録する
settings-compose-open-tracking-help = 対象となる書式付きメッセージに、トラッキング画像を追加します。開封の記録は、受信者が画像を読み込むかどうかに依存し、メッセージが読まれたことの証明にはなりません。管理者はトラッキングを無効にしたり、この既定値をロックしたりできます。
settings-compose-open-tracking-inherit = 継承されたトラッキングの既定値を使用
settings-reply-images = 返信と転送での画像
settings-reply-images-ask = 毎回確認
settings-reply-images-allow = 常に含める
settings-reply-images-trusted = 信頼する送信者のドメインでは含め、それ以外は確認
settings-reply-images-deny = 含めない
settings-reply-images-help = 返信や転送に、元のメッセージの画像を含めるかどうかを選択します。含めた画像は、添付ファイルとして下書きにコピーされます。
settings-reply-image-domains = 信頼する画像の送信者ドメイン
settings-reply-image-domains-help = 完全一致の送信者ドメインを 1 行に 1 つ、最大 200 件入力します。上の選択が「信頼するドメイン」のときに使用されます。
settings-reply-image-domains-save = 信頼する画像のドメインを保存
settings-ident-group-basics = 基本
settings-ident-group-replies = 返信の処理
settings-ident-group-signature = 署名
settings-ident-group-sending = 送信オプション
settings-ident-name-required = 表示名を入力してください。
settings-ident-email-invalid = name@example.org のような有効なメールアドレスを入力してください。
settings-ident-email-duplicate = このアドレスの差出人情報はすでにあります。
settings-ident-replyto-invalid = 有効なメールアドレスを入力するか、空のままにしてください。
settings-ident-replyto-help = この差出人情報から送信したメッセージへの返信は、送信者アドレスではなくこのアドレスに届きます。
settings-ident-bcc-invalid = Bcc のリストを確認してください。有効なアドレスをカンマで区切り、最大 20 件までです。
settings-ident-delete-title = 差出人情報 { $email } を削除しますか?
settings-ident-delete-body = この差出人情報から送信できなくなります。下書きと送信済みメッセージには影響しません。
settings-ident-empty = 差出人情報はまだありません。追加すると、作成時に名前、アドレス、署名を選択できます。
settings-sig-delete-title = 署名 { $name } を削除しますか?
settings-sig-delete-body = この署名を使用している差出人情報は、アカウントの署名に切り替わります。送信済みのメッセージには影響しません。
settings-sig-deleted = 署名を削除しました。
settings-reply-from = 返信の差出人
settings-reply-from-match = メッセージが送信された宛先のアドレス
settings-reply-from-default = 常に既定の差出人情報
settings-reply-from-ask = 複数の差出人情報が一致する場合は確認
settings-reply-from-keep = すでに選択されている差出人情報を維持
settings-reply-from-help = 返信または転送するときに、使用する差出人情報を選択します。メッセージの宛先アドレスに一致する差出人情報がない場合は、既定の差出人情報が使用されます。name+tag@example.org のようなプラス付きアドレスは、name@example.org に一致します。
settings-ident-default-badge = 既定
settings-ident-make-default = 既定にする
settings-ident-default-set = { $email } が既定の差出人情報になりました。
settings-ident-create-variant = バリエーションを作成
settings-ident-variant-count = { $count } { $count ->
       *[other] 件のバリエーション
    }
settings-ident-inherits = { $base } から次を引き継ぎ: { $fields }
settings-ident-field-replyTo = Reply-To
settings-ident-field-autoBcc = 自動 Bcc
settings-ident-field-organization = 組織
settings-ident-field-pgpKeyId = OpenPGP 鍵
settings-ident-field-signatureName = 署名
settings-ident-delete-default-body = これは既定の差出人情報です。残りの差出人情報のうち最も古いものが既定になります。この差出人情報から送信できなくなります。下書きと送信済みメッセージには影響しません。
settings-ident-variants-choice-title = { $email } を削除しますか?
settings-ident-variants-choice-body = この差出人情報には { $count } { $count ->
       *[other] 件のバリエーション
    }があります。{ $count ->
       *[other] それらも一緒に削除
    }するか、独立した差出人情報として残すことができます。残したバリエーションには、この差出人情報から引き継いでいた設定がコピーされます。
settings-ident-delete-with-variants = 差出人情報とバリエーションを削除
settings-ident-delete-keep-variants = 差出人情報を削除し、バリエーションは残す
settings-ident-new-variant = 新しいバリエーション
settings-ident-edit-variant-title = バリエーションを編集
settings-ident-variant-intro = { $base } のバリエーションです。項目を空にすると、その差出人情報の値が使用されます。
settings-ident-label-label = バリエーションのラベル
settings-ident-label-required = このバリエーションのラベルを入力してください(例: ニュースレター)。
settings-ident-inherit-hint = 空の場合: 元の差出人情報の { $value } を使用します。
settings-ident-inherit-none = なし
settings-check-policy-allow = 確認せずに送信
settings-check-policy-warn = 毎回確認
settings-check-policy-warn-remember = 確認する(今後は確認しない設定も可能)
settings-check-policy-deny = 送信をブロック
settings-check-invalid = 無効な受信者アドレス
settings-check-attachment = 添付ファイルに言及しているが添付されていない
settings-check-subject = 件名が空
settings-check-help = チェックで問題が見つかったときの動作です。「確認」では常に「このまま送信」が表示されます。メッセージを止めるのは「送信をブロック」だけです。
settings-external-warn-remember = 確認する(今後は確認しない設定も可能)
settings-external-deny = 送信をブロック
settings-auto-chip = スペース、カンマ、セミコロンを入力したら受信者を確定する
settings-auto-chip-help = 有効なアドレスは確定した受信者になり、区切り文字が自動的に追加されます。
settings-recipient-separator = 受信者の区切り文字
settings-recipient-separator-comma = カンマ
settings-recipient-separator-semicolon = セミコロン
settings-header-collapse = メッセージヘッダー(宛先、Cc、件名)
settings-header-collapse-expanded = 常に表示
settings-header-collapse-collapsed = 1 行の概要に折りたたむ
settings-header-collapse-auto = 本文の入力中は折りたたむ
settings-header-collapse-help = 折りたたむと、ヘッダーは「宛先: Ana +2」のような 1 行で表示され、クリックしたとき、または入力欄に注意が必要なときに開きます。
settings-retention-title = 保持
settings-folder-icon-camera = カメラ
settings-folder-icon-music = 音楽
settings-folder-icon-film = 映画
settings-folder-icon-lightbulb = アイデア
settings-folder-icon-rocket = ロケット
settings-folder-icon-trophy = トロフィー
settings-folder-icon-leaf = 葉
settings-folder-icon-globe = 地球
settings-folder-icon-cloud = 雲
settings-folder-icon-bookmark = ブックマーク
settings-folder-icon-wallet = 財布
settings-folder-icon-medical = 医療
settings-folder-icon-bell = ベル
settings-folder-icon-key = 鍵
settings-folder-icon-receipt = 領収書
settings-folder-icon-cart = ショッピングカート
settings-folder-icon-package = 荷物
settings-folder-icon-wrench = ツール
settings-folder-icon-chart = グラフ
settings-folder-icon-team = チーム
settings-folder-icon-store = ストア
settings-folder-icon-office = オフィス
settings-folder-icon-laptop = ノートパソコン
settings-folder-icon-crown = 王冠
settings-folder-icon-family = 家族
settings-folder-icon-group-general = 一般
settings-folder-icon-group-mail = メール
settings-folder-icon-group-work = 仕事
settings-folder-icon-group-finance = 金融とショッピング
settings-folder-icon-group-travel = 旅行と家
settings-folder-icon-group-personal = 個人と趣味
settings-folder-icon-group-tech = テクノロジーとツール
settings-folder-icon-search-label = アイコンを検索
settings-folder-icon-no-match = 一致するアイコンがありません

# -- Machine translation (2026-11): added to match en ---------------------
settings-folder-role-auto = 自動検出
settings-folder-role-inherit = 管理者の既定値を使用
settings-folder-role-path = 設定されたパス: { $path }
settings-folder-role-missing = 設定されたフォルダーを利用できません
settings-folder-role-conflict = この移動先が存在しない、または別の役割と競合しています。マッピングが修正されるまで、その役割は利用できません。
settings-folder-role-refresh-error = 保存しましたが、フォルダー一覧を更新できませんでした。新しいマッピングを使う前にフォルダーを更新してください。
settings-folder-role-unsupported = これらの上書き設定には、ネイティブのメールフォルダーが必要です。上流の JMAP アカウントでは、サーバーのフォルダーの役割がそのまま使われます。
settings-folder-role-inbox = 受信トレイ
settings-folder-role-drafts = 下書き
settings-folder-role-sent = 送信済みメール
settings-folder-role-trash = ゴミ箱
settings-folder-role-junk = 迷惑メール
settings-folder-role-archive = アーカイブ
settings-sessions-select = 選択
settings-sessions-select-device = { $device }のセッションを選択
settings-sessions-select-visible = 表示中の他のセッションをすべて選択
settings-sessions-current-protected = このセッションはログインしたままで、ここでは選択できません。終了するには「ログアウト」を使用してください。
settings-sessions-selected-title = 選択したセッションをログアウト({ $count })
settings-sessions-selected-confirm = 選択した他のセッションのみを終了します。このセッションはログインしたままです。失敗またはスキップしたリクエストは報告され、自動的には再試行されません。
settings-sessions-selected-action = 選択したセッションをログアウト
settings-sessions-cancel = キャンセル
settings-sessions-selected-result = ログアウト済み: { $revoked }。すでに終了: { $unchanged }。確認できず: { $failed }。未実行: { $skipped }。
settings-sessions-refresh-failed = セッション一覧を更新できませんでした。さらにセッションを選択する前に、再読み込みしてください。
settings-sig-format = 署名の形式
settings-sig-format-plain = プレーンテキスト
settings-sig-format-html = リッチテキスト(HTML)
settings-sig-html-help = HTML は 2 MiB までに制限され、保存時と送信時にサニタイズされます。リモート画像とアクティブなコンテンツは削除されます。
settings-sig-html-source = HTML ソースを編集
settings-sig-editor-loading = 署名エディターを読み込み中…
settings-sig-rich-body = リッチ署名
settings-sig-plain-alternative = プレーンテキストの代替
settings-sig-plain-help = プレーンテキストのメッセージにはこのテキストが使われます。カスタマイズするまでは、リッチ編集の内容が反映されます。HTML ソースの編集では、代替テキストは保持されます。
settings-sig-account-changed = ログインしているアカウントが変更されました。正しいアカウントでこの署名を開き直してください。
settings-sig-image-type = { $name } は PNG、JPEG、GIF、WebP のいずれの画像でもありません。
settings-sig-image-size = { $name } は収まりません。画像を含む署名の HTML は 2 MiB までです。
settings-sig-too-large = この署名の HTML が 2 MiB を超えています。画像または書式の一部を削除してください。
settings-sig-images-help = 画像は署名内に保存され、各メッセージにインライン画像として送信されます。
settings-folder-icon-inbox = 受信トレイ
settings-folder-icon-sent = 送信済み
settings-folder-icon-edit = 編集
settings-folder-icon-delete = 削除
settings-folder-icon-spam = 迷惑メール
settings-folder-icon-search = 検索
settings-folder-icon-pin = ピン
settings-folder-icon-link = リンク
settings-folder-icon-shield = 盾
settings-folder-icon-unlock = ロック解除
settings-folder-icon-fileAudio = オーディオ
settings-folder-icon-fileVideo = ビデオ
settings-folder-icon-filePdf = PDF
settings-folder-icon-fileCode = コード
settings-folder-icon-print = 印刷
settings-folder-icon-download = ダウンロード
settings-folder-icon-preferences = 環境設定
settings-folder-icon-dashboard = ダッシュボード
settings-folder-icon-analytics = 分析
settings-folder-icon-nextcloud = クラウドファイル
settings-folder-icon-domains = ドメイン
settings-folder-icon-twofactor = 認証
settings-folder-icon-assistant = アシスタント
settings-folder-icon-integrations = 連携
settings-folder-icon-logs = ログ
settings-folder-icon-home = ホーム
settings-folder-icon-briefcase = ブリーフケース
settings-folder-icon-heart = ハート
settings-folder-icon-shopping = ショッピング
settings-folder-icon-education = 教育
settings-folder-icon-travel = 旅行
settings-folder-icon-coffee = コーヒー
settings-folder-icon-gift = ギフト
settings-folder-icon-bank = 銀行
settings-folder-icon-book = 本
settings-nav-keyboard-display = キーボードと表示
settings-2fa-passkey-policy-disabled = 管理者が、このアカウントでのパスキーによるログインと登録を無効にしています。既存の認証情報は一覧に残るため、管理または削除できます。
settings-mail-interactions = メッセージの選択と移動
settings-mail-modifier-selection = 修飾キーでメッセージを選択
settings-mail-modifier-selection-help = Ctrl または Command で個別のメッセージの選択を切り替えます。Shift で範囲を選択し、Shift と矢印キーで範囲を広げます。オフの場合でも、表示されている選択ボタンは引き続き使えます。
settings-mail-drag-folder = メッセージをフォルダーにドラッグ
settings-mail-drag-folder-help = 選択済みのメッセージをドラッグすると選択全体を移動し、未選択のメッセージをドラッグするとそのメッセージだけを移動します。このアカウント内のフォルダーのみ受け付けます。タッチ操作やキーボードで操作する場合は、選択ツールバーを使用できます。
settings-search-sections = 設定を検索
settings-search-no-sections = 一致する設定セクションがありません。
settings-time-local = このデバイスのタイムゾーン
settings-time-none = 比較タイムゾーンなし
settings-folder-counts = フォルダーの件数
settings-folder-counts-none = 件数を非表示
settings-folder-counts-unread = 未読メッセージ
settings-folder-counts-total = メッセージの合計
settings-folder-counts-both = 未読と合計のメッセージ
settings-folder-counts-unreadIfAny = 未読メッセージ(ある場合のみ)
settings-folder-count-scope = 件数を表示するフォルダー
settings-folder-count-scope-all = すべてのフォルダー
settings-folder-count-scope-main = 主要なフォルダーのみ
settings-folder-count-scope-help = 主要なフォルダーは、受信トレイ、下書き、送信済み、アーカイブ、迷惑メール、ゴミ箱、送信トレイ、その他の特別な用途のフォルダーです。カスタムフォルダーとそのサブフォルダーには件数が表示されません。
settings-account-unread-badge = メールボックス名の横に未読数を表示
settings-folder-counts-compact = 受信トレイ以外は小さいバッジにする
settings-focused-inbox = 優先受信トレイ
settings-focused-inbox-help = 受信トレイを、一覧の上で「優先」と「その他」のタブに分けます。ニュースレターや自動送信者からのメッセージは「その他」に入ります。他のフォルダーと検索結果には、すべてのメッセージが表示されます。
settings-focused-senders-title = 優先する送信者を管理
settings-focused-senders-help = 常に「優先」または「その他」に移動するよう選択した送信者です。削除すると、自動の振り分けが再び判断します。
settings-focused-senders-empty = 送信者はまだありません。メッセージで「常に優先に移動」または「常にその他に移動」を使用してください。
settings-focused-senders-to-focused = 常に優先
settings-focused-senders-to-other = 常にその他
settings-focused-senders-remove = 削除
settings-focused-senders-remove-label = { $sender }を削除
settings-focused-senders-remove-failed = 送信者を削除できませんでした。もう一度お試しください。
settings-folder-counts-reset = フォルダー件数の設定を継承
settings-folder-counts-reset-size = バッジサイズの設定を継承
settings-mail-resume-tabs = 開いていたタブを再開する
settings-mail-resume-tabs-help = Mailwoman を開き直したときに、アカウントに紐づくタブの参照を復元します。メッセージ本文と添付ファイルの URL は、ワークスペースの記録には保存されません。
settings-row-actions-mode = メッセージ一覧のクイックアクション
settings-row-actions-mode-open = 常に表示
settings-row-actions-mode-hover = ホバー時に表示
settings-row-actions-mode-click = クリック時に表示
settings-row-actions-mode-off = オフ
settings-row-actions-mode-help = 各メッセージ行のクイックアクションの表示方法です。ホバー時は 3 つのコントロールと「その他」ボタンを表示し、クリック時はそれらを開く 1 つのボタンを表示します。タッチ画面では、ホバーはクリックと同じ動作になります。オフにすると行から削除されます(コンテキストメニューとツールバーには、すべての操作が残ります)。
settings-mail-feature-showGoToTop = メッセージ一覧に「先頭へ移動」ボタンを表示
settings-mail-feature-showGoToTop-help = メッセージ一覧を最初のメッセージまでスクロールして戻すボタンを追加します。既定ではオフです。
settings-mail-feature-showCreateFolderButton = 「作成」の横に「フォルダーを作成」ボタンを表示
settings-mail-feature-showCreateFolderButton-help = フォルダーを作成する、アイコンのみのボタンをサイドバーに追加します。既定ではオフです。
settings-draft-save = アカウントの下書き
settings-draft-save-help = 別のデバイスからアクセスできるよう、アカウントの下書きフォルダーに保存します。保存は目立たずに行われ、失敗したときだけ通知が表示されます。対応していないアカウントや、保護された復号済みコンテンツが、気づかないうちにアップロードされることはありません。
settings-draft-save-mode = 自動保存
settings-draft-realtime = 入力が止まったとき
settings-draft-periodic = 一定の間隔で
settings-draft-manual = 「保存」を選択したときのみ
settings-draft-debounce = 入力の停止時間(ミリ秒)
settings-draft-interval = 保存間隔(秒)
settings-permissions-push-other-registration = このブラウザーには、このアカウントに登録されていない購読があります。再登録する前に削除してください。
settings-show-from = 作成時に差出人を表示
settings-show-from-help = 差出人フィールドを表示した状態で開始します。リボンから、1 通のメッセージに限って表示することもできます。承認された送信アドレスのみ使用できます。
settings-sessions-table = アクティブなセッションの表
settings-folders-table = フォルダー一覧
settings-kbd-reference = プリセットの割り当てを表示
settings-kbd-reference-help = プリセットの参照用です。カスタム割り当てや管理者のポリシーにより、実際に有効なショートカットが変わる場合があります。
settings-sessions-client-details = クライアントの詳細
settings-2fa-passkey-rules = 組織では、次の条件を満たすパスキーのみを受け付けます:
settings-2fa-passkey-rule-platform = このデバイスに保存されている(Windows Hello、Touch ID、Android の画面ロック)
settings-2fa-passkey-rule-cross-platform = セキュリティキーまたはスマートフォンにある
settings-2fa-passkey-rule-user-verification = PIN、指紋、顔を確認する
settings-2fa-passkey-rule-resident-key = 認証器に保存されている(検出可能)
settings-2fa-passkey-gap-allowed = 現在のポリシーより前に追加されたもので、ポリシーを満たしていません。引き続き使用できます。新しいパスキーを追加して、これを削除してください。
settings-2fa-passkey-gap-refused = 現在のポリシーを満たしておらず、ログインには使用できなくなりました。新しいパスキーを追加して、これを削除してください。
settings-2fa-passkey-browser-refused-policy = ブラウザーが、組織のパスキーポリシーを満たす認証器を見つけられなかったか、リクエストがキャンセルされました。
settings-2fa-policy-user-verification = 組織では、PIN、指紋、顔を確認するパスキーが必要です。この認証器では本人確認が行われませんでした。
settings-2fa-policy-platform = 組織では、このデバイスに保存されたパスキー(Windows Hello、Touch ID、Android の画面ロックなど)のみを受け付けます。セキュリティキーやスマートフォンは使用できません。
settings-2fa-policy-cross-platform = 組織では、セキュリティキーまたはスマートフォンのパスキーのみを受け付けます。このデバイスに保存されたパスキーは使用できません。
settings-2fa-policy-attachment-unknown = ブラウザーが認証器の種類を通知しなかったため、組織のパスキーポリシーと照合できません。別のブラウザーをお試しください。
settings-2fa-policy-resident-key = 組織では、認証器に保存されるパスキーが必要です。この認証器には保存されませんでした。
settings-2fa-policy-legacy = このパスキーは、組織の現在のパスキーポリシーより前に追加されたもので、要件を満たさなくなりました。別の要素またはリカバリーコードでログインし、新しいパスキーを追加してください。
settings-2fa-policy-last-factor-blocked = 組織では、最後の第 2 要素を削除することは許可されていません。先に別のログイン方法を追加してください。
settings-2fa-policy-last-factor-required = 組織では、2 要素認証が必須です。これを削除する前に、別のログイン方法を追加してください。
settings-2fa-last-factor-hint = 先に別のログイン方法を追加してください。
settings-2fa-last-factor-tooltip = 先に別のログイン方法を追加してください。組織では、アカウントに少なくとも 1 つのパスキーまたは認証アプリが必要です。
settings-2fa-challenge-passkey-not-used = パスキーは使用されませんでした。リクエストがキャンセルまたはタイムアウトしたか、この認証器が組織の要件(PIN、指紋、顔の確認など)を満たせません。もう一度お試しいただくか、別の方法を使用してください。
settings-folders-hidden-help = 非表示のフォルダーとそのサブフォルダーは、サイドバーに表示されません。削除やアクセス制限はされません。サブフォルダーをここで表示すると、非表示の親フォルダーも表示されます。受信トレイは常に利用できます。
settings-folders-collapse-user = サイドバーでユーザーフォルダーを折りたたむ
settings-folders-show = { $name }をサイドバーに表示
settings-folders-hide = { $name }をサイドバーから非表示
settings-folders-show-short = 表示
settings-folders-hide-short = 非表示
settings-folders-search = フォルダーを検索
settings-folders-select = フォルダーを選択: { $name }
settings-folders-selected = 選択したフォルダー: { $count }
settings-folders-select-visible = 表示中のものを選択
settings-folders-select-all = すべてのフォルダーを選択
settings-folders-clear = 選択を解除
settings-folders-pin-selected = 選択したものを固定
settings-folders-unpin-selected = 選択したものの固定を解除
settings-folders-show-selected = 選択したものを表示
settings-folders-hide-selected = 選択したものを非表示
settings-folders-visible = 表示中
settings-folders-hidden = 非表示
settings-folders-pinned = 固定済み
settings-folders-location = 場所
settings-folders-counts = 未読 { $unread } / 合計 { $total }
settings-folders-more = { $name }のその他の操作
settings-folders-empty = 検索に一致するフォルダーがありません。
settings-folders-root = 最上位
settings-folders-order-help = フォルダーをドラッグして、サイドバー内の順序を変更します。固定したフォルダーはまとまって表示されます。
cache-title = メールのキャッシュとプリロード
cache-help = 最近開いたフォルダーとメッセージを、このタブのメモリに保持します。キャッシュされた内容はバックグラウンドで更新されます。タブを閉じるかログアウトすると消去されます。プリロードでメールが既読になったり、リモート画像が読み込まれたりすることはありません。
cache-enabled = セッションキャッシュを有効にする
cache-mailboxes = フォルダーのメッセージ一覧をキャッシュ
cache-messages = 開いたメッセージをキャッシュ
cache-freshSeconds = 鮮度の間隔(秒)
cache-maxAgeSeconds = キャッシュの最大保持時間(秒)
cache-maxEntries = キャッシュエントリの最大数
cache-maxBytes = メモリの上限(バイト)
cache-prefetchEnabled = 先読みプリロードを有効にする
cache-hoverEnabled = フォルダーにポインターを合わせたときにプリロード
cache-focusEnabled = フォルダーにキーボードフォーカスが移ったときにプリロード
cache-dwellMs = ポインターとフォーカスの待ち時間(ミリ秒)
cache-idleEnabled = アイドル時にフォルダーをプリロード
cache-idleDelayMs = アイドルまでの待ち時間(ミリ秒)
cache-maxConcurrency = 同時プリロードの最大数
cache-maxQueued = 待機中プリロードの最大数
cache-preloadMessages = メッセージ本文の先読みを許可
cache-excludedFolders = プリロードから除外するフォルダー
cache-exclusions-help = 1 行に 1 つのルール: role:trash、role:junk、role:inbox、role:drafts、role:sent、role:archive、または id: に続けて正確なフォルダー ID。
cache-refreshOnOpen = 開いたときにキャッシュされた内容を更新

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
settings-badge-auth = 認証 (DMARC)

settings-badge-auth-help = DMARC の結果。セキュリティの詳細がすべて表示されます。

settings-badge-density = バッジのサイズ

settings-badge-density-comfortable = 快適

settings-badge-density-compact = コンパクト

settings-badge-density-help = すべてのバッジのコンパクトまたは快適な高さと間隔。

settings-badge-folder = フォルダー

settings-badge-folder-help = メッセージが保存される場所。重要な場合にのみ、ジャンクとゴミ箱に表示されます。

settings-badge-labels = バッジラベル

settings-badge-labels-help = テキスト付きのアイコン、またはアイコンのみ (全文はツールヒントとスクリーン リーダーに表示されます)。

settings-badge-labels-icon = アイコンのみ

settings-badge-labels-text = アイコンとテキスト

settings-badge-list = メーリングリスト

settings-badge-list-help = メッセージの送信元のメーリング リスト。サブスクリプションの詳細とアクションが開きます。購読を解除した後もメールが届き続ける場合にのみ表示されます。

settings-badge-mode-always = いつも

settings-badge-mode-never = 決してしない

settings-badge-mode-notable = 目立った時のみ

settings-badge-sanitized = サニタイズ済み HTML

settings-badge-sanitized-help = HTML サニタイザーがメッセージに対して行ったこと。何かがブロックされたときにそれが表示されるのは、注目に値する場合のみです。

settings-badge-spam = スパムスコア

settings-badge-spam-help = 受信サーバーがメッセージ ヘッダーに書き込んだスコア。

settings-folders-hide-empty = サイドバーの空のフォルダーを非表示にする

settings-folders-hide-empty-help = フォルダーにメッセージが含まれている場合、フォルダーが再表示されます。受信トレイ、検索フォルダー、数が不明なフォルダー、および表示されているフォルダーの親は表示されたままになります。 [プロパティ]、[表示] で個々のフォルダーをオーバーライドします。すべてのフォルダーがこのリストに残ります。

settings-folders-list-card = フォルダーのリストと順序

settings-header-auto-collapse = 書き込み後にヘッダーを折ります(秒)

settings-header-auto-collapse-help = 本文にこの秒数フォーカスがあり、ヘッダーに受信者と件名がある場合、本文は概要行に折りたたまれます。 0 はこれをオフにします。

settings-message-badges = メッセージバッジ

settings-message-badges-help = 開いているメッセージのヘッダーに表示するバッジを選択します。優先マークはフォルダーとサイドバーの下に設定されます。バッジを右クリックして非表示にすることもできます。

settings-reply-from-headers = ID を選択するヘッダー

settings-reply-from-headers-help = 1 行に 1 つのヘッダー名が次の順序で試行されます: Delivered-To、X-Original-To、Envelope-To、X-Forwarded-To、Resent-To、To、Cc。省略されたヘッダーは読み込まれません。

settings-reply-from-headers-invalid = 次の名前のみを 1 行に 1 つ使用します: Delivered-To、X-Original-To、Envelope-To、X-Forwarded-To、Resent-To、To、Cc。

settings-reply-from-headers-save = ヘッダーの順序を保存する

settings-sessions-linked-created = リンクが作成されました

settings-sessions-linked-details = リンクの詳細

settings-sessions-linked-kind = リンクタイプ

settings-sessions-linked-kind-linked = リンクされたメールボックス

settings-sessions-linked-last-used = 最後に使用した

settings-sessions-linked-line = リンクされたメールボックスへのアクセス · { $origin } から

settings-sessions-linked-line-hidden = リンクされたメールボックスへのアクセス

settings-sessions-linked-origin = 由来

settings-sessions-linked-removed = 不明 (アカウントが削除されました)

settings-sessions-linked-revoke = リンクへのアクセスを取り消す

settings-sessions-linked-revoke-label = { $mailbox } のリンク アクセスを取り消します

settings-sessions-linked-target = メールボックスが開かれました

settings-sessions-origin-full = 完全な住所

settings-sessions-origin-help = このテーブルが、リンクされたメールボックス セッションの元のアカウントにどのような名前を付けるかを制御します。あなた自身のセッションで、あなただけがそれを見ることができます。

settings-sessions-origin-hidden = 隠された

settings-sessions-origin-masked-local-part = 最初の部分をマスクしたアドレス

settings-sessions-origin-setting = リンクされたメールボックスへのアクセス元を表示する

settings-shortcuts-search = 検索ショートカット

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
cache-help-durable = 最近アクセスしたフォルダーとメッセージをいつでも開けるようにしておきます。デフォルトでは、コピーはこのタブのメモリに残ります。無期限のストレージを有効にして、リロード後も暗号化されたコピーを保持します。プリロードでは、メールに既読のマークが付けられたり、リモート画像が読み込まれたりすることはありません。

cache-keepIndefinitely = キャッシュされたメールを無期限に保持する

cache-keepIndefinitely-help = デフォルトではオフです。リロード後も有効期限なしで暗号化されたコピーをこのブラウザに保持します。オンラインメッセージは引き続き更新されます。エントリ制限とバイト制限は引き続き適用されます。これをオフにするか、キャッシュされたメールをクリアすると、保存されているコピーが削除されます。ブラウザのデータを消去することでも削除できます。添付ファイルはオフラインで使用するためにダウンロードされません。
