#!/usr/bin/env node
// Runner for the bulk-operation benchmark (t60). Builds and runs the ignored
// Rust suite once per requested combination, keeps the raw output, extracts the
// records and writes a markdown report. See docs/perf/bulk-operations.md.
//
//   node scripts/perf/bulk-ops/run.mjs --backend sqlite --layer http --mailbox 10000 --operation 1000 --release
//   node scripts/perf/bulk-ops/run.mjs --matrix --release --plan     (print the full matrix, run nothing)
//
// PostgreSQL: pass the DSN in MW_LOAD_FIXTURE_PG_DSN (environment only, never an
// argument) and --backend postgres; the database name must start with mw_fixture.
import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { parseRecords } from './records.mjs';
import { renderMarkdown } from './report.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
export const SCENARIOS = Object.freeze(['ingest', 'open', 'flag', 'move', 'mark-folder-read', 'import', 'trash', 'destroy', 'delete-folder']);
const TEST = 'bulk_mailbox_operations_benchmark';

const DEFAULTS = Object.freeze({
  backend: 'sqlite', layer: 'http', mailbox: 10000, operation: 1000, scenarios: 'all', batch: 'max',
  onLimit: 'stop', reader: 'on', readerRate: 2, deadline: 3600, ready: 7200, workers: 4, profile: 'enforced',
  release: false, out: null, hardware: null, plan: false, matrix: false, ack: null, accounting: null, guard: null, sqliteWriter: null, keepDir: false,
});
const NUMERIC = { mailbox: [100, 100000], operation: [1, 100000], readerRate: [1, 20], deadline: [10, 86400], ready: [10, 86400], workers: [1, 32] };
const CHOICES = {
  backend: ['sqlite', 'postgres'], layer: ['http', 'engine'], onLimit: ['stop', 'retry-after'], reader: ['on', 'off'],
  profile: ['enforced', 'defaults'], ack: ['commit', 'journal'], accounting: ['synchronous', 'write-behind'], guard: ['off', 'bounded'],
};
const FLAGS = { release: true, plan: true, matrix: true, keepDir: true };
const NAMES = { 'on-limit': 'onLimit', 'reader-rate': 'readerRate', 'sqlite-writer': 'sqliteWriter', 'keep-dir': 'keepDir' };

export function parseArgs(argv) {
  const options = { ...DEFAULTS };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (!arg.startsWith('--')) throw new Error(`unexpected argument ${arg}`);
    const name = NAMES[arg.slice(2)] ?? arg.slice(2);
    if (!(name in DEFAULTS)) throw new Error(`unknown option ${arg}`);
    if (FLAGS[name]) { options[name] = true; continue; }
    const value = argv[++i];
    if (value === undefined) throw new Error(`${arg} needs a value`);
    if (/postgres(ql)?:\/\//.test(value)) throw new Error('pass the PostgreSQL DSN in MW_LOAD_FIXTURE_PG_DSN, not as an argument');
    if (NUMERIC[name]) {
      const [low, high] = NUMERIC[name];
      if (!/^\d+$/.test(value) || Number(value) < low || Number(value) > high) throw new Error(`${arg} must be an integer from ${low} to ${high}`);
      options[name] = Number(value);
    } else if (CHOICES[name]) {
      if (!CHOICES[name].includes(value)) throw new Error(`${arg} must be one of ${CHOICES[name].join(', ')}`);
      options[name] = value;
    } else if (name === 'scenarios') {
      const list = value === 'all' ? SCENARIOS : value.split(',');
      for (const scenario of list) if (!SCENARIOS.includes(scenario)) throw new Error(`unknown scenario ${scenario}`);
      options.scenarios = value;
    } else if (name === 'batch') {
      if (value !== 'max' && !(/^\d+$/.test(value) && Number(value) >= 1 && Number(value) <= 5000)) throw new Error('--batch must be max or 1..5000');
      options.batch = value;
    } else {
      options[name] = value;
    }
  }
  if (options.operation > options.mailbox) throw new Error('--operation cannot exceed --mailbox');
  return options;
}

/** The combinations the user asked for: 10k and 100k mailboxes, 1k and 10k operations. */
export function matrix(options) {
  const runs = [];
  for (const backend of ['sqlite', 'postgres'])
    for (const layer of ['engine', 'http'])
      for (const mailbox of [10000, 100000])
        for (const operation of [1000, 10000]) runs.push({ ...options, backend, layer, mailbox, operation, matrix: false });
  return runs;
}

export function environment(options, base = process.env) {
  const env = {
    ...base,
    MW_RUN_BULK_OPS_BENCH: '1',
    MW_BULK_LAYER: options.layer,
    MW_BULK_MAILBOX_MESSAGES: String(options.mailbox),
    MW_BULK_OPERATION_MESSAGES: String(options.operation),
    MW_BULK_SCENARIOS: options.scenarios,
    MW_BULK_BATCH: String(options.batch),
    MW_BULK_ON_LIMIT: options.onLimit,
    MW_BULK_CONCURRENT_READER: options.reader,
    MW_BULK_READER_RATE: String(options.readerRate),
    MW_BULK_SCENARIO_DEADLINE_SECONDS: String(options.deadline),
    MW_BULK_READY_SECONDS: String(options.ready),
    MW_BULK_WORKERS: String(options.workers),
    MW_BULK_SECURITY_PROFILE: options.profile,
    MW_LOAD_FIXTURE_BACKEND: options.backend,
  };
  if (options.backend === 'postgres') {
    if (!base.MW_LOAD_FIXTURE_PG_DSN) throw new Error('--backend postgres needs MW_LOAD_FIXTURE_PG_DSN in the environment');
    env.MW_LOAD_FIXTURE_PG_ACK_SCHEMA = '1';
  }
  if (options.ack) env.MW_INDEX_PUBLICATION_ACK = options.ack;
  if (options.accounting) env.MW_REQUEST_PROTECTION_ACCOUNTING = options.accounting;
  if (options.guard) env.MW_GUARD_SNAPSHOT = options.guard;
  if (options.sqliteWriter) env.MW_SQLITE_WRITER = options.sqliteWriter;
  if (options.keepDir) env.MW_BULK_KEEP_DIR = '1';
  // Windows debug builds need a large main-thread stack (the repository rule).
  if (process.platform === 'win32' && !options.release && !env.RUST_MIN_STACK) env.RUST_MIN_STACK = '67108864';
  return env;
}

export function cargoArgs(options) {
  return ['test', '--locked', ...(options.release ? ['--release'] : []), '-p', 'mw-server', '--features', 'load-test-fixtures', '--test', 't60_bulk_ops', TEST, '--', '--exact', '--ignored', '--nocapture', '--test-threads=1'];
}

export function label(options) {
  return `${options.backend}-${options.layer}-${options.mailbox}x${options.operation}`;
}

function git(args) {
  const result = spawnSync('git', args, { cwd: ROOT, encoding: 'utf8' });
  return result.status === 0 ? result.stdout.trim() : null;
}
export function hardware(extra) {
  return {
    supplied: true, collectedBy: 'scripts/perf/bulk-ops/run.mjs', platform: process.platform, release: os.release(), arch: process.arch,
    logicalCpus: os.cpus().length, cpuModel: os.cpus()[0]?.model ?? null, totalMemoryBytes: os.totalmem(), ...(extra ?? {}),
  };
}

function runOne(options, outDir) {
  const name = label(options);
  const env = environment(options);
  const extra = options.hardware ? JSON.parse(readFileSync(options.hardware, 'utf8')) : null;
  env.MW_BULK_HARDWARE_JSON = JSON.stringify(hardware(extra));
  env.MW_BULK_SOURCE_REVISION = git(['rev-parse', 'HEAD']) ?? 'unknown';
  env.MW_BULK_SOURCE_DIRTY = git(['status', '--porcelain', '--untracked-files=no']) ? 'dirty' : 'clean';
  const records = path.join(outDir, `${name}.jsonl`);
  env.MW_BULK_REPORT = records;
  writeFileSync(records, '');
  console.error(`bulk-ops: ${name}: cargo ${cargoArgs(options).join(' ')}`);
  const started = Date.now();
  const result = spawnSync('cargo', cargoArgs(options), { cwd: ROOT, env, encoding: 'utf8', maxBuffer: 1 << 30 });
  writeFileSync(path.join(outDir, `${name}.log`), `${result.stdout ?? ''}\n--- stderr ---\n${result.stderr ?? ''}`);
  const parsed = parseRecords(readFileSync(records, 'utf8'));
  console.error(`bulk-ops: ${name}: exit ${result.status}, ${parsed.length} records, ${((Date.now() - started) / 1000).toFixed(0)} s`);
  return { name, status: result.status, records: parsed };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  let options;
  try { options = parseArgs(process.argv.slice(2)); } catch (error) { console.error(error.message); process.exit(2); }
  const runs = options.matrix ? matrix(options) : [options];
  if (options.plan) {
    for (const run of runs) {
      const env = environment(run, { MW_LOAD_FIXTURE_PG_DSN: '<from your environment>' });
      const shown = Object.entries(env).filter(([key]) => key.startsWith('MW_') || key === 'RUST_MIN_STACK').map(([k, v]) => `${k}=${v}`).join(' ');
      console.log(`${label(run)}: ${shown} cargo ${cargoArgs(run).join(' ')}`);
    }
    process.exit(0);
  }
  const outDir = path.resolve(options.out ?? path.join(ROOT, '.orchestration/reports/bulk-ops', new Date().toISOString().replace(/[:.]/g, '-')));
  if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true });
  const all = [];
  let failed = false;
  for (const run of runs) {
    const outcome = runOne(run, outDir);
    all.push(...outcome.records);
    failed ||= outcome.status !== 0;
  }
  writeFileSync(path.join(outDir, 'report.md'), renderMarkdown(all) + '\n');
  console.error(`bulk-ops: ${all.length} records; report ${path.join(outDir, 'report.md')}`);
  process.exit(failed ? 1 : 0);
}
