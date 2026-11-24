#!/usr/bin/env node
// goal-65-35 write-shape smoke (t60): one validated pass of every write shape of
// the goal mix, plus candidate shapes for delete-to-trash, draft save and search,
// against a real seeded fixture server. A smoke, not a measurement.
// Selected by MW_NATIVE_LOAD_DRIVER=goal-mix-smoke with MW_NATIVE_LOAD_WORKLOAD=goal-65-35.
import { readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { representativeSpecs, goalMixWriteSpecs, GOAL_MIX_TABLE } from '../representative-workload.mjs';

const USING = ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail'];

/** Candidate templates for the shapes the mix does not have yet (for t53s-e1). */
export function candidateSpecs(p) {
  const call = (method, args, tag) => [method, { accountId: p.accountId, ...args }, tag];
  const batch = (name, calls, validate) => ({ name, body: JSON.stringify({ using: USING, methodCalls: calls }), validate: value => {
    const rows = value?.methodResponses;
    return Boolean(Array.isArray(rows) && rows.length === calls.length && rows.every((row, i) => row[0] === calls[i][0] && row[2] === calls[i][2]) && validate(rows.map(row => row[1])));
  } });
  const id = p.messageIds[4];
  const inFolder = (row, folder) => row.list?.length === 1 && row.list[0].id === id && row.list[0].mailboxIds?.[folder] === true && Object.keys(row.list[0].mailboxIds).length === 1;
  const changed = row => row.updated && Object.hasOwn(row.updated, id) && !row.notUpdated;
  return [
    batch('delete-to-trash-and-restore', [
      call('Email/set', { update: { [id]: { [`mailboxIds/${p.trashId}`]: true, [`mailboxIds/${p.inboxId}`]: null } } }, 'trash'),
      call('Email/get', { ids: [id], properties: ['id', 'mailboxIds'] }, 'trashed'),
      call('Email/set', { update: { [id]: { [`mailboxIds/${p.inboxId}`]: true, [`mailboxIds/${p.trashId}`]: null } } }, 'restore'),
      call('Email/get', { ids: [id], properties: ['id', 'mailboxIds'] }, 'restored'),
    ], ([a, ab, b, bb]) => changed(a) && inFolder(ab, p.trashId) && changed(b) && inFolder(bb, p.inboxId)),
    // Single-request form with creation back-references; kept to show whether the engine resolves them.
    batch('draft-save-backreference', [
      call('Email/set', { create: { d1: { mailboxIds: { [p.draftsId]: true }, keywords: { $draft: true, $seen: true }, to: [{ email: 'someone@native-load.invalid' }], subject: 'Goal mix draft', bodyValues: { b1: { value: 'Draft body.' } }, textBody: [{ partId: 'b1', type: 'text/plain' }] } } }, 'save'),
      call('Email/get', { '#ids': { resultOf: 'save', name: 'Email/set', path: '/created/d1/id' }, properties: ['id', 'mailboxIds', 'keywords', 'subject'] }, 'saved'),
      call('Email/set', { destroy: ['#d1'] }, 'discard'),
    ], ([save, saved, discard]) => {
      const created = save.created?.d1?.id;
      return Boolean(created) && saved.list?.length === 1 && saved.list[0].mailboxIds?.[p.draftsId] === true && saved.list[0].keywords?.$draft === true
        && Array.isArray(discard.destroyed) && discard.destroyed.includes(created) && !discard.notDestroyed;
    }),
    // Two-request form: save, then discard by the id the save returned.
    { ...batch('draft-save', [
      call('Email/set', { create: { d1: { mailboxIds: { [p.draftsId]: true }, keywords: { $draft: true, $seen: true }, to: [{ email: 'someone@native-load.invalid' }], subject: 'Goal mix draft', bodyValues: { b1: { value: 'Draft body.' } }, textBody: [{ partId: 'b1', type: 'text/plain' }] } } }, 'save'),
    ], ([save]) => Boolean(save.created?.d1?.id) && !save.notCreated),
      followUp: value => {
        const created = value.methodResponses[0][1].created.d1.id;
        return batch('draft-discard', [
          call('Email/get', { ids: [created], properties: ['id', 'mailboxIds', 'keywords'] }, 'saved'),
          call('Email/set', { destroy: [created] }, 'discard'),
        ], ([saved, discard]) => saved.list?.length === 1 && saved.list[0].mailboxIds?.[p.draftsId] === true && saved.list[0].keywords?.$draft === true && discard.destroyed?.includes(created) && !discard.notDestroyed);
      } },
    batch('search-planted-term', [
      call('Email/query', { filter: { inMailbox: p.inboxId, text: 'quarterly' }, sort: [{ property: 'receivedAt', isAscending: false }], limit: 50, calculateTotal: true }, 'q'),
      call('Email/get', { '#ids': { resultOf: 'q', name: 'Email/query', path: '/ids' }, properties: ['id', 'subject', 'receivedAt'] }, 'g'),
    ], ([q, g]) => q.total === p.plantedQuarterly && Array.isArray(q.ids) && q.ids.length === Math.min(50, p.plantedQuarterly) && g.list?.length === q.ids.length),
  ];
}

async function send(handoff, p, spec) {
  const response = await fetch(`${handoff.base}/jmap/api`, { method: 'POST', headers: { cookie: p.cookie, 'x-csrf-token': p.csrfToken, 'x-forwarded-for': p.clientAddress, origin: handoff.base, 'content-type': 'application/json' }, body: spec.body });
  const text = await response.text();
  let value = null;
  try { value = JSON.parse(text); } catch {}
  const ok = response.status === 200 && Boolean(value) && spec.validate(value);
  if (ok && spec.followUp) {
    const next = await send(handoff, p, spec.followUp(value));
    return { status: next.status, ok: next.ok, excerpt: next.excerpt, followUp: true };
  }
  return { status: response.status, ok, excerpt: text.slice(0, 1500) };
}
async function upstream(handoff) {
  const response = await fetch(`${handoff.base}${handoff.diagnostics.path}`, { headers: { 'x-fixture-token': handoff.diagnostics.token } });
  return response.status === 200 ? (await response.json()).upstreamCommands ?? null : null;
}

export async function run(handoff) {
  const p = handoff.principals[0];
  const writes = new Set(GOAL_MIX_TABLE.filter(row => row.kind === 'write').map(row => row.name));
  const mix = [...representativeSpecs(p), ...goalMixWriteSpecs(p)].filter(spec => writes.has(spec.name));
  const shapes = [...mix.map(spec => ({ group: 'goal-mix-write', spec })), ...candidateSpecs(p).map(spec => ({ group: 'candidate', spec }))];
  const results = [];
  for (const { group, spec } of shapes) {
    const before = await upstream(handoff);
    const passes = [];
    for (let pass = 0; pass < 2; pass++) passes.push(await send(handoff, p, spec));
    const after = await upstream(handoff);
    const delta = before && after ? Object.fromEntries(Object.entries(after).filter(([, v]) => typeof v === 'number').map(([k, v]) => [k, v - (before[k] ?? 0)])) : null;
    results.push({ group, name: spec.name, validated: passes.every(r => r.ok), passes: passes.map(r => ({ status: r.status, ok: r.ok, ...(r.ok ? {} : { excerpt: r.excerpt }) })), upstreamCommandsForTwoPasses: delta });
  }
  const missing = [...writes].filter(name => !mix.some(spec => spec.name === name));
  return { kind: 'mw-goal-mix-smoke-v1', note: 'two passes per shape on one principal; a smoke, not a measurement', missingTemplates: missing, results, status: results.filter(r => r.name !== 'draft-save-backreference').every(r => r.validated) && !missing.length ? 'OK' : 'FAILED' };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const [handoffPath, reportPath] = process.argv.slice(2);
  const report = await run(JSON.parse(readFileSync(handoffPath, 'utf8')));
  writeFileSync(reportPath, JSON.stringify(report, null, 2));
  for (const r of report.results) console.error(`goal-mix smoke: ${r.group} ${r.name} ${r.validated ? 'OK' : 'FAILED'}`);
  process.exit(report.status === 'OK' ? 0 : 1);
}
