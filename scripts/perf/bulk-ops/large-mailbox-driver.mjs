#!/usr/bin/env node
// Large-mailbox cohort driver for the native load fixture (t60).
// Selected by MW_NATIVE_LOAD_DRIVER=large-mailbox with MW_NATIVE_LOAD_PROFILE=large-mailbox:
// 1,000 users = 2 x 100,000 + 8 x 20,000 + 990 x 50 messages, 30% unread, 2% flagged.
//
// Phases:
//  1. isolated: each heavy group alone against each big account, sequential,
//     with fixture diagnostics before/after -> per-request documents loaded,
//     hits enumerated and process CPU-ms.
//  2. baseline: ordinary users only (identity + list batch) at a fixed offered rate.
//  3. heavy: the same ordinary traffic while every big account runs the heavy
//     groups in a closed loop.
//  4. recovery: ordinary users only again.
// The headline number is the ordinary users' p99 in "heavy" against "baseline".
// Every response is validated against the seeded counts; any failure makes the
// report FAILED and the process exit 1.
import { readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

export const CONTRACT = JSON.parse(readFileSync(new URL('../../../apps/web/src/api/native-read-intent-contract.json', import.meta.url), 'utf8'));
const USING = ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail'];
const PAGE = 50;

export function settings(env = process.env) {
  const int = (name, def, low, high) => {
    const raw = env[name];
    if (raw === undefined || raw === '') return def;
    if (!/^\d+$/.test(raw) || Number(raw) < low || Number(raw) > high) throw new Error(`${name} must be an integer ${low}..${high}`);
    return Number(raw);
  };
  return {
    phaseSeconds: int('MW_LARGE_DRIVER_PHASE_SECONDS', 60, 5, 3600),
    ordinaryRate: int('MW_LARGE_DRIVER_ORDINARY_RATE', 50, 1, 5000),
    isolatedRequests: int('MW_LARGE_DRIVER_ISOLATED_REQUESTS', 5, 1, 100),
    heavyLoopsPerAccount: int('MW_LARGE_DRIVER_HEAVY_LOOPS', 1, 1, 16),
    maxOutstanding: int('MW_LARGE_DRIVER_MAX_OUTSTANDING', 256, 1, 4096),
  };
}

const listGet = (p, ref = 'q') => ['Email/get', { accountId: p.accountId, '#ids': { resultOf: ref, name: 'Email/query', path: '/ids' }, properties: CONTRACT.properties }, 'g'];
const query = (p, filter, extra = {}) => ['Email/query', { accountId: p.accountId, filter, sort: [{ property: 'receivedAt', isAscending: false }], limit: PAGE, calculateTotal: true, ...extra }, 'q'];
const page = (p, total) => Math.min(PAGE, Math.max(0, total));

/** Heavy groups aimed at the big accounts, each with its exact expectation. */
export function heavyGroups(p) {
  const quarterly = p.plantedQuarterly;
  return {
    'unread-page': { calls: [query(p, { inMailbox: p.inboxId, notKeyword: '$seen' }), listGet(p)], total: p.unreadCount, rows: page(p, p.unreadCount) },
    'flagged-page': { calls: [query(p, { inMailbox: p.inboxId, hasKeyword: '$flagged' }), listGet(p)], total: p.flaggedCount, rows: page(p, p.flaggedCount) },
    'size-sort-page': { calls: [query(p, { inMailbox: p.inboxId }, { sort: [{ property: 'size', isAscending: false }] }), listGet(p)], total: p.messageCount, rows: page(p, p.messageCount) },
    'text-search-page-1': { calls: [query(p, { inMailbox: p.inboxId, text: 'quarterly' }), listGet(p)], total: quarterly, rows: page(p, quarterly) },
    'text-search-page-5': { calls: [query(p, { inMailbox: p.inboxId, text: 'quarterly' }, { position: 4 * PAGE }), listGet(p)], total: quarterly, rows: page(p, quarterly - 4 * PAGE) },
    // apps/web/src/api/workspaceMail.ts: the unread count.
    'unread-count': { calls: [['Email/query', { accountId: p.accountId, filter: { inMailbox: p.inboxId, notKeyword: '$seen' }, limit: 1, calculateTotal: true }, 'q']], total: p.unreadCount, rows: null },
    'list-batch': { calls: [query(p, { inMailbox: p.inboxId }), listGet(p)], total: p.messageCount, rows: page(p, p.messageCount) },
  };
}

export function validate(value, expectation) {
  const responses = value?.methodResponses;
  if (!Array.isArray(responses) || responses.some(r => r[0] === 'error')) return false;
  if (responses[0]?.[1]?.total !== expectation.total) return false;
  if (expectation.rows !== null && responses[1]?.[1]?.list?.length !== expectation.rows) return false;
  return true;
}

export function percentiles(samples) {
  const sorted = [...samples].sort((a, b) => a - b);
  const at = p => (sorted.length ? sorted[Math.min(sorted.length, Math.max(1, Math.ceil((p / 100) * sorted.length))) - 1] : null);
  return { count: sorted.length, p50: at(50), p95: at(95), p99: at(99), max: sorted.length ? sorted[sorted.length - 1] : null };
}

async function post(handoff, p, calls) {
  const response = await fetch(`${handoff.base}/jmap/api`, {
    method: 'POST',
    headers: { cookie: p.cookie, 'x-csrf-token': p.csrfToken, 'x-forwarded-for': p.clientAddress, origin: handoff.base, 'content-type': 'application/json' },
    body: JSON.stringify({ using: USING, methodCalls: calls }),
  });
  const text = await response.text();
  return { status: response.status, value: response.status === 200 ? JSON.parse(text) : null };
}
async function identity(handoff, p) {
  const response = await fetch(`${handoff.base}/api/me`, { headers: { cookie: p.cookie, 'x-forwarded-for': p.clientAddress } });
  await response.arrayBuffer();
  return { status: response.status };
}
async function diagnostics(handoff) {
  const response = await fetch(`${handoff.base}${handoff.diagnostics.path}`, { headers: { 'x-fixture-token': handoff.diagnostics.token } });
  if (response.status !== 200) throw new Error(`diagnostics ${response.status}`);
  const value = await response.json();
  return {
    documentFetches: value.searchIndex?.documentFetches ?? 0,
    listProjectionDocumentReads: value.searchIndex?.listProjectionDocumentReads ?? 0,
    hitsMaterialised: value.searchIndex?.writeAndHitCounts?.hitsMaterialised ?? 0,
    cpuSeconds: value.processCpuSeconds ?? null,
  };
}
export function counterDelta(before, after, requests) {
  const per = key => (after[key] - before[key]) / Math.max(1, requests);
  return {
    requests,
    hitsEnumeratedPerRequest: per('hitsMaterialised'),
    storedDocumentsLoadedPerRequest: per('hitsMaterialised') + per('documentFetches') + per('listProjectionDocumentReads'),
    cpuMsPerRequest: before.cpuSeconds === null || after.cpuSeconds === null ? null : ((after.cpuSeconds - before.cpuSeconds) * 1000) / Math.max(1, requests),
    scope: 'process-wide counters around sequential requests with no other traffic',
  };
}

async function ordinaryTraffic(handoff, ordinary, cfg, seconds, record) {
  const interval = 1000 / cfg.ordinaryRate;
  const start = performance.now();
  let outstanding = 0, index = 0, dropped = 0;
  const pending = new Set();
  while (performance.now() - start < seconds * 1000) {
    const scheduled = start + index * interval;
    const wait = scheduled - performance.now();
    if (wait > 0) await new Promise(resolve => setTimeout(resolve, wait));
    const p = ordinary[index % ordinary.length];
    const kind = index % 2 === 0 ? 'identity' : 'list';
    index++;
    if (outstanding >= cfg.maxOutstanding) { dropped++; continue; }
    outstanding++;
    const task = (async () => {
      try {
        let ok;
        if (kind === 'identity') ok = (await identity(handoff, p)).status === 200;
        else {
          const expectation = { total: p.messageCount, rows: page(p, p.messageCount) };
          const r = await post(handoff, p, [query(p, { inMailbox: p.inboxId }), listGet(p)]);
          ok = r.status === 200 && validate(r.value, expectation);
        }
        record(kind, performance.now() - scheduled, ok);
      } catch { record(kind, performance.now() - scheduled, false); } finally { outstanding--; }
    })();
    pending.add(task); task.finally(() => pending.delete(task));
  }
  await Promise.all(pending);
  return { offered: index, dropped };
}

async function heavyLoop(handoff, p, stop, record) {
  const groups = Object.entries(heavyGroups(p));
  let i = 0;
  while (!stop.done) {
    const [name, group] = groups[i++ % groups.length];
    const begun = performance.now();
    try {
      const r = await post(handoff, p, group.calls);
      if (r.status === 429) {
        // A request-protection refusal is a limit, not a wrong answer: count it and back off.
        record(`${name}:throttled`, performance.now() - begun, true);
        await new Promise(resolve => setTimeout(resolve, 1000));
        continue;
      }
      record(name, performance.now() - begun, r.status === 200 && validate(r.value, group));
    } catch { record(name, performance.now() - begun, false); }
  }
}

function collector() {
  const samples = new Map();
  const record = (kind, ms, ok) => {
    if (!samples.has(kind)) samples.set(kind, { latencies: [], failures: 0 });
    const s = samples.get(kind);
    if (ok) s.latencies.push(ms); else s.failures++;
  };
  const report = () => Object.fromEntries([...samples].map(([k, s]) => [k, { latencyMs: percentiles(s.latencies), failures: s.failures }]));
  return { record, report, failures: () => [...samples.values()].reduce((n, s) => n + s.failures, 0) };
}

export async function run(handoff, cfg) {
  const big = handoff.principals.filter(p => p.cohort !== 'ordinary' && p.cohort !== 'default');
  const ordinary = handoff.principals.filter(p => p.cohort === 'ordinary');
  if (!big.length || !ordinary.length) throw new Error('handoff is not the large-mailbox cohort profile');
  const report = { kind: 'mw-large-mailbox-driver-v1', settings: cfg, workloadProfile: handoff.workloadProfile, cohorts: { big: big.map(p => ({ cohort: p.cohort, messages: p.messageCount, unread: p.unreadCount, flagged: p.flaggedCount })), ordinary: ordinary.length }, phases: {} };
  let failures = 0;
  // 1. isolated per-group cost, one big account of each size.
  const isolated = {};
  for (const p of [big.find(b => b.cohort === 'big-100000'), big.find(b => b.cohort === 'big-20000')].filter(Boolean)) {
    for (const [name, group] of Object.entries(heavyGroups(p))) {
      const before = await diagnostics(handoff);
      const latencies = [];
      let bad = 0;
      for (let n = 0; n < cfg.isolatedRequests; n++) {
        const begun = performance.now();
        const r = await post(handoff, p, group.calls);
        latencies.push(performance.now() - begun);
        if (!(r.status === 200 && validate(r.value, group))) bad++;
      }
      const after = await diagnostics(handoff);
      failures += bad;
      isolated[`${p.cohort}/${name}`] = { latencyMs: percentiles(latencies), failures: bad, expectedTotal: group.total, counters: counterDelta(before, after, cfg.isolatedRequests) };
    }
  }
  report.phases.isolated = isolated;
  // 2-4. ordinary latency without, with and after the heavy load.
  for (const phase of ['baseline', 'heavy', 'recovery']) {
    const ordinaryStats = collector();
    const heavyStats = collector();
    const stop = { done: false };
    const loops = phase === 'heavy' ? big.flatMap(p => Array.from({ length: cfg.heavyLoopsPerAccount }, () => heavyLoop(handoff, p, stop, heavyStats.record))) : [];
    const before = await diagnostics(handoff);
    const offered = await ordinaryTraffic(handoff, ordinary, cfg, cfg.phaseSeconds, ordinaryStats.record);
    stop.done = true;
    await Promise.all(loops);
    const after = await diagnostics(handoff);
    failures += ordinaryStats.failures() + heavyStats.failures();
    report.phases[phase] = { seconds: cfg.phaseSeconds, ordinary: ordinaryStats.report(), ordinaryOffered: offered, heavy: phase === 'heavy' ? heavyStats.report() : null, processCpuSeconds: after.cpuSeconds === null ? null : after.cpuSeconds - before.cpuSeconds };
  }
  const p99 = (phase, kind) => report.phases[phase].ordinary[kind]?.latencyMs?.p99 ?? null;
  report.headline = {
    ordinaryIdentityP99Ms: { baseline: p99('baseline', 'identity'), heavy: p99('heavy', 'identity'), recovery: p99('recovery', 'identity') },
    ordinaryListP99Ms: { baseline: p99('baseline', 'list'), heavy: p99('heavy', 'list'), recovery: p99('recovery', 'list') },
    latencyNote: 'ordinary latency is measured from the scheduled send time (open loop), so it includes any queueing in the driver',
  };
  report.status = failures === 0 ? 'OK' : 'FAILED';
  report.validationFailures = failures;
  return report;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const [handoffPath, reportPath] = process.argv.slice(2);
  if (!handoffPath || !reportPath) { console.error('usage: large-mailbox-driver.mjs <handoff.json> <report.json>'); process.exit(2); }
  const handoff = JSON.parse(readFileSync(handoffPath, 'utf8'));
  const report = await run(handoff, settings());
  // The report carries no cookies, tokens or account ids.
  writeFileSync(reportPath, JSON.stringify(report, null, 2));
  console.error(`large-mailbox driver: ${report.status}; ordinary list p99 ${JSON.stringify(report.headline.ordinaryListP99Ms)}`);
  process.exit(report.status === 'OK' ? 0 : 1);
}
