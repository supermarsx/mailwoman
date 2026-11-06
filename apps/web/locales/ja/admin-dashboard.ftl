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
admin-dash-h-migrations = マイグレーション
admin-dash-h-engine = メール同期
admin-dash-h-outbox = 送信トレイ
admin-dash-h-index = 検索インデックス
admin-dash-h-sandbox = レンダリングサンドボックス
admin-dash-h-plugins = プラグインランタイム

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = ストレージ
admin-dash-db-size = データベースファイル
admin-dash-wal-size = 先行書き込みログ
admin-dash-index-size = 検索インデックス
admin-dash-volume-free = データボリュームの空き容量
admin-dash-volume-of = / { $total }
admin-dash-na-wal = このデータベースは先行書き込みログファイルを使用していないか、サーバーから報告されていません。
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

# -- Machine translation (2026-11): added to match en ---------------------
admin-dash-warn-geoip-missing = 国のポリシーが有効ですが、使用可能な国データベースが読み込まれていないため、対象のサインインは拒否されます。
admin-dash-warn-geoip-stale = 国のポリシーが有効ですが、国データベースが古くなっています。
admin-dash-h-status-ok = 正常
admin-dash-h-status-warn = 警告
admin-dash-h-status-fail = 失敗
admin-dash-h-status-disabled = 無効
admin-dash-h-checked = 確認日時: { $time }
admin-dash-h-refresh = 再確認
admin-dash-h-refresh-failed = ヘルスチェックを再実行できませんでした(ステータス { $status })。前回の結果を表示しています。
admin-dash-h-refresh-network = ヘルスチェックを再実行できませんでした。サーバーが応答しませんでした。前回の結果を表示しています。
admin-dash-h-enable = 有効にするには
admin-dash-h-probe-time = プローブ所要時間: { $time }
admin-dash-h-outbox-probe-time = ステータスの集計に { $time } かかりました。送信ハンドシェイクの時間は別に表示されます。
admin-dash-h-history = 直近 { $n } 回のチェック
admin-dash-h-open-diagnostics = 診断
admin-dash-h-open-logs = ログ
admin-dash-h-open-plugins = プラグイン
admin-dash-h-open-maintenance = メンテナンス
admin-dash-h-not-tracked = 追跡対象外
admin-dash-h-unlimited = 制限なし
admin-dash-h-m-none = なし
admin-dash-h-m-ago = { $age } 前
admin-dash-h-m-ping = 往復時間
admin-dash-h-m-pool = 接続プール
admin-dash-h-m-pool-value = 使用中 { $inUse }、アイドル { $idle }(オープン { $size })
admin-dash-h-m-engine = データベースサーバー
admin-dash-h-m-location = 場所
admin-dash-h-m-db-size = データベースサイズ
admin-dash-h-m-wal-size = 先行書き込みログ
admin-dash-h-m-journal = ジャーナルモード
admin-dash-h-m-slow = 低速クエリ(直近 1 時間)
admin-dash-h-m-applied = 適用済み
admin-dash-h-m-applied-value = このビルドの { $expected } 件中 { $applied } 件
admin-dash-h-m-latest = 最新の適用済み
admin-dash-h-m-pending = 未適用
admin-dash-h-m-failed = 開始したが未完了
admin-dash-h-m-unknown = より新しいビルドのもの
admin-dash-h-m-accounts = メールボックス
admin-dash-h-m-accounts-value = 登録 { $total } 件: 同期中 { $syncing }、アイドル { $idle }、失敗 { $erroring }
admin-dash-h-m-last-sync = 最後に完了した同期以降
admin-dash-h-m-last-sync-value = 中央値 { $p50 }、最長 { $max }
admin-dash-h-m-never-synced = サーバー起動後に同期されていません
admin-dash-h-m-queue = 実行中の同期
admin-dash-h-m-errors-hour = 同期エラー(直近 1 時間)
admin-dash-h-m-by-kind = バックエンド別の接続
admin-dash-h-m-kind-value = { $kind }: メールボックス { $accounts } 件、監視接続 { $watch } 件
admin-dash-h-m-recent-errors = 最近のエラー
admin-dash-h-m-queued = 送信待ち
admin-dash-h-m-queued-value = 送信予定 { $queued }、再試行中 { $deferred }、送信中 { $sending }
admin-dash-h-m-oldest = 最も古い待機
admin-dash-h-m-scheduled = 予約または保留
admin-dash-h-m-scheduled-value = 予約 { $scheduled }、保留 { $held }
admin-dash-h-m-failed-total = 失敗(送信トレイに残っている)
admin-dash-h-m-failed-total-value = 失敗 { $failed }、配信不明 { $uncertain }
admin-dash-h-m-sends-hour = 送信(直近 1 時間)
admin-dash-h-m-sends-hour-value = 送信済み { $sent }、延期 { $deferred }、失敗 { $failed }
admin-dash-h-m-last-send = 最後の送信
admin-dash-h-m-last-send-value = { $outcome }、{ $age } 前
admin-dash-h-m-last-send-none = サーバー起動後の送信はありません
admin-dash-h-outcome-sent = 送信済み
admin-dash-h-outcome-deferred = 延期
admin-dash-h-outcome-failed = 失敗
admin-dash-h-outcome-uncertain = 配信不明
admin-dash-h-m-submission = 送信サーバー
admin-dash-h-m-submission-value = { $checked } 件中 { $reachable } 件が応答(ハンドシェイクのみ、サインインなし、最も遅いもの { $slowest })、{ $age } 前に確認
admin-dash-h-m-submission-none = 送信サーバーは設定されていません。
admin-dash-h-m-submission-pending = 接続を確認中です。
admin-dash-h-m-submission-unmeasured = { $count } 台のサーバーは認証付きのプロバイダー確認が必要です。認証情報は送信されませんでした。
admin-dash-h-m-documents = ドキュメント
admin-dash-h-m-documents-value = 保存済みメッセージ { $messages } 件中 { $documents } 件
admin-dash-h-m-index-pending = まだインデックスされていません
admin-dash-h-m-index-size = ディスク上のサイズ
admin-dash-h-m-index-memory = メモリ上
admin-dash-h-m-last-commit = 最後のコミット
admin-dash-h-m-rebuild = 再構築
admin-dash-h-m-rebuild-running = 実行中: { $pct }({ $total } 件中 { $done } 件)
admin-dash-h-m-rebuild-idle = 実行されていません
admin-dash-h-m-version = ワーカーのバージョン
admin-dash-h-m-worker-missing = ワーカーのバイナリがありません
admin-dash-h-m-render-ping = Ping(起動、jail、応答)
admin-dash-h-m-render-jobs = 現在実行中のジョブ
admin-dash-h-m-render-failures = 問題(直近 1 時間)
admin-dash-h-m-render-failures-value = 失敗 { $failed }、タイムアウト { $timeouts }、ビジー時の拒否 { $busy }
admin-dash-h-m-jail = カーネル jail
admin-dash-h-m-jail-required = 必須
admin-dash-h-m-jail-optional = このプラットフォームでは不要
admin-dash-h-m-jail-degraded = 縮退: { $why }
admin-dash-h-m-plugins = プラグイン
admin-dash-h-m-plugins-value = インストール済み { $installed }、有効 { $enabled }、読み込み済み { $loaded }
admin-dash-h-m-plugin-list = 有効なプラグイン
admin-dash-h-m-plugin-loaded = 読み込み済み
admin-dash-h-m-plugin-not-loaded = 未読み込み({ $reason })
admin-dash-h-m-plugin-limits = メモリ { $memory }、期限 { $deadline }、燃料 { $fuel }
admin-dash-h-m-plugin-calls = 呼び出しの失敗(直近 1 時間)
admin-dash-h-m-plugin-calls-value = トラップ { $traps }、制限超過 { $limits }、その他のエラー { $errors }
admin-dash-h-m-host-ping = プラグインホストのロック
admin-dash-h-t-ping = { $warn } を超えると警告、{ $fail } を超えると失敗
admin-dash-h-t-oldest = メッセージの待機が { $warn } を超えると警告、{ $fail } を超えると失敗
admin-dash-h-t-pending = 未インデックスのメッセージが { $n } 件を超えると警告
admin-dash-h-t-render = { $warn } を超えると警告。{ $timeout } 以内に応答がない場合は失敗
admin-dash-h-t-sync = メールボックスの { $pct } 以上で同期に失敗すると失敗
admin-dash-h-chart-db = データベースの往復時間
admin-dash-h-chart-render = レンダーワーカーの Ping
admin-dash-h-chart-outbox = 送信待ちのメッセージ
admin-dash-h-chart-search = 未インデックスのメッセージ
admin-dash-na-index-proxy = プロキシモード: このサーバーは検索インデックスを保持しません。
admin-dash-na-index-memory = 検索インデックスはメモリ上にあるため、ディスク上のサイズはありません。
admin-dash-customize = ダッシュボードをカスタマイズ
admin-dash-limitations = データの利用可否
