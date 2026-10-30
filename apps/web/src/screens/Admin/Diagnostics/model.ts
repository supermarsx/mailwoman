// t105: how the Diagnostics screen groups, names and summarises checks.
//
// The server reports a flat list of results per run (`server` suite: database,
// network and posture checks; `backends` job: one row per saved endpoint). The
// screen shows them as cards on purpose-named tabs. This module is the one table
// that maps a check id to its tab, its title, its plain-language purpose and its
// "How to fix" text, and the pure helpers the tabs share (latest results, status
// counts, run-to-run differences). No rendering here.

import type { Check, Run } from './api.ts';

export const DIAG_TABS = ['overview', 'server', 'connectivity', 'backends', 'posture', 'tools', 'history', 'report'] as const;
export type DiagTab = (typeof DIAG_TABS)[number];
/** The tabs whose cards come from runs (Overview counts these). */
export const CHECK_GROUPS = ['server', 'connectivity', 'backends', 'posture'] as const;
export type CheckGroup = (typeof CHECK_GROUPS)[number];

/** `#/diagnostics/<tab>`; an unknown or empty segment opens the Overview. */
export function tabOf(segment: string | undefined): DiagTab {
  return (DIAG_TABS as readonly string[]).includes(segment ?? '') ? segment as DiagTab : 'overview';
}

export type CardStatus = 'ok' | 'warn' | 'fail' | 'skipped' | 'notRun' | 'running';
/** The server's words (`pass`/`warn`/`fail`/`skipped`/`error`) as a card status. */
export function cardStatus(status: string | undefined): CardStatus {
  switch (status) {
    case 'pass': return 'ok';
    case 'warn': return 'warn';
    case 'fail': case 'error': return 'fail';
    case 'skipped': return 'skipped';
    case 'running': return 'running';
    default: return 'notRun';
  }
}

/** The server-suite checks this build knows, by tab, in reading order. */
export const KNOWN_CHECKS: Readonly<Record<Exclude<CheckGroup, 'backends'>, readonly string[]>> = {
  server: ['database-connectivity', 'database-write', 'database-migrations', 'server-key-roundtrip'],
  connectivity: ['network-public-dns', 'network-public-port', 'network-backend-ports', 'egress-private-policy'],
  posture: [
    'security-https', 'security-secure-cookies', 'security-hsts', 'security-csrf', 'security-admin-factors',
    'security-signin-pacing', 'security-credential-locks', 'security-geography', 'security-render-isolation',
    'security-deployment', 'security-plaintext-backends', 'fail2ban-client-address',
  ],
};
const GROUP_OF = new Map<string, CheckGroup>(
  (Object.entries(KNOWN_CHECKS) as [CheckGroup, readonly string[]][]).flatMap(([group, ids]) => ids.map((id) => [id, group] as const)),
);
/** Message ids exist for these: `admin-diagx-check-<id>`, `-purpose`, `-fix`. */
export const isKnownCheck = (id: string): boolean => GROUP_OF.has(id);

/** The tab a result belongs on. A backends-job row is always a mail backend. */
export function groupOf(check: Check, suite: string): CheckGroup {
  if (suite === 'backends') return 'backends';
  const known = GROUP_OF.get(check.id);
  if (known) return known;
  if (check.id.startsWith('security-') || check.id.startsWith('fail2ban-')) return 'posture';
  if (check.id.startsWith('network-') || check.id.startsWith('egress-')) return 'connectivity';
  return 'server';
}

export interface LatestCheck { check: Check; run: Run }
/** For each check id, its result in the newest run that has it. `runs` is newest first. */
export function latestChecks(runs: readonly Run[]): Map<string, LatestCheck> {
  const out = new Map<string, LatestCheck>();
  for (const run of runs) {
    if (run.status === 'running') continue;
    for (const check of run.results) {
      const key = keyOf(check, run.suite);
      if (!out.has(key)) out.set(key, { check, run });
    }
  }
  return out;
}
/** A card key: the server-suite id, or `backends:<domain:backend>` for an endpoint. */
export const keyOf = (check: Check, suite: string): string => suite === 'backends' ? `backends:${check.id}` : check.id;

/** The newest finished run of `suite`, if any. */
export const latestRun = (runs: readonly Run[], suite: string): Run | undefined => runs.find((r) => r.suite === suite && r.status !== 'running');

export type Counts = Record<'ok' | 'warn' | 'fail' | 'skipped' | 'notRun', number>;
export function countStatuses(statuses: readonly CardStatus[]): Counts {
  const counts: Counts = { ok: 0, warn: 0, fail: 0, skipped: 0, notRun: 0 };
  for (const s of statuses) if (s !== 'running') counts[s] += 1;
  return counts;
}
/** The worst status in a set (what a group's tile shows). */
export function worst(statuses: readonly CardStatus[]): CardStatus {
  for (const s of ['fail', 'warn', 'running', 'ok', 'skipped'] as const) if (statuses.includes(s)) return s;
  return 'notRun';
}

export interface RunChange { id: string; summary: string; before: string | undefined; after: string | undefined }
/** What changed from `older` to `newer`: status changes, new checks, checks gone. */
export function diffRuns(newer: Run, older: Run): RunChange[] {
  const before = new Map(older.results.map((c) => [c.id, c]));
  const after = new Map(newer.results.map((c) => [c.id, c]));
  const changes: RunChange[] = [];
  for (const [id, c] of after) {
    const prior = before.get(id);
    if (!prior || prior.status !== c.status) changes.push({ id, summary: c.summary, before: prior?.status, after: c.status });
  }
  for (const [id, c] of before) if (!after.has(id)) changes.push({ id, summary: c.summary, before: c.status, after: undefined });
  return changes;
}
/** The run of the same suite saved just before `run` (runs are newest first). */
export function previousOf(runs: readonly Run[], run: Run): Run | undefined {
  const at = runs.findIndex((r) => r.id === run.id);
  return at < 0 ? undefined : runs.slice(at + 1).find((r) => r.suite === run.suite && r.status !== 'running');
}

/** The labelled values a backend handshake reported (diagnostic_backend in
 *  domain_probe.rs: no host, greeting or credentials are sent back). */
export interface BackendFacts { reachable: boolean | undefined; tls: string | undefined; tlsVersion: string | undefined; capabilityCount: number | undefined; selectedReason: string | undefined; credentialsSent: boolean | undefined }
export function backendFacts(check: Check): BackendFacts {
  const d = check.detail ?? {};
  const str = (k: string): string | undefined => (typeof d[k] === 'string' && d[k] !== '' ? d[k] as string : undefined);
  const num = (k: string): number | undefined => (typeof d[k] === 'number' ? d[k] as number : undefined);
  const bool = (k: string): boolean | undefined => (typeof d[k] === 'boolean' ? d[k] as boolean : undefined);
  return { reachable: bool('reachable'), tls: str('tls'), tlsVersion: str('tlsVersion'), capabilityCount: num('capabilityCount'), selectedReason: str('selectedReason'), credentialsSent: bool('credentialsSent') };
}
