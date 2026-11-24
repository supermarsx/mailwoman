import test from 'node:test';
import assert from 'node:assert/strict';
import { parseRecords, validateRecord } from './records.mjs';
import { renderMarkdown } from './report.mjs';
import { parseArgs, environment, cargoArgs, matrix, label } from './run.mjs';

const inputs = { config: { mailboxMessages: 10000, operationMessages: 1000, batch: 'maxObjectsInSet', securityProfile: 'enforced' }, buildProfile: 'release', sourceRevision: 'abc123', encryption: { sqlcipher: true }, limits: { indexPublication: { ack: 'commit' } }, hardware: { supplied: false } };
const base = { kind: 'mw-bulk-ops-v1', backend: 'sqlite', layer: 'http', inputs, checks: [{ name: 'INBOX-counts', pass: true, detail: {} }], failure: null };
const ok = {
  ...base, scenario: 'destroy', status: 'OK',
  results: {
    messages: 1000, elapsedMs: 2500, messagesPerSecond: 400, requests: { requests: 1, latencyMs: { p50: 2400, p95: 2400 } }, listCorrectAfterLastResponseMs: 12,
    counters: { statements: { perMessage: 3.5 }, searchIndex: { documentsWrittenPerMessage: 0, bodyBytesTokenised: 0 }, upstream: { expunge: 1, store: 0 } },
    concurrentReader: { windows: { before: { inboxFirstPage: { latencyMs: { p95: 8 } } }, during: { inboxFirstPage: { latencyMs: { p95: 90 } } }, after: { inboxFirstPage: { latencyMs: { p95: 9 } } } } },
  },
};
const failed = { ...base, scenario: 'move', status: 'FAILED', results: null, checks: [{ name: 'destination-uids-match-upstream', pass: false, detail: {} }], failure: { reason: 'end-state verification failed' } };
const refused = { ...base, scenario: 'flag-seen', status: 'REFUSED', partialResults: { messages: 5000 }, failure: { stage: 'Email/set after 0 of 10000 messages', refusal: { kind: 'http', status: 429 } } };
const read = { ...base, scenario: 'open-unread', status: 'OK', results: { returned: 50, total: 10000, coldMs: 41.2, warm: { latencyMs: { p50: 30, max: 33 } }, coldCounters: { searchIndex: { hitsMaterialised: 10000 } } } };

test('records are extracted from test output and validated strictly', () => {
  const text = ['noise', `BULK_OPS_RECORD ${JSON.stringify(ok)}`, JSON.stringify(read), 'test result: ok'].join('\n');
  assert.deepEqual(parseRecords(text).map(r => r.scenario), ['destroy', 'open-unread']);
  assert.throws(() => validateRecord({ ...failed, results: { messages: 1 } }), /FAILED record carries results/);
  assert.throws(() => validateRecord({ ...ok, checks: [{ name: 'x', pass: false }] }), /failing check/);
  assert.throws(() => validateRecord({ ...ok, status: 'GOOD' }), /invalid status/);
  assert.throws(() => validateRecord({ ...ok, inputs: undefined }), /declared inputs/);
  assert.throws(() => validateRecord({ ...ok, kind: 'other' }), /unknown record kind/);
});

test('the markdown report shows numbers only for verified scenarios', () => {
  const markdown = renderMarkdown([ok, failed, refused, read]);
  assert.match(markdown, /\| destroy \| OK \| 1,000 \| 2.5 \| 400 \| 1 \| 2,400 \/ 2,400 \| 12 \| 3.5 \| 0 \| 0 \| EXPUNGE 1 \| 8 \/ 90 \/ 9 \|/);
  assert.match(markdown, /\| move \| FAILED \| verification failed: destination-uids-match-upstream \|/);
  assert.match(markdown, /\| flag-seen \| REFUSED \| Email\/set after 0 of 10000 messages: HTTP 429 \|/);
  assert.match(markdown, /\| open-unread \| OK \| 50 \/ 10,000 \| 41.2 \| 30 \/ 33 \| 10,000 \|/);
  assert.doesNotMatch(markdown.split('\n').find(line => line.startsWith('| move |')), /\d{3,}/);
});

test('runner arguments are bounded and the DSN never travels as an argument', () => {
  const options = parseArgs(['--backend', 'sqlite', '--layer', 'engine', '--mailbox', '100000', '--operation', '10000', '--release', '--batch', '100', '--ack', 'journal']);
  assert.equal(label(options), 'sqlite-engine-100000x10000');
  const env = environment(options, {});
  assert.equal(env.MW_BULK_LAYER, 'engine');
  assert.equal(env.MW_BULK_BATCH, '100');
  assert.equal(env.MW_INDEX_PUBLICATION_ACK, 'journal');
  assert.equal(env.MW_RUN_BULK_OPS_BENCH, '1');
  assert.ok(cargoArgs(options).includes('--release'));
  assert.deepEqual(cargoArgs(options).slice(-4), ['--exact', '--ignored', '--nocapture', '--test-threads=1']);
  for (const bad of [['--mailbox', '99'], ['--operation', '2', '--mailbox', '100', '--operation', '101'], ['--layer', 'grpc'], ['--scenarios', 'open,nuke'], ['--batch', '0'], ['--unknown', '1'], ['--out', 'postgres://u:p@localhost/mw_fixture']]) {
    assert.throws(() => parseArgs(bad), JSON.stringify(bad));
  }
  assert.throws(() => environment(parseArgs(['--backend', 'postgres']), {}), /MW_LOAD_FIXTURE_PG_DSN/);
  assert.equal(environment(parseArgs(['--backend', 'postgres']), { MW_LOAD_FIXTURE_PG_DSN: 'x' }).MW_LOAD_FIXTURE_PG_ACK_SCHEMA, '1');
});

test('the matrix covers both backends, both layers, 10k/100k mailboxes and 1k/10k operations', () => {
  const runs = matrix(parseArgs([]));
  assert.equal(runs.length, 16);
  assert.ok(runs.every(run => run.operation <= run.mailbox));
  assert.deepEqual([...new Set(runs.map(label))].length, 16);
});

import { heavyGroups, validate, percentiles, counterDelta, settings } from './large-mailbox-driver.mjs';

test('large-mailbox driver groups carry exact expectations from the seeded counts', () => {
  const p = { accountId: 'a', inboxId: 'i', messageCount: 100000, unreadCount: 30000, flaggedCount: 2000, plantedQuarterly: 20000 };
  const groups = heavyGroups(p);
  assert.deepEqual(Object.keys(groups), ['unread-page', 'flagged-page', 'size-sort-page', 'text-search-page-1', 'text-search-page-5', 'unread-count', 'list-batch']);
  assert.equal(groups['unread-count'].calls[0][1].limit, 1);
  assert.equal(groups['text-search-page-5'].calls[0][1].position, 200);
  const ok = { methodResponses: [['Email/query', { total: 30000 }, 'q'], ['Email/get', { list: new Array(50).fill({}) }, 'g']] };
  assert.equal(validate(ok, groups['unread-page']), true);
  assert.equal(validate({ methodResponses: [['Email/query', { total: 29999 }, 'q'], ok.methodResponses[1]] }, groups['unread-page']), false);
  assert.equal(validate({ methodResponses: [['error', { type: 'serverFail' }, 'q']] }, groups['unread-count']), false);
  const small = heavyGroups({ ...p, plantedQuarterly: 210 });
  assert.equal(small['text-search-page-5'].rows, 10);
});

test('driver statistics use nearest rank and per-request deltas', () => {
  assert.deepEqual(percentiles([1, 2, 3, 4]), { count: 4, p50: 2, p95: 4, p99: 4, max: 4 });
  assert.equal(percentiles([]).p99, null);
  const d = counterDelta({ hitsMaterialised: 0, documentFetches: 0, listProjectionDocumentReads: 0, cpuSeconds: 1 }, { hitsMaterialised: 500, documentFetches: 10, listProjectionDocumentReads: 40, cpuSeconds: 1.5 }, 5);
  assert.deepEqual([d.hitsEnumeratedPerRequest, d.storedDocumentsLoadedPerRequest, d.cpuMsPerRequest], [100, 110, 100]);
  assert.equal(settings({}).ordinaryRate, 50);
  assert.throws(() => settings({ MW_LARGE_DRIVER_ORDINARY_RATE: '0' }));
});

test('search records render their own table with oracle counts and decode costs', () => {
  const search = { ...base, scenario: 'search-common-term', status: 'OK', results: { hitsMatched: 2000, returned: 50, coldMs: 80, warm: { latencyMs: { p50: 40, max: 45 } }, cold: { hitsMaterialisedPerRequest: 2000, storedDocumentsDecodedPerRequest: 2050, sealedIndexBytesReadPerRequest: 0, processCpuMsPerRequest: 75.5 } } };
  const fresh = { ...base, scenario: 'search-fresh-after-move-1000', status: 'OK', results: { expectedTotal: 1000, firstQueryTotal: 1000, firstQueryMs: 30, correctAfterChangeMs: 31, changeMs: 9000, messagesChanged: 1000, counters: { publication: { readFenceWaits: 0 } } } };
  const markdown = renderMarkdown([search, fresh]);
  assert.match(markdown, /\| search-common-term \| OK \| 2,000 \/ 50 \| 80 \| 40 \/ 45 \| 2,000 \| 2,050 \| 0 \| 75.5 \|/);
  assert.match(markdown, /search-fresh-after-move-1000 \| OK \| 1,000 expected, first query 1,000/);
});
