# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: calendar. Source: ../en/calendar.ftl

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = カレンダー
calendar-prev = 前へ
calendar-next = 次へ
calendar-today = 今日
calendar-views = カレンダーの表示
calendar-new-event = 新しい予定
calendar-import = インポート
calendar-export = エクスポート
calendar-import-file = カレンダーファイルをインポート

# -- View names --------------------------------------------------------------
calendar-view-day = 日
calendar-view-3day = 3 日
calendar-view-work-week = 稼働日
calendar-view-week = 週
calendar-view-month = 月
calendar-view-tri-month = 四半期
calendar-view-schedule = スケジュール
calendar-view-agenda = 予定リスト
calendar-view-year = 年

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = カレンダー
calendar-toggle = { $name } の表示を切り替え
calendar-color-for = { $name } の色
calendar-add-calendar = カレンダーを追加
calendar-new-calendar-name = 新しいカレンダー
calendar-holidays-heading = 祝日
calendar-subscribe-holidays = 祝日を購読
calendar-add-region = 地域を追加…
calendar-synced = 同期済みのカレンダー

# -- Grid / views ------------------------------------------------------------
calendar-all-day = 終日
calendar-all-day-full = 終日
calendar-more = 他 { $count } 件
calendar-conflict = 重複
calendar-no-events-30 = 今後 30 日間に予定はありません。
calendar-events-count = { $count ->
    [zero] 予定なし
   *[other] { $count } 件の予定
}
calendar-month-grid = { $month } の月表示
calendar-time-grid = 時間グリッド
calendar-cell = { $date }、{ $events }
calendar-announce = { $date }、{ $events }
calendar-event-at = { $time } { $title }
calendar-event-allday = 終日: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = 新しい予定
calendar-editor-edit = 予定を編集
calendar-invitation = 招待
calendar-invited = 招待されています ({ $status })。
calendar-accept = 承諾
calendar-tentative = 仮承諾
calendar-decline = 辞退
calendar-counter = 別の日時を提案…
calendar-propose-start = 新しい開始日時を提案
calendar-send-counter = 提案を送信
calendar-overlaps = この予定は他の { $count ->
   *[other] { $count } 件の予定
} と重なっています。
calendar-field-title = タイトル
calendar-untitled = (タイトルなし)
calendar-field-calendar = カレンダー
calendar-field-all-day = 終日
calendar-field-start = 開始
calendar-field-duration = 所要時間 (分)
calendar-field-location = 場所
calendar-field-shows-as = 表示方法
calendar-shows-busy = 予定あり
calendar-shows-free = 空き時間
calendar-field-status = ステータス
calendar-status-confirmed = 確定
calendar-status-tentative = 仮
calendar-status-cancelled = キャンセル済み
calendar-repeats = 繰り返し
calendar-frequency = 頻度
calendar-freq-daily = 毎日
calendar-freq-weekly = 毎週
calendar-freq-monthly = 毎月
calendar-freq-yearly = 毎年
calendar-every = 間隔
calendar-interval = 間隔
calendar-ends = 終了
calendar-end-mode = 終了条件
calendar-end-never = なし
calendar-end-count = 回数指定
calendar-end-until = 日付指定
calendar-occurrences = 回数
calendar-until-date = 終了日
calendar-reminders = リマインダー
calendar-reminder-before = { $min } 分前
calendar-remove-reminder = リマインダー { $min } を削除
calendar-add-reminder = リマインダーを追加
calendar-reminder-none = + リマインダー
calendar-reminder-5 = 5 分
calendar-reminder-15 = 15 分
calendar-reminder-30 = 30 分
calendar-reminder-60 = 1 時間
calendar-reminder-1440 = 1 日
calendar-attendees = 出席者
calendar-add-attendee = 出席者を追加
calendar-attendee-invalid = メールアドレスではありません。name@example.com のようにアドレスを 1 つ入力してください。
calendar-save-failed = 予定は保存されませんでした。サーバーに拒否されました。
calendar-remove-attendee = { $email } を削除
calendar-notes = メモ

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = { $email } の役割
calendar-attendee-cutype-for = { $email } の種類
calendar-role-chair = 議長
calendar-role-required = 必須
calendar-role-optional = 任意
calendar-role-non-participant = 参加者以外
calendar-cutype-individual = 個人
calendar-cutype-group = グループ
calendar-cutype-resource = リソース
calendar-cutype-room = 会議室
calendar-partstat-needs-action = 未返信
calendar-partstat-accepted = 承諾
calendar-partstat-declined = 辞退
calendar-partstat-tentative = 仮承諾

# -- Schedule view (distinct from Agenda) ------------------------------------
calendar-schedule-row = { $start }～{ $end }: { $title }
calendar-schedule-free = { $gap } 空き
calendar-gap-hm = { $h } 時間 { $m } 分
calendar-gap-h = { $h } 時間
calendar-gap-m = { $m } 分

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = { $count ->
   *[other] { $count } 件の重複
}を解決
calendar-resolver-title = 重複を解決
calendar-resolver-none = 解決が必要な重複はありません。
calendar-resolver-pick = 重複
calendar-resolver-pair-n = { $total } 件中 { $n } 件目: { $a } と { $b }
calendar-resolver-time = 時間
calendar-resolver-overlap = 重複 { $start } – { $end }
calendar-resolver-reschedule = 後の予定を変更
calendar-resolver-shorten = 前の予定を短縮
calendar-resolver-tentative = 後の予定を仮にする
calendar-resolver-double-book = 重複登録 (空き時間にする)
calendar-resolver-keep = 両方残す
calendar-events-count-people = { $count ->
    [zero] 出席者なし
   *[other] 出席者 { $count } 人
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-caption = 重複がある日の予定あり時間 (1 時間ごと)。
calendar-fb-whose = カレンダー
calendar-fb-own = 自分
calendar-fb-own-only = 出席者の空き状況は表示されません。サーバーは自分のカレンダーの予定あり時間のみを報告します。
calendar-fb-unavailable = 予定あり時間を読み込めませんでした。
calendar-fb-cell = { $principal } の { $hour }: { $status }
calendar-fb-busy = 予定あり
calendar-fb-tentative = 仮
calendar-fb-free = 空き

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = 予定をすばやく追加
calendar-quick-add-placeholder = すばやく追加 (例: Lunch Friday 1pm)
calendar-quick-add-btn = 追加
calendar-quick-add-do = すばやく追加

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = カテゴリ
calendar-add-category = カテゴリを追加
calendar-category-placeholder = カテゴリを追加
calendar-remove-category = カテゴリ { $name } を削除
calendar-filter-heading = 絞り込み
calendar-filter-category = カテゴリで絞り込み
calendar-filter-category-placeholder = 表示するカテゴリ

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = 添付ファイル
calendar-attachment-title = 添付ファイル名
calendar-attachment-title-placeholder = 名前 (任意)
calendar-attachment-uri = 添付ファイルのリンク
calendar-attachment-add = 添付ファイルを追加
calendar-remove-attachment = 添付ファイル { $name } を削除

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = URL で購読
calendar-subscribe-url = カレンダーの URL
calendar-subscribe-add = 購読
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = そのアドレスのカレンダーを取得できなかったため、追加されませんでした。
calendar-subscribe-hol-many-locations = この祝日ファイルには複数の地域が含まれています。ダウンロードしてから「インポート」を使い、必要な地域を選んでください。
calendar-subscribe-refresh = { $name } を更新
calendar-subscribe-refresh-btn = 更新
calendar-subscribe-refresh-failed = カレンダーフィードを取得できませんでした。予定は変更されていません。
calendar-import-failed = そのファイルをカレンダーとして読み込めませんでした。何もインポートされていません。
calendar-import-done = { $count ->
   *[other] { $count } 件の予定をインポートしました。
}
calendar-import-locations-title = インポートする地域を選択
calendar-import-locations-intro = このファイルには { $count } 地域の祝日が含まれています。選択した地域のみがインポートされます。
calendar-import-locations-filter = 地域を絞り込み
calendar-import-locations-list = ファイル内の地域
calendar-import-locations-none = 条件に一致する地域はありません。
calendar-import-locations-picked = { $count ->
   *[other] { $count } 地域を選択しました。
}
calendar-import-locations-import = 選択した地域をインポート
calendar-export-failed = カレンダーをエクスポートできませんでした。
calendar-quick-add-failed = その予定を追加できませんでした。
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = 共有
calendar-share-for = { $name } を共有
calendar-share-title = { $name } を共有
calendar-share-intro = 共有した相手はこのカレンダーを閲覧できます。予定の追加と編集を許可するには、読み書きのアクセス権を付与してください。
calendar-share-people = 共有相手
calendar-share-empty = このカレンダーは誰とも共有されていません。
calendar-share-add = ユーザーを追加
calendar-share-new-access = 新しいユーザーのアクセス権
calendar-share-access-for = { $principal } のアクセス権
calendar-share-remove = { $principal } を削除
calendar-share-read = 読み取り専用
calendar-share-readwrite = 読み書き
