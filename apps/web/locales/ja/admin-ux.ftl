# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-ux. Source: ../en/admin-ux.ftl

admin-ux-close = 閉じる

# -- Page states (UX11) ---------------------------------------------------------
admin-ux-state-unavailable = 利用できません — { $reason }
admin-ux-unavailable-default = このサーバーはまだこの機能を提供していません。
admin-ux-state-code = エラーコード: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = 開く
admin-ux-list-back = 一覧に戻る

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = 未保存の変更はありません
admin-ux-form-changes =
    { $count ->
       *[other] 未保存の変更 { $count } 件
    }
admin-ux-form-discard = 変更を破棄
admin-ux-form-saving = 保存しています…
admin-ux-form-saved =
    { $count ->
       *[other] { $count } 件の変更を保存しました。
    }
admin-ux-form-fix =
    { $count ->
       *[other] 何も保存されませんでした。下に示した { $count } 個の項目を修正してください。
    }
admin-ux-learn-more = 詳細
admin-ux-learn-more-field = （{ $label } について）
admin-ux-propagation-hot = すべてのサーバープロセスで 30 秒以内に反映されます。
admin-ux-propagation-restart = サーバーの再起動後に反映されます。
admin-ux-secret-set = 値が保存されています。置き換えるには新しい値を入力してください。
admin-ux-secret-unset = 値は保存されていません。

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = 値を入力してください。
admin-ux-err-number = 数値を入力してください。
admin-ux-err-whole = 整数を入力してください。
admin-ux-err-min = { $min } 以上の値を入力してください。
admin-ux-err-max = { $max } 以下の値を入力してください。
admin-ux-err-max-length = { $max } 文字以内で入力してください。
admin-ux-err-url = https:// または http:// で始まる完全なアドレスを入力してください。
admin-ux-err-email = name@example.org のようなメールアドレスを入力してください。
admin-ux-err-pattern = この値は想定された形式ではありません。
admin-ux-err-option = 一覧の選択肢から 1 つ選んでください。
admin-ux-err-out-of-range = 許可された範囲外のため、サーバーがこの値を拒否しました。
admin-ux-err-too-long = 長すぎるため、サーバーがこの値を拒否しました。
admin-ux-err-invalid = サーバーがこの値を拒否しました。確認してもう一度保存してください。
admin-ux-err-pinned = この値はサーバーの環境で設定されているため、ここでは変更できません。
admin-ux-err-locked = この値はより広いスコープでロックされているため、ここでは変更できません。

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = 既定値
admin-ux-source-stored = 保存済み
admin-ux-source-env-pin = { $name } で設定
admin-ux-source-env-fallback = { $name } からのフォールバック
admin-ux-source-build = ビルド時に設定
admin-ux-source-locked = { $scope } でロック
admin-ux-source-read-only = 読み取り専用
admin-ux-source-restart = 再起動が必要
admin-ux-source-env-pin-reason = この値はサーバーの環境で { $name } により設定されています。そこで変更してサーバーを再起動してください。
admin-ux-source-build-reason = この値はサーバーのビルド時に固定されます。
admin-ux-source-locked-reason = { $scope } スコープのロックによりこの値は固定されています。変更するには、そこでロックを解除してください。
admin-ux-source-read-only-reason = コンソールはこの値を表示しますが、変更はできません。設定場所はドキュメントに記載されています。

# -- History (UX8) ------------------------------------------------------------
admin-ux-history-changed-by-at = 最終変更: { $time }、{ $who }
admin-ux-history-changed-by = 最終変更者: { $who }
admin-ux-history-changed-at = 最終変更日時: { $time }
admin-ux-history-never = コンソールからは変更されていません
admin-ux-history-link = 履歴

# -- Revert to default (UX7) --------------------------------------------------
admin-ux-revert-default-is = 既定値: { $value }
admin-ux-revert = 既定値に戻す
admin-ux-revert-field = （{ $label }）
admin-ux-revert-done = 既定値（{ $value }）に戻しました。

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = 編集中に { $who } が { $time } にこれを変更しました。
admin-ux-stale-by = 編集中に { $who } がこれを変更しました。
admin-ux-stale-unknown = 編集中に他のユーザーがこれを変更しました。
admin-ux-stale-choice = 再読み込みして相手の版を表示し自分の編集を破棄するか、自分の編集を残して相手の変更に上書き保存してください。
admin-ux-stale-reload = 再読み込み
admin-ux-stale-keep = 自分の編集を残す

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = この変更の影響を確認しています…
admin-ux-impact-heading = この変更の影響
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
       *[other] ほか { $count } 件
    }
admin-ux-impact-none = ほかに変わるものはありません。
admin-ux-impact-unavailable = 影響のプレビューは利用できません — この変更の影響をサーバーが報告しませんでした。

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = フィルター
admin-ux-stream-pause = 一時停止
admin-ux-stream-resume = 再開
admin-ux-stream-clear = クリア
admin-ux-stream-announce = 新しい項目を読み上げる
admin-ux-stream-connecting = 接続しています…
admin-ux-stream-live = ライブ
admin-ux-stream-ended = 終了
admin-ux-stream-lost = 接続が切れました
admin-ux-stream-reconnect = 再接続
admin-ux-stream-paused =
    { $count ->
        [0] 一時停止中
       *[other] 一時停止中 — 新しい項目 { $count } 件を保留中
    }
admin-ux-stream-no-match = フィルターに一致する項目はありません。

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = 項目

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = 外観
admin-ux-appearance-title = コンソールの外観
admin-ux-appearance-note = ここでの選択は、このブラウザーの管理コンソールにのみ適用されます。
admin-ux-appearance-following = ここで選択するまで、コンソールはこのブラウザーのウェブメールのテーマに従います。ここでの選択は、このブラウザーの管理コンソールにのみ適用されます。
admin-ux-appearance-mode = モード
admin-ux-appearance-mode-light = ライト
admin-ux-appearance-mode-dark = ダーク
admin-ux-appearance-mode-system = システムに合わせる
admin-ux-appearance-theme = テーマ
admin-ux-appearance-accent = アクセントカラー
admin-ux-appearance-light = ライト
admin-ux-appearance-dark = ダーク
admin-ux-appearance-reset = ウェブメールのテーマを使用
