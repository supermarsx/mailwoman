# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin. Source: ../en/admin.ftl

# -- Shell / nav -------------------------------------------------------------
admin-brand = Mailwoman 管理
admin-nav = 管理セクション
admin-sign-out = サインアウト
admin-nav-domains = ドメイン
admin-nav-users = ユーザー
admin-nav-integrations = 連携
admin-nav-observability = 監視
admin-nav-plugins = プラグイン
admin-nav-assist = AI アシスタント
admin-nav-sso = シングルサインオン
admin-nav-servermeta = サーバーメタデータ
admin-nav-rethread = メンテナンス
admin-nav-2fa = 2 要素認証の必須化

admin-delete = 削除
admin-revoke = 取り消す
admin-remove = 削除
admin-saved = 保存しました。

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = 管理者サインイン
admin-login-note = このパネルは別の管理者セッションで動作します。セッションは操作がないまま 30 分経過すると終了し、遅くともサインインから 12 時間後に終了します。
admin-login-session-ended = 管理者セッションが終了しました。もう一度サインインしてください。
admin-login-username = 管理者ユーザー名
admin-login-password = パスワード
admin-login-sign-in = サインイン
admin-login-signing-in = サインインしています…
admin-login-invalid = 管理者の認証情報が正しくありません
admin-login-unreachable = サーバーに接続できませんでした

# -- Domains -----------------------------------------------------------------
admin-domains-title = ドメイン
admin-domains-load-error = ドメインを読み込めませんでした
admin-domains-save-error = ドメインを保存できませんでした
admin-domains-delete-error = ドメインを削除できませんでした
admin-domains-add = ドメインを追加
admin-domains-name = ドメイン名
admin-domains-name-placeholder = example.com
admin-domains-note = ドメインは名前で登録されます。登録された名前は、ルールを 1 つのドメインに適用するときに「2 要素認証の必須化」画面で選択肢として表示されます。ドメインを登録しても、メールのルーティングやフィルタリングは行われません。
admin-domains-save = ドメインを保存
admin-domains-empty = ドメインはまだありません。
admin-domains-delete-for = { $name } を削除

# -- Users -------------------------------------------------------------------
admin-users-title = ユーザー
admin-users-load-error = ユーザーを読み込めませんでした
admin-users-provision-error = ユーザーを作成できませんでした
admin-users-flag-error = フラグを更新できませんでした
admin-users-revoke-error = セッションを取り消せませんでした
admin-users-provision = ユーザーを作成
admin-users-username = ユーザー名
admin-users-domain = ドメイン
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = 容量上限（バイト、0 = 無制限）
admin-users-quota-msgs = メッセージ数上限（0 = 無制限）
admin-users-empty = ユーザーはまだいません。
admin-users-col-account = アカウント
admin-users-col-quota = 上限（バイト/件数）
admin-users-col-zeroaccess = ゼロアクセス
admin-users-col-flags = フラグ
admin-users-col-sessions = セッション
admin-users-zeroaccess-for = { $account } のゼロアクセス
admin-users-zeroaccess-help = 記録用のみです。ゼロアクセス保存は、ユーザー自身が自分の設定で自分の鍵を使ってオン・オフします。このボックスにチェックを入れてもアカウントの保存済みメールは暗号化されず、チェックを外しても復号されません。
admin-users-disable-for = { $account } を無効化
admin-users-disabled = 無効
admin-users-disabled-help = このアカウントのサインインをブロックし、開いているすべてのセッションを終了します。API キーとトークンは拒否されます。メールサーバー上のメールボックスは無効になりません。そちらでも無効にするまで、他のメールクライアントからは引き続きアクセスできます。
admin-users-force-change-for = { $account } にパスワード変更を強制
admin-users-force-change = 変更を強制
admin-users-force-change-help = ユーザーは次回のサインイン時およびすでに開いているセッションでパスワード変更画面に留め置かれ、変更が成功するまでメールにアクセスできません。これには動作するパスワード変更バックエンド（MW_PASSWD_BACKEND）が必要です。バックエンドがない場合、変更は成功せず、このボックスのチェックを外すまでユーザーはその画面に留まります。
admin-users-revoke-for = { $account } のセッションを取り消す

# -- Integrations ------------------------------------------------------------
admin-integrations-title = 連携
admin-integrations-load-error = 連携を読み込めませんでした
admin-integrations-revoke-error = キーを取り消せませんでした
admin-integrations-ldap = LDAP / GAL ディレクトリ
admin-integrations-nextcloud = Nextcloud ブリッジ
admin-integrations-active = 有効
admin-integrations-configured = 設定済み
admin-integrations-not-configured = 未設定
admin-integrations-unknown = 状態不明
admin-integrations-config-note = 「設定済み」は、このデプロイにその連携の設定があることを意味します。データベース内の有効な LDAP ディレクトリ項目、または 3 つの MW_NEXTCLOUD_* 環境変数です。どちらもサーバーの起動時に読み込まれます。リモートサービスに接続したことや、到達可能であることを意味するものではありません。
admin-integrations-webhooks = Webhook
admin-integrations-webhooks-empty = 登録された Webhook はありません。
admin-integrations-keys = API キーと MCP キー
admin-integrations-keys-empty = 発行されたキーはありません。
admin-integrations-col-prefix = プレフィックス
admin-integrations-col-account = アカウント
admin-integrations-col-scopes = スコープ
admin-integrations-col-status = 状態
admin-integrations-status-revoked = 取り消し済み
admin-integrations-status-active = 有効
admin-integrations-revoke-key = キー { $prefix } を取り消す
admin-integrations-col-unattended = 無人送信
admin-integrations-unattended-not-requested = 申請なし
admin-integrations-unattended-requested = 申請済み、未承認
admin-integrations-unattended-approved = 承認済み
admin-integrations-unattended-approve = 承認
admin-integrations-unattended-withdraw = 撤回
admin-integrations-unattended-approve-key = キー { $prefix } の無人送信を承認
admin-integrations-unattended-withdraw-key = キー { $prefix } の無人送信の承認を撤回
admin-integrations-unattended-approve-title = このキーの無人送信を承認しますか？
admin-integrations-unattended-withdraw-title = このキーの承認を撤回しますか？
admin-integrations-unattended-owner = 所有者: { $account }
admin-integrations-unattended-key = キー: { $prefix }
admin-integrations-unattended-scope = 権限: { $permissions }。MCP ツール: { $tools }。
admin-integrations-unattended-scope-unreadable = このキーのスコープを読み取れませんでした。
admin-integrations-scope-read = 読み取り
admin-integrations-scope-send = 送信
admin-integrations-scope-delete = 削除
admin-integrations-scope-mail = メール
admin-integrations-scope-pim = カレンダー、タスク、メモ、連絡先
admin-integrations-scope-none = なし
admin-integrations-unattended-approve-meaning = このキーで MCP 経由で送信されたメッセージは、人による送信許可なしに送信されます。
admin-integrations-unattended-withdraw-meaning = このキーで MCP 経由で行われる送信は、キーが再び承認されるまで拒否されます。メッセージは送信されず、所有者の送信トレイにも入りません。
admin-integrations-unattended-approve-effect = 承認はすぐに有効になり、キーの次回の送信から適用されます。再起動は不要です。
admin-integrations-unattended-withdraw-effect = 撤回はすぐに有効になります。キーは次回の送信から無人送信を行いません。再起動は不要です。
admin-integrations-unattended-cancel = キャンセル
admin-integrations-unattended-saving = 保存しています…
admin-integrations-unattended-approved-done = キー { $prefix } の無人送信を承認しました。現在有効です。
admin-integrations-unattended-withdrawn-done = キー { $prefix } の承認を撤回しました。このキーは無人送信を行いません。
admin-integrations-unattended-error-gone = キー { $prefix } は存在しないか、取り消されています。何も変更されていません。
admin-integrations-unattended-error-conflict = キー { $prefix } は取り消されているか、所有者が無人送信を申請していません。何も承認されていません。
admin-integrations-unattended-error = キー { $prefix } の承認を変更できませんでした。

# -- Observability -----------------------------------------------------------
admin-obs-title = 監視
admin-obs-load-error = 監視データを読み込めませんでした
admin-obs-export-error = 監査ログをエクスポートできませんでした
admin-obs-ban-add-error = ブロックを追加できませんでした
admin-obs-unban-error = ブロックを解除できませんでした
admin-obs-telemetry-note = ログとテレメトリはここでは設定しません。ログフィルター（MW_LOG）、OTLP コレクター（MW_OTLP_ENDPOINT）、メトリクスエンドポイント（MW_METRICS_TOKEN）は、サーバーの起動時に環境から読み込まれます。
admin-obs-audit = 監査ログ
admin-obs-export = JSONL をエクスポート
admin-obs-audit-empty = 監査エントリはありません。
admin-obs-col-time = 日時
admin-obs-col-actor = 実行者
admin-obs-col-action = 操作
admin-obs-col-target = 対象
admin-obs-bans = ログイン監視 / ブロックリスト
admin-obs-bans-note = このリストは記録です。Mailwoman はここにあるアドレスからの接続を拒否せず、エントリはサインインに影響しません。管理者サインインの失敗ごとに、fail2ban の jail が処理できる行がサーバーログに書き込まれ、1 つのアドレスから 5 回失敗するとそのアドレスがここに追加されます。ブロックはその jail またはファイアウォールが行います。
admin-obs-ban-add = ブロックを追加
admin-obs-ban-ip = IP アドレス
admin-obs-ban-reason = 理由
admin-obs-ban-btn = IP をブロック
admin-obs-bans-empty = 有効なブロックはありません。
admin-obs-unban-for = { $ip } のブロックを解除
admin-obs-unban-btn = ブロック解除

# -- Plugins (§22) -----------------------------------------------------------
admin-plugins-title = プラグイン
admin-plugins-intro = エンジンプラグインは、機能単位で権限を制御する WebAssembly サンドボックス内で動作します。プラグインは承認してから有効にし、必要な機能だけを付与してください。
admin-plugins-empty = 登録されたプラグインはありません。
admin-plugins-signed = 署名あり
admin-plugins-unsigned = 署名なし
admin-plugins-approved = 承認済み
admin-plugins-enabled = 有効
admin-plugins-approve = 承認
admin-plugins-enable = 有効にする
admin-plugins-disable = 無効にする
admin-plugins-net = ネットワーク: { $hosts }
admin-plugins-limits = 制限: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = 制限: { $memory } MiB · { $deadline } ms · { $fuel } fuel
admin-plugins-allow-unsigned-for = 署名なしプラグイン { $name } を許可
admin-plugins-allow-unsigned = この署名なしプラグインの実行を許可する
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
admin-allowlist-title = サードパーティプラグインの許可リスト
admin-allowlist-intro = サードパーティ（ファーストパーティ以外）のコンポーネントは、管理者がその正確な SHA-256 ダイジェストを承認した後にのみ読み込まれます。以下のダイジェストは、ディスク上のコンポーネントのバイト列からサーバーが計算したものです。承認すると、そのバイト列に固定されます。ファーストパーティのコンポーネントはビルド時に固定されており、ここでは管理しません。
admin-allowlist-load-error = プラグインの許可リストを読み込めませんでした。
admin-allowlist-present-heading = ディスク上のコンポーネント
admin-allowlist-present-empty = サードパーティのコンポーネントはありません。サードパーティプラグインのディレクトリにコンポーネントを置くと、ここに表示されます。
admin-allowlist-digest-label = 計算された SHA-256
admin-allowlist-status-approved = 承認済み
admin-allowlist-status-pending = 未承認
admin-allowlist-status-firstparty = ファーストパーティ
admin-allowlist-unsigned-note = ダイジェストの固定により許可されています。このコンポーネントには署名がありません。承認は、表示されているダイジェストを持つバイト列そのものを信頼するもので、この方法で承認されたコンポーネントとしては想定どおりの状態です。
admin-allowlist-highpower-note = 高権限の機能（{ $caps }）はサードパーティプラグインに付与できません。サーバーは承認の有無にかかわらずこれらを拒否します。これらはファーストパーティのコンポーネントのみが利用できます。
admin-allowlist-firstparty-note = この ID はファーストパーティのコンポーネントと一致します。ファーストパーティの固定が常に優先されるため、この ID をサードパーティとして承認することはできません。
admin-allowlist-approve = ダイジェストを承認
admin-allowlist-approve-for = { $id } のダイジェストを承認
admin-allowlist-revoke = 取り消す
admin-allowlist-revoke-for = { $id } の固定を取り消す
admin-allowlist-uninstall = アンインストール
admin-allowlist-uninstall-for = { $id } をアンインストール
admin-allowlist-pins-heading = 承認済みおよび取り消し済みの固定
admin-allowlist-pins-empty = 記録された固定はありません。
admin-allowlist-pin-approved-by = { $at } に { $by } が承認
admin-allowlist-pin-revoked = 取り消し済み
admin-allowlist-cancel = キャンセル

admin-allowlist-approve-title = このコンポーネントの読み込みを承認しますか？
admin-allowlist-approve-detail = 承認すると、以下に SHA-256 が表示されているバイト列そのものに固定されます。承認後、この ID ではこのダイジェストに一致するバイト列のみが読み込まれます。コンポーネントを変更するとダイジェストが変わり、再承認されるまで読み込まれません。承認だけでは機能は付与されません。
admin-allowlist-approve-confirm = ダイジェストを承認

admin-allowlist-revoke-title = この固定を取り消しますか？
admin-allowlist-revoke-detail = 取り消すと、このダイジェストの承認が削除され、プラグインが無効になります。次回の読み込み時に有効になり、すでに実行中のインスタンスはそれまで停止されません。
admin-allowlist-revoke-confirm = 固定を取り消す

admin-allowlist-uninstall-title = このプラグインをアンインストールしますか？
admin-allowlist-uninstall-detail = アンインストールすると、すべてのアカウントについてプラグインが保存したキー/値データが削除され、許可リストの固定も削除され、プラグインが無効になります。ディスク上のコンポーネントファイルは削除されないため、後で再承認できます。
admin-allowlist-uninstall-confirm = プラグインをアンインストール

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = シングルサインオン
admin-sso-intro = OIDC と SAML 2.0 のログインバックエンドを設定します。有効なバックエンドは、メールボックスのログイン画面に「…でサインイン」ボタンとして表示されます。対象はデプロイ全体または 1 つのドメインです。
admin-sso-add = ログインバックエンドを追加
admin-sso-edit = 編集
admin-sso-create = バックエンドを追加
admin-sso-update = 変更を保存
admin-sso-cancel = キャンセル
admin-sso-empty = SSO バックエンドは設定されていません。
admin-sso-load-error = SSO バックエンドを読み込めませんでした。
admin-sso-save-error = バックエンドを保存できませんでした。
admin-sso-delete-error = バックエンドを削除できませんでした。

admin-sso-id = バックエンド ID
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = 表示名
admin-sso-display-name-placeholder = Acme SSO でサインイン
admin-sso-kind = プロトコル
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = 対象範囲
admin-sso-scope-deployment = デプロイ全体
admin-sso-scope-domain = 1 つのドメイン
admin-sso-domain = ドメイン
admin-sso-domain-placeholder = example.org
admin-sso-enabled = 有効
admin-sso-first-login = 初回ログインのポリシー
admin-sso-policy-allowlist = 許可リスト（不明なユーザーを拒否）
admin-sso-policy-autocreate = 初回ログイン時にアカウントを自動作成

admin-sso-issuer = 発行者 URL
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = クライアント ID
admin-sso-client-secret = クライアントシークレット
admin-sso-secret-unchanged = 保存済みのシークレットを維持するには空欄のままにします
admin-sso-redirect = リダイレクト URL
admin-sso-scopes = スコープ
admin-sso-metadata = SP メタデータ

admin-sso-sp-entity-id = SP エンティティ ID
admin-sso-acs-url = ACS URL
admin-sso-idp-metadata-url = IdP メタデータ URL
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = IdP メタデータ XML
admin-sso-idp-metadata-xml-placeholder = IdP メタデータ XML を貼り付けるか、上の URL を使用します
admin-sso-idp-sso-url = IdP SSO URL
admin-sso-idp-slo-url = IdP ログアウト（SLO）URL
admin-sso-idp-certs = IdP 署名証明書（PEM）
admin-sso-idp-certs-placeholder = 1 ブロックに 1 つの PEM 証明書、空行で区切ります
admin-sso-nameid-format = NameID 形式
admin-sso-want-signed = 署名付きアサーションを必須にする
admin-sso-want-encrypted = 暗号化されたアサーションを必須にする

admin-sso-claims = クレームのマッピング
admin-sso-claim-email = メールアドレスのクレーム
admin-sso-claim-username = ユーザー名のクレーム
admin-sso-claim-display = 表示名のクレーム
admin-sso-claim-groups = グループのクレーム

admin-sso-badge-enabled = 有効
admin-sso-badge-disabled = 無効
admin-sso-enable = 有効にする
admin-sso-disable = 無効にする
admin-sso-delete = 削除
admin-sso-enable-for = { $name } を有効にする
admin-sso-disable-for = { $name } を無効にする
admin-sso-delete-for = { $name } を削除

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
admin-servermeta-title = サーバーメタデータ
admin-servermeta-intro = 作成済みアカウントの RFC 5464 サーバーアノテーションを表示・編集します。変更はメールサーバーに直接書き込まれ、アカウントがそれを設定できるかどうかはメールサーバーが判断します。
admin-servermeta-account = アカウント
admin-servermeta-select-option = アカウントを選択…
admin-servermeta-select-prompt = サーバーアノテーションを表示・編集するアカウントを選択してください。
admin-servermeta-load-error = アカウント一覧を読み込めませんでした。
admin-servermeta-no-accounts = 作成済みのアカウントはありません。

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
admin-rethread-title = メールボックスのスレッド再構築
admin-rethread-intro = 作成済みアカウントの保存済みメールに対して会話のスレッド化（JWZ）を再実行し、スレッドのグループ化を再設定します。これは 1 回限りのメンテナンス操作で、自動的には実行されません。
admin-rethread-account = アカウント
admin-rethread-select-option = アカウントを選択…
admin-rethread-no-accounts = 作成済みのアカウントはありません。
admin-rethread-load-error = アカウント一覧を読み込めませんでした。
admin-rethread-run = スレッドを再構築
admin-rethread-confirm-title = このメールボックスのスレッドを再構築しますか？
admin-rethread-confirm-warning = スレッドを再構築すると、このアカウントの会話のグループ化が再設定されます。既存のスレッドが統合または分割され、ユーザーには会話がまとめ直されて見える場合があります。
admin-rethread-confirm-detail = これは 1 回限りのメンテナンス操作です。今 1 回だけ実行されます。再実行しても安全で、メールは削除されません。
admin-rethread-confirm = スレッドを再構築
admin-rethread-running = スレッドを再構築しています…
admin-rethread-cancel = キャンセル
admin-rethread-summary = { $accounts } 件のアカウントで { $messages } 件のメッセージを { $threads } 件のスレッドに再構築しました。{ $reassigned } 件のメッセージが別のスレッドに移動しました。
admin-rethread-error = スレッドの再構築に失敗しました。サーバーが要求を拒否した場合、スレッドのグループ化は変更されていません。サーバーログを確認して、もう一度お試しください。

# -- Search index: status and rebuild (t28-e13) --------------------------------
admin-searchindex-title = 検索インデックス
admin-searchindex-intro = 検索は保存済みメールから作成されたインデックスを使用します。インデックスが保存済みメールと一致しない場合、サーバーは起動時に再構築します。ゼロアクセスアカウントのメールはインデックスに登録されないため、これらのアカウントでは検索結果が表示されません。
admin-searchindex-count = 保存済みメッセージ { $messages } 件のうち { $documents } 件がインデックス登録済みです。
admin-searchindex-on-disk = インデックスはディスクに保存されています。
admin-searchindex-in-memory = インデックスはメモリ上に保持され、サーバーの起動ごとに再構築されます。
admin-searchindex-progress = 再構築を実行中です: { $total } 件中 { $done } 件のメッセージが完了しました。
admin-searchindex-run = 検索インデックスを再構築
admin-searchindex-running = 再構築しています…
admin-searchindex-summary = { $accounts } 件のアカウントで保存済みメッセージ { $messages } 件を読み込みました: { $indexed } 件を登録、{ $removed } 件をインデックスから削除、{ $failed } 件は読み込めませんでした。ゼロアクセスアカウント { $zeroAccess } 件はインデックス未登録のままです。
admin-searchindex-busy = 再構築はすでに実行中です。完了するまでお待ちください。
admin-searchindex-error = 再構築に失敗しました。インデックスは以前の内容を保持しています。サーバーログを確認して、もう一度お試しください。
admin-searchindex-status-error = 検索インデックスの状態を読み取れませんでした。
admin-searchindex-unavailable = このサーバーはプロキシモードで動作しており、検索インデックスはありません。

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
admin-2fa-title = 2 要素認証の必須化
admin-2fa-intro = サインインに 2 つ目の要素（パスキーまたは認証アプリ）を必須にします。どのユーザーも自分で要素を登録できます。ここで必須にすると、対象のアカウントは次回のサインイン時に登録を求められます。
admin-2fa-load-error = 2 要素認証のポリシーを読み込めませんでした。
admin-2fa-save-error = 2 要素認証のポリシーを保存できませんでした。
admin-2fa-global = デプロイ全体で 2 要素認証を必須にする
admin-2fa-global-label = デプロイ全体で 2 要素認証を必須にする
admin-2fa-global-note = オンにすると、すべてのアカウントに 2 つ目の要素が必要になります。下のドメイン別ルールを使うと、全体ではなく 1 つのドメインだけで必須にすることもできます。
admin-2fa-domains-heading = ドメイン別の必須設定
admin-2fa-domains-empty = ドメイン別の必須設定はありません。
admin-2fa-col-domain = ドメイン
admin-2fa-col-require = 必須
admin-2fa-require-for = { $domain } で 2 要素認証を必須にする
admin-2fa-add-domain = ドメイン
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = このドメインで 2 要素認証を必須にする
admin-2fa-add-require-label = このドメインで 2 要素認証を必須にする
admin-2fa-add-save = ドメインルールを追加

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = 送信経路
admin-egress-heading = 送信プロキシのルート
admin-egress-intro = サーバーからの外部取得を HTTP CONNECT または SOCKS5 プロキシ経由にします。Mailwoman は引き続き独自のアドレスポリシーを適用します。プロキシに名前解決を依頼することはありません。
admin-egress-empty = ルートは設定されていません。外部取得は直接行われます。
admin-egress-none-active = 使用中のルートはありません。外部取得は直接行われます。保存したルートは「このルートを使用」を選んだ後にのみ通信を中継します。
admin-egress-in-use = 外部取得はルート「{ $id }」を使用しています。
admin-egress-deactivate = プロキシの使用を停止
admin-egress-activate = このルートを使用
admin-egress-activate-for = ルート { $id } を使用
admin-egress-activate-error = ルートに切り替えられませんでした
admin-egress-deactivate-error = ルートの使用を停止できませんでした
admin-egress-col-state = 状態
admin-egress-state-active = 使用中
admin-egress-state-inactive = 未使用
admin-egress-test-scope = テストできるのは使用中のルートのみです。テストではルート経由で固定の URL を 1 つ取得し、どこまで到達したかを報告します。何も変更されません。
admin-egress-probe-url = 取得先: { $url }
admin-egress-load-error = 送信経路のルートを読み込めませんでした
admin-egress-save-error = ルートを保存できませんでした
admin-egress-delete-error = ルートを削除できませんでした
admin-egress-invalid = ID、ホスト、0 より大きいポートが必要です。
admin-egress-username-required = パスワードにはユーザー名が必要です。ユーザー名がないと、ルートは誰としても認証されません。
admin-egress-col-id = ID
admin-egress-col-route = ルート
admin-egress-col-username = ユーザー名
admin-egress-col-credentials = パスワード
admin-egress-col-plaintext = 平文
admin-egress-col-actions = 操作
admin-egress-cred-set = 設定済み
admin-egress-cred-none = なし
admin-egress-plaintext-on = 許可
admin-egress-plaintext-off = 拒否
admin-egress-add-heading = ルートを追加
admin-egress-edit-heading = ルートを編集
admin-egress-id = ID
admin-egress-scheme = スキーム
admin-egress-host = ホスト
admin-egress-port = ポート
admin-egress-username = ユーザー名
admin-egress-password = パスワード
admin-egress-password-unchanged = 保存済みのパスワードを維持するには空欄のままにします
admin-egress-allow-plaintext = このルートで平文の http オリジンを許可する
admin-egress-allow-plaintext-note = 既定ではオフです。https オリジンであれば、プロキシは中継する通信を読み取れません。http を許可すると、プロキシから通信が見えます。
admin-egress-test = テスト
admin-egress-testing = テストしています…
admin-egress-test-failed = テストを実行できなかったため、このルートについては何もわかっていません。
admin-egress-outcome-connected = 接続しました
admin-egress-outcome-auth-rejected = 認証が拒否されました
admin-egress-outcome-refused-by-policy = ポリシーにより拒否されました
admin-egress-outcome-dns-failed = 名前を解決できませんでした
admin-egress-outcome-unreachable = 到達できません
admin-egress-outcome-origin-tls-failed = オリジンの TLS に失敗しました
admin-egress-outcome-route-invalid = ルートが無効です
admin-egress-outcome-unknown = 不明な結果
admin-egress-stage-dns = 名前解決の段階で停止
admin-egress-stage-connect = プロキシへの接続中に停止
admin-egress-stage-tunnel = トンネルの確立中に停止
admin-egress-stage-origin = オリジンで停止
admin-egress-stage-unknown = 不明な段階で停止
admin-egress-proxied-yes = プロキシを経由しました
admin-egress-proxied-no = プロキシを経由しませんでした
