# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-users. Source: ../en/admin-users.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-userlist-caption = ユーザー。列見出しでリストを並べ替えられます。
admin-userlist-search = ユーザーを検索
admin-userlist-search-placeholder = 名前、アドレス、エイリアス、ドメイン
admin-userlist-count = { $count ->
   *[other] { $count } 人のユーザー
}
admin-userlist-count-filtered = { $count ->
   *[other] { $total } 人中 { $count } 人が一致
}
admin-userlist-range = { $total } 件中 { $from }–{ $to }
admin-userlist-loading = ユーザーを読み込み中…
admin-userlist-retry = 再試行
admin-userlist-empty-filtered = この検索とフィルターに一致するユーザーはいません。
admin-userlist-compact = コンパクトな行
admin-userlist-columns = 列
admin-userlist-columns-legend = 表示する列
admin-userlist-actions = 操作
admin-userlist-paging = ユーザーのページ
admin-userlist-page-size = 1 ページあたりの行数
admin-userlist-prev = 前へ
admin-userlist-next = 次へ
admin-userlist-sort-by = 並べ替え
admin-userlist-sort-asc = { $column }、昇順
admin-userlist-sort-desc = { $column }、降順
admin-userlist-col-user = ユーザー
admin-userlist-col-domain = ドメイン
admin-userlist-col-status = ステータス
admin-userlist-col-2fa = 2 要素認証
admin-userlist-col-role = コンソールのロール
admin-userlist-col-last-signin = 最終サインイン
admin-userlist-col-last-seen = 最終確認
admin-userlist-col-created = 作成日
admin-userlist-col-quota = 容量
admin-userlist-col-profile = 機能プロファイル
admin-userlist-col-linked = リンクされたメールボックス
admin-userlist-state-active = アクティブ
admin-userlist-state-pending = 未サインイン
admin-userlist-state-blocked = ブロック中
admin-userlist-state-disabled = 無効
admin-userlist-state-deleted = 削除済み
admin-userlist-2fa-passkey = パスキー
admin-userlist-2fa-totp = 認証アプリ
admin-userlist-2fa-none = なし
admin-userlist-signin-never = なし
admin-userlist-signin-at = { $relative }({ $exact })
admin-userlist-seen-never = 記録されたアクティビティなし
admin-userlist-seen-at = 最終確認 { $relative }({ $exact })
admin-userlist-quota-unlimited = 無制限
admin-userlist-profile-default = デフォルト
admin-userlist-profile-via-user = このユーザーに割り当て済み
admin-userlist-profile-via-group = グループから
admin-userlist-profile-via-domain = ドメインから
admin-userlist-more = その他の操作
admin-userlist-more-for = { $account } のその他の操作
admin-userlist-select = { $account } を選択
admin-userlist-select-page = このページのすべてのユーザーを選択
admin-userlist-filters = フィルター
admin-userlist-add-filter = フィルターを追加
admin-userlist-clear-all = すべてクリア
admin-userlist-chip-edit = フィルター { $label } を編集
admin-userlist-chip-remove = フィルター { $label } を削除
admin-userlist-filter-domain = ドメイン
admin-userlist-filter-status = ステータス
admin-userlist-filter-tfa = 2 要素認証
admin-userlist-filter-role = コンソールのロール
admin-userlist-filter-profile = 機能プロファイル
admin-userlist-filter-signin = 最終サインイン
admin-userlist-filter-created = 作成日
admin-userlist-filter-no-choices = 選択できる項目がまだありません。
admin-userlist-filter-remove = フィルターを削除
admin-userlist-apply = 適用
admin-userlist-cancel = キャンセル
admin-userlist-role-any = すべてのコンソールロール
admin-userlist-role-none = コンソールのロールなし
admin-userlist-profile-none = プロファイルなし(デフォルト)
admin-userlist-signin-older = { $days ->
   *[other] { $days } 日より前
}
admin-userlist-signin-within = { $days ->
   *[other] 過去 { $days } 日以内
}
admin-userlist-signin-custom = この日数より前:
admin-userlist-signin-days = 日数
admin-userlist-days-invalid = 1 から 36500 の整数で日数を入力してください。
admin-userlist-created-from = 開始
admin-userlist-created-to = 終了
admin-userlist-created-between = { $from } ~ { $to }
admin-userlist-created-since = { $date } 以降
admin-userlist-created-until = { $date } まで
admin-userlist-dates-invalid = 開始日が終了日より後になっています。
admin-userlist-views = 保存済みビュー
admin-userlist-view-save = このビューを保存…
admin-userlist-view-name = ビューの名前
admin-userlist-view-save-confirm = ビューを保存
admin-userlist-view-delete = ビュー「{ $name }」を削除
admin-userlist-bulk = 選択したユーザーへの操作
admin-userlist-selected = { $count ->
   *[other] { $count } 人を選択中
}
admin-userlist-clear-selection = 選択を解除
admin-userlist-block = ログインをブロック
admin-userlist-unblock = ログインのブロックを解除
admin-userlist-sign-out = すべての場所でサインアウト
admin-userlist-delete = ローカルアクセスを削除
admin-userlist-bulk-profile = 機能プロファイルを割り当て
admin-userlist-bulk-profile-clear = ユーザー自身のプロファイルを削除
admin-userlist-confirm-skipped = { $count ->
   *[other] 選択した { $count } 人のユーザーはスキップされます。この操作は対象外です。
}
admin-userlist-confirm-block = { $count ->
   *[other] { $count } 人のユーザーのログインをブロックしますか?
}
admin-userlist-confirm-block-one = { $account } のログインをブロックしますか?
admin-userlist-confirm-block-body = 新しいサインインは拒否され、現在の Mailwoman セッションは終了します。アップストリームのメールボックスは変更されません。
admin-userlist-confirm-block-action = ログインをブロック
admin-userlist-confirm-unblock = { $count ->
   *[other] { $count } 人のユーザーのログインのブロックを解除しますか?
}
admin-userlist-confirm-unblock-one = { $account } のログインのブロックを解除しますか?
admin-userlist-confirm-unblock-body = ローカルのログインブロックが解除されます。その他のセキュリティ制限は引き続き適用されます。
admin-userlist-confirm-unblock-action = ログインのブロックを解除
admin-userlist-confirm-signout = { $count ->
   *[other] { $count } 人のユーザーをすべての場所でサインアウトしますか?
}
admin-userlist-confirm-signout-one = { $account } をすべての場所でサインアウトしますか?
admin-userlist-confirm-signout-body = これらのユーザーの開いているすべての Mailwoman セッションが終了します。再度サインインできます。
admin-userlist-confirm-signout-action = サインアウト
admin-userlist-confirm-delete = { $count ->
   *[other] { $count } 人のユーザーのローカルアクセスを削除しますか?
}
admin-userlist-confirm-delete-one = { $account } のローカルアクセスを削除しますか?
admin-userlist-confirm-delete-body = 削除されたアクセス記録は保持されるため、ユーザーが自動的に再登録されることはありません。Mailwoman の設定、履歴、アップストリームのメールボックスは保存されたままです。確認のためアドレスを入力してください。
admin-userlist-confirm-delete-action = ローカルアクセスを削除
admin-userlist-confirm-profile = { $count ->
   *[other] { $count } 人のユーザーの機能プロファイルを変更しますか?
}
admin-userlist-confirm-profile-one = { $account } の機能プロファイルを変更しますか?
admin-userlist-confirm-profile-body = プロファイルは各ユーザーに直接割り当てられ、次のリクエストから適用されます。ネイティブ、プロキシ、SSO のメールボックスIDに対応しています。
admin-users-profile-help = 「デフォルト」を選択すると、このユーザーへの直接の割り当てが削除され、グループまたはドメインから継承されたプロファイルが使用されます。
admin-users-profile-ambiguous = このユーザーには複数のサインインIDがあります。機能プロファイルで対象のIDを選択してください。
admin-users-profile-unavailable = 検証済みのメールボックスIDがありません。プロファイルを割り当てるには、このユーザーが先にサインインする必要があります。
admin-userlist-confirm-profile-action = プロファイルを変更
admin-userlist-bulk-done-block = { $count ->
   *[other] { $count } 人のユーザーのログインをブロックしました。
}
admin-userlist-bulk-done-unblock = { $count ->
   *[other] { $count } 人のユーザーのログインのブロックを解除しました。
}
admin-userlist-bulk-done-signout = { $count ->
   *[other] { $count } 人のユーザーをサインアウトしました。
}
admin-userlist-bulk-done-delete = { $count ->
   *[other] { $count } 人のユーザーのローカルアクセスを削除しました。
}
admin-userlist-bulk-done-profile = { $count ->
   *[other] { $count } 人のユーザーの機能プロファイルを変更しました。
}
admin-userlist-bulk-failed = { $done } 件完了、{ $failed } 件失敗: { $reason }
