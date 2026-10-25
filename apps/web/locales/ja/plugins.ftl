# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: plugins. Source: ../en/plugins.ftl

# -- Unsigned-plugin trust banner --------------------------------------------
plugins-unsigned-warning-label = 署名なし UI プラグインの警告
plugins-unsigned-title = 署名なしのプラグインが実行されています
plugins-unsigned-text = これらのプラグインは、検証済みの署名なしで許可されています。サンドボックス内で実行されますが、コードは信頼済みの鍵で検証されていません。

# -- Plugin frames: the region around each sandboxed frame and the frame title -
plugins-frame-label = プラグイン: { $name }
plugins-frame-title = plugin:{ $id }

# -- Admin: engine plugins (src/screens/Admin/Plugins) ------------------------
plugins-admin-intro = エンジンプラグインは WebAssembly サンドボックス内で動作します。プラグインは、登録・承認され、1 つ以上の機能が付与され、有効にされた後にのみ実行されます。使用できるのは、ここで付与された機能だけです。
plugins-admin-unsigned-banner = 署名のないサードパーティプラグインが読み込まれています。管理者が許可したため実行されています。サーバーはそのファイルを承認済みのダイジェストと照合するだけです。
plugins-admin-load-error = プラグインを読み込めませんでした。
plugins-admin-register-heading = プラグインを登録
plugins-admin-register-component = コンポーネント
plugins-admin-register-third-party = サードパーティのコンポーネント
plugins-admin-register-id = プラグイン ID
plugins-admin-register-id-hint = サーバーは、サードパーティプラグインのディレクトリにあるこの ID の名前のファイルからコンポーネントを読み込みます。先に下の許可リストでそのファイルのダイジェストを承認してください。
plugins-admin-register-name = 名前
plugins-admin-register-version = バージョン
plugins-admin-register-capabilities = プラグインが宣言する機能
plugins-admin-register-hosts = プラグインが接続できるホスト
plugins-admin-register-hosts-hint-first-party = ホスト名をカンマ区切りで入力します。空欄にするとコンポーネントの既定のホストを使用します。
plugins-admin-register-hosts-hint-third-party = ホスト名をカンマ区切りで入力します。空欄にするとネットワークにアクセスできません。
plugins-admin-register-submit = 登録
plugins-admin-trust-first-party = ファーストパーティ、ダイジェスト組み込み済み
plugins-admin-trust-third-party = サードパーティ、ダイジェスト承認済み
plugins-admin-status-loaded = 読み込み済み
plugins-admin-status-not-loaded = 未読み込み
plugins-admin-status-restart = 再起動が必要
plugins-admin-restart-to-load = このプラグインを読み込むには再起動が必要です。
plugins-admin-restart-to-apply = これらの変更を適用するには再起動が必要です。それまでは、このブリッジの一部のアカウントが処理されません。
plugins-admin-reason-not-approved = 未読み込み: 管理者がこのプラグインを承認していません。
plugins-admin-reason-disabled = 未読み込み: プラグインが有効になっていません。
plugins-admin-reason-unsigned-not-allowed = 未読み込み: プラグインに署名がなく、署名なしでの実行が許可されていません。
plugins-admin-reason-no-grant = 未読み込み: 必要な機能が付与されていません。
plugins-admin-reason-component-unavailable = 未読み込み: ダイジェストの確認に合格したコンポーネントファイルがありません。
plugins-admin-reason-load-failed = 未読み込み: サーバーがコンポーネントを読み込めませんでした。サーバーログを確認してください。
plugins-admin-reason-proxy-mode = 未読み込み: このサーバーはプロキシモードで動作しており、メールを自身では処理しません。
plugins-admin-reason-no-account-binding = 未読み込み: このブリッジに紐付けられたアカウントがありません。
plugins-admin-reason-no-host-caller = 未読み込み: このバージョンのサーバーは、このプラグインが提供するフックを呼び出しません。
plugins-admin-reason-another-classifier-active = 未読み込み: 別のスパム分類器が読み込まれています。同時に実行できるのは 1 つだけです。
plugins-admin-running-with = 実行中の機能: { $capabilities }
plugins-admin-hosts = ホスト: { $hosts }
plugins-admin-hosts-none = ホスト: なし
plugins-admin-grants-heading = 付与された機能
plugins-admin-grants-hint = プラグインが使用できるのは、ここでチェックした機能だけです。保存すると以前の付与内容が置き換えられます。
plugins-admin-grant-for = { $name } に { $capability } を付与
plugins-admin-grants-save = 付与内容を保存
plugins-admin-endpoint = 分類器のアドレス
plugins-admin-endpoint-for = { $name } の分類器のアドレス
plugins-admin-endpoint-hint = プラグインがデーモンに接続する先です。ホストはプラグインのホスト一覧にも含まれている必要があります。空欄にするとコンポーネントの既定値を使用します。
plugins-admin-endpoint-save = アドレスを保存
plugins-admin-test = 分類器をテスト
plugins-admin-test-verdict = テストの判定: { $verdict }
plugins-admin-uninstall = アンインストール
plugins-admin-uninstall-confirm = アンインストールを確定
plugins-admin-uninstall-note = アンインストールすると、登録、付与された機能、保存された設定が削除されます。
plugins-admin-uninstall-detail = アンインストールすると、すべてのアカウントについてプラグインが保存したデータ、許可リストの固定、登録と付与された機能が削除されます。ディスク上のコンポーネントファイルは削除されません。
plugins-admin-revoke-detail = 取り消すと、このダイジェストの承認が削除され、プラグインが無効になります。プラグインが実行中の場合は、すぐに停止されます。
plugins-admin-error-already-registered = このプラグインはすでに登録されています。再登録する前にアンインストールしてください。
plugins-admin-error-component-unavailable = サーバーに組み込まれたダイジェストに一致する、このプラグインのコンポーネントファイルがありません。
plugins-admin-error-digest-not-approved = この ID について、承認済みのダイジェストを持つコンポーネントファイルが見つかりませんでした。先に許可リストでファイルのダイジェストを承認してください。
plugins-admin-error-not-approved = 有効にする前にプラグインを承認してください。
plugins-admin-error-unsigned-not-allowed = このプラグインには署名がありません。有効にする前に、署名なしでの実行を許可してください。
plugins-admin-error-not-loaded = プラグインが読み込まれていないため、テストできません。
plugins-admin-error-classifier-error = 分類器の呼び出しに失敗しました: { $detail }
plugins-admin-error-other = サーバーが要求を拒否しました: { $detail }

# -- Admin: UI plugins (src/screens/Admin/UiPlugins) --------------------------
plugins-admin-ui-nav = UI プラグイン
plugins-admin-ui-title = UI プラグイン
plugins-admin-ui-intro = UI プラグインはウェブクライアントにコントロールを追加します。各プラグインはサンドボックス化されたフレーム内で動作し、ここで付与された機能だけを使用できます。プラグインは承認され有効にされると、サインインしたユーザーに提供されます。
plugins-admin-ui-empty = 登録された UI プラグインはありません。
plugins-admin-ui-load-error = UI プラグインを読み込めませんでした。
plugins-admin-ui-register-heading = UI プラグインを登録
plugins-admin-ui-manifest = マニフェスト（JSON）
plugins-admin-ui-bundle = バンドルファイル
plugins-admin-ui-bundle-hint = 署名付きプラグインでは必須です。署名はこのファイルに対して検証されます。
plugins-admin-ui-allow-unsigned = このプラグインを署名なしで登録することを許可する
plugins-admin-ui-register = 登録
plugins-admin-ui-manifest-invalid = マニフェストが有効な JSON ではありません。
plugins-admin-ui-refused = サーバーが要求を拒否しました: { $detail }
plugins-admin-ui-approve = 承認して有効にする
plugins-admin-ui-extension-points = 拡張ポイント: { $points }
plugins-admin-ui-grants-heading = 機能を付与
plugins-admin-ui-grants-note = サーバーは、すでに付与されている機能を報告しません。機能を再度付与すると、以前の付与内容が置き換えられます。
plugins-admin-ui-grant = { $capability } を付与
plugins-admin-ui-grant-for = { $name } に { $capability } を付与
plugins-admin-ui-grant-hosts = { $name } の { $capability } 用ホスト
plugins-admin-ui-grant-hosts-hint = プラグインが取得できるホスト名をカンマ区切りで入力します。
plugins-admin-ui-granted = { $capability } を付与しました。
plugins-admin-ui-delete = 削除
plugins-admin-ui-delete-confirm = 削除を確定
