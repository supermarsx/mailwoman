// Bulk-operation benchmark records (t60): extraction and validation.
// The Rust suite (crates/mw-server/tests/t60_bulk_ops.rs) prints one
// `BULK_OPS_RECORD {json}` line per scenario; this module reads them back.
export const RECORD_PREFIX = 'BULK_OPS_RECORD ';
export const RECORD_KIND = 'mw-bulk-ops-v1';
export const STATUSES = Object.freeze(['OK', 'FAILED', 'REFUSED', 'INCOMPLETE']);

export function validateRecord(record) {
  if (!record || typeof record !== 'object' || Array.isArray(record)) throw new Error('record must be an object');
  if (record.kind !== RECORD_KIND) throw new Error(`unknown record kind ${JSON.stringify(record.kind)}`);
  if (typeof record.scenario !== 'string' || !record.scenario) throw new Error('record needs a scenario');
  if (!STATUSES.includes(record.status)) throw new Error(`invalid status ${JSON.stringify(record.status)}`);
  if (!['sqlite', 'postgres'].includes(record.backend)) throw new Error('record needs backend sqlite|postgres');
  if (!['engine', 'http'].includes(record.layer)) throw new Error('record needs layer engine|http');
  if (!record.inputs || typeof record.inputs !== 'object') throw new Error('record needs declared inputs');
  if (!Array.isArray(record.checks)) throw new Error('record needs its checks');
  // A failed verification must never carry numbers.
  if (record.status === 'FAILED' && record.results !== null) throw new Error('FAILED record carries results');
  if (record.status === 'OK') {
    if (!record.results || typeof record.results !== 'object') throw new Error('OK record needs results');
    if (record.checks.some(check => check.pass !== true)) throw new Error('OK record has a failing check');
  }
  return record;
}

/** Records from captured test output (stdout) or JSON-lines text. */
export function parseRecords(text) {
  const records = [];
  for (const raw of String(text).split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) continue;
    let payload;
    if (line.startsWith(RECORD_PREFIX)) payload = line.slice(RECORD_PREFIX.length);
    else if (line.startsWith('{') && line.includes(`"kind":"${RECORD_KIND}"`)) payload = line;
    else continue;
    records.push(validateRecord(JSON.parse(payload)));
  }
  return records;
}
