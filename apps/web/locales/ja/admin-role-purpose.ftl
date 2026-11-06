# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-role-purpose. Source: ../en/admin-role-purpose.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-role-purpose-super_admin = 管理者、ロール、セキュリティ、メールルーティング、サービス設定を含む、すべての機能にわたる完全な管理権限です。このロールは、デプロイ全体に責任を持つ信頼できる運用担当者のみに限定してください。
admin-role-purpose-domain_admin = 日常的なユーザーとドメインの管理です。ユーザーの作成、更新、無効化、削除、クォータ、ユーザー機能、ドメインのサインインルール、設定、ブランディングの管理、セッションの取り消し、禁止の管理ができます。メール診断と利用状況・データのエクスポートも含みます。管理者ロールの管理や上流サーバーの変更は含みません。
admin-role-purpose-helpdesk = アカウントのサポートとインシデントの切り分けです。ユーザー、監査アクティビティ、利用状況の確認、ユーザーフラグの変更、アカウントの無効化、セッションの取り消し、禁止の表示、サインインシミュレーションの実行ができます。ユーザーの作成、削除、クォータの変更、一般設定の書き込みは含みません。
admin-role-purpose-auditor = 確認とレポートです。.read で終わるすべての機能に加え、監査と利用状況のエクスポートが付与されます。設定の変更やアカウントの変更は含みません。エクスポートしたレポートには、管理上の機微な情報が含まれる場合があります。
admin-role-purpose-security_officer = セキュリティ運用です。セキュリティと MFA のポリシー、サインインポリシー、禁止の管理、ユーザーの無効化、セッションや API キーの取り消し、監査アクティビティの確認とエクスポート、認証ログの閲覧、診断の実行ができます。SSO の設定は読み取れますが、SSO の編集や管理者ロールの管理は含みません。
admin-role-purpose-custom = 選択された { $count } 個の権限を持つカスタムロールです。権限は名前ではなく、下の正確な権限リストによって決まります。割り当てる前に、各操作を確認してください。
admin-role-purpose-scope = ドメインのスコープは、このロールを管理者に付与するときに割り当てられます。アカウント単位の許可・拒否の上書きにより、実際のアクセス権が変わることがあります。説明によって追加の権限が付与されることはありません。
admin-role-purpose-editor = この管理者が行う必要のある作業を選び、対応する権限を選択します。閲覧、データのエクスポート、レコードの変更は、それぞれ別の操作です。保存する前に、機能の正確な識別子を確認してください。
admin-role-purpose-selection = これらの説明は、選択した操作を解説するものです。サーバーに送信される権限が広がることはありません。
admin-role-purpose-read = この領域の情報を表示します。
admin-role-purpose-export = この領域から情報をエクスポートします。
admin-role-purpose-revoke = この権限で指定されたセッションまたは認証情報を取り消します。
admin-role-purpose-delete = この権限の対象となるレコードを削除します。
admin-role-purpose-disable = この権限の対象となるアカウントを無効にします。
admin-role-purpose-create = この権限の対象となるレコードを作成します。
admin-role-purpose-specific = この機能で指定された特定の操作を実行します。付与する前に識別子を確認してください。
admin-role-purpose-description = 目的と責任
admin-role-purpose-description-help = このカスタムロールを誰に付与するべきか、どのような作業を担当するかを説明します（1024 文字まで）。このテキストによって権限が変わることはありません。
