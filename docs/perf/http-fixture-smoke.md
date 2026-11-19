# HTTP load report

Target: isolated ephemeral loopback fixture

Scenario: simulated authenticated transport; NOT Mailwoman application capacity; runtime: v24.14.1. bounded open-loop fixed arrivals; includes generator drops.

| Fixture protection | Offered RPS | Attempted | Accepted | Completed RPS | Accepted RPS | p50/p95/p99 ms | Errors | 429 | Timeouts | Capacity/scheduler drops | Lag p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | --- | ---: |
| off (simulated) | 20000 | 256 | 256 | 839.3 | 839.3 | 37/104/108 | 0 | 0 | 0 | 5448/296 | 60 |
| off (simulated) | 50000 | 642 | 642 | 2083.2 | 2083.2 | 21/29/30 | 0 | 0 | 0 | 14353/5 | 31 |
| off (simulated) | 100000 | 897 | 897 | 2815.9 | 2815.9 | 14/22/26 | 0 | 0 | 0 | 28674/429 | 18 |
| on (simulated) | 20000 | 704 | 470 | 2227.1 | 1486.9 | 14/75/76 | 234 | 234 | 0 | 5130/166 | 18 |
| on (simulated) | 50000 | 832 | 554 | 2720.8 | 1811.7 | 13/31/31 | 278 | 278 | 0 | 14063/105 | 19 |
| on (simulated) | 100000 | 961 | 641 | 3036.1 | 2025.1 | 13/24/24 | 320 | 320 | 0 | 28849/190 | 16 |

Dropped arrivals have no response latency and must be evaluated alongside percentiles. This report alone is not a production capacity claim.
