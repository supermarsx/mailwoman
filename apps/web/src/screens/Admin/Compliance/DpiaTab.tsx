// Compliance > Impact assessments: a register of data protection impact
// assessments with templates, a likelihood by severity matrix, residual risk,
// a status workflow, version history, review reminders and export. It records
// the administrator's assessment; it does not decide whether one is required.

import { For, Show, createResource, createSignal, type JSX } from 'solid-js';
import { formatDateTime, t } from '../../../i18n';
import { complianceApi, saveText, type ComplianceApi, type Dpia, type DpiaBody, type DpiaStatus, type Lang, type Risk } from '../../../api/admin/compliance.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';
import * as ccss from './compliance.css.ts';

export const STATUSES: readonly DpiaStatus[] = ['draft', 'review', 'approved', 'needs-prior-consultation'];
export const TEMPLATES = ['generic', 'assistant', 'email-tracking', 'large-scale-monitoring', 'new-processor'] as const;
const SCALE = [1, 2, 3, 4] as const;

/** Likelihood times severity. */
export const riskScore = (l: number | null | undefined, s: number | null | undefined): number | null => (l && s ? l * s : null);
/** The band of a score: low up to 3, medium up to 6, high up to 9, then very high. */
export function riskLevel(score: number | null): 'low' | 'medium' | 'high' | 'veryHigh' | null {
  if (score === null) return null;
  return score <= 3 ? 'low' : score <= 6 ? 'medium' : score <= 9 ? 'high' : 'veryHigh';
}
const levelText = (score: number | null): string => { const l = riskLevel(score); return l === null ? '' : `${score} ${t(`compliance-dpia-level-${l}`)}`; };

const TEXT_FIELDS = ['description', 'purposes', 'necessity', 'proportionality', 'mitigations', 'dpoOpinion', 'consultation', 'notes'] as const;

function ScoreSelect(props: { value: number | null | undefined; label: string; onChange: (v: number | null) => void; optional?: boolean }): JSX.Element {
  return (
    <select class={`${ccss.field} ${ccss.score}`} aria-label={props.label} value={props.value ?? ''} onChange={(e) => props.onChange(e.currentTarget.value === '' ? null : Number(e.currentTarget.value))}>
      <Show when={props.optional}><option value="">-</option></Show>
      <For each={SCALE}>{(n) => <option value={n}>{n}</option>}</For>
    </select>
  );
}

function Matrix(props: { risks: Risk[] }): JSX.Element {
  return (
    <table class={ccss.matrix} aria-label={t('compliance-dpia-matrix')} data-testid="dpia-matrix">
      <thead><tr><th scope="col">{t('compliance-dpia-severity')} / {t('compliance-dpia-likelihood')}</th><For each={SCALE}>{(l) => <th scope="col">{l}</th>}</For></tr></thead>
      <tbody>
        <For each={[...SCALE].reverse()}>{(s) => (
          <tr>
            <th scope="row">{s}</th>
            <For each={SCALE}>{(l) => (
              <td data-level={riskLevel(l * s)}>{props.risks.map((r, i) => (r.likelihood === l && r.severity === s ? i + 1 : 0)).filter(Boolean).join(', ')}</td>
            )}</For>
          </tr>
        )}</For>
      </tbody>
    </table>
  );
}

function Editor(props: { dpia: Dpia; api: ComplianceApi; canWrite: boolean; onChanged: (d: Dpia) => void; onClose: () => void }): JSX.Element {
  const [body, setBody] = createSignal<DpiaBody>(structuredClone(props.dpia.body));
  const [title, setTitle] = createSignal(props.dpia.title);
  const [status, setStatus] = createSignal<DpiaStatus>(props.dpia.status);
  const [review, setReview] = createSignal(props.dpia.reviewAt ? new Date(props.dpia.reviewAt * 1000).toISOString().slice(0, 10) : '');
  const [error, setError] = createSignal<unknown>(null);
  const [saved, setSaved] = createSignal(false);
  const patchRisk = (i: number, p: Partial<Risk>): void => { setBody({ ...body(), risks: body().risks.map((r, j) => (j === i ? ({ ...r, ...p } as Risk) : r)) }); };
  const save = async (): Promise<void> => {
    try {
      setError(null);
      const reviewAt = review() === '' ? undefined : Math.floor(Date.parse(`${review()}T00:00:00Z`) / 1000);
      const d = await props.api.saveDpia(props.dpia.id, props.dpia.version, { title: title(), status: status(), body: body(), ...(reviewAt !== undefined && !Number.isNaN(reviewAt) ? { reviewAt } : {}) });
      setSaved(true);
      props.onChanged(d);
    } catch (e) { setError(e); }
  };
  const exportAs = async (format: 'html' | 'json'): Promise<void> => {
    try { saveText(await props.api.exportDpia(props.dpia.id, format), `dpia-${props.dpia.id.slice(0, 8)}.${format}`, format === 'html' ? 'text/html;charset=utf-8' : 'application/json'); } catch (e) { setError(e); }
  };
  return (
    <div class={ccss.sectionBox} data-testid="dpia-editor">
      <label>{t('compliance-dpia-title')}
        <input class={ccss.field} value={title()} onInput={(e) => setTitle(e.currentTarget.value)} maxLength={200} disabled={!props.canWrite} data-testid="dpia-title" />
      </label>
      <div class={ccss.row}>
        <label>{t('compliance-dpia-status')}{' '}
          <select class={ccss.field} value={status()} onChange={(e) => setStatus(e.currentTarget.value as DpiaStatus)} disabled={!props.canWrite} data-testid="dpia-status">
            <For each={STATUSES}>{(s) => <option value={s}>{t(`compliance-dpia-status-${s}`)}</option>}</For>
          </select>
        </label>
        <label>{t('compliance-dpia-review-date')}{' '}
          <input type="date" class={ccss.field} value={review()} onInput={(e) => setReview(e.currentTarget.value)} disabled={!props.canWrite} />
        </label>
      </div>
      <For each={TEXT_FIELDS}>{(f) => (
        <label>{t(`compliance-dpia-field-${f}`)}
          <textarea class={ccss.textarea} value={body()[f]} onInput={(e) => setBody({ ...body(), [f]: e.currentTarget.value } as DpiaBody)} maxLength={8000} disabled={!props.canWrite} />
        </label>
      )}</For>

      <h4 class={css.heading}>{t('compliance-dpia-risks')}</h4>
      <Matrix risks={body().risks} />
      <div class={css.tableWrap}>
        <table class={css.table} data-testid="dpia-risks">
          <thead>
            <tr>
              <th scope="col">#</th>
              <th scope="col">{t('compliance-dpia-risk')}</th>
              <th scope="col">{t('compliance-dpia-likelihood')}</th>
              <th scope="col">{t('compliance-dpia-severity')}</th>
              <th scope="col">{t('compliance-dpia-score')}</th>
              <th scope="col">{t('compliance-dpia-mitigations')}</th>
              <th scope="col">{t('compliance-dpia-residual-likelihood')}</th>
              <th scope="col">{t('compliance-dpia-residual-severity')}</th>
              <th scope="col">{t('compliance-dpia-residual')}</th>
              <th scope="col"><span class="visually-hidden">{t('compliance-records-actions')}</span></th>
            </tr>
          </thead>
          <tbody>
            <For each={body().risks}>{(r, i) => (
              <tr data-testid={`dpia-risk-${i()}`}>
                <td>{i() + 1}</td>
                <td>
                  <input class={ccss.field} aria-label={t('compliance-dpia-risk')} value={r.title} onInput={(e) => patchRisk(i(), { title: e.currentTarget.value })} disabled={!props.canWrite} />
                  <textarea class={ccss.textarea} aria-label={t('compliance-dpia-risk-description')} value={r.description} onInput={(e) => patchRisk(i(), { description: e.currentTarget.value })} disabled={!props.canWrite} />
                </td>
                <td><ScoreSelect value={r.likelihood} label={t('compliance-dpia-likelihood')} onChange={(v) => patchRisk(i(), { likelihood: v ?? 1 })} /></td>
                <td><ScoreSelect value={r.severity} label={t('compliance-dpia-severity')} onChange={(v) => patchRisk(i(), { severity: v ?? 1 })} /></td>
                <td data-testid={`dpia-score-${i()}`}>{levelText(riskScore(r.likelihood, r.severity))}</td>
                <td><textarea class={ccss.textarea} aria-label={t('compliance-dpia-mitigations')} value={r.mitigations} onInput={(e) => patchRisk(i(), { mitigations: e.currentTarget.value })} disabled={!props.canWrite} /></td>
                <td><ScoreSelect optional value={r.residualLikelihood} label={t('compliance-dpia-residual-likelihood')} onChange={(v) => patchRisk(i(), { residualLikelihood: v })} /></td>
                <td><ScoreSelect optional value={r.residualSeverity} label={t('compliance-dpia-residual-severity')} onChange={(v) => patchRisk(i(), { residualSeverity: v })} /></td>
                <td data-testid={`dpia-residual-${i()}`}>{levelText(riskScore(r.residualLikelihood, r.residualSeverity))}</td>
                <td><Show when={props.canWrite}><button type="button" class={ccss.linkButton} onClick={() => setBody({ ...body(), risks: body().risks.filter((_, j) => j !== i()) })}>{t('compliance-records-delete')}</button></Show></td>
              </tr>
            )}</For>
          </tbody>
        </table>
      </div>
      <Show when={props.canWrite}>
        <button type="button" class="btn btn--ghost" onClick={() => setBody({ ...body(), risks: [...body().risks, { title: '', description: '', mitigations: '', likelihood: 2, severity: 2 }] })} data-testid="dpia-add-risk">{t('compliance-dpia-add-risk')}</button>
      </Show>

      <h4 class={css.heading}>{t('compliance-dpia-attachments')}</h4>
      <For each={body().links}>{(l, i) => (
        <div class={ccss.row}>
          <input class={ccss.field} aria-label={t('compliance-dpia-link-label')} placeholder={t('compliance-dpia-link-label')} value={l.label} onInput={(e) => setBody({ ...body(), links: body().links.map((x, j) => (j === i() ? { ...x, label: e.currentTarget.value } : x)) })} disabled={!props.canWrite} />
          <input class={ccss.field} aria-label={t('compliance-dpia-link-url')} placeholder="https://" value={l.url} onInput={(e) => setBody({ ...body(), links: body().links.map((x, j) => (j === i() ? { ...x, url: e.currentTarget.value } : x)) })} disabled={!props.canWrite} />
          <input class={ccss.field} aria-label={t('compliance-dpia-link-text')} placeholder={t('compliance-dpia-link-text')} value={l.text} onInput={(e) => setBody({ ...body(), links: body().links.map((x, j) => (j === i() ? { ...x, text: e.currentTarget.value } : x)) })} disabled={!props.canWrite} />
          <Show when={props.canWrite}><button type="button" class={ccss.linkButton} onClick={() => setBody({ ...body(), links: body().links.filter((_, j) => j !== i()) })}>{t('compliance-records-delete')}</button></Show>
        </div>
      )}</For>
      <Show when={props.canWrite}>
        <button type="button" class="btn btn--ghost" onClick={() => setBody({ ...body(), links: [...body().links, { label: '', url: '', text: '' }] })}>{t('compliance-dpia-add-link')}</button>
      </Show>

      <Show when={(props.dpia.history ?? []).length > 0}>
        <h4 class={css.heading}>{t('compliance-policy-history')}</h4>
        <ul data-testid="dpia-history">
          <For each={props.dpia.history}>{(h) => <li>{t('compliance-dpia-history-row', { version: h.version, status: t(`compliance-dpia-status-${h.status}`), by: h.changedBy, date: formatDateTime(h.at * 1000) })}</li>}</For>
        </ul>
      </Show>

      <Show when={error()}><p role="alert">{describeAdminError(error())}</p></Show>
      <Show when={saved()}><p role="status">{t('compliance-dpia-saved')}</p></Show>
      <div class={ccss.row}>
        <Show when={props.canWrite}><button type="button" class="btn" onClick={() => void save()} data-testid="dpia-save">{t('compliance-records-save')}</button></Show>
        <button type="button" class="btn btn--ghost" onClick={() => void exportAs('html')}>{t('compliance-dpia-export-html')}</button>
        <button type="button" class="btn btn--ghost" onClick={() => void exportAs('json')}>{t('compliance-dpia-export-json')}</button>
        <button type="button" class="btn btn--ghost" onClick={props.onClose}>{t('compliance-close')}</button>
      </div>
    </div>
  );
}

export default function DpiaTab(props: { api?: ComplianceApi; canWrite?: boolean }): JSX.Element {
  const api = props.api ?? complianceApi;
  const canWrite = (): boolean => props.canWrite ?? true;
  const [lang, setLang] = createSignal<Lang>('en');
  const [list, { refetch }] = createResource(() => api.dpias());
  const [suggestions, { refetch: refetchSuggestions }] = createResource(() => api.suggestions());
  const [open, setOpen] = createSignal<Dpia | null>(null);
  const [title, setTitle] = createSignal('');
  const [template, setTemplate] = createSignal<string>('generic');
  const [failure, setFailure] = createSignal<unknown>(null);
  const run = async (fn: () => Promise<unknown>): Promise<void> => {
    try { setFailure(null); await fn(); await Promise.all([refetch(), refetchSuggestions()]); } catch (e) { setFailure(e); }
  };
  const create = (name: string, tpl: string): Promise<void> => run(async () => { setOpen(await api.createDpia(name, tpl, lang())); setTitle(''); });
  const openFull = (id: string): Promise<void> => run(async () => { setOpen(await api.dpia(id)); });

  return (
    <div data-testid="compliance-dpia" class={css.section}>
      <p class={css.note}>{t('compliance-dpia-intro')}</p>
      <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
      <Show when={(suggestions() ?? []).length > 0}>
        <AdminSubsection title={t('compliance-dpia-suggested')} description={t('compliance-dpia-suggested-note')}>
          <ul data-testid="dpia-suggestions">
            <For each={suggestions() ?? []}>{(s) => (
              <li>
                {t(`compliance-dpia-feature-${s.feature}`)}{' '}
                <Show when={canWrite()}><button type="button" class={ccss.linkButton} onClick={() => void create(t(`compliance-dpia-feature-${s.feature}`), s.template)}>{t('compliance-dpia-start')}</button></Show>
              </li>
            )}</For>
          </ul>
        </AdminSubsection>
      </Show>

      <Show when={open()}>
        {(d) => (
          <Editor dpia={d()} api={api} canWrite={canWrite()} onClose={() => { setOpen(null); void refetch(); }}
            onChanged={(next) => { setOpen(next); void run(async () => undefined); }} />
        )}
      </Show>

      <AdminSubsection title={t('compliance-dpia-register')}>
        <Show when={list.error}><p role="alert">{describeAdminError(list.error)}</p></Show>
        <Show when={(list() ?? []).length > 0} fallback={<p>{t('compliance-dpia-empty')}</p>}>
          <div class={css.tableWrap}>
            <table class={css.table} data-testid="dpia-list">
              <thead><tr><th scope="col">{t('compliance-dpia-title')}</th><th scope="col">{t('compliance-dpia-status')}</th><th scope="col">{t('compliance-dpia-review-date')}</th><th scope="col">{t('compliance-dpia-version')}</th><th scope="col"><span class="visually-hidden">{t('compliance-records-actions')}</span></th></tr></thead>
              <tbody>
                <For each={list() ?? []}>{(d) => (
                  <tr data-testid={`dpia-row-${d.id}`}>
                    <th scope="row">{d.title}</th>
                    <td>{t(`compliance-dpia-status-${d.status}`)}</td>
                    <td>
                      {d.reviewAt ? formatDateTime(d.reviewAt * 1000) : ''}
                      <Show when={d.due}>{' '}<span class={css.badge}>{d.overdue ? t('compliance-dpia-overdue') : t('compliance-dpia-due')}</span></Show>
                    </td>
                    <td>{d.version}</td>
                    <td>
                      <button type="button" class={ccss.linkButton} onClick={() => void openFull(d.id)}>{t('compliance-records-edit')}</button>{' '}
                      <Show when={canWrite()}><button type="button" class={ccss.linkButton} onClick={() => void run(() => api.deleteDpia(d.id))}>{t('compliance-records-delete')}</button></Show>
                    </td>
                  </tr>
                )}</For>
              </tbody>
            </table>
          </div>
        </Show>
      </AdminSubsection>

      <Show when={canWrite()}>
        <AdminSubsection title={t('compliance-dpia-new')}>
          <form class={ccss.row} onSubmit={(e) => { e.preventDefault(); if (title().trim() !== '') void create(title(), template()); }}>
            <label>{t('compliance-dpia-title')}{' '}
              <input class={ccss.field} value={title()} onInput={(e) => setTitle(e.currentTarget.value)} maxLength={200} data-testid="dpia-new-title" />
            </label>
            <label>{t('compliance-dpia-template')}{' '}
              <select class={ccss.field} value={template()} onChange={(e) => setTemplate(e.currentTarget.value)} data-testid="dpia-new-template">
                <For each={TEMPLATES}>{(tpl) => <option value={tpl}>{t(`compliance-dpia-template-${tpl}`)}</option>}</For>
              </select>
            </label>
            <label>{t('compliance-policy-language')}{' '}
              <select class={ccss.field} value={lang()} onChange={(e) => setLang(e.currentTarget.value as Lang)}>
                <option value="en">en</option>
                <option value="pt-PT">pt-PT</option>
              </select>
            </label>
            <button type="submit" class="btn" data-testid="dpia-create">{t('compliance-dpia-create')}</button>
          </form>
        </AdminSubsection>
      </Show>
    </div>
  );
}
