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
calendar-fb-whose = カレンダー
calendar-fb-own = 自分
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

# -- Machine translation (2026-11): added to match en ---------------------
calendar-mini = 日付ナビゲーター
calendar-event-range = { $title }、{ $start } から { $end } まで
calendar-busy-free = 空き時間として表示
calendar-event-with-status = { $event }({ $status })
calendar-more-label = { $date } にあと { $count } { $count ->
   *[other] 件の予定
  }があります
calendar-fb-caption-people = 競合する日の予定あり時間を、自分と招待された各ユーザーについて時間ごとに表示します。
calendar-fb-unknown = { $principal } の空き状況を取得できませんでした。
calendar-fb-loading = 空き状況を読み込み中…
calendar-import-itip-message = このファイルはメールで送信された返信またはキャンセルです。適用するには、添付元のメッセージを開いてください。何もインポートされていません。
calendar-import-skipped = { $count ->
   *[other] { $count } 件の予定はすでにカレンダーにあるため、そのままにしました。
}
calendar-invite-title = カレンダーの招待
calendar-invite-sender = 送信者: { $sender }
calendar-invite-add = カレンダーに追加
calendar-invite-update = 予定の更新を適用
calendar-invite-cancel = キャンセルを適用
calendar-invite-reply = 出席者の返信を記録
calendar-invite-new = この招待をカレンダーに追加できます。
calendar-invite-newer = 主催者が更新された予定を送信しました。
calendar-invite-current = カレンダーにはすでにこのバージョンがあります。
calendar-invite-stale = このメッセージは、カレンダー内の予定より古いものです。
calendar-invite-cancelled = 主催者がこの予定をキャンセルしました。
calendar-invite-response = 出席者から返信が届きました。
calendar-invite-not-organizer = 送信者は保存されている主催者ではありません。何も変更されません。
calendar-invite-not-attendee = 送信者は一致する出席者ではありません。何も変更されません。
calendar-invite-unknown = このメッセージは、カレンダーにない予定に関するものです。
calendar-invite-unsupported = この提案は自動的に適用できません。
calendar-invite-no-reply = 返信は送信せず、カレンダーを更新します。
calendar-invite-applied = カレンダーを更新しました。
calendar-invite-unchanged = 予定は変更されませんでした。招待の現在の状態を確認してください。
calendar-invite-apply-failed = 招待を適用できませんでした。もう一度お試しください。
calendar-invite-load-failed = 招待を確認できませんでした。
calendar-invite-retry = 再試行
calendar-field-time-zone = 予定のタイムゾーン
calendar-field-time-zone-help = Europe/Lisbon のような IANA 名を使用します。他の場所から表示しても、予定はこのタイムゾーンを保持します。
calendar-time-invalid = 有効なタイムゾーンと現地時刻を入力してください。時計が進む際に存在しない現地時刻があります。
calendar-mini-prev = 前の月
calendar-mini-next = 次の月
calendar-card-region-event = カレンダーの予定
calendar-card-kind-request = 会議の招待
calendar-card-kind-update = 更新された招待
calendar-card-kind-cancel = キャンセルされた会議
calendar-card-kind-reply = 会議への返信
calendar-card-kind-counter = 新しい時間の提案
calendar-card-kind-publish = { $count ->
   *[other] { $count } 件のカレンダー予定
}
calendar-card-when = 日時
calendar-card-where = 場所
calendar-card-organizer = 主催者
calendar-card-attendees = 出席者
calendar-card-all-day = { $when }(終日)
calendar-card-organizer-time = 主催者の時刻: { $when }({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] 毎日
   *[other] { $interval } 日ごと
}
calendar-card-repeat-weekly = { $interval ->
    [one] 毎週
   *[other] { $interval } 週間ごと
}
calendar-card-repeat-weekly-on = { $interval ->
    [one] 毎週{ $days }
   *[other] { $interval } 週間ごとの{ $days }
}
calendar-card-repeat-monthly = { $interval ->
    [one] 毎月
   *[other] { $interval } か月ごと
}
calendar-card-repeat-monthly-day = { $interval ->
    [one] 毎月 { $day } 日
   *[other] { $interval } か月ごとの { $day } 日
}
calendar-card-repeat-yearly = { $interval ->
    [one] 毎年
   *[other] { $interval } 年ごと
}
calendar-card-repeat-until = { $rule }、{ $date } まで
calendar-card-repeat-count = { $rule }、{ $count ->
    [one] 1 回
   *[other] { $count } 回
}
calendar-card-repeat-except = { $count ->
    [one] { $rule }(1 日をスキップ)
   *[other] { $rule }({ $count } 日をスキップ)
}
calendar-card-repeat-custom = カスタムスケジュールで繰り返し
calendar-card-join = { $provider } に参加
calendar-card-organizer-is = 主催者: { $who }
calendar-card-people-all = { $count } 人すべてを表示
calendar-card-people-fewer = 表示を減らす
calendar-card-rsvp-accepted = 承諾
calendar-card-rsvp-tentative = 仮承諾
calendar-card-rsvp-declined = 辞退
calendar-card-rsvp-none = 未返信
calendar-card-in-calendar = カレンダーに登録済み
calendar-card-cancelled = キャンセル済み
calendar-card-open = カレンダーで開く
calendar-card-agenda = { $day } のカレンダー
calendar-card-agenda-title = 今日の予定
calendar-card-agenda-this-event = この予定
calendar-card-agenda-unavailable = この日のカレンダーを読み込めませんでした。
calendar-card-conflicts = { $count ->
    [one] { $titles } と重複しています
   *[other] { $count } 件の予定と重複しています: { $titles }
}
calendar-card-no-conflicts = カレンダーに重複する予定はありません
calendar-card-invited-as = { $address } として招待されました。
calendar-card-not-invited = この招待には、あなたのアドレスが含まれていません。カレンダーに追加することはできますが、返信はできません。
calendar-card-not-invited-short = この招待には、あなたのアドレスが含まれていません。
calendar-card-out-of-date = この招待は古くなっています。より新しいバージョンがすでにカレンダーにあります。
calendar-card-replied = { $who }さんは{ $answer ->
    [accepted] 承諾しました。
    [tentative] 仮承諾しました。
    [declined] 辞退しました。
   *[other] まだ返信していません。
}
calendar-card-counter = { $who } さんが新しい時間を提案しました。提案は自動的には適用されません。
calendar-card-todos-skipped = { $count ->
    [one] このファイルにはタスクが 1 件含まれていますが、追加されません。
   *[other] このファイルにはタスクが { $count } 件含まれていますが、追加されません。
}
calendar-card-respond = 招待に返信
calendar-card-accept = 承諾
calendar-card-tentative = 仮承諾
calendar-card-decline = 辞退
calendar-card-propose = 新しい時間を提案
calendar-card-propose-start = 新しい開始時刻
calendar-card-propose-send = 提案を送信
calendar-card-propose-invalid = 有効な新しい開始時刻を入力してください。
calendar-card-send-response = { $organizer } に返信を送信
calendar-card-comment = コメント(任意)
calendar-card-calendar = カレンダー
calendar-card-events = このファイル内の予定
calendar-card-add-one = 追加
calendar-card-add-all = すべて追加({ $count })
calendar-card-added = { $count ->
    [one] カレンダーに追加しました。
   *[other] { $count } 件の予定をカレンダーに追加しました。
}
calendar-card-remove = カレンダーから削除
calendar-card-removed = カレンダーから削除しました。
calendar-card-record-reply = 出席者の状況を更新
calendar-card-answered-sent = { $answer }。{ $organizer } に返信を送信しました。
calendar-card-answered-not-sent = { $answer }。カレンダーは更新されましたが、返信を送信できませんでした。
calendar-card-answered-silently = { $answer }。返信は送信されていません。
calendar-card-proposal-sent = { $organizer } に提案を送信しました。
calendar-card-not-sent = メッセージを送信できませんでした。
calendar-card-more = { $count ->
    [one] ほか 1 件
   *[other] ほか { $count } 件
}
calendar-card-attendee-summary = { $count ->
    [one] 出席者 1 人 · 承諾 { $accepted }、仮承諾 { $tentative }、辞退 { $declined }
   *[other] 出席者 { $count } 人 · 承諾 { $accepted }、仮承諾 { $tentative }、辞退 { $declined }
}
