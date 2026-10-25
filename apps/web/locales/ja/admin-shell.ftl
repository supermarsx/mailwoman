# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-shell. Source: ../en/admin-shell.ftl

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = ダッシュボード
admin-shell-group-overview = 概要
admin-shell-group-people = ユーザー
admin-shell-group-security = セキュリティ
admin-shell-group-configuration = 設定
admin-shell-group-operations = 運用
admin-shell-menu = メニュー
admin-shell-close-menu = メニューを閉じる
admin-shell-search = 検索
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = { $name } としてサインイン中
admin-shell-language-label = 言語
admin-shell-breadcrumb = パンくずリスト
admin-shell-unknown-section = このセクションは存在しないか、あなたのロールに含まれていません。
admin-shell-back-to-dashboard = ダッシュボードに戻る
admin-shell-cap-missing = あなたのロールには { $capability } が含まれていません。

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = 読み込んでいます…
admin-shell-retry = 再試行
admin-shell-cancel = キャンセル
admin-shell-save = 保存
admin-shell-not-available = 利用できません

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = このページを離れますか？
admin-shell-leave-body = このページの変更は保存されていません。ページを離れると変更は破棄されます。
admin-shell-leave-confirm = 変更を破棄して離れる

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = 確認のため { $name } と入力してください

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = 管理コンソールを検索
admin-shell-palette-label = セクション、ユーザー、ドメイン、設定
admin-shell-palette-results = 結果
admin-shell-palette-none = 一致するものはありません。
admin-shell-palette-searching = ユーザー、ドメイン、設定を検索しています…
admin-shell-palette-sections-only = このサーバーではまだユーザー、ドメイン、設定を検索できないため、セクションのみ表示しています。
admin-shell-palette-error = ユーザー、ドメイン、設定の検索に失敗しました。セクションのみ表示しています。
admin-shell-palette-kind-section = セクション
admin-shell-palette-kind-user = ユーザー
admin-shell-palette-kind-domain = ドメイン
admin-shell-palette-kind-setting = 設定

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = フィルター
admin-shell-table-actions = 操作
admin-shell-table-error = 一覧を読み込めませんでした。
admin-shell-table-paging = { $table } のページ
admin-shell-table-prev = 前のページ
admin-shell-table-next = 次のページ

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = 最新 { $latest }、最大 { $max }
admin-shell-chart-col-time = 日時
admin-shell-chart-col-item = 項目
admin-shell-chart-col-value = 値

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = 設定元
admin-shell-pinned-sr = この値はサーバーの環境から読み込まれており、ここでは変更できません。
admin-shell-lock-text = ユーザーに対してロック
admin-shell-lock-setting = { $setting }
admin-shell-restore = 以前の値に戻す
admin-shell-restore-note = { $time } に { $actor } が変更しました。以前の値: { $value }
admin-shell-restore-done = 以前の値に戻しました。

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = ステップ
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = ステップ { $n }/{ $total }: { $title }
admin-shell-wizard-review = 確認
admin-shell-wizard-back = 戻る
admin-shell-wizard-next = 次へ
admin-shell-wizard-saved = 保存しました。

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = サーバーに接続できませんでした。接続を確認して、もう一度お試しください。
admin-shell-err-session = 管理者セッションが終了しました。もう一度サインインしてください。
admin-shell-err-forbidden = あなたのロールではこの操作は許可されていません。適切なロールを持つ管理者に依頼してください。
admin-shell-err-out-of-scope = このオブジェクトは、あなたのロールの対象外のドメインに属しています。
admin-shell-err-not-found = この項目はすでに存在しません。一覧を再読み込みしてください。
admin-shell-err-conflict = その間に他の人が変更しました。再読み込みして、もう一度お試しください。
admin-shell-err-validation = サーバーが値を拒否しました。入力内容を確認して、もう一度お試しください。
admin-shell-err-rate-limited = リクエストが多すぎます。1 分ほど待ってから、もう一度お試しください。
admin-shell-err-server = サーバーがエラーを返しました（{ $status }）。もう一度お試しください。繰り返し発生する場合は、サーバーログを確認してください。
admin-shell-err-upstream-dns = ホスト名を解決できませんでした。ホスト名の綴りを確認してください。
admin-shell-err-upstream-connect = そのホストとポートで応答がありませんでした。ポートと、間にあるファイアウォールを確認してください。
admin-shell-err-upstream-tls = サーバーは応答しましたが、TLS ハンドシェイクに失敗しました。ポートと接続のセキュリティ設定を確認してください。
admin-shell-err-upstream-auth = サーバーが認証情報を拒否しました。ユーザー名とパスワードを確認してください。

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = サインインの試行回数が多すぎます。数分待ってから、もう一度お試しください。
admin-shell-totp-form = 管理者サインイン（2 段階目）
admin-shell-totp-note = { $name } 用に設定した認証アプリの 6 桁のコードを入力してください。
admin-shell-totp-code = 認証コード
admin-shell-totp-verify = 確認
admin-shell-totp-start-over = 最初からやり直す
admin-shell-totp-invalid = コードが正しくありません。認証アプリに表示されている現在のコードを入力してください。
admin-shell-totp-expired = サインインに時間がかかりすぎました。最初からやり直して、パスワードを入力してください。
