# rspamd normal (scan) worker for the spam-classify live-E2E (t10-e14, SPEC §10.8).
# Binds the HTTP scan worker on all interfaces so the host test / jail fetcher can POST
# /checkv2. This matches the spam-rspamd guest's compiled default endpoint
# (http://rspamd:11333). GTUBE detection is built into rspamd — no training needed for
# the deterministic spam fixture.
bind_socket = "*:11333";
