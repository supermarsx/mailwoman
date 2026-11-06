# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: auth. Source: ../en/auth.ftl

# -- Mailbox login -----------------------------------------------------------
auth-app-name = Mailwoman
auth-email = メールアドレス
auth-password = パスワード
auth-sign-in = サインイン
auth-signing-in = サインインしています…
auth-invalid-credentials = 認証情報が正しくありません
auth-refused-note = メールアドレスとパスワードが正しい場合、アカウントが管理者によって無効にされている可能性があります。無効にされたアカウントは、管理者が再び有効にするまでサインインできません。
auth-unreachable = サーバーに接続できませんでした
auth-address-not-configured = このアドレスはこのサーバーで設定されていません。管理者に問い合わせてください。

# -- Single sign-on (t9) -----------------------------------------------------
auth-sso-heading = シングルサインオン
auth-sso-divider = または次で続行
auth-sso-button = { $name } でサインイン
auth-sso-error = シングルサインオンが完了しませんでした。もう一度お試しいただくか、パスワードでサインインしてください。

# -- OAuth 2.1 consent -------------------------------------------------------
auth-consent-dialog = アプリケーションの承認
auth-consent-title = アクセスの承認
auth-consent-loading = リクエストを読み込んでいます…
auth-consent-intro = があなたのアカウントへのアクセスを求めています。
auth-consent-approved = 管理者承認済みのクライアント
auth-consent-unapproved = 不明なクライアント — 管理者の承認なし
auth-consent-requesting = 要求している権限
auth-consent-redirects-to = リダイレクト先
auth-consent-for-resource = 対象リソース
auth-consent-deny = 拒否
auth-consent-allow = 許可
auth-consent-error = 決定を記録できませんでした

# -- Machine translation (2026-11): added to match en ---------------------
auth-login-rate-limited = サインインの試行回数が多すぎます。しばらくしてからもう一度お試しください。
auth-retry-countdown = { $seconds } 秒後に再試行できます
auth-upstream-protocol = サインインを中止しました。メールサーバーから、このサーバーが解釈できない応答が返されました。後でもう一度お試しいただくか、管理者にお知らせください。
auth-account-disabled = このアカウントは管理者により無効にされています。管理者が再び有効にするまでサインインできません。
auth-signin-not-allowed = このアドレスはこのサーバーにサインインできません。管理者にお問い合わせください。
auth-signin-not-provisioned = このアドレスのアカウントはまだ存在せず、新しいアカウントは自動作成されません。管理者に追加を依頼してください。
auth-reference = 参照ID: { $id }
auth-domain-hint = { $domain } ではユーザー名だけでサインインできます。
auth-passkey-sign-in = パスキーでサインイン
auth-passkey-failed = パスキーでサインインできませんでした。もう一度お試しいただくか、パスワードをご利用ください。
auth-remembered = 以前のサインイン
auth-remembered-help = このデバイスで正常に使用されたアドレスです。選択するとアドレスのみが入力されます。
auth-forget-address = このデバイスから { $address } を削除
auth-login-timeout = サインインがタイムアウトしました。自動での再試行は行っていません。接続を確認し、準備ができたらもう一度お試しください。
auth-verification-title = サインイン確認
auth-verification-loading = サインイン確認を読み込んでいます…
auth-verification-error = 確認を完了できませんでした。サインインする前にもう一度確認をやり直してください。
auth-verification-required = もう一度試す前に、新しいサインイン確認を完了してください。
auth-identity-title = ID情報の共有
auth-identity-sign-in = このID要求を続けるには、もう一度サインインしてください。
auth-identity-sign-out = Mailwoman からサインアウト
auth-identity-expired = このID要求は期限切れか、内容が変更されました。要求元のアプリケーションに戻り、最初からやり直してください。
auth-identity-retry = 再試行
auth-identity-disclosure = このアプリケーションは、以下のID情報を受け取ります。この許可によってメールへのアクセスが付与されることはありません。
auth-identity-scope-openid = 固定のID識別子
auth-identity-scope-profile = 利用可能なプロフィール情報
auth-identity-scope-email = サインイン中のメールアドレス (独立して確認済みであることは保証されません)
auth-identity-scope-offline_access = この Mailwoman セッションが有効な間、サインインから最長15日間IDアクセスを更新
auth-identity-deny = キャンセル
auth-identity-approve = IDアクセスを許可
auth-identity-grants = IDを使用しているアプリケーション
auth-identity-grants-help = アプリケーションのIDアクセスを取り消します。メール API の個別の権限は削除されません。
auth-identity-no-grants = 現在、IDの同意があるアプリケーションはありません。
auth-identity-revoke = IDアクセスを取り消す
auth-maintenance = ウェブメールはメンテナンスのため一時停止しています。後でもう一度お試しください。
