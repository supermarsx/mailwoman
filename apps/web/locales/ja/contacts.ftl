# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: contacts. Source: ../en/contacts.ftl

contacts-title = 連絡先
contacts-search = 連絡先を検索
contacts-new = 新しい連絡先
contacts-find-duplicates = 重複を検索
contacts-loading = 読み込んでいます…
contacts-none = 連絡先はありません。
contacts-list = 連絡先リスト
contacts-favorite = { $name } をお気に入りに追加
contacts-export-vcard = vCard をエクスポート
contacts-export-csv = CSV をエクスポート
contacts-select-hint = 連絡先を選択すると、カードが表示されます。

# -- Sidebar -----------------------------------------------------------------
contacts-books-groups = アドレス帳とグループ
contacts-address-books = アドレス帳
contacts-all = すべての連絡先
contacts-favorites = ★ お気に入り
contacts-groups = グループ
contacts-new-group = 新しいグループ
contacts-new-group-name = 新しいグループ名
contacts-group-name-placeholder = グループ名
contacts-create = 作成
contacts-data = データ
contacts-import = インポート…

# -- Business card -----------------------------------------------------------
contacts-card = 連絡先 { $name }
contacts-favorited = ★ お気に入り済み
contacts-favorite-action = ☆ お気に入り
contacts-email = メール
contacts-phone = 電話
contacts-dates = 日付
contacts-groups-label = グループ
contacts-notes = メモ
contacts-security-key = セキュリティキー
contacts-key-on-file = 鍵/証明書が登録されています (PGP/S-MIME 対応までは表示のみ)。
contacts-no-key = 鍵は登録されていません。
contacts-directory-security = ディレクトリのセキュリティ
contacts-member-of = 所属グループ
contacts-group-membership = { $name } のメンバー

# -- Editor ------------------------------------------------------------------
contacts-new-contact = 新しい連絡先
contacts-edit-contact = 連絡先を編集
contacts-full-name = 氏名
contacts-given = 名
contacts-given-placeholder = 名
contacts-surname = 姓
contacts-surname-placeholder = 姓
contacts-organization = 組織
contacts-job-title = 役職
contacts-job-title-placeholder = 役職
contacts-email-n = メール { $n }
contacts-email-label = メール { $n } のラベル
contacts-email-label-placeholder = 勤務先
contacts-remove-email = メール { $n } を削除
contacts-add-email = メールを追加
contacts-phone-n = 電話 { $n }
contacts-remove-phone = 電話 { $n } を削除
contacts-add-phone = 電話を追加
contacts-security-key-label = セキュリティキー (内容は解釈されません)
contacts-security-key-placeholder = PGP 鍵 / S-MIME 証明書 (表示のみ)
contacts-favorite-word = お気に入り

# -- Import dialog -----------------------------------------------------------
contacts-import-title = 連絡先をインポート
contacts-imported = { $count ->
   *[other] { $count } 件の連絡先
}をインポートしました。
contacts-import-hint = vCard (.vcf) または CSV を貼り付けるか、ファイルを選択してください。形式は自動で判別されます。
contacts-import-file = ファイルをインポート
contacts-paste = vCard または CSV を貼り付け
contacts-preview = プレビュー
contacts-map-columns = 列の対応付け
contacts-col = 列
contacts-maps-to = 対応先
contacts-sample = サンプル
contacts-map-column = 列 { $header } の対応付け
contacts-preview-count = プレビュー ({ $count })
contacts-import-preview = インポートのプレビュー
contacts-no-name = (名前なし)
contacts-no-email = メールなし
contacts-import-n = { $count ->
   *[other] { $count } 件の連絡先
}をインポート

# -- Merge dialog ------------------------------------------------------------
contacts-merge-title = 重複を統合
contacts-merge-review-hint = 統合後のカードを確認してください。他のカードは削除済みとして残ります (元に戻せます)。
contacts-merged-preview = 統合後のプレビュー
contacts-emails = メール
contacts-phones = 電話
contacts-merge-confirm = 連絡先を統合
contacts-no-duplicates = 重複は見つかりませんでした。
contacts-possible-duplicates = 重複の可能性: { $count } 件
contacts-review-merge = 統合を確認
