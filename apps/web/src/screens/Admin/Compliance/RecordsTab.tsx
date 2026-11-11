// Compliance > Records of processing: the activities generated from the
// configuration plus the operator's own, and the sub-processor / recipient list.
// Exports are CSV, JSON and print-ready HTML. The records describe what is
// configured; completeness and fitness for any law are for the operator to judge.

import { For, Show, createEffect, createResource, createSignal, createUniqueId, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { complianceApi, saveText, type ComplianceApi, type Lang, type RecordRow } from '../../../api/admin/compliance.ts';
import { InsertDialog } from '../../../components/compose/insert/InsertDialog.tsx';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';
import * as ccss from './compliance.css.ts';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('admin-compliance');

export const ACTIVITY_FIELDS = ['purpose', 'legalBasis', 'subjects', 'categories', 'recipients', 'transfers', 'retention', 'security'] as const;
export const PROCESSOR_FIELDS = ['purpose', 'location', 'safeguards', 'url', 'notes'] as const;
type Kind = 'activity' | 'processor';

const MIME = { csv: 'text/csv;charset=utf-8', json: 'application/json', html: 'text/html;charset=utf-8' } as const;

function Editor(props: { kind: Kind; row: RecordRow | null; onSave: (name: string, fields: Record<string, string>, enabled: boolean) => Promise<void>; onCancel: () => void }): JSX.Element {
  const names = props.kind === 'activity' ? ACTIVITY_FIELDS : PROCESSOR_FIELDS;
  const [name, setName] = createSignal(props.row?.name ?? '');
  const [enabled, setEnabled] = createSignal(props.row?.enabled ?? true);
  const [fields, setFields] = createSignal<Record<string, string>>(Object.fromEntries(names.map((n) => [n, String((props.row as Record<string, unknown> | null)?.[n] ?? '')])));
  const [step, setStep] = createSignal(0);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<unknown>();
  const id = createUniqueId();
  let heading: HTMLHeadingElement | undefined;
  createEffect(() => {
    const current = step();
    if (current > 0) heading?.focus();
  });
  const label = (key: string): string => t(key === 'name' ? 'compliance-records-name' : `compliance-records-field-${key}`);
  const group = () => step() === 0 ? names.slice(0, 2) : names.slice(2);
  const close = () => { if (!saving()) props.onCancel(); };
  async function save(): Promise<void> {
    if (saving() || !name().trim()) return;
    setSaving(true);
    setError(undefined);
    try { await props.onSave(name().trim(), fields(), enabled()); }
    catch (failure) { setError(failure); }
    finally { setSaving(false); }
  }

  return (
    <InsertDialog title={t(props.row ? 'compliance-records-edit' : `compliance-records-add-${props.kind}`)}
      testId="record-editor" busy={saving()} onClose={close}>
      <form class={ccss.sectionBox} onSubmit={(event) => {
        event.preventDefault();
        if (step() < 2) setStep(step() + 1);
        else void save();
      }}>
        <h3 tabindex="-1" ref={heading}>{t(`compliance-record-guide-step-${step()}`)}</h3>
        <p>{t(`compliance-record-guide-${props.kind}-${step()}`)}</p>
        <Show when={step() === 0}>
          <label>{label('name')}
            <input class={ccss.field} value={name()} onInput={(event) => setName(event.currentTarget.value)}
              maxLength={200} required data-testid="record-name" aria-describedby={`${id}-name`} />
          </label>
          <small id={`${id}-name`}>{t(`compliance-record-guide-name-${props.kind}`)}</small>
        </Show>
        <Show when={step() < 2} fallback={
          <>
            <dl>
              <dt>{label('name')}</dt><dd>{name()}</dd>
              <For each={names}>{(key) => <><dt>{label(key)}</dt><dd>{fields()[key] || '—'}</dd></>}</For>
            </dl>
            <label><input type="checkbox" checked={enabled()} disabled={saving()}
              onChange={(event) => setEnabled(event.currentTarget.checked)} /> {t('compliance-records-enabled')}</label>
          </>
        }>
          <For each={group()}>{(key) => (
            <div>
              <label>{label(key)}
                <textarea class={ccss.textarea} value={fields()[key] ?? ''} maxLength={2000}
                  aria-describedby={`${id}-${key}`} title={t(`compliance-record-guide-field-${key}`)}
                  onInput={(event) => setFields({ ...fields(), [key]: event.currentTarget.value })} />
              </label>
              <small id={`${id}-${key}`}>{t(`compliance-record-guide-field-${key}`)}</small>
            </div>
          )}</For>
        </Show>
        <Show when={error()}><p role="alert">{describeAdminError(error())}</p></Show>
        <div class={ccss.row}>
          <Show when={step() > 0}>
            <button type="button" class="btn btn--ghost" disabled={saving()}
              onClick={() => setStep(step() - 1)}>{t('common-back')}</button>
          </Show>
          <button type="submit" class="btn" disabled={saving() || !name().trim()}
            data-testid={step() === 2 ? 'record-save' : 'record-next'}>
            {t(step() === 2 ? 'compliance-records-save' : 'common-next')}
          </button>
        </div>
      </form>
    </InsertDialog>
  );
}

export default function RecordsTab(props: { api?: ComplianceApi; canWrite?: boolean }): JSX.Element {
  const api = props.api ?? complianceApi;
  const canWrite = (): boolean => props.canWrite ?? true;
  const [lang, setLang] = createSignal<Lang>('en');
  const [data, { refetch }] = createResource(lang, (l) => api.records(l));
  const [editing, setEditing] = createSignal<{ kind: Kind; row: RecordRow | null } | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);

  const run = async (fn: () => Promise<unknown>): Promise<void> => {
    try { setFailure(null); await fn(); await refetch(); } catch (e) { setFailure(e); }
  };
  const exportAs = (set: 'activities' | 'processors', format: 'csv' | 'json' | 'html'): void => {
    void run(async () => {
      const text = await api.exportRecords(set, format, lang());
      saveText(text, `${set === 'activities' ? 'processing-records' : 'sub-processors'}.${format}`, MIME[format]);
    });
  };
  const table = (set: 'activities' | 'processors', rows: RecordRow[], kind: Kind): JSX.Element => (
    <>
      <div class={ccss.row}>
        <For each={['csv', 'json', 'html'] as const}>{(f) => (
          <button type="button" class="btn btn--ghost" onClick={() => exportAs(set, f)} data-testid={`export-${set}-${f}`}>{t(`compliance-records-export-${f}`)}</button>
        )}</For>
        <Show when={canWrite()}>
          <button type="button" class="btn" onClick={() => setEditing({ kind, row: null })} data-testid={`add-${kind}`}>{t(`compliance-records-add-${kind}`)}</button>
        </Show>
      </div>
      <div class={css.tableWrap}>
        <table class={css.table} data-testid={`records-${set}`}>
          <thead>
            <tr>
              <th scope="col">{t('compliance-records-name')}</th>
              <th scope="col">{t('compliance-records-field-purpose')}</th>
              <th scope="col">{t(kind === 'activity' ? 'compliance-records-field-recipients' : 'compliance-records-field-location')}</th>
              <th scope="col">{t(kind === 'activity' ? 'compliance-records-field-retention' : 'compliance-records-field-safeguards')}</th>
              <th scope="col">{t('compliance-records-source')}</th>
              <th scope="col"><span class="visually-hidden">{t('compliance-records-actions')}</span></th>
            </tr>
          </thead>
          <tbody>
            <For each={rows}>{(r) => (
              <tr data-testid={`record-${r.id}`}>
                <th scope="row">{r.name}{r.enabled ? '' : ` (${t('compliance-records-off')})`}</th>
                <td>{r.purpose ?? ''}</td>
                <td>{(kind === 'activity' ? r.recipients : r.location) ?? ''}</td>
                <td>{(kind === 'activity' ? r.retention : r.safeguards) ?? ''}</td>
                <td>{r.builtin ? t('compliance-records-generated') : t('compliance-records-custom')}</td>
                <td>
                  <Show when={!r.builtin && canWrite()}>
                    <button type="button" class={ccss.linkButton} onClick={() => setEditing({ kind, row: r })}>{t('compliance-records-edit')}</button>{' '}
                    <button type="button" class={ccss.linkButton} onClick={() => void run(() => api.deleteRecord(r.id))}>{t('compliance-records-delete')}</button>
                  </Show>
                </td>
              </tr>
            )}</For>
          </tbody>
        </table>
      </div>
    </>
  );

  return (
    <div data-testid="compliance-records" class={css.section}>
      <p class={css.note}>{t('compliance-records-intro')}</p>
      <label>{t('compliance-policy-language')}{' '}
        <select class={ccss.field} value={lang()} onChange={(e) => setLang(e.currentTarget.value as Lang)}>
          <option value="en">en</option>
          <option value="pt-PT">pt-PT</option>
        </select>
      </label>
      <Show when={data.error}><p role="alert">{describeAdminError(data.error)}</p></Show>
      <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
      <Show when={editing()} keyed>
        {(e) => (
          <Editor kind={e.kind} row={e.row} onCancel={() => setEditing(null)}
            onSave={async (name, fields, enabled) => {
              if (e.row) await api.updateRecord(e.row.id, e.kind, name, fields, enabled);
              else await api.addRecord(e.kind, name, fields, enabled);
              setEditing(null);
              await run(async () => {});
            }} />
        )}
      </Show>
      <Show when={!data.error ? data.latest : undefined} fallback={<p role="status">{t('admin-shell-loading')}</p>}>
        {(d) => (
          <>
            <AdminSubsection title={t('compliance-records-activities')}>{table('activities', d().activities, 'activity')}</AdminSubsection>
            <AdminSubsection title={t('compliance-records-processors')} description={t('compliance-records-processors-note')}>{table('processors', d().processors, 'processor')}</AdminSubsection>
          </>
        )}
      </Show>
    </div>
  );
}
