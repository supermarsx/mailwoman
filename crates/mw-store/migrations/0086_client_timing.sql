-- Aggregate browser load-time histograms, hourly, per metric and device class.
-- No session, account, address, path or user agent is stored: only counts.
CREATE TABLE client_timing_bucket (
    instance_id TEXT NOT NULL,
    bucket_start BIGINT NOT NULL,
    metric TEXT NOT NULL,
    device TEXT NOT NULL,
    sample_count BIGINT NOT NULL CHECK(sample_count >= 0),
    sum_ms BIGINT NOT NULL CHECK(sum_ms >= 0),
    histogram_json TEXT NOT NULL,
    PRIMARY KEY(instance_id,bucket_start,metric,device)
);
CREATE INDEX client_timing_range ON client_timing_bucket(bucket_start);
