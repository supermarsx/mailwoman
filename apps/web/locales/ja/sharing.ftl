# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: sharing. Source: ../en/sharing.ftl

# -- editor frame ------------------------------------------------------------
sharing-editor-label = メールボックスのアクセス権
sharing-title = このメールボックスにアクセスできるユーザー
sharing-title-named = { $mailbox } にアクセスできるユーザー
sharing-intro = アクセス権はメールサーバーによって適用されます。ここでの変更は、あなたのアカウントにこのメールボックスの管理が許可されている場合のみ有効になります。
sharing-loading = アクセス権を読み込んでいます…
sharing-load-failed = メールボックスのアクセス権一覧を読み込めませんでした。
sharing-op-failed = 変更は保存されませんでした。サーバーが拒否したか、接続できませんでした。

# -- your own access (MYRIGHTS) ----------------------------------------------
sharing-your-access = あなたのアクセス権
sharing-no-access = このメールボックスに対するアクセス権はありません。
sharing-readonly = このメールボックスの管理権限がないため、アクセス権は読み取り専用で表示されます。変更するには管理者に依頼してください。

# -- grants (ACL entries) ----------------------------------------------------
sharing-grants = ユーザーとグループ
sharing-no-grants = まだアクセス権は付与されていません。
sharing-remove = 削除

# -- add a grant -------------------------------------------------------------
sharing-add-heading = アクセス権を付与
sharing-identifier-label = ユーザーまたはグループ
sharing-identifier-placeholder = ユーザー名、グループ、または anyone
sharing-rights-legend = 付与する権限
sharing-add-grant = アクセス権を追加

# -- RFC 4314 rights bits: label + plain-language description -----------------
sharing-right-l-label = 参照
sharing-right-l-desc = フォルダー一覧にこのメールボックスを表示します。
sharing-right-r-label = 読み取り
sharing-right-r-desc = メールボックスを開いてメッセージを読みます。
sharing-right-s-label = 既読状態の保持
sharing-right-s-desc = セッション間でどのメッセージが既読かを記憶します。
sharing-right-w-label = フラグの書き込み
sharing-right-w-desc = 既読と削除以外のフラグ (フラグ付きや返信済みなど) を変更します。
sharing-right-i-label = メッセージの挿入
sharing-right-i-desc = コピーまたは移動でこのメールボックスにメッセージを追加します。
sharing-right-p-label = 投稿
sharing-right-p-desc = 対応している場合、このメールボックスに関連付けられたアドレスにメールを送信します。
sharing-right-k-label = サブメールボックスの作成
sharing-right-k-desc = このメールボックスの中にメールボックスを作成します。
sharing-right-x-label = メールボックスの削除
sharing-right-x-desc = このメールボックス自体を削除または名前変更します。
sharing-right-t-label = メッセージの削除
sharing-right-t-desc = このメールボックス内のメッセージに削除済みの印を付けます。
sharing-right-e-label = 完全削除
sharing-right-e-desc = 削除済みの印が付いたメッセージを完全に削除します。
sharing-right-a-label = 管理
sharing-right-a-desc = このメールボックスにアクセスできるユーザーを変更します。
