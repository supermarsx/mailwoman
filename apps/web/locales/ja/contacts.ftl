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
contacts-favorites = お気に入り
contacts-groups = グループ
contacts-new-group = 新しいグループ
contacts-new-group-name = 新しいグループ名
contacts-group-name-placeholder = グループ名
contacts-create = 作成
contacts-data = データ
contacts-import = インポート…

# -- Business card -----------------------------------------------------------
contacts-card = 連絡先 { $name }
contacts-favorited = お気に入り済み
contacts-favorite-action = お気に入り
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

# -- Machine translation (2026-11): added to match en ---------------------
contacts-address-card = 連絡先の詳細
contacts-address-recipients = 宛先を表示
contacts-address-save = 連絡先を保存
contacts-address-compose = メールを送信
contacts-address-unsaved = このアドレスは連絡先に保存されていません。
contacts-address-book = アドレス帳
contacts-address-lookup-failed = 連絡先情報を読み込めませんでした。もう一度お試しください。
contacts-address-save-failed = 連絡先を保存できませんでした。もう一度お試しください。
contacts-address-saved = 連絡先を保存しました。
contacts-address-loading = 連絡先を読み込んでいます…
contacts-editor-person = 氏名と勤務先
contacts-editor-reach = メールと電話
contacts-editor-other = メモと設定
contacts-editor-save-error = 連絡先を保存できませんでした。変更内容は残っています。もう一度お試しください。
contacts-group-editor-help = アドレス帳にグループを作成し、連絡先カードからメンバーを追加します。
contacts-group-details = グループの詳細
contacts-group-save-error = グループを保存できませんでした。変更内容は残っています。もう一度お試しください。
contacts-edit-group = 連絡先グループを編集
contacts-edit-named-group = グループ { $name } を編集
contacts-group-name = グループ名
contacts-group-edit-help = このグループの名前を更新します。アドレス帳とメンバーは変更されません。
contacts-back-to-list = 連絡先に戻る
contacts-membership-title = グループのメンバー
contacts-membership-help = グループを選び、メンバーを追加または削除します。変更はその都度保存されます。
contacts-membership-no-groups = グループがまだありません。作成してメンバーを追加しましょう。
contacts-membership-pick = グループを選ぶとメンバーが表示されます。
contacts-membership-of = { $name } のメンバー
contacts-membership-members = メンバー
contacts-membership-add = 名前またはアドレスを入力してメンバーを追加
contacts-membership-add-group = グループ名を入力してこの連絡先を追加
contacts-membership-failed = 一部のメンバー変更を保存できませんでした。一覧には保存済みの内容が表示されています。
contacts-membership-manage = グループのメンバーを管理
contacts-import-source = ソース
contacts-import-book = アドレス帳
contacts-import-review = 確認
contacts-import-empty = ファイルを選ぶか vCard または CSV のテキストを貼り付けて、「プレビュー」を選択してください。見つかった連絡先がここに表示され、不要なものは除外できます。
contacts-merge-sets = 重複の可能性
contacts-merge-set = { $name } の重複
contacts-merge-keep = 残すカード
contacts-merge-done = { $name } に統合しました。
contacts-address-actions = 連絡先の操作
contacts-address-save-to = 連絡先に保存
contacts-address-goto = 連絡先へ移動
contacts-address-copy = 連絡先情報をコピー
contacts-address-copied = 連絡先情報をコピーしました。
contacts-address-copy-failed = 連絡先情報をコピーできませんでした。
contacts-share = 連絡先を共有
contacts-share-email = 連絡先をメールで送信
contacts-share-copy = vCard としてコピー
contacts-share-download = .vcf をダウンロード
contacts-share-native = 共有…
contacts-share-copied = vCard をコピーしました。
contacts-share-copy-failed = vCard をコピーできませんでした。
contacts-share-downloaded = vCard をダウンロードしました。
contacts-share-failed = 連絡先を共有できませんでした。
contacts-share-attach-failed = .vcf ファイルを添付できなかったため、vCard はメッセージ本文に入っています。
contacts-address-saved-favorite = 連絡先を保存し、お気に入りに追加しました。
contacts-favorite-add = お気に入りに追加
contacts-favorite-remove = お気に入りから削除
contacts-favorite-save = 保存してお気に入りに追加
contacts-favorite-added = { $name } をお気に入りに追加しました。
contacts-favorite-removed = { $name } をお気に入りから削除しました。
contacts-favorite-failed = お気に入りを更新できませんでした。
contacts-select-row = { $name } を選択
contacts-selected-count = { $count } 件選択中
contacts-select-clear = 選択を解除
contacts-bulk-actions = 選択した連絡先の操作
contacts-bulk-favorites-added = { $count } 件をお気に入りに追加しました。
contacts-bulk-favorites-removed = { $count } 件をお気に入りから削除しました。
contacts-row-menu = { $name } の操作
contacts-favorites-empty = お気に入りはまだありません。連絡先の星印を使って追加できます。

# -- Machine translation (2026-11): added to match en ---------------------
contacts-membership-count = { $count ->
   *[other] { $count } 人のメンバー
}

# Machine translation refresh (2026-11-09). Not reviewed by a native speaker.
contacts-save-already = { $name } はすでに連絡先にあります。

contacts-save-already-badge = すでに保存されています

contacts-save-already-many = すべての { $count } の連絡先がすでに保存されています。

contacts-save-automated = これは自動送信者 (返信なし、通知) のように見えます。

contacts-save-book = アドレス帳

contacts-save-book-readonly = { $name } (読み取り専用)

contacts-save-cancel = キャンセル

contacts-save-choose = 連絡先を選択してください…

contacts-save-contacts = このメッセージから連絡先を保存します

contacts-save-dialog-title = このメッセージから連絡先を保存します

contacts-save-email = 電子メール

contacts-save-failed = 連絡先を保存できませんでした。もう一度やり直してください。

contacts-save-group = グループ

contacts-save-group-none = グループなし

contacts-save-loading = アドレスを読み込み中…

contacts-save-name = 名前

contacts-save-new = 新しい連絡先を保存する

contacts-save-no-book = 新しい連絡先を取得できるアドレス帳はありません。

contacts-save-no-sender = このメッセージには送信者アドレスがありません。

contacts-save-note = { $date } のメッセージから保存されました

contacts-save-nothing = 保存する人がいません: 誰もがすでに連絡先に登録されています、あなたですか、あるいは自動化されているように見えます。

contacts-save-open = 開く

contacts-save-open-contact = オープンコンタクト

contacts-save-open-link = オープンコンタクト

contacts-save-own = これはあなた自身のアドレスの 1 つです。

contacts-save-partial = { $total } 件中 { $saved } 件の連絡先を保存しました。他の人は救われませんでした。

contacts-save-pick = { $email } を節約

contacts-save-recipients = すべての受信者を保存する

contacts-save-removed = 保存された連絡先が削除されました。

contacts-save-role-bcc = Bcc

contacts-save-role-cc = Cc

contacts-save-role-from = から

contacts-save-role-replyTo = 返信先

contacts-save-role-sender = 送信者

contacts-save-role-to = に

contacts-save-saved-heading = すでに連絡先にあります

contacts-save-saved-many = { $count } 件の連絡先を { $book } に保存しました

contacts-save-saved-one = { $name } から { $book } に保存しました

contacts-save-sender = 送信者を連絡先として保存

contacts-save-settings-book = アドレス帳

contacts-save-settings-book-default = デフォルトのアドレス帳

contacts-save-settings-enabled = このメッセージから連絡先を保存する

contacts-save-settings-enabled-help = メッセージ メニューとリーダーの [その他] メニューに項目を追加します。

contacts-save-settings-note = 日付を記したメモを追加する

contacts-save-settings-note-help = メモには「メッセージを保存しました」と日付が続きます。

contacts-save-settings-reply-to = リストと通知の送信者に Reply-To を使用する

contacts-save-settings-reply-to-help = これらの送信者の From ではなく Reply-To アドレスを保存します。

contacts-save-settings-scope = ダイアログで最初にチェックを入れます

contacts-save-settings-scope-all = メッセージに参加している皆さん

contacts-save-settings-scope-sender = 送信者のみ

contacts-save-settings-sender = オファー 送信者を連絡先として保存

contacts-save-settings-sender-help = メッセージ メニューのワンクリック項目。

contacts-save-settings-skip = 自動送信者をスキップする

contacts-save-settings-skip-help = チェックを入れない限り、非応答アドレス、通知アドレス、および返送アドレスは除外されます。

contacts-save-settings-title = メッセージから連絡先を保存する

contacts-save-show-automated = 自動アドレスと応答なしアドレスを表示する

contacts-save-submit = { $count ->
       *[other] { $count } 件の連絡先を保存
    }

contacts-save-undo = 元に戻す
