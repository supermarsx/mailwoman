# Inbound connection negotiation limits

The built-in listener bounds TLS handshakes and PROXY protocol header negotiation. These are startup settings: changing a value requires restarting the process. The configuration catalogue and Diagnostics → Deployment security show the configured values. They do not measure limits at an external reverse proxy. Writing these startup settings from the administrator UI is not implemented.

| Environment variable | Default | Accepted range | Applies to |
| --- | ---: | ---: | --- |
| `MW_TLS_HANDSHAKE_TIMEOUT_MS` | 10000 | 1000–60000 | Complete external-certificate or ACME TLS handshake |
| `MW_TLS_MAX_HANDSHAKES` | 128 | 1–1024 | Concurrent handshakes inside the TLS listener |
| `MW_PROXY_HEADER_TIMEOUT_MS` | 5000 | 1000–30000 | Complete PROXY header classification and parsing |
| `MW_PROXY_MAX_HANDSHAKES` | 128 | 1–1024 | Concurrent PROXY header negotiations |

Unset variables use the defaults. Empty, malformed or out-of-range values refuse listener startup. These limits are fixed; no adaptive increase is enabled.

A silent TLS peer occupies one bounded handshake slot until its deadline. Other available slots continue to accept handshakes. When every slot is occupied, admission pauses until a slot completes or expires; the kernel TCP backlog remains subject to operating-system settings. The listener owns pending handshake futures, so dropping it closes their sockets. PROXY negotiation uses a bounded task set and a bounded completed-connection queue; shutdown aborts those tasks too. PROXY trust and required-header checks still apply before TLS or HTTP.

These bounds do not constitute protection against malformed TCP/IP packets, distributed denial of service, or an external proxy with different limits. HTTP request-header/body deadlines and HTTP/2 stream budgets require their own settings and acceptance tests; the handshake limits do not replace them. Keep operating-system and front-proxy admission controls in place.
