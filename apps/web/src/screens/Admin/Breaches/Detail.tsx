// One incident: the notice clock, risk questionnaire, details, timeline and
// evidence, the authority notice, data subject communication, the containment
// checklist with one-click playbooks, and the trail and delivery log.
import { For, Show, createResource, createSignal, onCleanup, type JSX } from 'solid-js';
import { formatDateTime, t } from '../../../i18n';
import {
  addEvidence, addTimeline, assessBreach, assignBreach, authorityFile, changeAwareAt, checkPlaybook, confirmBreach, loadBreach,
  lockdown, moveBreach, previewSubjectNotice, recordAuthorityNotice, reportFile, sendSubjectNotice, updateBreach,
  type BreachDetail, type BreachIncident, type BreachMeta, type BreachSeverity, type BreachStatus, type LockdownResult, type SubjectNoticePlan,
} from '../../../api/admin/breaches.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';
import { NEXT_STATUSES, clockTone, fileName, fromLocalInput, needsReason, remaining, saveFile, splitList, toLocalInput } from './model.ts';

const when = (s: number | null | undefined): string => (s === null || s === undefined ? '' : formatDateTime(s * 1000));

export function Countdown(props: { clock: BreachIncident['clock']; now: number }): JSX.Element {
  const tone = (): string => clockTone(props.clock, props.now);
  const text = (): string => {
    const c = props.clock;
    if (c.phase === 'not-started') return t('admin-breach-clock-draft');
    if (c.phase === 'stopped-on-time') return t('admin-breach-clock-on-time', { at: when(c.notifiedAt) });
    if (c.phase === 'stopped-late') return t('admin-breach-clock-late', { at: when(c.notifiedAt) });
    if (c.phase === 'not-required') return t('admin-breach-clock-not-required');
    const r = remaining(c.deadlineAt ?? 0, props.now);
    return r.overdue ? t('admin-breach-clock-overdue', { hours: r.hours, minutes: r.minutes }) : t('admin-breach-clock-left', { hours: r.hours, minutes: r.minutes });
  };
  return <span data-testid="breach-countdown" data-tone={tone()} role={tone() === 'overdue' ? 'alert' : undefined}>{text()}</span>;
}

export function Detail(props: { id: string; meta: BreachMeta | undefined; now: () => number; onChange: () => void; onClose: () => void }): JSX.Element {
  const [data, { refetch }] = createResource(() => props.id, (id) => loadBreach(id));
  const [error, setError] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  async function act(work: () => Promise<unknown>): Promise<void> {
    setBusy(true);
    setError('');
    try { await work(); await refetch(); props.onChange(); } catch (e) { setError(describeAdminError(e)); } finally { setBusy(false); }
  }
  const timer = setInterval(() => void 0, 60_000);
  onCleanup(() => clearInterval(timer));
  return (
    <Show when={data()} fallback={<p role="status">{t('admin-breach-loading')}</p>}>
      {(d) => <Body detail={d()} meta={props.meta} now={props.now} busy={busy()} error={error()} setError={setError} act={act} onClose={props.onClose} />}
    </Show>
  );
}

type Act = (work: () => Promise<unknown>) => Promise<void>;

function Body(props: { detail: BreachDetail; meta: BreachMeta | undefined; now: () => number; busy: boolean; error: string; setError: (s: string) => void; act: Act; onClose: () => void }): JSX.Element {
  const i = (): BreachIncident => props.detail.incident;
  const b = (): BreachIncident['body'] => i().body;
  const [reason, setReason] = createSignal('');
  const [target, setTarget] = createSignal<BreachStatus>('assessing');
  const [awareEdit, setAwareEdit] = createSignal('');
  const [awareReason, setAwareReason] = createSignal('');
  const [answers, setAnswers] = createSignal<Record<string, string>>({ ...i().body.answers });
  const [severity, setSeverity] = createSignal<string>('');
  const [rationale, setRationale] = createSignal(i().body.assessment?.rationale ?? '');
  const [tlKind, setTlKind] = createSignal('containment');
  const [tlText, setTlText] = createSignal('');
  const [evKind, setEvKind] = createSignal<'text' | 'link'>('text');
  const [evLabel, setEvLabel] = createSignal('');
  const [evValue, setEvValue] = createSignal('');
  const [assignee, setAssignee] = createSignal(i().assignee ?? '');
  const [recPhase, setRecPhase] = createSignal('');
  const [recRef, setRecRef] = createSignal('');
  const [recAt, setRecAt] = createSignal('');
  const [recDelay, setRecDelay] = createSignal(i().body.delayReasons);
  const [audKind, setAudKind] = createSignal<'all' | 'selected' | 'domain'>('all');
  const [audList, setAudList] = createSignal('');
  const [audDomain, setAudDomain] = createSignal('');
  const [locale, setLocale] = createSignal('');
  const [advice, setAdvice] = createSignal('');
  const [plan, setPlan] = createSignal<SubjectNoticePlan | null>(null);
  const [lock, setLock] = createSignal<LockdownResult | null>(null);
  const [form, setForm] = createSignal({
    description: i().body.description, consequences: i().body.consequences, measuresTaken: i().body.measuresTaken, measuresProposed: i().body.measuresProposed,
    subjects: i().body.subjectCategories.join(', '), data: i().body.dataCategories.join(', '), approxSubjects: i().body.approxSubjects === null ? '' : String(i().body.approxSubjects),
    approxRecords: i().body.approxRecords === null ? '' : String(i().body.approxRecords), affected: i().body.affected.map((a) => a.address).join('\n'),
  });
  const setF = (k: keyof ReturnType<typeof form>, v: string): void => { const next = { ...form(), [k]: v }; setForm(() => next); };
  const num = (s: string): number | null => (s.trim() === '' || !Number.isFinite(Number(s)) ? null : Math.max(0, Math.floor(Number(s))));
  const audience = () => ({ kind: audKind(), addresses: splitList(audList()), domain: audDomain() });
  const open = (): boolean => !['closed', 'not-notifiable'].includes(i().status);
  const download = async (kind: 'report' | 'html' | 'json'): Promise<void> => {
    try {
      if (kind === 'report') saveFile(fileName(i().reference, 'report', 'html'), await reportFile(i().id, 'html'), 'text/html');
      else if (kind === 'html') saveFile(fileName(i().reference, 'authority', 'html'), await authorityFile(i().id, 'html'), 'text/html');
      else saveFile(fileName(i().reference, 'authority', 'json'), await authorityFile(i().id, 'json-file'), 'application/json');
    } catch (e) { props.setError(describeAdminError(e)); }
  };
  return (
    <section class={css.card} aria-label={t('admin-breach-detail')} data-testid="breach-detail">
      <div class={css.headingRow}>
        <h3 class={css.subHeading}>{i().reference} · {i().title}</h3>
        <button type="button" class="btn btn--ghost" onClick={props.onClose}>{t('admin-breach-close')}</button>
      </div>
      <dl>
        <dt>{t('admin-breach-col-status')}</dt><dd>{t(`admin-breach-status-${i().status}`)} · {t(`admin-breach-sev-${i().severity}`)}</dd>
        <dt>{t('admin-breach-clock')}</dt><dd><Countdown clock={i().clock} now={props.now()} /></dd>
        <Show when={i().awareAt !== null}><dt>{t('admin-breach-aware-at')}</dt><dd>{when(i().awareAt)}</dd><dt>{t('admin-breach-deadline')}</dt><dd>{when(i().deadlineAt)} ({t('admin-breach-window', { hours: i().noticeHours })})</dd></Show>
        <dt>{t('admin-breach-detected-at')}</dt><dd>{when(i().detectedAt)} · {t(`admin-breach-source-${i().source}`)}</dd>
      </dl>
      <Show when={i().draft}>
        <div role="status" data-testid="breach-draft">
          <p>{t('admin-breach-draft-note')}</p>
          <button type="button" class="btn" disabled={props.busy} onClick={() => void props.act(() => confirmBreach(i().id))} data-testid="breach-confirm">{t('admin-breach-confirm')}</button>
        </div>
      </Show>
      <Show when={i().delayReasonsRequired}>
        <p role="alert" data-testid="breach-delay-required">{t('admin-breach-delay-required')}</p>
      </Show>

      <AdminSubsection title={t('admin-breach-h-status')}>
        <div class={css.grid}>
          <label>{t('admin-breach-move-to')}
            <select value={target()} onChange={(e) => setTarget(e.currentTarget.value as BreachStatus)} data-testid="breach-target">
              <For each={NEXT_STATUSES.filter((s) => s !== i().status)}>{(s) => <option value={s}>{t(`admin-breach-status-${s}`)}</option>}</For>
            </select>
          </label>
          <label>{t('admin-breach-reason')}<input value={reason()} onInput={(e) => setReason(e.currentTarget.value)} data-testid="breach-reason" /></label>
          <button type="button" class="btn" disabled={props.busy || (needsReason(i().status, target()) && reason().trim().length < 10)} onClick={() => void props.act(async () => { await moveBreach(i().id, target(), reason()); setReason(''); })} data-testid="breach-move">{t('admin-breach-move')}</button>
        </div>
        <div class={css.grid}>
          <label>{t('admin-breach-assignee')}<input value={assignee()} onInput={(e) => setAssignee(e.currentTarget.value)} /></label>
          <button type="button" class="btn" disabled={props.busy} onClick={() => void props.act(() => assignBreach(i().id, assignee().trim() || null))}>{t('admin-breach-assign')}</button>
        </div>
        <Show when={i().awareAt !== null}>
          <div class={css.grid}>
            <label>{t('admin-breach-aware-new')}<input type="datetime-local" value={awareEdit() || toLocalInput(i().awareAt ?? 0)} onInput={(e) => setAwareEdit(e.currentTarget.value)} data-testid="breach-aware-input" /></label>
            <label>{t('admin-breach-aware-why')}<input value={awareReason()} onInput={(e) => setAwareReason(e.currentTarget.value)} data-testid="breach-aware-reason" /></label>
            <button type="button" class="btn" disabled={props.busy || awareReason().trim().length < 5 || fromLocalInput(awareEdit()) === undefined} onClick={() => void props.act(async () => { await changeAwareAt(i().id, fromLocalInput(awareEdit()) ?? 0, awareReason()); setAwareEdit(''); setAwareReason(''); })} data-testid="breach-aware-save">{t('admin-breach-aware-save')}</button>
          </div>
        </Show>
      </AdminSubsection>

      <AdminSubsection title={t('admin-breach-h-assess')} description={t('admin-breach-assess-note')}>
        <For each={props.meta?.questions ?? []}>{(q) => (
          <label>{t(`admin-breach-q-${q.id}`)}
            <select value={answers()[q.id] ?? ''} onChange={(e) => setAnswers({ ...answers(), [q.id]: e.currentTarget.value })} data-testid={`breach-q-${q.id}`}>
              <option value="">{t('admin-breach-unanswered')}</option>
              <For each={q.options}>{(o) => <option value={o}>{t(`admin-breach-q-${q.id}-${o}`)}</option>}</For>
            </select>
          </label>
        )}</For>
        <p data-testid="breach-suggestion">{t('admin-breach-suggestion', { level: t(`admin-breach-sev-${i().suggestion.level}`), score: i().suggestion.score, answered: i().suggestion.answered, total: i().suggestion.total })}</p>
        <label>{t('admin-breach-risk-level')}
          <select value={severity()} onChange={(e) => setSeverity(e.currentTarget.value)} data-testid="breach-severity">
            <option value="">{t('admin-breach-use-suggestion')}</option>
            <For each={(props.meta?.severities ?? []).filter((s) => s !== 'unassessed')}>{(s) => <option value={s}>{t(`admin-breach-sev-${s}`)}</option>}</For>
          </select>
        </label>
        <label>{t('admin-breach-rationale')}<textarea rows="2" value={rationale()} onInput={(e) => setRationale(e.currentTarget.value)} data-testid="breach-rationale" /></label>
        <button type="button" class="btn" disabled={props.busy} onClick={() => void props.act(() => assessBreach(i().id, answers(), (severity() || undefined) as BreachSeverity | undefined, rationale()))} data-testid="breach-assess">{t('admin-breach-assess-save')}</button>
      </AdminSubsection>

      <AdminSubsection title={t('admin-breach-h-details')}>
        <label>{t('admin-breach-description')}<textarea rows="3" value={form().description} onInput={(e) => setF('description', e.currentTarget.value)} /></label>
        <label>{t('admin-breach-subject-categories')}<input value={form().subjects} onInput={(e) => setF('subjects', e.currentTarget.value)} /></label>
        <label>{t('admin-breach-data-categories')}<input value={form().data} onInput={(e) => setF('data', e.currentTarget.value)} /></label>
        <div class={css.grid}>
          <label>{t('admin-breach-approx-subjects')}<input type="number" min="0" value={form().approxSubjects} onInput={(e) => setF('approxSubjects', e.currentTarget.value)} /></label>
          <label>{t('admin-breach-approx-records')}<input type="number" min="0" value={form().approxRecords} onInput={(e) => setF('approxRecords', e.currentTarget.value)} /></label>
        </div>
        <label>{t('admin-breach-consequences')}<textarea rows="2" value={form().consequences} onInput={(e) => setF('consequences', e.currentTarget.value)} /></label>
        <label>{t('admin-breach-measures-taken')}<textarea rows="2" value={form().measuresTaken} onInput={(e) => setF('measuresTaken', e.currentTarget.value)} /></label>
        <label>{t('admin-breach-measures-proposed')}<textarea rows="2" value={form().measuresProposed} onInput={(e) => setF('measuresProposed', e.currentTarget.value)} /></label>
        <label>{t('admin-breach-affected')}<textarea rows="3" value={form().affected} onInput={(e) => setF('affected', e.currentTarget.value)} data-testid="breach-affected" /></label>
        <button type="button" class="btn" disabled={props.busy} data-testid="breach-details-save" onClick={() => void props.act(() => updateBreach(i().id, {
          description: form().description, consequences: form().consequences, measuresTaken: form().measuresTaken, measuresProposed: form().measuresProposed,
          subjectCategories: splitList(form().subjects), dataCategories: splitList(form().data), approxSubjects: num(form().approxSubjects), approxRecords: num(form().approxRecords),
          affected: splitList(form().affected),
        }))}>{t('admin-breach-details-save')}</button>
      </AdminSubsection>

      <AdminSubsection title={t('admin-breach-h-timeline')}>
        <ol data-testid="breach-timeline"><For each={b().timeline}>{(e) => <li>{when(e.ts)} · {e.actor} · {t(`admin-breach-tl-${e.kind}`)}: {e.text}</li>}</For></ol>
        <div class={css.grid}>
          <label>{t('admin-breach-tl-kind')}
            <select value={tlKind()} onChange={(e) => setTlKind(e.currentTarget.value)}>
              <For each={['detection', 'containment', 'closure', 'note', 'subjects-informed']}>{(k) => <option value={k}>{t(`admin-breach-tl-${k}`)}</option>}</For>
            </select>
          </label>
          <label>{t('admin-breach-tl-text')}<input value={tlText()} onInput={(e) => setTlText(e.currentTarget.value)} data-testid="breach-tl-text" /></label>
          <button type="button" class="btn" disabled={props.busy || !tlText().trim()} onClick={() => void props.act(async () => { await addTimeline(i().id, tlKind(), tlText()); setTlText(''); })} data-testid="breach-tl-add">{t('admin-breach-add')}</button>
        </div>
        <h4 class={css.subHeading}>{t('admin-breach-h-evidence')}</h4>
        <ul data-testid="breach-evidence"><For each={b().evidence}>{(e) => <li>{e.label}: <Show when={e.kind === 'link'} fallback={e.value}><a href={e.value} rel="noopener noreferrer" target="_blank">{e.value}</a></Show></li>}</For></ul>
        <div class={css.grid}>
          <label>{t('admin-breach-ev-kind')}
            <select value={evKind()} onChange={(e) => setEvKind(e.currentTarget.value as 'text' | 'link')}>
              <option value="text">{t('admin-breach-ev-text')}</option><option value="link">{t('admin-breach-ev-link')}</option>
            </select>
          </label>
          <label>{t('admin-breach-ev-label')}<input value={evLabel()} onInput={(e) => setEvLabel(e.currentTarget.value)} /></label>
          <label>{t('admin-breach-ev-value')}<input value={evValue()} onInput={(e) => setEvValue(e.currentTarget.value)} /></label>
          <button type="button" class="btn" disabled={props.busy || !evLabel().trim() || !evValue().trim()} onClick={() => void props.act(async () => { await addEvidence(i().id, evKind(), evLabel(), evValue()); setEvLabel(''); setEvValue(''); })}>{t('admin-breach-add')}</button>
        </div>
      </AdminSubsection>

      <AdminSubsection title={t('admin-breach-h-authority')} description={t('admin-breach-authority-note')}>
        <div class={css.headingRow}>
          <button type="button" class="btn" onClick={() => void download('html')} data-testid="breach-authority-html">{t('admin-breach-authority-html')}</button>
          <button type="button" class="btn" onClick={() => void download('json')} data-testid="breach-authority-json">{t('admin-breach-authority-json')}</button>
          <button type="button" class="btn btn--ghost" onClick={() => void download('report')}>{t('admin-breach-report-html')}</button>
        </div>
        <ul data-testid="breach-authority-list"><For each={b().authorityNotices}>{(n) => <li>{t(`admin-breach-phase-${n.phase}`)} · {when(n.submittedAt)} · {n.reference}<Show when={n.late}> <span class={css.badge}>{t('admin-breach-late')}</span></Show></li>}</For></ul>
        <Show when={open() && i().awareAt !== null}>
          <div class={css.grid}>
            <label>{t('admin-breach-phase')}
              <select value={recPhase()} onChange={(e) => setRecPhase(e.currentTarget.value)}>
                <option value="">{t('admin-breach-phase-auto')}</option>
                <For each={['initial', 'update', 'final']}>{(p) => <option value={p}>{t(`admin-breach-phase-${p}`)}</option>}</For>
              </select>
            </label>
            <label>{t('admin-breach-submitted-at')}<input type="datetime-local" value={recAt()} onInput={(e) => setRecAt(e.currentTarget.value)} /></label>
            <label>{t('admin-breach-authority-ref')}<input value={recRef()} onInput={(e) => setRecRef(e.currentTarget.value)} /></label>
          </div>
          <label>{t('admin-breach-delay-reasons')}<textarea rows="2" value={recDelay()} onInput={(e) => setRecDelay(e.currentTarget.value)} data-testid="breach-delay-reasons" /></label>
          <button type="button" class="btn" disabled={props.busy} data-testid="breach-record" onClick={() => void props.act(() => recordAuthorityNotice(i().id, { phase: recPhase() || undefined, submittedAt: fromLocalInput(recAt()), reference: recRef(), delayReasons: recDelay() } as never))}>{t('admin-breach-record')}</button>
        </Show>
      </AdminSubsection>

      <AdminSubsection title={t('admin-breach-h-subjects')} description={t('admin-breach-subjects-note')}>
        <p>{t('admin-breach-variables', { names: (props.meta?.variables ?? []).map((v) => `{{${v}}}`).join(' ') })}</p>
        <div class={css.grid}>
          <label>{t('admin-breach-audience')}
            <select value={audKind()} onChange={(e) => setAudKind(e.currentTarget.value as 'all' | 'selected' | 'domain')} data-testid="breach-audience">
              <option value="all">{t('admin-breach-aud-all')}</option><option value="selected">{t('admin-breach-aud-selected')}</option><option value="domain">{t('admin-breach-aud-domain')}</option>
            </select>
          </label>
          <label>{t('admin-breach-locale')}
            <select value={locale()} onChange={(e) => setLocale(e.currentTarget.value)}>
              <option value="">{t('admin-breach-locale-auto')}</option><option value="en">{t('admin-breach-lang-en')}</option><option value="pt-PT">{t('admin-breach-lang-pt')}</option>
            </select>
          </label>
        </div>
        <Show when={audKind() === 'selected'}><label>{t('admin-breach-aud-addresses')}<textarea rows="2" value={audList()} onInput={(e) => setAudList(e.currentTarget.value)} /></label></Show>
        <Show when={audKind() === 'domain'}><label>{t('admin-breach-aud-domain-name')}<input value={audDomain()} onInput={(e) => setAudDomain(e.currentTarget.value)} /></label></Show>
        <label>{t('admin-breach-advice')}<textarea rows="2" value={advice()} onInput={(e) => setAdvice(e.currentTarget.value)} /></label>
        <div class={css.headingRow}>
          <button type="button" class="btn" disabled={props.busy} data-testid="breach-preview" onClick={() => void props.act(async () => setPlan(await previewSubjectNotice(i().id, { audience: audience(), locale: locale(), advice: advice() })))}>{t('admin-breach-preview')}</button>
          <button type="button" class="btn" disabled={props.busy || plan() === null || plan()?.dryRun !== true} data-testid="breach-send" onClick={() => void props.act(async () => { await sendSubjectNotice(i().id, { audience: audience(), locale: locale(), advice: advice() }); setPlan(null); })}>{t('admin-breach-send')}</button>
        </div>
        <Show when={plan()}>
          {(p) => (
            <div role="status" data-testid="breach-plan">
              <p>{t('admin-breach-plan', { count: p().recipients.length, channels: p().channels.join(', ') })}</p>
              <Show when={p().preview.unknownVariables.subject.length + p().preview.unknownVariables.body.length > 0}><p role="alert">{t('admin-breach-unknown-vars', { names: [...p().preview.unknownVariables.subject, ...p().preview.unknownVariables.body].join(', ') })}</p></Show>
              <p><strong>{p().preview.subject}</strong></p>
              <pre class={css.mono}>{p().preview.body}</pre>
              <ul><For each={p().recipients}>{(r) => <li>{r.address} · {r.channels.join(', ') || '-'}<For each={r.skipped}>{(s) => <> · {s.channel}: {s.reason}</>}</For></li>}</For></ul>
            </div>
          )}
        </Show>
        <table class={css.table} data-testid="breach-deliveries">
          <caption>{t('admin-breach-deliveries')}</caption>
          <thead><tr><th scope="col">{t('admin-breach-col-recipient')}</th><th scope="col">{t('admin-breach-col-channel')}</th><th scope="col">{t('admin-breach-col-status')}</th><th scope="col">{t('admin-breach-col-when')}</th></tr></thead>
          <tbody><For each={props.detail.deliveries}>{(d) => <tr><td>{d.recipient}</td><td>{d.channel}</td><td>{d.status}{d.detail ? ` (${d.detail})` : ''}</td><td>{when(d.ts)}</td></tr>}</For></tbody>
        </table>
      </AdminSubsection>

      <AdminSubsection title={t('admin-breach-h-playbook')} description={t('admin-breach-playbook-note')}>
        <ul data-testid="breach-playbook">
          <For each={i().playbook}>{(p) => (
            <li><label><input type="checkbox" checked={p.done} disabled={props.busy} onChange={(e) => void props.act(() => checkPlaybook(i().id, p.id, e.currentTarget.checked))} /> {t(`admin-breach-pb-${p.id}`)}</label></li>
          )}</For>
        </ul>
        <div class={css.headingRow}>
          <button type="button" class="btn" disabled={props.busy} data-testid="breach-lock-dry-sessions" onClick={() => void props.act(async () => setLock(await lockdown(i().id, { action: 'revoke-sessions', dryRun: true })))}>{t('admin-breach-lock-dry-sessions')}</button>
          <button type="button" class="btn" disabled={props.busy} data-testid="breach-lock-dry-password" onClick={() => void props.act(async () => setLock(await lockdown(i().id, { action: 'force-password-reset', dryRun: true })))}>{t('admin-breach-lock-dry-password')}</button>
        </div>
        <Show when={lock()}>
          {(l) => (
            <div role="status" data-testid="breach-lock-result">
              <p>{t('admin-breach-lock-result', { action: t(`admin-breach-lock-${l().action}`), count: l().results.length })}</p>
              <ul><For each={l().results}>{(r) => <li>{r.address}: {r.status}{r.sessions !== undefined ? ` (${r.sessions})` : ''}</li>}</For></ul>
              <Show when={l().dryRun}>
                <button type="button" class="btn" disabled={props.busy} data-testid="breach-lock-apply" onClick={() => void props.act(async () => setLock(await lockdown(i().id, { action: l().action as 'revoke-sessions', dryRun: false, confirm: true })))}>{t('admin-breach-lock-apply')}</button>
              </Show>
            </div>
          )}
        </Show>
      </AdminSubsection>

      <AdminSubsection title={t('admin-breach-h-trail')}>
        <ol data-testid="breach-trail"><For each={props.detail.trail}>{(e) => <li>{when(e.ts)} · {e.actor} · {t(`admin-breach-trail-${e.kind}`)}</li>}</For></ol>
      </AdminSubsection>
      <Show when={props.error}><p role="alert" class={css.error}>{props.error}</p></Show>
    </section>
  );
}
