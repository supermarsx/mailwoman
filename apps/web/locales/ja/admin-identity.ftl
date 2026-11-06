# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-identity. Source: ../en/admin-identity.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-identity-title = ID プロバイダー
admin-identity-description = 承認済みのアプリケーションが Mailwoman でサインインできるようにします。これは外部のサインインプロバイダーやメール API アクセスとは別のものです。PKCE 付きの認可コードが必須で、他のグラントは利用できません。
admin-identity-configure = 発行者を設定
admin-identity-new = ID クライアントを追加
admin-identity-edit = クライアントを編集
admin-identity-enabled = 有効
admin-identity-disabled = 無効
admin-identity-all = すべて
admin-identity-discovery = ディスカバリードキュメントを開く
admin-identity-search = クライアントを検索
admin-identity-status = ステータス
admin-identity-clients = ID クライアント
admin-identity-client = クライアント ID 情報
admin-identity-client-id = クライアント ID
admin-identity-name = 名前
admin-identity-method = クライアント認証
admin-identity-public = パブリッククライアント（PKCE、シークレットなし）
admin-identity-pkce = すべてのクライアントで PKCE S256 を使う必要があります。機密クライアントのシークレットはここで生成され、一度だけ表示されます。
admin-identity-rotate-secret = 既存のクライアントシークレットを置き換える
admin-identity-redirects = 登録済みの宛先
admin-identity-redirect-help = 1行に1つ、正確な HTTPS URI を入力してください。ローカルアプリでは、数値のループバック HTTP も使えます。ワイルドカードとフラグメントは拒否されます。
admin-identity-callbacks = 認可コールバック URI
admin-identity-logout-uris = ログアウト後のリダイレクト URI
admin-identity-scopes = 許可する ID スコープ
admin-identity-actions = 操作
admin-identity-delete = クライアントを削除
admin-identity-delete-confirm = { $name } の ID アクセスを削除しますか？既存の ID グラントは機能しなくなり、ID 専用のクライアント登録も削除されます。
admin-identity-empty = 一致する ID クライアントはありません。
admin-identity-cancel = キャンセル
admin-identity-save = 保存
admin-identity-done = 完了
admin-identity-secret = 生成されたクライアントシークレット
admin-identity-secret-once = 今すぐこのシークレットを利用側アプリケーションにコピーしてください。再表示はできません。機密シークレットをブラウザーのコードに含めないでください。
admin-identity-issuer = 公開 HTTPS の発行者 URL
admin-identity-issuer-help = このデプロイの正確な HTTPS オリジンと /oidc パスを使用してください。デプロイのプレフィックスがある場合はそれも含めます。発行者または設定を変更すると、既存の ID グラントは取り消されます。
admin-identity-rotate-key = 署名キーをローテーション
admin-identity-key-help = ローテーションには、管理者の再確認が必要です。以前の公開鍵は 15 分間利用でき、直前に発行されたトークンが短い有効期間を全うできます。
