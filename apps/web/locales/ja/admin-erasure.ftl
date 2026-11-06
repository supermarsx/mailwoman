# Machine translation (t32, 2026-10). Not reviewed by a native speaker.
# Locale: ja · Module: admin-erasure. Source: ../en/admin-erasure.ftl


# -- Machine translation (2026-11): added to match en ---------------------
admin-nav-erasure-coverage = 削除とスクラブ
compliance-card-erasure-coverage = 監査履歴とログ内の特定の人の識別子を置き換え、削除済みの人がまだ残っているバックアップを確認します。
admin-erasure-title = 削除とスクラブ
admin-erasure-intro = 「人をスクラブ」は、監査履歴とローテーション済みのログファイル内にある人のアドレスとアカウント ID を、安定したキー付き仮名に置き換えます。監査ハッシュチェーンは再封印されるため、検証は引き続き成功します。削除とスクラブのたびにトゥームストーンも残るため、古いバックアップを復元しても、それらの人は再び削除されます。
admin-erasure-docs = テーブルごとに何を削除、エクスポート、保持するかの一覧は docs/admin/erasure-coverage.md にあります。
admin-erasure-scrub = 人をスクラブ
admin-erasure-scrub-note = 先にプレビューを実行してください。変更される行数とログ行数を数えるだけで、何も変更しません。
admin-erasure-address = アドレス
admin-erasure-preview = プレビュー
admin-erasure-run = 人をスクラブ
admin-erasure-dry-note = プレビューのみです。何も変更されていません。
admin-erasure-done-note = スクラブしました。この人は { $pseudonym } として表示されます。
admin-erasure-nothing = この人の名前が記載された履歴はありません。
admin-erasure-col-table = テーブルまたはファイル
admin-erasure-col-rows = 行数
admin-erasure-confirm-title = この人を履歴からスクラブしますか？
admin-erasure-confirm-body = その人の名前が記載された監査エントリとログ行が、仮名で書き換えられます。この操作は元に戻せません。キーがなければ、仮名からアドレスを復元することはできません。
admin-erasure-settings = 設定
admin-erasure-keys = 仮名キー
admin-erasure-keys-note = 仮名は現在のキー世代で作られます。ローテーションすると世代が追加され、古い世代は、上で設定した保持期間によって削除されるまで残ります。
admin-erasure-rotate = キーをローテーション
admin-erasure-keys-none = キーはまだ作成されていません。最初のスクラブで作成されます。
admin-erasure-col-gen = 世代
admin-erasure-col-created = 作成日時
admin-erasure-col-state = 状態
admin-erasure-key-current = 現在
admin-erasure-key-retired = 退役済み・保持
admin-erasure-key-destroy = 退役済み・{ $when } に削除
admin-erasure-chain = 監査チェーン
admin-erasure-chain-note = 各監査エントリには、直前のエントリのハッシュが含まれています。スクラブはエントリを書き換えてチェーンを再封印し、古い先頭ハッシュと新しい先頭ハッシュを記録します。
admin-erasure-seal = 新しいエントリを封印
admin-erasure-chain-ok = チェーンの検証に成功しました。{ $rows } 件のエントリが封印済み、{ $unsealed } 件が封印待ちです。
admin-erasure-chain-bad = チェーンの検証に失敗しました。最初の不正なエントリは位置 { $seq } です。
admin-erasure-col-time = 時刻
admin-erasure-col-range = 位置
admin-erasure-backups = バックアップ
admin-erasure-backups-note = バックアップは作成時にトゥームストーンのウォーターマークを記録します。現在のウォーターマークは { $watermark } です。
admin-erasure-col-label = ラベル
admin-erasure-col-location = 場所
admin-erasure-col-watermark = ウォーターマーク
admin-erasure-col-marker = 削除の基準
admin-erasure-col-actions = 操作
admin-erasure-backups-none = 登録されたバックアップはありません。
admin-erasure-contains-erased = 削除済みの対象者を含む（{ $count }）
admin-erasure-no-marker = なし
admin-erasure-forget = 登録解除
admin-erasure-register = バックアップを登録
