# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-inbound. Source: ../en/admin-inbound.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-inbound-title = 受信アクセス
admin-inbound-help = 管理コンソールとウェブメールについて、信頼する接続元 IP を個別に制限します。どちらのポリシーも初期状態では無効です。これらのルールは、信頼するプロキシ、API スコープ、外向きアクセスを変更しません。
admin-inbound-unscoped = このデプロイ全体のポリシーには、スコープなしのセキュリティアクセス権を持つ管理者が必要です。
admin-inbound-admin = 管理コンソールと API
admin-inbound-admin-help = サインインとこのエディターを含みます。既存の管理者認証があっても、ネットワークポリシーは回避されません。
admin-inbound-webmail = ウェブメールとアカウント API
admin-inbound-webmail-help = メール、DAV、署名付きコンテンツ、SSO、OIDC、API のルートが対象です。アップグレード済みの既存の接続は、ハンドシェイク時の判断を維持します。
admin-inbound-enabled = この対象を許可リストに制限する
admin-inbound-cidrs = 復旧用の IP と CIDR
admin-inbound-cidrs-help = 1行に1つ、IP またはその CIDR を最大 128 個まで入力します。有効にする場合は、少なくとも1つが必要です。復旧に使える信頼できる接続元を含めてください。
admin-inbound-hostnames = 許可する DNS ホスト名
admin-inbound-hostnames-help = 1行に1つ、小文字の ASCII の DNS 名を入力します。国際化ドメイン名には IDNA の ASCII 形式を使ってください。URL、ポート、ワイルドカード、IP リテラル、末尾のドットは使えません。名前によって許可されるのは、現在解決されているアドレスのみです。
admin-inbound-dns = DNS 解決の制限
admin-inbound-dns-help = 新規、期限切れ、失敗した名前にはアクセスが許可されません。成功した応答は、DNS の TTL と設定されたキャッシュ上限に従います。リクエストが DNS の解決を待つことはありません。
admin-inbound-servers = 再帰 DNS サーバー
admin-inbound-servers-help = システムのリゾルバーを使う場合は空欄にするか、1行に1つ、最大4つの IP:ポートを入力します（例：10.0.0.53:53 または [fd00::53]:53）。明示的に指定したサーバーはシステムのリゾルバーを置き換え、公開サーバーへのフォールバックはありません。リゾルバーの信頼は利用者の責任であり、独立した DNSSEC 検証は保証されません。
admin-inbound-dns-timeoutMs = 検索のタイムアウト（ミリ秒）
admin-inbound-dns-cacheSeconds = キャッシュの最大保持時間（秒）
admin-inbound-dns-maxHosts = 異なるホスト名の最大数
admin-inbound-dns-maxAddresses = ホスト名ごとの最大アドレス数
admin-inbound-dns-concurrency = 同時検索数
admin-inbound-bounds = 許可される範囲：{ $min }〜{ $max }。
admin-inbound-lockout = このポリシーにより、このエディターを含め、自分自身のアクセスがすぐにブロックされる可能性があります。復旧用の IP が、設定された信頼するプロキシで判定される実際の接続元を含んでいることを確認してください。ループバックやサインイン済み管理者の自動的な回避はありません。ロックアウトからの復旧にはデプロイ側での復元が必要で、通常のサインインでは回避できません。
admin-inbound-confirm-title = 受信アクセスのポリシーを適用しますか？
admin-inbound-confirm-word = APPLY
admin-inbound-save = 確認済みのポリシーを適用
admin-inbound-review = 変更内容を確認
admin-inbound-refresh = 保存済みのポリシーと DNS の状態を更新
admin-inbound-reset = 下書きを読み込んだポリシーに戻す
admin-inbound-saved = サーバーが受信アクセスのポリシーを受け付けました。
admin-inbound-conflict = 編集中に、保存済みのポリシーが変更されました。下書きをリセットして更新し、現在のリビジョンを読み込んでから、もう一度編集してください。
admin-inbound-no-revision = サーバーがポリシーのリビジョンを返しませんでした。保存する前に更新してください。
admin-inbound-status = 現在の DNS による許可
admin-inbound-status-help = これは最後に状態を更新した時点のスナップショットで、カウントダウンでも接続確認でもありません。現在の許可を確認するには更新してください。元の有効期限が切れるまでは、エラーと、まだ有効な以前の応答が共存することがあります。
admin-inbound-refreshing = 範囲を限定した DNS の更新が進行中です。
admin-inbound-idle = 最後に状態を読み取った時点では、DNS の更新は実行されていませんでした。
admin-inbound-no-hosts = 有効なポリシーに、有効なホスト名はありません。
admin-inbound-hostname = ホスト名
admin-inbound-state = アクセスの許可
admin-inbound-addresses = 利用可能なアドレス
admin-inbound-validity = 最後の更新時点の有効期間
admin-inbound-error = DNS の結果コード
admin-inbound-usable = 利用可能
admin-inbound-unavailable = 現在の許可なし
admin-inbound-seconds = { $seconds } 秒
admin-inbound-on = 有効
admin-inbound-off = 無効
admin-inbound-invalid-dns = DNS の制限またはリゾルバーの数が、許可された範囲外です。
admin-inbound-rescue-required = 有効にした各対象には、少なくとも1つの復旧用 IP または CIDR が必要です。
admin-inbound-too-many = ポリシーが、設定されたホスト名またはリテラルアドレスの上限を超えています。
admin-inbound-invalid-host = URL、ポート、ワイルドカード、IP リテラル、末尾のドットを含まない、小文字の ASCII の DNS 名を使ってください。
