import test from 'node:test';
import assert from 'node:assert/strict';
import { postgresProfile, postgresArgs, parseBytes } from './postgres-profile.mjs';

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

test('a 1 GiB co-located host keeps stock-sized memory settings', () => {
  const s = postgresProfile({ cpus: 1, memoryBytes: 1 * GiB, profile: 'colocated' });
  assert.equal(s.shared_buffers, '153MB');
  assert.equal(s.max_connections, '100');
  assert.equal(s.work_mem, '4MB');
  assert.equal(s.maintenance_work_mem, '64MB');
  assert.equal(s.max_wal_size, '2GB');
  assert.equal(s.autovacuum_naptime, '1min');
});

test('only the benchmark profile loads diagnosis extensions', () => {
  const base = postgresProfile({ cpus: 4, memoryBytes: 8 * GiB });
  const bench = postgresProfile({ cpus: 4, memoryBytes: 8 * GiB, profile: 'benchmark' });
  assert.equal(base.shared_preload_libraries, undefined);
  assert.equal(bench.shared_preload_libraries, 'pg_stat_statements');
  assert.equal(bench.track_io_timing, 'on');
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
