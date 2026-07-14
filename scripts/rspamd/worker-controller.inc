# rspamd controller worker (t10-e14). Exposes the controller HTTP on 11334 for the
# trainer path (/learnham + /learnspam). Inside the isolated CI compose network every
# client is treated as `secure_ip`, so no controller password is needed for the train
# legs. GTUBE/verdict CLASSIFICATION uses the scan worker on 11333 (worker-normal.inc);
# the controller is only exercised by the optional train* legs.
bind_socket = "*:11334";
secure_ip = "0.0.0.0/0";
