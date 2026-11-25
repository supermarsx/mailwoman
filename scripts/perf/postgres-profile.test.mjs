import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { postgresProfile, postgresArgs, parseBytes, memoryShares, colocatedProfile, appConnections, BACKEND_MIB, DEPLOY_SAFE_SETTINGS, alterSystemSql } from './postgres-profile.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

const GiB = 1024 ** 3;

test('16 CPUs / 64 GiB dedicated follows the documented formulas', () => {
  const s = postgresProfile({ cpus: 16, memoryBytes: 64 * GiB });
  assert.equal(s.shared_buffers, '16384MB');
  assert.equal(s.effective_cache_size, '49152MB');
  assert.equal(s.max_connections, '160');
  assert.equal(s.work_mem, '64MB');
  assert.equal(s.maintenance_work_mem, '2048MB');
  assert.equal(s.autovacuum_work_mem, '1024MB');
  assert.equal(s.wal_buffers, '64MB');
  assert.equal(s.max_wal_size, '32768MB');
  assert.equal(s.autovacuum_max_workers, '4');
  assert.equal(s.jit, 'off');
  assert.equal(s.random_page_cost, undefined, 'random_page_cost stays at the default 4');
});

test('no profile turns durability off', () => {
  for (const profile of ['dedicated', 'colocated', 'benchmark']) {
    for (const [cpus, memory] of [[1, 1], [4, 8], [16, 64], [64, 512]]) {
      const s = postgresProfile({ cpus, memoryBytes: memory * GiB, profile });
      assert.equal(s.fsync, 'on');
      assert.equal(s.full_page_writes, 'on');
      assert.equal(s.synchronous_commit, 'on');
      assert.equal(s.commit_delay, undefined, 'group-commit delay is not part of any profile');
    }
  }
});

test('a 1 GiB co-located host: PostgreSQL fits its share with the connections the app opens', () => {
  const shares = memoryShares({ hostMiB: 1024 });
  assert.deepEqual(shares, { app: 437, postgres: 235, valkey: 64, reserve: 256, caddy: 0 });
  const s = colocatedProfile({ pgMiB: shares.postgres, hostMiB: 1024, cpus: 1, appCpus: 1 });
  assert.equal(s.shared_buffers, '58MB');
  assert.equal(s.max_connections, '30');
  assert.equal(s.work_mem, '4MB');
  assert.equal(s.max_wal_size, '1GB');
  // shared_buffers + the app's 12 connections x (backend 8 MiB + one 4 MiB
  // work_mem node) stays inside PostgreSQL's share.
  const active = appConnections(1);
  assert.equal(active, 12);
  assert.ok(58 + active * (BACKEND_MIB + 4) <= shares.postgres);
});

test('memory shares never give a component more than the host has', () => {
  for (const hostMiB of [1024, 2048, 4096, 8192, 16384, 65536]) {
    for (const stack of ['full', 'single']) {
      for (const caddy of [false, true]) {
        const m = memoryShares({ hostMiB, stack, caddy });
        assert.equal(m.app + m.postgres + m.valkey + (stack === 'full' ? 32 : 0) + m.reserve + m.caddy, hostMiB);
        assert.ok(m.app > 0);
      }
    }
  }
});

test('arguments are rendered as -c pairs', () => {
  assert.deepEqual(postgresArgs({ jit: 'off' }), ['-c', 'jit=off']);
});

test('inputs are validated', () => {
  assert.equal(parseBytes('8GiB'), 8 * GiB);
  assert.equal(parseBytes('512m'), 512 * 1024 ** 2);
  assert.throws(() => parseBytes('lots'));
  assert.throws(() => postgresProfile({ cpus: 0, memoryBytes: GiB }));
  assert.throws(() => postgresProfile({ cpus: 2, memoryBytes: 100 }));
  assert.throws(() => postgresProfile({ cpus: 2, memoryBytes: GiB, profile: 'fast' }));
});

const bash = (script) => execFileSync('bash', ['-c', `source scripts/deploy-stack.sh; ${script}`], { cwd: root, encoding: 'utf8' });

test('deploy.sh full stack renders stock PostgreSQL plus the safe settings, and the same shares', () => {
  for (const hostMiB of [1024, 2048, 4096, 16384, 65536, 262144]) {
    for (const [stack, caddy] of [['full', 0], ['full', 1], ['single', 0]]) {
      const m = memoryShares({ hostMiB, stack, caddy: caddy === 1 });
      assert.equal(bash(`stack_memory_shares ${hostMiB} ${stack} ${caddy}`).trim(), `${m.app} ${m.postgres} ${m.valkey} ${m.reserve} ${m.caddy}`);
    }
  }
  const lines = bash('stack_postgres_settings').split(/\s+/).filter(Boolean);
  const rendered = Object.fromEntries(lines.map((line) => [line.slice(0, line.indexOf('=')), line.slice(line.indexOf('=') + 1)]));
  assert.deepEqual(rendered, DEPLOY_SAFE_SETTINGS);
  for (const key of Object.keys(rendered)) {
    assert.ok(!/buffers|work_mem|wal|checkpoint|cost|jit|connections|cache|fsync|commit|parallel|autovacuum_(vacuum|analyze|max)/.test(key) || key === 'log_checkpoints', `${key} would change the query path or durability`);
  }
});

test('the benchmark profile carries diagnostics only when asked', () => {
  const plain = postgresProfile({ cpus: 4, memoryBytes: 4 * GiB, profile: 'benchmark' });
  assert.equal(plain.shared_preload_libraries, undefined);
  assert.equal(plain.track_io_timing, undefined);
  const diag = postgresProfile({ cpus: 4, memoryBytes: 4 * GiB, profile: 'benchmark', diagnostics: true });
  assert.equal(diag.shared_preload_libraries, 'pg_stat_statements');
  assert.equal(diag.track_io_timing, 'on');
  assert.deepEqual(alterSystemSql({ jit: 'off' }), ["ALTER SYSTEM SET jit = 'off';"]);
});
