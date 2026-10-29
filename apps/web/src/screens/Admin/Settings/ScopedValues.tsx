// Domain values and per-user values of one setting (worker f01; t32-admin §2.4).
//
// A table of the stored values with Remove (confirmed, naming the object), and a
// form to add or replace one: the domain (or user) and the value, plus "Locked
// for the domain's users" for a domain. A domain lock over per-user values shows
// what it overrules first (dry run, UX6). Refusals show at the field the server's
// pointer names (UX5); writes send If-Match from the row's tag (UX19).

import { Show, createResource, createSignal, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import {
  deleteDomainValue,
  deleteUserValue,
  listScopedValues,
  previewDomain,
  putDomainValue,
  putUserValue,
  type AdminSetting,
  type ScopedValue,
} from '../../../api/admin/settings.ts';
import { AdminProblemError } from '../shell/adminApi.ts';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { DataTable } from '../shell/DataTable.tsx';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { FormField } from '../shell/FormField.tsx';
import { HelpNote } from '../shell/HelpNote.tsx';
import { ImpactPreview } from '../shell/ImpactPreview.tsx';
import { resourceValue } from '../../../components/resourceValue.ts';
import { ValueInput } from './ValueInput.tsx';
import { displayValue, parseDraft, toDraft, type Draft } from './value.ts';
import { impactLabels } from './impact.ts';
import * as shell from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from './settings.css.ts';

export interface ScopedValuesProps {
  setting: AdminSetting;
  /** The setting's plain name. */
  label: string;
  scope: 'domain' | 'admin_user';
  /** The console may write (settings.write for domains, settings.user_values for users). */
  canWrite: boolean;
  disabledReason?: string | null;
  onChanged?: () => void;
}

export function ScopedValues(props: ScopedValuesProps): JSX.Element {
  const isDomain = (): boolean => props.scope === 'domain';
  // The source is the key and scope only: a new tuple on every write of the
  // global value (the parent hands down a new setting object) fetched the rows
  // again for nothing.
  const [rows, { refetch }] = createResource(
    () => props.setting.key,
    (key) => listScopedValues(key, props.scope).then((r) => r.values),
  );
  // Never `rows()` (t75): read while the rows reload after a write it suspends
  // the console's page boundary, and the whole setting page is swapped for the
  // page skeleton and back. The last rows stay on screen while they refresh.
  const current = (): ScopedValue[] | undefined => resourceValue(rows);
  const [target, setTarget] = createSignal('');
  const [draft, setDraft] = createSignal<Draft>(toDraft(props.setting.kind, props.setting.effective.value));
  const [locked, setLocked] = createSignal(false);
  const [targetError, setTargetError] = createSignal<string | null>(null);
  const [valueError, setValueError] = createSignal<string | null>(null);
  const [formError, setFormError] = createSignal<string | null>(null);
  const [done, setDone] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [removing, setRemoving] = createSignal<ScopedValue | null>(null);
  const [preview, setPreview] = createSignal<{ value: unknown; locked: boolean } | null>(null);

  const existing = (id: string): ScopedValue | undefined =>
    current()?.find((r) => r.scopeId.toLowerCase() === id.toLowerCase());

  function edit(row: ScopedValue): void {
    setTarget(row.scopeId);
    setDraft(toDraft(props.setting.kind, row.value));
    setLocked(row.locked);
    setTargetError(null);
    setValueError(null);
  }

  async function write(value: unknown, lock: boolean): Promise<void> {
    const id = target().trim();
    const row = existing(id);
    setBusy(true);
    setFormError(null);
    try {
      if (isDomain()) {
        await putDomainValue(props.setting.key, id, { value, locked: lock }, row?.etag ?? null);
      } else {
        await putUserValue(props.setting.key, id, value, row?.etag ?? null);
      }
      setDone(t('admin-settings-saved'));
      setTarget('');
      await refetch();
      props.onChanged?.();
    } catch (err) {
      if (err instanceof AdminProblemError && err.field === '/value') setValueError(describeAdminError(err));
      else setFormError(describeAdminError(err));
      throw err;
    } finally {
      setBusy(false);
    }
  }

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    setDone(null);
    const id = target().trim();
    setTargetError(id === '' ? t(isDomain() ? 'admin-settings-domain-required' : 'admin-settings-user-required') : null);
    const parsed = parseDraft(props.setting.kind, draft(), props.setting.nullable);
    setValueError(parsed.ok ? null : parsed.error);
    if (id === '' || !parsed.ok) return;
    const lock = isDomain() && locked();
    const newLock = lock && !(existing(id)?.locked ?? false);
    if (newLock && (props.setting.overrides.user ?? 0) + (props.setting.overrides.admin_user ?? 0) > 0) {
      setPreview({ value: parsed.value, locked: lock });
      return;
    }
    await write(parsed.value, lock).catch(() => undefined);
  }

  async function remove(row: ScopedValue): Promise<void> {
    if (isDomain()) await deleteDomainValue(props.setting.key, row.scopeId, row.etag);
    else await deleteUserValue(props.setting.key, row.scopeId, row.etag);
    setRemoving(null);
    setDone(null);
    await refetch();
    props.onChanged?.();
  }

  const heading = (): string => t(isDomain() ? 'admin-settings-domains' : 'admin-settings-users');
  const headingId = (): string => `settings-${props.scope}-heading`;

  return (
    <section class={ux.fieldset} aria-labelledby={headingId()}>
      <h3 class={ux.legend} id={headingId()}>
        {heading()}
      </h3>
      <p class={ux.fieldsetHelp}>{t(isDomain() ? 'admin-settings-domains-help' : 'admin-settings-users-help')}</p>
      <DataTable<ScopedValue>
        caption={t(isDomain() ? 'admin-settings-domains-caption' : 'admin-settings-users-caption', {
          setting: isolate(props.label),
        })}
        columns={[
          {
            id: 'target',
            header: t(isDomain() ? 'admin-settings-domain' : 'admin-settings-user'),
            cell: (r) => <span class={css.keyName}>{r.scopeId}</span>,
          },
          { id: 'value', header: t('admin-settings-col-value'), cell: (r) => displayValue(props.setting.kind, r.value) },
          ...(isDomain()
            ? [
                {
                  id: 'locked',
                  header: t('admin-settings-domain-locked'),
                  cell: (r: ScopedValue) => (r.locked ? t('admin-settings-locked-yes') : t('admin-settings-locked-no')),
                },
              ]
            : []),
        ]}
        rows={current() ?? []}
        rowKey={(r) => r.scopeId}
        state={rows.error ? 'error' : rows.loading ? 'loading' : 'ready'}
        errorText={rows.error ? describeAdminError(rows.error) : ''}
        onRetry={() => void refetch()}
        emptyText={t(isDomain() ? 'admin-settings-domains-empty' : 'admin-settings-users-empty')}
        rowActions={(r) => (
          <Show when={props.canWrite}>
            <button type="button" class="btn btn--ghost" onClick={() => edit(r)}>
              {t('admin-settings-open')}
              <span class={shell.srOnly}> {r.scopeId}</span>
            </button>
            <button type="button" class="btn btn--ghost" onClick={() => setRemoving(r)}>
              {t('admin-settings-domain-remove')}
              <span class={shell.srOnly}> {r.scopeId}</span>
            </button>
          </Show>
        )}
      />

      <Show
        when={props.canWrite}
        fallback={<Show when={props.disabledReason}>{(r) => <HelpNote>{r()}</HelpNote>}</Show>}
      >
        <form class={css.inlineForm} noValidate onSubmit={(e) => void submit(e)} aria-labelledby={`${headingId()}-add`}>
          <h4 class={ux.legend} id={`${headingId()}-add`}>
            {t(isDomain() ? 'admin-settings-domain-add' : 'admin-settings-user-add')}
          </h4>
          <FormField
            label={t(isDomain() ? 'admin-settings-domain' : 'admin-settings-user')}
            {...(isDomain() ? {} : { help: t('admin-settings-user-hint') })}
            error={targetError()}
          >
            {(w) => (
              <input
                id={w.id}
                class={ux.control}
                type="text"
                autocapitalize="none"
                spellcheck={false}
                aria-describedby={w.describedBy}
                aria-invalid={w.invalid}
                value={target()}
                onInput={(e) => setTarget(e.currentTarget.value)}
              />
            )}
          </FormField>
          <FormField
            label={t(isDomain() ? 'admin-settings-domain-value' : 'admin-settings-user-value')}
            error={valueError()}
          >
            {(w) => (
              <ValueInput
                kind={props.setting.kind}
                draft={draft()}
                onDraft={setDraft}
                wiring={w}
                nullable={props.setting.nullable}
              />
            )}
          </FormField>
          <Show when={isDomain()}>
            <label class={ux.checkboxRow}>
              <input type="checkbox" checked={locked()} onChange={(e) => setLocked(e.currentTarget.checked)} />
              <span>{t('admin-settings-domain-locked')}</span>
            </label>
          </Show>
          <button type="submit" class="btn btn--primary" disabled={busy()}>
            {busy() ? t('admin-ux-form-saving') : t(isDomain() ? 'admin-settings-domain-save' : 'admin-settings-user-save')}
          </button>
        </form>
      </Show>
      <div aria-live="polite">
        <Feedback kind="success" message={done()} />
      </div>
      <Feedback kind="error" message={formError()} />

      <ConfirmDialog
        open={removing() !== null}
        title={t(isDomain() ? 'admin-settings-domain-remove-title' : 'admin-settings-user-remove-title', {
          domain: isolate(removing()?.scopeId ?? ''),
          user: isolate(removing()?.scopeId ?? ''),
        })}
        body={t(isDomain() ? 'admin-settings-domain-remove-body' : 'admin-settings-user-remove-body', {
          domain: isolate(removing()?.scopeId ?? ''),
        })}
        confirmLabel={t('admin-settings-domain-remove')}
        onConfirm={() => {
          const r = removing();
          return r ? remove(r) : undefined;
        }}
        onCancel={() => setRemoving(null)}
      />
      <ImpactPreview
        open={preview() !== null}
        title={t('admin-settings-lock-title', { setting: isolate(`${props.label} — ${target()}`) })}
        body={t('admin-settings-lock-body')}
        loadImpact={() => {
          const p = preview();
          return p ? previewDomain(props.setting.key, target().trim(), p) : Promise.resolve(null);
        }}
        labels={impactLabels()}
        confirmLabel={t('admin-settings-lock-confirm')}
        danger={false}
        onConfirm={async () => {
          const p = preview();
          if (p) await write(p.value, p.locked);
          setPreview(null);
        }}
        onCancel={() => setPreview(null)}
      />
    </section>
  );
}
