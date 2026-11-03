-- PostgreSQL counterpart of 0084_open_tracking_policy_subject.sql.
ALTER TABLE open_tracking ADD COLUMN policy_subject_sealed BYTEA;
