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
