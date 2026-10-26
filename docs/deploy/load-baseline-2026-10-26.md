# Local HTTP stress baseline, 2026-10-26

This is functional stress evidence, **not production capacity acceptance**.
The server was an earlier native test binary on a shared Windows development
machine with other compiler/test processes active. It does not include all later
security commits. The exact binary SHA256 was
`2648C73BDBDEE127E2ABAF96CE7C2F12984D83BF7C4C26CD872E522F87D2BDAB`.
The target was isolated loopback port 42214, SQLite, one fixture mailbox, and
Node v24.14.1. The production proxy/TLS path was not involved.

## Public reads

Four 30-second stages, plus a three-second warm-up. Health, application-shell and
public-configuration reads completed **166,253 requests**, with no unexpected
responses, timeouts or post-run health failure.

| Concurrent clients | Requests | Requests/s | p95 ms | p99 ms |
|---|---:|---:|---:|---:|
| 1 | 8,825 | 294.1 | 9 | 10 |
| 8 | 55,672 | 1,855.4 | 11 | 23 |
| 32 | 50,124 | 1,668.1 | 61 | 86 |
| 64 | 51,632 | 1,715.2 | 118 | 145 |

## Authenticated mail reads and admission control

The first four 30-second stages completed 13,139 requests, including 3,986 HTTP
429 refusals at higher concurrency. Those correctly failed the initial
zero-refusal gate. Inspection established that all failures were the existing
process-wide four-request JMAP admission ceiling; there were no 5xx responses or
timeouts. This does **not** establish fairness across multiple accounts.

A separate four-stage, 20-seconds-per-stage run explicitly accepted and counted
throttles. It completed 8,652 requests, with 2,638 HTTP 429 refusals, no unexpected
responses or timeouts, and a healthy post-run probe.

| Concurrent clients | Requests | Total requests/s | p95 ms | p99 ms | HTTP 429 |
|---|---:|---:|---:|---:|---:|
| 1 | 453 | 22.6 | 73 | 77 | 0 |
| 8 | 1,845 | 91.5 | 183 | 189 | 246 |
| 32 | 3,087 | 148.3 | 857 | 882 | 1,134 |
| 64 | 3,267 | 151.7 | 1,803 | 1,829 | 1,258 |

Total throughput includes refused work and must not be presented as successful
mail-operation capacity. These stages mixed health, current-user/session reads,
mailbox enumeration and paged message queries against one authenticated account.

## Malformed requests and parser smoke tests

Three 15-second stages at 1, 8 and 32 concurrent clients completed 7,442 mixed
health and invalid JSON/JMAP-envelope requests. Every malformed request was
refused as expected; there were no unexpected responses, timeouts or post-run
health failures. p95 was 33, 49 and 187 ms respectively.

The five native MIME smoke tests also passed: fixed corpus, every corpus prefix,
adversarial inputs, deeply nested multipart and pseudo-random byte streams.
This is a bounded smoke run, not a coverage-guided fuzz campaign.

## Reproduction and remaining gates

Use [load-testing.md](load-testing.md). Raw reports from this run are in the
ignored local `.orchestration/reports/codex-http-load-*.json` artifacts. The
harness's three own tests verify percentiles, application-level errors under
HTTP 200 and separately visible expected throttling.

Repeat against the final integrated **release** image, with a populated mailbox
matrix, multiple accounts, SQLite/PostgreSQL and the real HTTP/2/HTTP/3 proxy
paths. Add long soaks, open-loop arrival-rate tests, resource measurements and
transport framing/slow-client acceptance. Compare protective policies off/on
without changing production settings. TLS/PROXY handshake and HTTP header/body
bounds were identified as separate runtime work; adaptive protection remains a
separate disabled-by-default feature under implementation.
