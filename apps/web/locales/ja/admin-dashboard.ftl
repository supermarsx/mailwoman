# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-dashboard. Source: ../en/admin-dashboard.ftl

admin-dash-refresh = 更新
admin-dash-unavailable = ダッシュボードの数値はまだ利用できません。
admin-dash-unavailable-detail = このサーバーはダッシュボードのデータを提供していません（GET /admin/dashboard が 404 を返します）。今後のサーバー更新で提供されます。他のセクションはこれまでどおり動作します。
admin-dash-error-network = ダッシュボードを読み込めませんでした: サーバーが応答しませんでした。もう一度お試しください。
admin-dash-error-status = ダッシュボードを読み込めませんでした: サーバーが { $status } を返しました。もう一度お試しください。繰り返し発生する場合は、サーバーログを確認してください。

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = 要対応
admin-dash-warn-fix = 対処するセクションを開く
admin-dash-warn-blocked-accounts = { $count ->
   *[other] ドメインに設定がないため、{ $count } 件のアカウントがサインインできません。
}
admin-dash-warn-no-trusted-proxy = IP ルールが有効ですが、信頼済みプロキシが設定されていないため、すべてのクライアントがプロキシのアドレスから接続しているように見える可能性があります。
admin-dash-warn-allowlist-bypass = 環境変数によって IP 許可リストが無効になっています。
admin-dash-warn-metrics-token = メトリクスエンドポイントにトークンが設定されていません。
admin-dash-warn-pending-migrations = { $count ->
   *[other] { $count } 件のデータベースマイグレーションが適用されていません。サーバーを再起動して適用してください。
}
admin-dash-warn-cert-expiring = { $count ->
   *[other] { $count } 件のアップストリーム証明書がまもなく期限切れになります（前回の接続テストより）。
}
admin-dash-warn-unknown = サーバー警告 { $code }（{ $count }）

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
   *[other] { $count } 件のアカウントがブロック中: ドメイン設定なし
}
admin-dash-blocked-note = これらのアカウントはこの更新より前にサインインしていましたが、ドメインにコンソール上の設定がないため、現在はサインインできません。ドメインを設定すると、再びサインインできるようになります。
admin-dash-blocked-chart = ドメイン別のブロック中アカウント
admin-dash-blocked-row = { $count ->
   *[other] { $count } 件のアカウント
}
admin-dash-blocked-seen = （最後に使用したメールホスト { $host }）
admin-dash-configure-domain = { $domain } を設定
admin-dash-blocked-no-access = あなたのロールではドメインを設定できません。

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] セットアップチェックリスト: 未完了の項目なし
   *[other] セットアップチェックリスト: 未完了の項目 { $open } 件
}
admin-dash-check-done = 完了
admin-dash-check-open = 未完了
admin-dash-check-optional = （任意）
admin-dash-check-go = 開く
admin-dash-check-hide = 非表示
admin-dash-check-unhide = 再表示
admin-dash-check-hidden = { $count ->
   *[other] { $count } 件の項目が非表示です。非表示にしても完了にはなりません。
}
admin-dash-check-show-hidden = 非表示の項目を表示
admin-dash-check-hide-hidden = 非表示の項目を除外
admin-dash-check-domain-exists = メールドメインを追加する
admin-dash-check-domain-upstream = ドメインにメールサーバーを指定するか、既定のアップストリームを設定する
admin-dash-check-test-connection = 接続テストを成功させる（過去 7 日以内）
admin-dash-check-preferred-domain = ユーザーが入力しなくてもサインインできるよう、優先ドメインを選ぶ
admin-dash-check-second-admin = 2 つ目の管理者アカウントを作成するか、環境変数によるサインインをオフにする
admin-dash-check-trusted-proxies = このサーバーの前段にある信頼済みプロキシを設定する
admin-dash-check-security-policy = セキュリティポリシーを保存する
admin-dash-check-metrics-token = メトリクスエンドポイントのトークンを設定する
admin-dash-check-branding = ブランディングを確認する

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = アカウントとセッション
admin-dash-users = 作成済みユーザー
admin-dash-signed-in-24h = サインインしたアカウント（過去 24 時間）
admin-dash-signed-in-7d = サインインしたアカウント（過去 7 日間）
admin-dash-domains = ドメイン
admin-dash-domains-detail = メールサーバーあり { $upstream }、無効 { $disabled }
admin-dash-sessions = 有効なウェブメールセッション
admin-dash-admin-sessions = 有効な管理者セッション
admin-dash-security = サインインのセキュリティ
admin-dash-failed-webmail = ウェブメールのサインイン失敗（過去 24 時間）
admin-dash-failed-admin = 管理者のサインイン失敗（過去 24 時間）
admin-dash-active-blocks = ブロック中のアドレス
admin-dash-allowlists = IP 許可リスト
admin-dash-allowlists-value = 管理 { $admin }、ウェブメール { $webmail }
admin-dash-on = オン
admin-dash-off = オフ
admin-dash-trusted-proxies = 信頼済みプロキシ
admin-dash-configured = 設定済み
admin-dash-not-configured = 未設定
admin-dash-untrusted-forwarded = { $count ->
   *[other] 24 時間で { $count } 件のリクエストに、信頼されていない接続元からの転送ヘッダーが含まれていました
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = トラフィック（過去 { $mins } 分）
admin-dash-traffic-note = このサーバープロセスが起動してから集計した値です。他のプロセスやレプリカは含まれません。
admin-dash-traffic-empty = この期間に集計されたリクエストはありません。
admin-dash-requests = 1 分あたりのリクエスト数
admin-dash-errors = 1 分あたりのサーバーエラー（5xx）
admin-dash-p95 = 応答時間（95 パーセンタイル）

# -- Health ----------------------------------------------------------------------------
admin-dash-health = 稼働状態
admin-dash-h-ok = 正常
admin-dash-h-attention = 要対応
admin-dash-h-db = データベース
admin-dash-h-db-ok = { $ms } で応答
admin-dash-h-migrations = マイグレーション
admin-dash-h-migrations-ok = 最新（{ $n } 件適用済み）
admin-dash-h-migrations-behind = { $expected } 件中 { $applied } 件適用済み
admin-dash-h-engine = メール同期
admin-dash-h-engine-value = 登録アカウント { $accounts } 件、同期エラー { $errors } 件
admin-dash-h-outbox = 送信トレイ
admin-dash-h-outbox-value = 待機中 { $pending } 件（最古 { $oldest }）、24 時間の失敗 { $failed } 件
admin-dash-h-outbox-none = なし
admin-dash-h-index = 検索インデックス
admin-dash-h-index-value = { $n } 件のドキュメント
admin-dash-h-index-rebuilding = 再構築中（現在 { $n } 件のドキュメント）
admin-dash-h-sandbox = レンダリングサンドボックス
admin-dash-h-sandbox-ok = 利用可能: 信頼されていないドキュメントは別プロセスでレンダリングされます。
admin-dash-h-sandbox-missing = なし: 信頼されていないドキュメントはサーバープロセス内でレンダリングされます。
admin-dash-h-sandbox-required = なし（このサーバーでは必須）: 信頼されていないドキュメントのレンダリングは失敗します。
admin-dash-h-plugins = プラグインランタイム
admin-dash-h-plugins-ok = 利用可能
admin-dash-h-plugins-off = オフ
admin-dash-na-proxy = このサーバーはプロキシモードで動作しており、メールの同期を自身では行いません。
admin-dash-na-unreported = サーバーから報告されていません。

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = ストレージ
admin-dash-db-size = データベースファイル
admin-dash-wal-size = 先行書き込みログ
admin-dash-index-size = 検索インデックス
admin-dash-volume-free = データボリュームの空き容量
admin-dash-volume-of = / { $total }
admin-dash-na-wal = このデータベースは先行書き込みログファイルを使用していないか、サーバーから報告されていません。
admin-dash-na-index = このサーバーには検索インデックスがありません。
admin-dash-na-volume = このプラットフォームではサーバーが空き容量を読み取れません。

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = 最近の管理操作
admin-dash-audit-empty = 記録された管理操作はまだありません。
admin-dash-audit-when = 日時
admin-dash-audit-actor = 実行者
admin-dash-audit-action = 操作
admin-dash-audit-target = 対象
admin-dash-audit-all = 監査ログ全体を開く
admin-dash-server = サーバー
admin-dash-version = バージョン
admin-dash-build = ビルド
admin-dash-mode = モード
admin-dash-mode-engine = エンジン: このサーバーがメールを同期・保存します
admin-dash-mode-proxy = プロキシ: このサーバーはリクエストをメールサーバーに中継します
admin-dash-uptime = 稼働時間
admin-dash-base-path = 公開パス
admin-dash-base-root = サイトのルート
admin-dash-duration-dh = { $d } 日 { $h } 時間
admin-dash-duration-hm = { $h } 時間 { $m } 分
admin-dash-duration-m = { $m } 分
admin-dash-duration-s = { $s } 秒
