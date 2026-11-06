# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: nextcloud. Source: ../en/nextcloud.ftl

# -- File picker -------------------------------------------------------------
nextcloud-up = 上へ
nextcloud-loading = 読み込んでいます…
nextcloud-list-error = このフォルダーの内容を表示できませんでした。
nextcloud-empty = このフォルダーは空です。
nextcloud-file-list = Nextcloud のファイル
nextcloud-open-folder = フォルダー { $name } を開く
nextcloud-select-file = { $name } を選択

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Nextcloud から添付
nextcloud-attach-action = { $count ->
    [0] 添付
   *[other] { $count } 件のファイルを添付
  }
nextcloud-error-select-file = 添付するファイルを 1 つ以上選択してください
nextcloud-error-attach-failed = 選択したファイルを添付できませんでした

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Nextcloud に保存
nextcloud-save-title = 「{ $name }」を Nextcloud に保存
nextcloud-save-action = ここに保存 ({ $dir })
nextcloud-saved = { $path } に保存しました
nextcloud-error-save-failed = Nextcloud に保存できませんでした

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = 共有リンクを作成
nextcloud-share-title = 共有リンク
nextcloud-share-intro = ファイルを添付する代わりに、{ $path } への公開リンクを作成します。
nextcloud-protect-password = パスワードで保護
nextcloud-password-label = パスワード
nextcloud-share-password = 共有パスワード
nextcloud-set-expiry = 有効期限を設定
nextcloud-expires-on = 有効期限
nextcloud-expiry-date = 有効期限日
nextcloud-create-link = リンクを作成
nextcloud-password-protected = パスワード保護あり
nextcloud-no-password = パスワードなし
nextcloud-expires = 有効期限 { $date }
nextcloud-no-expiry = 有効期限なし
nextcloud-error-need-password = パスワードを入力するか、パスワード保護をオフにしてください
nextcloud-error-need-expiry = 有効期限日を選ぶか、有効期限をオフにしてください
nextcloud-error-share-failed = 共有リンクを作成できませんでした

# -- Machine translation (2026-11): added to match en ---------------------
nextcloud-share-upload-intro = { $path } は添付ファイルの上限を超えています。Nextcloud の「Mail attachments」フォルダーにアップロードし、代わりに共有リンクをメッセージに挿入してください。
nextcloud-upload-and-link = アップロードしてリンクを挿入
nextcloud-share-not-now = 今はしない
nextcloud-upload-progress = アップロード中：{ $percent }%
nextcloud-upload-progress-label = アップロードの進行状況
nextcloud-share-inserted = { $name } の共有リンクをメッセージに挿入しました。
nextcloud-share-body = { $name }（{ $size }）：{ $url }
nextcloud-share-body-expires = このリンクは { $date } まで有効です。
nextcloud-share-body-password = このリンクはパスワードで保護されています。
nextcloud-account-title = Nextcloud アカウント
nextcloud-account-help = 管理者が承認したインスタンスに、自分のアカウントを連携します。認証情報は、このメールボックスアカウント専用に暗号化して保存されます。
nextcloud-account-load-error = Nextcloud の連携を読み込めませんでした。再読み込みしてもう一度お試しください。
nextcloud-account-unlink-error = 連携を解除できませんでした。再読み込みしてからもう一度お試しください。
nextcloud-account-reload = 再読み込み
nextcloud-account-no-instances = 利用できる有効な Nextcloud インスタンスがありません。管理者に設定を依頼してください。
nextcloud-account-ready = 連携済み・利用可能
nextcloud-account-stale = インスタンスの設定が変更されました。連携をもう一度確認してください。
nextcloud-account-unlinked = 未連携
nextcloud-account-relink = 連携を再確認
nextcloud-account-link = アカウントを連携
nextcloud-account-unlink = アカウントの連携を解除
nextcloud-account-password-help = Nextcloud の個人用セキュリティ設定でアプリパスワードを作成し、そのユーザー名とパスワードをここにコピーしてください。確認では範囲を限定したリクエストを1回送信し、拒否された認証情報は再試行されません。
nextcloud-account-username = Nextcloud アプリパスワードのユーザー名
nextcloud-account-password = Nextcloud アプリパスワード
nextcloud-account-verify = 確認して連携
nextcloud-account-verify-error = 確認または保存に失敗しました。インスタンス、アプリパスワードのユーザー名、パスワードを確認してください。パスワード欄は消去されました。すでに動作している連携は置き換えられていません。
nextcloud-admin-title = Nextcloud インスタンス
nextcloud-admin-help = メールボックスユーザーが個別に連携できる HTTPS インスタンスを承認します。共有のサービスパスワードがメールボックスアカウントに渡されることはありません。ファイル操作は 8 MiB までに制限されます。ネイティブエンジンのユーザーは、連携後にカレンダー、タスク、アドレス帳のコレクションを明示的に検出して接続します。
nextcloud-admin-add = Nextcloud インスタンスを追加
nextcloud-admin-edit = Nextcloud インスタンスを編集
nextcloud-admin-enabled = 有効
nextcloud-admin-disabled = 無効
nextcloud-admin-missing = このインスタンスは存在しません。インスタンス一覧に戻ってください。
nextcloud-admin-private-help = プライベートアドレスには、Nextcloud への明示的な外向き通信の許可が必要です。設定済みのプロキシ経路が優先され、プロキシに拒否されたリクエストが直接接続に切り替わることはありません。
nextcloud-admin-revalidation = インスタンスの変更を保存すると、ユーザーは連携を再確認する必要があります。インスタンスを無効にすると、新しい操作は直ちに停止します。
nextcloud-admin-authority = インスタンスの権限
nextcloud-admin-name = 表示名
nextcloud-admin-name-help = ユーザーがインスタンスを選ぶときに表示される短い名前です。
nextcloud-admin-url = HTTPS のインストール URL
nextcloud-admin-url-help = https://cloud.example.org/ や https://example.org/nextcloud/ のように、インストールのルートを指定してください。証明書は信頼されたもので、ホスト名と一致している必要があります。認証情報、クエリパラメーター、フラグメントは受け付けません。
nextcloud-admin-enable = ユーザーがこのインスタンスを連携することを許可
nextcloud-admin-ca = 追加の認証局（PEM）
nextcloud-admin-ca-help = この Nextcloud サーバー用の任意のプライベート CA 証明書です。公的な信頼も引き続き利用でき、ホスト名の検証も必須のままです。変更するとユーザーは連携を再確認する必要があります。プロキシ証明書の信頼は別です。
nextcloud-instance-select = Nextcloud インスタンス
nextcloud-instance-placeholder = 連携済みのインスタンスを選択
nextcloud-instances-empty = 確認済みの Nextcloud 連携がありません。「ユーザー設定 → Nextcloud」で連携を追加または確認してください。
nextcloud-instances-error = 連携済みのインスタンスを読み込めませんでした。選択画面を開き直してもう一度お試しください。
nextcloud-save-warning = この添付ファイルをメールボックスの外に出す前に、管理者による確認が必要です。
nextcloud-save-acknowledge = 添付ファイルに関する警告を理解した上で、このファイルを選択した Nextcloud インスタンスに保存します。
nextcloud-save-attachment = { $name } を Nextcloud に保存
nextcloud-save-close = Nextcloud の保存先を閉じる
nextcloud-save-options = 添付ファイルを Nextcloud に保存
nextcloud-pim-title = 連絡先とカレンダー
nextcloud-pim-help = 確認済みのこの Nextcloud アカウントからコレクションを検出し、必要なものを明示的に接続します。同期すると最新のデータを取得します。接続済みのコレクションの編集はリモートのコレクションに対して行われ、連携や権限が取り消されると失敗します。
nextcloud-pim-category = コレクションのカテゴリ
nextcloud-pim-calendar = カレンダー
nextcloud-pim-contacts = アドレス帳
nextcloud-pim-tasks = タスクリスト
nextcloud-pim-discover = コレクションを検出
nextcloud-pim-discovered = 利用可能なコレクション
nextcloud-pim-empty = 一致するコレクションは返されませんでした。
nextcloud-pim-connect = 接続
nextcloud-pim-connected = 接続済みのコレクション
nextcloud-pim-sync = 今すぐ同期
nextcloud-pim-sync-needed = コレクションを接続しました。内容を読み込むには今すぐ同期してください。
nextcloud-pim-synced = 同期が完了しました。
nextcloud-pim-error = 操作に失敗しました。アカウントの連携と権限を確認し、明示的にもう一度お試しください。既存のキャッシュデータは破棄されていません。
nextcloud-pim-native-required = 連絡先とカレンダーの同期にはネイティブメールエンジンが必要です。ファイルへのアクセスは、アカウントの連携を通じて引き続き利用できます。
nextcloud-pim-disconnect = 接続を解除
nextcloud-pim-disconnect-help = 接続を解除してもキャッシュデータは保持され、リモートへの書き込みは停止します。再連携したアカウントでは、再度検出と明示的な接続が必要です。
nextcloud-pim-ready = 準備完了
nextcloud-pim-disconnected = 接続解除済み、または連携が変更されました
nextcloud-pim-unnamed = 名前のないコレクション
nextcloud-pim-back = Nextcloud アカウントに戻る
nextcloud-pim-create-title = Nextcloud に新しいコレクションを作成
nextcloud-pim-create-help = この Nextcloud アカウントに空のカレンダーまたはタスクリストを作成し、ここに接続します。
nextcloud-pim-create-name = 新しいコレクションの名前
nextcloud-pim-create = Nextcloud に作成
nextcloud-pim-created = { $name } を Nextcloud に作成し、接続しました。読み込むには今すぐ同期してください。
nextcloud-pim-interval = 自動更新の間隔（秒）
nextcloud-pim-interval-help = 0 の場合は手動更新のみです。正の値は最短 60 秒です。接続済みのアカウントはバックグラウンドで更新され、失敗時は間隔を延ばし、認証情報が拒否された場合は再連携するまで以降のリクエストを停止します。
nextcloud-pim-interval-reset = 管理者の既定値を使う
nextcloud-admin-empty = 承認済みのインスタンスはまだありません。アカウントの連携に使えるよう、インスタンスを追加してください。
nextcloud-admin-status = ステータス
nextcloud-admin-open = { $name } を開く
