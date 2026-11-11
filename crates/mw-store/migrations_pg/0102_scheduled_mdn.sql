-- Pending read receipts carry only message identity and delivery time.
CREATE TABLE scheduled_mdn (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    email_id TEXT NOT NULL REFERENCES messages(stable_id) ON DELETE CASCADE,
    requested_by TEXT NOT NULL,
    send_at BIGINT NOT NULL,
    lease_until BIGINT,
    retry_at BIGINT,
    PRIMARY KEY (account_id, email_id)
);
CREATE INDEX scheduled_mdn_due ON scheduled_mdn(send_at, lease_until);

CREATE TABLE mdn_sender_schedule (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    address TEXT NOT NULL,
    send_at BIGINT NOT NULL,
    PRIMARY KEY (account_id, address)
);
CREATE INDEX mdn_sender_schedule_expiry ON mdn_sender_schedule(send_at);
