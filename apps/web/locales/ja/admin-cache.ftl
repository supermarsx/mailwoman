admin-cache-title = キャッシュのメンテナンス
admin-cache-help = 既定では、60日間使用されていないメール内容のキャッシュは期限切れになります。期限切れのメッセージを開くと、内容が再ダウンロードされます。元のメッセージ、下書き、保護されたローカル内容は保持されます。
admin-cache-expiry = キャッシュされたメッセージの有効期間
admin-cache-orphans = 参照されていない本文と容量の回収
admin-cache-advanced = バッチの上限と実行間隔
admin-cache-settings-help = 変更には通常の管理者設定ポリシー、検証、監査記録が適用されます。
admin-cache-expiry-enabled = キャッシュされたメッセージ本文を自動で期限切れにする
admin-cache-retention-days = 期限切れになるまでの未使用日数
admin-cache-expiry-status = キャッシュ本文の有効期限
admin-cache-expiry-result = スキャン済み: { $scanned } · 切り離されたキャッシュ参照: { $detached } · 保護対象: { $protected }
admin-cache-expiry-help = 元のメールボックスから復元できることが確認された本文のみが対象です。スキャンやヘッダーの更新では未使用期間は更新されません。
admin-cache-inspect = キャッシュストレージを調査
admin-cache-inspect-help = データベースの可観測性を開いて、キャッシュ内容、参照中の本文、孤立したコピーを測定します。
admin-cache-no-status = このサーバープロセスでは期限切れ処理のバッチがまだ完了していません。
admin-cache-settings-unavailable = キャッシュ設定を読み込めませんでした。
admin-cache-change-log = 同期履歴
admin-cache-change-log-help = 既定では、古い変更記録は60日後に期限切れになります。古い同期トークンを使用するクライアントには、完全な再同期が必要であることを示す応答が返されます。
admin-cache-no-sources = 現在、再取得経路が確認された接続済みメールボックスはありません。保護された内容は保持されます。
admin-cache-wal-recycled = 割り当て済みWAL領域を再利用可能にしました。クリーンアップを続行できます。
admin-cache-wal-recovery = クリーンアップはWALの復旧またはチェックポイントを待っています。処理待ちのジャーナルデータは保持されます。

admin-cache-removed-payload = 削除したキャッシュ内容
admin-cache-oversized = 削除を延期した大きな本文
admin-cache-change-removed = 削除した同期履歴の行

admin-cache-parsed-copies = 解析済み本文の削除件数
admin-cache-parsed-bytes = 削除した解析済み本文データ
admin-cache-change-chunked = 分割して整理した同期状態

# Public static representations only; process-local counters.
admin-cache-assets-title = 公開静的アセットの RAM キャッシュ
admin-cache-assets-clear = 静的キャッシュを消去
admin-cache-assets-help = 公開アセットのみを保持します。予算は所有する常駐データが対象で、処理中の応答やプロセス全体のメモリではありません。配備された初期ページとサービスワーカーは再検証間隔中キャッシュに残る場合があります。
admin-cache-assets-clear-help = 消去すると保持エントリが解放されます。実行中の応答は通常どおり完了します。事前読み込みが有効な場合は再び格納されます。カウンターはこのサーバープロセスが対象です。
admin-cache-assets-stale = ポリシーの更新に失敗しました。最後に適用された設定が引き続き有効です。
admin-cache-assets-applied = 設定の適用時刻: { $time }
admin-cache-assets-resident = 所有データ / 予算
admin-cache-assets-entries = エントリ / 上限
admin-cache-assets-file = 所有ファイルの実効上限
admin-cache-assets-loads = 実行中 / 上限、待機中 / 上限
admin-cache-assets-hits = ヒット / 受理した読み込み / 共有待機
admin-cache-assets-evictions = 削除 / 期限切れ / サイズ超過
admin-cache-assets-failures = 混雑による拒否 / 失敗したジョブ
admin-cache-assets-preload = 事前読み込みモード、保持 / スキップ

admin-cache-assets-encoded = 圧縮データ / 所有部分
admin-cache-assets-encoding-loads = 検証済みファイル / 生成したエンコード
admin-cache-assets-encoding-help = 公開静的ファイルの圧縮データはキャッシュ予算を共有します。元のバイト列と照合し、調整したファイルは上限付きのキャッシュ読み込み時に一度エンコードできます。メール、APIデータ、非公開HTMLは対象外です。
