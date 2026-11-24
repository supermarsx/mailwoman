#!/usr/bin/env node
// Markdown tables from bulk-operation benchmark records (t60).
// Usage: node scripts/perf/bulk-ops/report.mjs <records.jsonl|test-output.log>... > report.md
// Only OK records show numbers. FAILED rows show the failed checks; REFUSED and
// INCOMPLETE rows show where the run stopped and why.
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { parseRecords } from './records.mjs';

const fmt = (value, digits = 1) => {
  if (value === null || value === undefined || Number.isNaN(value)) return '–';
  if (typeof value !== 'number') return String(value);
  if (Number.isInteger(value)) return value.toLocaleString('en-US');
  return value.toLocaleString('en-US', { maximumFractionDigits: digits, minimumFractionDigits: 0 });
};
const seconds = ms => (typeof ms === 'number' ? ms / 1000 : null);
const cell = text => String(text).replace(/\|/g, '\\|').replace(/\n/g, ' ');

function upstream(counters) {
  const u = counters?.upstream;
  if (!u) return '–';
  const parts = [['store', 'STORE'], ['move', 'MOVE'], ['expunge', 'EXPUNGE'], ['append', 'APPEND'], ['fetch', 'FETCH'], ['deleteMailbox', 'DELETE']]
    .filter(([key]) => u[key] > 0)
    .map(([key, name]) => `${name} ${fmt(u[key])}`);
  return parts.length ? parts.join(', ') : 'none';
}
function readerCell(results) {
  const windows = results?.concurrentReader?.windows;
  if (!windows) return '–';
  const p95 = name => windows[name]?.inboxFirstPage?.latencyMs?.p95;
  return `${fmt(p95('before'))} / ${fmt(p95('during'))} / ${fmt(p95('after'))}`;
}
function why(record) {
  if (record.status === 'FAILED') {
    const failed = record.checks.filter(check => !check.pass).map(check => check.name);
    return `verification failed: ${failed.join(', ') || record.failure?.reason || 'unknown'}`;
  }
  const failure = record.failure ?? {};
  const refusal = failure.refusal ?? {};
  const detail = refusal.status ? `HTTP ${refusal.status}` : refusal.error?.type ?? refusal.kind ?? failure.reason ?? '';
  return [failure.stage, detail].filter(Boolean).join(': ');
}

export function groupKey(record) {
  const c = record.inputs?.config ?? {};
  return `${record.backend} · ${record.layer} · ${fmt(c.mailboxMessages)} per mailbox · ${fmt(c.operationMessages)} per operation · ${record.inputs?.buildProfile ?? '?'} · ${String(record.inputs?.sourceRevision ?? 'unknown').slice(0, 12)}`;
}

const BULK_HEADER = '| Scenario | Status | Messages | Elapsed (s) | Messages/s | Requests | Request p50 / p95 (ms) | List correct after (ms) | Statements / message | Index docs / message | Body bytes / message | Upstream commands | Reader p95 before / during / after (ms) |';
const READ_HEADER = '| Scenario | Status | Returned / total | Cold (ms) | Warm p50 / max (ms) | Hits materialised (cold) | Notes |';

function bulkRow(record) {
  if (record.status !== 'OK') return `| ${cell(record.scenario)} | ${record.status} | ${cell(why(record))} | | | | | | | | | | |`;
  const r = record.results;
  const statements = r.counters?.statements?.perMessage;
  const docs = r.counters?.searchIndex?.documentsWrittenPerMessage;
  const body = r.counters?.searchIndex?.bodyBytesTokenised;
  const latency = r.requests?.latencyMs ?? {};
  const visible = r.listCorrectAfterLastResponseMs ?? r.indexReflectsAfterLastResponseMs;
  return `| ${cell(record.scenario)} | OK | ${fmt(r.messages)} | ${fmt(seconds(r.elapsedMs), 2)} | ${fmt(r.messagesPerSecond)} | ${fmt(r.requests?.requests)} | ${fmt(latency.p50)} / ${fmt(latency.p95)} | ${fmt(visible)} | ${fmt(statements, 2)} | ${fmt(docs, 2)} | ${fmt(r.messages ? body / r.messages : null, 0)} | ${cell(upstream(r.counters))} | ${readerCell(r)} |`;
}
function readRow(record) {
  if (record.status !== 'OK') return `| ${cell(record.scenario)} | ${record.status} | ${cell(why(record))} | | | | |`;
  const r = record.results;
  const total = r.total ?? r.mailboxMessages;
  const hits = r.coldCounters?.searchIndex?.hitsMaterialised;
  const note = record.scenario === 'open-changes' ? `arrival sync ${fmt(r.arrivalSyncMs)} ms` : r.queryResponse?.totalTruncated ? 'total truncated' : '';
  return `| ${cell(record.scenario)} | OK | ${fmt(r.returned)} / ${fmt(total)} | ${fmt(r.coldMs)} | ${fmt(r.warm?.latencyMs?.p50)} / ${fmt(r.warm?.latencyMs?.max)} | ${fmt(hits)} | ${cell(note)} |`;
}
const SEARCH_HEADER = '| Scenario | Status | Matched / returned | Cold (ms) | Warm p50 / max (ms) | Hits materialised (cold) | Stored docs decoded (cold) | Index bytes read (cold) | CPU ms (cold) | Notes |';
function searchRow(record) {
  if (record.status !== 'OK') return `| ${cell(record.scenario)} | ${record.status} | ${cell(why(record))} | | | | | | | |`;
  const r = record.results;
  if (record.scenario.startsWith('search-fresh-')) {
    return `| ${cell(record.scenario)} | OK | ${fmt(r.expectedTotal)} expected, first query ${fmt(r.firstQueryTotal)} | ${fmt(r.firstQueryMs)} | | | | | | ${cell(`correct ${fmt(r.correctAfterChangeMs)} ms after the change; change ${fmt(r.changeMs)} ms for ${fmt(r.messagesChanged)} messages; read-fence waits ${fmt(r.counters?.publication?.readFenceWaits)}`)} |`;
  }
  if (record.scenario === 'search-as-you-type') {
    const keys = (r.keystrokes ?? []).map(k => `${k.typed}:${fmt(k.latencyMs)}`).join(' ');
    return `| ${cell(record.scenario)} | OK | | | ${fmt(r.requests?.latencyMs?.p50)} / ${fmt(r.requests?.latencyMs?.max)} | ${fmt(r.counters?.hitsMaterialisedPerRequest)} /req | ${fmt(r.counters?.storedDocumentsDecodedPerRequest)} /req | | ${fmt(r.counters?.processCpuMsPerRequest)} /req | ${cell(`${keys}; slower than 250 ms: ${fmt(r.responsesSlowerThanDebounce)}`)} |`;
  }
  if (record.scenario === 'search-index-rebuild') {
    return `| ${cell(record.scenario)} | OK | ${fmt(r.messagesIndexed)} indexed | ${fmt(r.rebuildMs)} | | | | | | ${cell(`${fmt(r.documentsPerSecond)} documents/s; index ${fmt((r.disk?.searchIndexBytes ?? 0) / 1048576, 1)} MiB`)} |`;
  }
  const c = r.cold ?? r.firstAfterReload ?? {};
  const cold = r.coldMs ?? r.firstAfterReloadMs;
  return `| ${cell(record.scenario)} | OK | ${fmt(r.hitsMatched ?? r.snippets)} / ${fmt(r.returned)} | ${fmt(cold)} | ${fmt(r.warm?.latencyMs?.p50)} / ${fmt(r.warm?.latencyMs?.max)} | ${fmt(c.hitsMaterialisedPerRequest)} | ${fmt(c.storedDocumentsDecodedPerRequest)} | ${fmt(c.sealedIndexBytesReadPerRequest)} | ${fmt(c.processCpuMsPerRequest, 1)} | ${cell(r.queryResponse?.totalTruncated ? 'total truncated' : '')} |`;
}

function setupRows(records) {
  const lines = [];
  for (const record of records) {
    if (record.scenario === 'seed') {
      if (record.status !== 'OK') { lines.push(`- seed and startup index: ${record.status} (${why(record)})`); continue; }
      const r = record.results;
      lines.push(`- seed: ${fmt(r.seededMessages)} messages in ${fmt(seconds(r.seedMs), 1)} s (${fmt(r.seedMessagesPerSecond)} messages/s); startup index build ${fmt(seconds(r.startupIndexBuildMs), 1)} s (${fmt(r.startupIndexDocumentsPerSecond)} documents/s)`);
    }
    if (record.scenario === 'ingest' && record.status === 'OK') {
      const r = record.results;
      lines.push(`- ingest: first rows ${fmt(r.timeToFirstRowsMs)} ms, first usable page ${fmt(r.timeToFirstUsablePageMs)} ms, all rows ${fmt(seconds(r.allRowsVisibleMs), 1)} s, index caught up ${fmt(seconds(r.searchIndexCaughtUpMs), 1)} s, read fence ${fmt(seconds(r.readFenceSatisfiedMs), 1)} s, sampled peak RSS ${fmt((r.memory?.sampledPeakResidentBytes ?? 0) / 1048576, 0)} MiB, database ${fmt((r.disk?.databaseBytes ?? r.disk?.schemaBytes ?? 0) / 1048576, 0)} MiB, WAL ${fmt((r.disk?.walBytes ?? 0) / 1048576, 0)} MiB, index ${fmt((r.disk?.searchIndexBytes ?? 0) / 1048576, 0)} MiB`);
    }
  }
  return lines;
}

export function renderMarkdown(records) {
  const groups = new Map();
  for (const record of records) {
    const key = groupKey(record);
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(record);
  }
  const out = ['# Bulk mailbox operation results', '', 'Generated by scripts/perf/bulk-ops/report.mjs. Read docs/perf/bulk-operations.md for what each column means and does not mean. Only verified (OK) scenarios show numbers.', ''];
  for (const [key, list] of groups) {
    const first = list[0];
    out.push(`## ${key}`, '');
    const hw = first.inputs?.hardware;
    out.push(`Inputs: encryption ${JSON.stringify(first.inputs?.encryption ?? {})}; security profile ${first.inputs?.config?.securityProfile}; batch ${first.inputs?.config?.batch}; index publication ack ${first.inputs?.limits?.indexPublication?.ack}; hardware ${hw && hw.supplied !== false ? JSON.stringify(hw) : 'not supplied'}.`, '');
    out.push(...setupRows(list), '');
    const reads = list.filter(r => r.scenario.startsWith('open-'));
    const searches = list.filter(r => r.scenario.startsWith('search-'));
    const bulk = list.filter(r => !r.scenario.startsWith('open-') && !r.scenario.startsWith('search-') && r.scenario !== 'seed');
    if (reads.length) out.push(READ_HEADER, '| --- | --- | ---: | ---: | ---: | ---: | --- |', ...reads.map(readRow), '');
    if (searches.length) out.push(SEARCH_HEADER, '| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |', ...searches.map(searchRow), '');
    if (bulk.length) out.push(BULK_HEADER, '| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: |', ...bulk.map(bulkRow), '');
  }
  return out.join('\n');
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const files = process.argv.slice(2);
  if (!files.length) {
    console.error('usage: node scripts/perf/bulk-ops/report.mjs <records.jsonl|output.log>...');
    process.exit(2);
  }
  const records = files.flatMap(file => parseRecords(readFileSync(file, 'utf8')));
  process.stdout.write(renderMarkdown(records) + '\n');
}
