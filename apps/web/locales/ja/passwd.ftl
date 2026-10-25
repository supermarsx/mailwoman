# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: passwd. Source: ../en/passwd.ftl

# -- Region / heading --------------------------------------------------------
passwd-region-label = パスワードの変更
passwd-heading = パスワードの変更

# -- Forced change -----------------------------------------------------------
passwd-force-change = 続行する前に、管理者の指示によりパスワードを変更する必要があります。

# -- Fields ------------------------------------------------------------------
passwd-current-label = 現在のパスワード
passwd-new-label = 新しいパスワード
passwd-confirm-label = 新しいパスワード（確認）

# -- Password match indicator (text, never colour alone) ---------------------
passwd-match-ok = 新しいパスワードが一致しています。
passwd-match-no = 新しいパスワードはまだ一致していません。

# -- Policy rules (also reused verbatim in validation error text) ------------
passwd-rule-min-length = { $count } 文字以上
passwd-rule-uppercase = 英大文字
passwd-rule-lowercase = 英小文字
passwd-rule-digit = 数字
passwd-rule-symbol = 記号

# -- Zero-access re-wrap notice ----------------------------------------------
passwd-rewrap-notice = このアカウントはゼロアクセスです。変更が適用される前にリカバリーフレーズが表示されます。問題が起きてもデータにアクセスできるよう、必ず保存してください。

# -- Actions -----------------------------------------------------------------
passwd-continue = 続行
passwd-submit = パスワードを変更

# -- Recovery-phrase phase ---------------------------------------------------
passwd-recovery-heading = リカバリーフレーズを保存してください
passwd-recovery-prose = このフレーズを書き留めて、安全な場所に保管してください。新しいパスワードを失ってもデータを復元できるよう、パスワード変更の前に表示しています。このフレーズはサーバーには保存されません。
passwd-recovery-ack-label = リカバリーフレーズを保存しました
passwd-recovery-ack-text = リカバリーフレーズを安全な場所に保存しました。

# -- Done phase --------------------------------------------------------------
passwd-done = パスワードを変更しました。
passwd-done-reencrypt = 保存されているサーバーの認証情報を新しいパスワードで再暗号化しました。
passwd-done-rewrap = ゼロアクセス鍵を新しいパスワードで再ラップしました。

# -- Validation / error messages ---------------------------------------------
passwd-error-enter-current = 現在のパスワードを入力してください
passwd-error-mismatch = 新しいパスワードと確認用のパスワードが一致しません
passwd-error-policy = 新しいパスワードには次が必要です：{ $rules }
passwd-error-ack-first = 先にリカバリーフレーズを保存したことを確認してください
passwd-error-prepare-recovery = リカバリーフレーズを準備できませんでした
passwd-error-change = パスワードを変更できませんでした
passwd-error-session-ended = セッションが終了しました。もう一度サインインしてください。
passwd-error-not-configured = このサーバーはパスワード変更に対応するよう設定されていません。管理者に連絡してください。
passwd-error-contact-admin = サーバーは変更を完了できませんでした。管理者に連絡してください。

# -- Forced change screen ----------------------------------------------------
passwd-forced-title = 続行するにはパスワードを変更してください
passwd-forced-signed-in-as = { $username } としてサインインしています。
passwd-forced-explain = 管理者がこのアカウントのパスワード変更を必須にしています。変更が完了するまで、メール、カレンダー、連絡先は利用できません。
passwd-forced-backend-note = 現在のパスワードが正しいのに変更が拒否される場合、このサーバーではパスワードを変更できない可能性があります。管理者に連絡してください。管理者はパスワード変更を設定するか、この要件を解除できます。
passwd-forced-sign-out = サインアウト
passwd-forced-still-required = サーバーは変更を受け付けましたが、このアカウントでは引き続きパスワード変更が必要です。管理者に連絡してください。
passwd-forced-reload-failed = パスワードは変更されましたが、アカウントを読み込めませんでした。接続を確認して、もう一度お試しください。
passwd-forced-retry = 再試行
