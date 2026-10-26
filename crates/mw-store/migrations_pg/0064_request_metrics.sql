-- Bounded response-ready request aggregates; no addresses, paths or identities.
CREATE TABLE request_metrics_bucket (
    instance_id TEXT NOT NULL,
    bucket_seconds BIGINT NOT NULL CHECK(bucket_seconds IN (60,3600)),
    bucket_start BIGINT NOT NULL,
    observed_from BIGINT NOT NULL,
    observed_to BIGINT NOT NULL,
    counts_json TEXT NOT NULL,
    PRIMARY KEY(instance_id,bucket_seconds,bucket_start),
    CHECK(observed_from >= bucket_start AND observed_to >= observed_from AND observed_to <= bucket_start + bucket_seconds)
);
CREATE INDEX request_metrics_range ON request_metrics_bucket(bucket_seconds,bucket_start);
