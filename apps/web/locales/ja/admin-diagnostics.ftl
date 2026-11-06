# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-diagnostics. Source: ../en/admin-diagnostics.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-diagx-tabs = 診断ページ
admin-diagx-tab-overview = 概要
admin-diagx-tab-server = サーバー
admin-diagx-tab-connectivity = 接続性
admin-diagx-tab-backends = メールバックエンド
admin-diagx-tab-posture = セキュリティ状況
admin-diagx-tab-tools = ツール
admin-diagx-tab-history = 履歴
admin-diagx-tab-report = レポート
admin-diagx-tab-server-intro = このサーバー自身の各部分（データベース、そのマイグレーション、サーバーキー）のチェックです。サーバーチェックと一緒に実行されます。
admin-diagx-tab-connectivity-intro = このサーバーが公開名でアクセスできるか、保存済みのバックエンドに到達できるか、プライベートアドレスに届かないようになっているかを確認します。下のドメインツールでは、DNS レコードの検索、TLS のテスト、ドメインの設定の検出ができます。
admin-diagx-tab-backends-intro = 保存済みのすべてのメール、送信、ルール、DAV エンドポイントとの接続およびセキュリティのハンドシェイクです。認証情報は送信されず、メールも配信されません。
admin-diagx-tab-posture-intro = サインインとセッションを保護する設定と、リクエストのアドレスが fail2ban にどう見えるかを確認します。
admin-diagx-tab-tools-intro = サービスを確認できるその他の場所と、診断の実行に適用される制限です。
admin-diagx-tab-history-intro = 保存された実行を新しい順に表示します。実行を開くと、その結果と、直前の実行からの変化を確認できます。
admin-diagx-status-ok = OK
admin-diagx-status-warn = 警告
admin-diagx-status-fail = 失敗
admin-diagx-status-skipped = スキップ
admin-diagx-status-not-run = 未実行
admin-diagx-status-running = 実行中
admin-diagx-run = 実行
admin-diagx-run-again = もう一度実行
admin-diagx-running = 実行中…
admin-diagx-run-server-note = サーバーチェックをもう一度実行します。その実行のすべてのカードが更新されます。
admin-diagx-run-backends-note = 保存済みのすべてのバックエンドをもう一度チェックします。すべてのバックエンドのカードが更新されます。
admin-diagx-how-to-fix = 対処方法
admin-diagx-details = 詳細
admin-diagx-copy-details = 詳細をコピー
admin-diagx-copy-value = { $label }をコピー
admin-diagx-copied = コピーしました。
admin-diagx-copy-failed = ブラウザーでコピーが許可されませんでした。テキストを選択して、手動でコピーしてください。
admin-diagx-not-run-yet = まだ実行されていません。
admin-diagx-not-reported = 前回のサーバー実行ではこのチェックが報告されませんでした。このデプロイには該当しません。
admin-diagx-posture-met = 満たしている
admin-diagx-posture-not-met = 満たしていない
admin-diagx-loading = 読み込み中…
admin-diagx-none = なし
admin-diagx-yes = はい
admin-diagx-no = いいえ
admin-diagx-open = 開く
admin-diagx-row-result = 結果
admin-diagx-row-duration = 所要時間
admin-diagx-row-checked = チェック日時
admin-diagx-row-id = チェック ID
admin-diagx-row-domain = ドメイン
admin-diagx-row-backend = バックエンド
admin-diagx-row-protocol = プロトコル
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = ケイパビリティ
admin-diagx-row-credentials = 送信した認証情報
admin-diagx-row-source = 検出元
admin-diagx-row-oauth = OAuth が必要
admin-diagx-overview-intro = グループごとに、各チェックの最新の結果を表示します。「すべて実行」は、サーバーチェックを実行してから、保存済みのすべてのバックエンドをチェックします。
admin-diagx-run-all = すべてのチェックを実行
admin-diagx-run-all-step = ステップ { $step } / { $steps }：{ $suite ->
    [backends] 保存済みのバックエンド
   *[server] サーバーチェック
}。
admin-diagx-run-all-done = すべてのチェックが完了しました。結果は履歴に保存されています。
admin-diagx-backends-progress = { $total } 件中 { $completed } 件のバックエンドをチェックしました。
admin-diagx-counts = OK { $ok } 件 · 警告 { $warn } 件 · 失敗 { $fail } 件 · 未実行 { $notRun } 件
admin-diagx-attention = 要確認
admin-diagx-attention-none = 最新の実行で、警告または失敗を報告したチェックはありません。
admin-diagx-never-run = まだチェックは実行されていません。「すべてのチェックを実行」で最初の結果を取得してください。
admin-diagx-last-server = 前回のサーバーチェック
admin-diagx-last-backends = 前回のバックエンドチェック
admin-diagx-never = 未実施
admin-diagx-open-health = サービスの状態
admin-diagx-check-database-connectivity = データベース接続
admin-diagx-check-database-connectivity-purpose = データベースが単純なクエリに応答します。
admin-diagx-check-database-connectivity-fix = データベースが稼働していて、MW_DB_PATH または MW_DATABASE_URL がそれを指しているか確認してください。接続エラーはサーバーログに記録されます。
admin-diagx-check-database-write = データベースへの書き込み
admin-diagx-check-database-write-purpose = 一時的な行を書き込み、その後ロールバックできます。
admin-diagx-check-database-write-fix = ディスクの空き容量と、データベースファイルまたはユーザーに書き込み権限があるか確認してください。読み取り専用のボリュームやディスクの空き不足があると、書き込めません。
admin-diagx-check-database-migrations = データベースのマイグレーション
admin-diagx-check-database-migrations-purpose = このサーバーのバージョンが必要とするすべてのマイグレーションがデータベースに適用されています。
admin-diagx-check-database-migrations-fix = サーバーを再起動して保留中のマイグレーションを適用し、失敗したマイグレーションがないかログを確認してください。新しいデータベースで古いサーバーを実行しないでください。
admin-diagx-check-server-key-roundtrip = サーバーキー
admin-diagx-check-server-key-roundtrip-purpose = サーバーキーでテスト値を封印し、開封できます。
admin-diagx-check-server-key-roundtrip-fix = MW_SERVER_KEY（またはキーファイル）が、このデータベースの作成時に使われたキーか確認してください。保存されたシークレットは、別のキーでは開封できません。
admin-diagx-check-other-purpose = このサーバーが報告したチェックです。
admin-diagx-check-other-fix = 「詳細」で元の結果を開き、実行時点のサーバーログを確認してください。
admin-diagx-check-network-public-dns = 公開名
admin-diagx-check-network-public-dns-purpose = MW_PUBLIC_URL のホスト名を、このサーバーから名前解決できます。
admin-diagx-check-network-public-dns-fix = 公開名の DNS レコードを確認するか、MW_PUBLIC_URL をユーザーがこのサーバーにアクセスするアドレスに設定してください。
admin-diagx-check-network-public-port = 公開ポート
admin-diagx-check-network-public-port-purpose = 公開 URL のポートが、このサーバーからの接続を受け付けます。
admin-diagx-check-network-public-port-fix = ファイアウォール、リバースプロキシ、ポート転送を確認してください。一部のルーターは自身の公開アドレスに接続できません（NAT ヘアピン）。ネットワークの外部からもテストしてください。
admin-diagx-check-network-backend-ports = バックエンドのポート
admin-diagx-check-network-backend-ports-purpose = 保存済みの各バックエンドのホストとポートが、直接の接続を受け付けます。
admin-diagx-check-network-backend-ports-fix = バックエンドのホストが稼働していて、ファイアウォールがこのサーバーからの接続を許可しているか確認してください。「メールバックエンド」で、どのエンドポイントのハンドシェイクが失敗しているかを確認できます。
admin-diagx-check-egress-private-policy = プライベートアドレスの保護
admin-diagx-check-egress-private-policy-purpose = デフォルトでは、ループバックとプライベートアドレスへの送信接続は拒否されます。
admin-diagx-check-egress-private-policy-fix = 「プライバシー」の送信設定を確認してください。プライベートアドレスは、必要な名前付きバックエンドにのみ許可してください。
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = リクエストは HTTPS 経由でこのサーバーに届いています。
admin-diagx-check-security-https-fix = コンソールを HTTPS で提供するか、リバースプロキシが TLS を終端する場合は、MW_PUBLIC_URL を https に設定し、MW_TRUSTED_PROXIES を設定してください。
admin-diagx-check-security-secure-cookies = Secure Cookie
admin-diagx-check-security-secure-cookies-purpose = セッション Cookie は HTTPS でのみ送信されます。
admin-diagx-check-security-secure-cookies-fix = サーバーに HTTPS でアクセスできるようになったら、MW_COOKIE_SECURE を true に設定してください。
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = ブラウザーに、このサイトで HTTPS を使うよう指示されています。
admin-diagx-check-security-hsts-fix = セキュリティ設定で Strict-Transport-Security をオンにするか、リバースプロキシから送信してください。
admin-diagx-check-security-csrf = CSRF 保護
admin-diagx-check-security-csrf-purpose = データを変更するリクエストには、CSRF トークンが必要です。
admin-diagx-check-security-csrf-fix = MW_CSRF_STRICT を true に設定してください。
admin-diagx-check-security-admin-factors = 管理者の第 2 要素
admin-diagx-check-security-admin-factors-purpose = すべての管理者が、第 2 のサインイン要素を持っています。
admin-diagx-check-security-admin-factors-fix = 第 2 要素のない管理者に、「個人設定」でパスキーまたは認証アプリを追加するよう依頼するか、セキュリティセンターで必須にしてください。
admin-diagx-check-security-signin-pacing = サインインの再試行待機
admin-diagx-check-security-signin-pacing-purpose = サインインに失敗すると、短い待機時間が入ります。
admin-diagx-check-security-signin-pacing-fix = セキュリティポリシーでサインインのペース制御をオンにしてください。
admin-diagx-check-security-credential-locks = 認証情報のロック
admin-diagx-check-security-credential-locks-purpose = サインインの失敗が繰り返されると、その認証情報が一定時間ロックされます。
admin-diagx-check-security-credential-locks-fix = セキュリティセンターで自動ロックをオンにしてください。
admin-diagx-check-security-geography = サインイン地域のデータ
admin-diagx-check-security-geography-purpose = サインイン地域ポリシーに必要な国データがあります。
admin-diagx-check-security-geography-fix = 国データベースをインストールまたは更新するか、地域ポリシーをオフにしてください。
admin-diagx-check-security-render-isolation = 隔離されたメッセージ表示
admin-diagx-check-security-render-isolation-purpose = メッセージは、別の隔離されたワーカーで表示されます。
admin-diagx-check-security-render-isolation-fix = MW_RENDER_BIN が mw-render プログラムを指していて、起動できるか確認してください。ダッシュボードのサービスの状態に、ワーカーの状態が表示されます。
admin-diagx-check-security-deployment = デプロイ設定
admin-diagx-check-security-deployment-purpose = デプロイのセキュリティ設定を読み取れます。
admin-diagx-check-security-deployment-fix = データベース接続を確認してから、デプロイのセキュリティ評価を開いてください。
admin-diagx-check-security-plaintext-backends = 暗号化されていない接続でのパスワード
admin-diagx-check-security-plaintext-backends-purpose = TLS なしでパスワードを受け付ける保存済みバックエンドはありません。
admin-diagx-check-security-plaintext-backends-fix = ドメインの接続を編集して、TLS または STARTTLS を選択してください。平文の接続は、管理下のプライベートネットワークでのみ許可してください。
admin-diagx-check-fail2ban-client-address = fail2ban のクライアントアドレス
admin-diagx-check-fail2ban-client-address-purpose = fail2ban のログ行に記録されるアドレスは、実際のクライアントのアドレスです。
admin-diagx-check-fail2ban-client-address-fix = リバースプロキシの背後では、MW_TRUSTED_PROXIES をプロキシのアドレスに設定してください。設定しないと、fail2ban がクライアントではなくプロキシを禁止します。サーバー自体でコンソールを開く場合は、ループバックアドレスになるのが正常です。
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = { $protocol } エンドポイント：サインインせずに行う、接続とセキュリティのハンドシェイク。
admin-diagx-backend-other-purpose = バックエンド実行からのメモです。
admin-diagx-backend-fix = このバックエンドに保存されているホスト名、ポート、TLS モードを確認し、DNS が解決できること、証明書がそのホスト名で有効であることを確認してください。その後、もう一度実行してください。
admin-diagx-backend-config-fix = ドメインの接続を開いて、もう一度保存してください。保存済みのバックエンド一覧を読み取れませんでした。
admin-diagx-backend-results = バックエンドチェック
admin-diagx-backends-not-run = 保存済みのバックエンドは、まだチェックされていません。
admin-diagx-backends-run-at = 前回のチェック { $date }：{ $count } 件の結果。
admin-diagx-failover-title = フェイルオーバー
admin-diagx-failover-purpose = 第 1 候補のメールバックエンドに接続できない場合に、ドメインが別のメールバックエンドへ切り替えられるかどうかです。
admin-diagx-failover-state = フェイルオーバー{ $state ->
    [on] オン
   *[off] オフ
} · メールバックエンド { $backends } 件{ $enabled ->
    [no] {" "}· ドメインは無効
   *[yes] {""}
}
admin-diagx-no-domains = 保存済みのドメインはありません。
admin-diagx-domain-tools = ドメインツール
admin-diagx-domain-tools-purpose = 保存済みのドメインに対する 1 回限りのチェックです。サインインやメール送信は行いません。
admin-diagx-domain = ドメイン
admin-diagx-tool-dns = DNS レコード
admin-diagx-tool-dns-purpose = ドメインの MX、SPF、DMARC、サービス（SRV）レコードを検索します。
admin-diagx-tool-dns-fix = 不足しているレコードを DNS プロバイダーで追加してください。MX がないとドメインはメールを受信できず、SPF と DMARC がないと、他のサーバーがそのメールを拒否する場合があります。
admin-diagx-srv-found = サービスレコード { $total } 件中 { $found } 件が見つかりました
admin-diagx-tool-tls = TLS とハンドシェイク
admin-diagx-tool-tls-purpose = ドメインの保存済みの各エンドポイントに接続し、TLS のバージョン、証明書の有効期限、所要時間を報告します。
admin-diagx-tool-tls-fix = 接続できないエンドポイントは、ホスト、ポート、ファイアウォールを確認してください。14 日以内に期限が切れる証明書は更新してください。
admin-diagx-probe-none = このドメインには、テストできる保存済みのエンドポイントがありません。
admin-diagx-probe-line = { $host } · 到達可能：{ $reachable ->
    [yes] はい
   *[no] いいえ
} · TLS { $tls } · 証明書{ $days ->
    [none] 報告なし
   *[other] 有効期限まで { $days } 日
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
admin-diagx-tool-detect = 設定を検出
admin-diagx-tool-detect-purpose = ドメイン名に対して公開されているメール設定（autoconfig、autodiscover、SRV）を検索します。何も保存されません。
admin-diagx-tool-detect-fix = ドメインに autoconfig または SRV レコードを公開するか、「ドメイン」で設定を手動で入力してください。
admin-diagx-detect-name = 検索するドメイン名
admin-diagx-tool-health = サービスの状態
admin-diagx-tool-health-purpose = ダッシュボードにある、メール同期、送信トレイ、検索インデックス、表示ワーカー、プラグインの状態です。
admin-diagx-tool-deployment-purpose = このサーバーが確認できるデプロイのセキュリティ設定すべてと、それを制御する設定です。
admin-diagx-tool-ssh = SSH トンネルのテスト
admin-diagx-tool-ssh-purpose = 別のネットワークにあるバックエンドへの接続に使う SSH トンネルをテストします。
admin-diagx-tool-geo = サインイン地域
admin-diagx-tool-geo-purpose = サインインの発信元と、地域ポリシーの基となる国データベースです。
admin-diagx-catalogue-purpose = 各診断スイートの対象範囲と、各実行が守る制限です。
admin-diagx-suite-unsupported = 利用不可：{ $reason }
admin-diagx-suite-server = サーバーチェック
admin-diagx-suite-backends = 保存済みのバックエンド
admin-diagx-run-title = { $suite ->
    [backends] 保存済みのバックエンド
   *[server] サーバーチェック
}、{ $date }
admin-diagx-changes = 変更点
admin-diagx-changes-first = 比較できる、この種類の以前の実行はありません。
admin-diagx-changes-none = { $date } の実行から変更はありません。
admin-diagx-changes-since = { $date } の実行からの変更：
admin-diagx-change = { $before } → { $after }
admin-diagx-report-title = トラブルシューティングレポート
admin-diagx-report-purpose = サポート向けの JSON レポートです。実行時のフラグ、最新の結果、選択した実行が含まれます。パスワード、トークン、Cookie、キー、設定の生データ、メールは含まれません。
admin-diagx-report-run = 含める実行
admin-diagx-report-preview = プレビュー
admin-crumb-server = サーバー
admin-crumb-connectivity = 接続性
admin-crumb-posture = セキュリティ状況
admin-crumb-tools = ツール
admin-crumb-history = 履歴
admin-crumb-report = レポート
admin-diagx-table-actions = 操作
admin-diagx-check-pdfjs-viewer-assets = PDF ビューアーのファイル
admin-diagx-check-pdfjs-viewer-assets-purpose = PDF ビューアーのファイルがすべてそろっていて、種類が正しく、このサーバーのビルドと一致しています。
admin-diagx-check-pdfjs-viewer-assets-fix = Web ビルドを再デプロイするか、MW_WEB_DIR から pdfjs フォルダーを削除して、サーバーに組み込まれたコピーを使ってください。警告ではなく失敗になっている場合は、サーバーイメージを再ビルドしてください。
