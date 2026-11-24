// PostgreSQL configuration profiles, sized by rule from the CPUs and memory
// the database server has. The formulas are the ones documented in
// docs/admin/postgresql-tuning.md; keep the two in step.
//
// Profiles:
// - 'dedicated': PostgreSQL has the machine (or container limit) to itself.
// - 'colocated': PostgreSQL shares the machine with Mailwoman (deploy.sh full
//   stack); smaller memory shares.
// - 'benchmark': the dedicated formulas plus diagnosis settings
//   (pg_stat_statements, track_io_timing). Used by PostgreSQL benchmark legs;
//   every report must name it. It never turns durability off: fsync,
//   full_page_writes and synchronous_commit stay on.
//
// CLI: node scripts/perf/postgres-profile.mjs --cpus 4 --memory 8GiB [--profile benchmark]
// prints one `-c name=value` argument pair per line for `postgres`.

const KiB = 1024, MiB = 1024 * KiB, GiB = 1024 * MiB;
const clamp = (value, low, high) => Math.min(high, Math.max(low, value));
const mib = (bytes) => `${Math.floor(bytes / MiB)}MB`;

export const PROFILES = ['dedicated', 'colocated', 'benchmark'];

export function parseBytes(text) {
  const match = /^(\d+(?:\.\d+)?)\s*(B|KiB|MiB|GiB|TiB|KB|MB|GB|TB|k|m|g|t)?$/i.exec(String(text).trim());
  if (!match) throw new Error(`invalid size: ${text}`);
  const unit = (match[2] || 'B').toLowerCase()[0];
  const scale = { b: 1, k: KiB, m: MiB, g: GiB, t: 1024 * GiB }[unit];
  return Math.floor(Number(match[1]) * scale);
}

/// The settings for a server with `cpus` CPUs and `memoryBytes` of memory.
export function postgresProfile({ cpus, memoryBytes, profile = 'dedicated' }) {
  if (!PROFILES.includes(profile)) throw new Error(`unknown profile ${profile}`);
  if (!Number.isInteger(cpus) || cpus < 1) throw new Error('cpus must be a positive integer');
  if (!Number.isFinite(memoryBytes) || memoryBytes < 512 * MiB) throw new Error('memory must be at least 512 MiB');
  const colocated = profile === 'colocated';
  const sharedBuffers = clamp(memoryBytes * (colocated ? 0.15 : 0.25), 128 * MiB, 32 * GiB);
  const maxConnections = clamp(10 * cpus, 100, 400);
  const settings = {
    // Memory
    shared_buffers: mib(sharedBuffers),
    effective_cache_size: mib(clamp(memoryBytes * (colocated ? 0.5 : 0.75), 256 * MiB, 1024 * GiB)),
    work_mem: mib(clamp((memoryBytes - sharedBuffers) / (4 * maxConnections), 4 * MiB, 64 * MiB)),
    maintenance_work_mem: mib(clamp(memoryBytes / 16, 64 * MiB, 2 * GiB)),
    autovacuum_work_mem: mib(clamp(memoryBytes / 64, 64 * MiB, 1 * GiB)),
    huge_pages: 'try',
    // Connections
    max_connections: String(maxConnections),
    // WAL and checkpoints (durability unchanged)
    wal_level: 'replica',
    fsync: 'on',
    full_page_writes: 'on',
    synchronous_commit: 'on',
    wal_compression: 'lz4',
    wal_buffers: sharedBuffers >= 4 * GiB ? '64MB' : '-1',
    max_wal_size: colocated ? '2GB' : mib(clamp(memoryBytes / 2, 2 * GiB, 64 * GiB)),
    min_wal_size: colocated ? '256MB' : mib(clamp(memoryBytes / 16, 256 * MiB, 8 * GiB)),
    checkpoint_timeout: '15min',
    checkpoint_completion_target: '0.9',
    // I/O on SSD. random_page_cost stays at its default 4: 1.1 changes plan
    // choices that the store's PostgreSQL plan tests assert (see
    // docs/admin/postgresql-tuning.md); it is a measured, per-deployment step.
    effective_io_concurrency: '200',
    maintenance_io_concurrency: '100',
    jit: 'off',
    default_toast_compression: 'lz4',
    // Parallelism
    max_worker_processes: String(Math.max(8, cpus)),
    max_parallel_workers: String(Math.max(2, Math.floor(cpus / 2))),
    max_parallel_workers_per_gather: '2',
    max_parallel_maintenance_workers: String(clamp(Math.floor(cpus / 4), 2, 4)),
    // Autovacuum
    autovacuum_max_workers: String(clamp(Math.floor(cpus / 4), 3, 8)),
    autovacuum_naptime: colocated ? '1min' : '15s',
    autovacuum_vacuum_cost_limit: '2000',
    autovacuum_vacuum_scale_factor: '0.05',
    autovacuum_vacuum_insert_scale_factor: '0.05',
    autovacuum_analyze_scale_factor: '0.02',
    // Safety and diagnosis
    idle_in_transaction_session_timeout: '10min',
    log_lock_waits: 'on',
    log_checkpoints: 'on',
    log_autovacuum_min_duration: '10s',
    log_temp_files: '16MB',
  };
  if (profile === 'benchmark') {
    Object.assign(settings, {
      shared_preload_libraries: 'pg_stat_statements',
      'pg_stat_statements.max': '5000',
      'pg_stat_statements.track': 'top',
      track_io_timing: 'on',
      track_wal_io_timing: 'on',
    });
  }
  return settings;
}

export function postgresArgs(settings) {
  return Object.entries(settings).flatMap(([name, value]) => ['-c', `${name}=${value}`]);
}

if (process.argv[1]?.endsWith('postgres-profile.mjs')) {
  const args = process.argv.slice(2);
  const value = (flag) => { const at = args.indexOf(flag); return at >= 0 ? args[at + 1] : undefined; };
  if (value('--cpus') && value('--memory')) {
    const settings = postgresProfile({
      cpus: Number(value('--cpus')),
      memoryBytes: parseBytes(value('--memory')),
      profile: value('--profile') || 'dedicated',
    });
    const pairs = postgresArgs(settings);
    for (let i = 0; i < pairs.length; i += 2) console.log(`${pairs[i]} ${pairs[i + 1]}`);
  } else if (args.length) {
    console.error('usage: postgres-profile.mjs --cpus N --memory 8GiB [--profile dedicated|colocated|benchmark]');
    process.exitCode = 2;
  }
}
