# Multi-client native debug smoke

The 20 November 2026 local native-engine fixture completed with **100 distinct
durable users, sessions and simulated client addresses** receiving validated
successful responses in both measured phases. These are forwarded client
addresses behind the isolated trusted loopback proxy, not 100 physical machines.
Production authentication, account/domain routing and admission policies remained
active. No production mailboxes or upstream servers were contacted.

The candidate worktree ran on Windows in the unoptimized debug profile, with
four Tokio workers, explicit 8 MiB debug worker stacks, a child Node generator
in the same resource group, 32 outstanding HTTP requests and two-second phases.
It was not CPU/RAM constrained to the deployment target. The workload mixed
authenticated identity reads, Mailbox/get, Email/query, Email/get and native
no-change refreshes against two seeded messages per user.

| Offered RPS | Successful responses | Successful RPS including drain | Successful RPS within window | Successful users / clients | Dropped arrivals | Errors / throttles / timeouts |
| ---: | ---: | ---: | ---: | --- | ---: | --- |
| 500 | 316 | 148.8 | 142 | 100 / 100 | 684 | 0 / 0 / 0 |
| 1,000 | 346 | 163.7 | 157 | 100 / 100 | 1,654 | 0 / 0 / 0 |

Both phases also observed all 100 users and clients completing successfully
within their scheduled windows. Offered arrivals above the bounded generator's
outstanding-request capacity were dropped and are not counted as successful
throughput. This smoke verifies the multi-user/client model, not production
capacity or the 40,000 successful RPS / 10,000 live-user target. Its different
engine mode, population and candidate source prevent attributing a speedup or
slowdown relative to the older single-user proxy smoke.

The run caught and corrected two fixture setup problems: insufficient debug
worker stacks and an invalid seeded TLS mode that caused normal startup routing
checks to revoke sessions. It also exposed prefix bias in the old rotation:
only 65–72 users were reached before overload phases ended. The corrected
rotation interleaves endpoint types while spreading early requests across the
population; every endpoint visits every user over a complete cycle.

Validation passed: 22 Node tests, the three native fixture tests (including the
10,000-address production resolver/spoof-rejection test), the rerun of the actual
HTTP campaign, and Dockerfile frontend checks. The complete Linux release
capacity campaign and 10,000 live WebSocket sessions remain unverified.

Raw local artifacts:

- `.orchestration/reports/perf-investigation/multi-client-debug-balanced/native-100.json`
- `.orchestration/reports/perf-investigation/multi-client-debug-balanced/native-100.md`
- `.orchestration/reports/perf-investigation/native-multi-client-debug-balanced.log`
- `.orchestration/reports/perf-investigation/multi-client-node-tests.log`

Use the [native fixture instructions](./native-http-fixtures.md) to reproduce.
