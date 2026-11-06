-- Persisted performance aggregates, time-bucketed, so the admin statistics survive a restart.
-- One row per instance, metric, bucket and bounded dimension set. The latency
-- distribution is a mergeable log-bucket sketch (counts per bucket), never a
-- percentile, so buckets and instances merge exactly. No person, address, path,
-- message or folder name is stored: dimensions are fixed role classes only.
CREATE TABLE metrics_bucket (
    instance_id TEXT NOT NULL,
    metric TEXT NOT NULL,
    bucket_start BIGINT NOT NULL,
    resolution_seconds INTEGER NOT NULL CHECK(resolution_seconds IN (60, 3600, 86400)),
    dim_domain TEXT NOT NULL DEFAULT '',
    dim_folder_class TEXT NOT NULL DEFAULT '',
    dim_client_kind TEXT NOT NULL DEFAULT '',
    dim_device_class TEXT NOT NULL DEFAULT '',
    dim_backend TEXT NOT NULL DEFAULT '',
    sample_count BIGINT NOT NULL CHECK(sample_count >= 0),
    sum_value BIGINT NOT NULL CHECK(sum_value >= 0),
    min_value BIGINT NOT NULL,
    max_value BIGINT NOT NULL,
    sketch BYTEA NOT NULL,
    flushed_at BIGINT NOT NULL,
    PRIMARY KEY(instance_id, metric, bucket_start, resolution_seconds, dim_domain, dim_folder_class, dim_client_kind, dim_device_class, dim_backend)
);
CREATE INDEX metrics_bucket_range ON metrics_bucket(resolution_seconds, bucket_start);
CREATE INDEX metrics_bucket_metric ON metrics_bucket(metric, bucket_start);
