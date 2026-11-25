-- 0131's change-log seq triggers are removed (see the PostgreSQL 0133: above the cost
-- gate). The SQLite message triggers are row triggers already; nothing else changes.
DROP TRIGGER mail_data_seq_changes_insert;
DROP TRIGGER mail_data_seq_changes_update;
DROP TRIGGER mail_data_seq_changes_delete;
DROP TRIGGER mail_data_seq_pim_changes_insert;
DROP TRIGGER mail_data_seq_pim_changes_update;
DROP TRIGGER mail_data_seq_pim_changes_delete;
DROP TRIGGER mail_data_seq_crypto_changes_insert;
DROP TRIGGER mail_data_seq_crypto_changes_update;
DROP TRIGGER mail_data_seq_crypto_changes_delete;
DROP TRIGGER mail_data_seq_change_log_checkpoint_insert;
DROP TRIGGER mail_data_seq_change_log_checkpoint_update;
DROP TRIGGER mail_data_seq_change_log_checkpoint_delete;
