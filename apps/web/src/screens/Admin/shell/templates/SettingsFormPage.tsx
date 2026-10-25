// Settings form page template (t32-cfg UX3, UX4, UX5, UX7, UX8, UX9, UX13, UX18, UX19).
//
// Fieldsets with headings, one field per setting. Each field has a label, one
// sentence of help with a "Learn more" link to `docs/admin/<area>.md#<field-id>`,
// its source badge (UX9), the propagation note (UX18), "last changed by" with the
// History link (UX8) and Revert to default (UX7). A value the console cannot
// change (env pin, build, lock) is disabled with the reason under it.
//
// A sticky bar at the bottom says how many fields changed and holds Discard and
// Save. Save first validates every field against its declared type and bounds
// (UX5; the bounds come from the registry or catalogue entry) and, when any field
// is wrong, shows each error at its field and focuses the first one; no request
// is made. Then the changed fields are sent with `If-Match` (UX19): a 412 shows
// "Changed by <who> at <time>" with Reload and Keep mine, a 422 shows the server's
// error at the field its JSON pointer names. The result is announced in a live
// region. While anything is unsaved the dirty-state guard asks before leaving.
//
// Every field's holder carries its field anchor, so `?field=<id>` focuses it.
//
// A field or group with `when` is shown only while `when(values)` holds (the
// OIDC fields while the kind is OIDC); a hidden field is neither validated nor
// counted as changed, and `save` still gets every value to pick from. With
// `frame: 'none'` the form is drawn without the page heading, for a form that
// sits inside a list's detail pane (`labelledBy` names the pane's heading).

import { For, Show, createMemo, createSignal, createUniqueId, onMount, untrack, type JSX } from 'solid-js';
import { t } from '../../../../i18n';
import { PasswordInput } from '../../../../components/PasswordInput.tsx';
import { Feedback } from '../Feedback.tsx';
import { Icon } from '../icons.tsx';
import { HelpNote } from '../HelpNote.tsx';
import { SourceBadge, sourceBlocksEdit, sourceReason, type SourceInfo } from '../SourceBadge.tsx';
import { FieldHistory } from '../FieldHistory.tsx';
import { RevertToDefault } from '../RevertToDefault.tsx';
import { StaleNotice } from '../StaleNotice.tsx';
import { fieldAnchorAttrs } from '../fieldAnchors.ts';
import { useFieldErrors } from '../useFieldErrors.ts';
import { useEtag } from '../useEtag.ts';
import { useDirtyGuard } from '../useDirtyGuard.ts';
import { AdminPage, PageState } from './PageFrame.tsx';
import * as css from '../shell.css.ts';
import * as ux from '../ux.css.ts';

export type SettingFieldKind = 'text' | 'number' | 'boolean' | 'select' | 'secret' | 'textarea' | 'url' | 'email';

export interface SettingOption {
  value: string;
  label: string;
}

/** What a custom control (`render`) gets. */
export interface FieldRenderContext {
  id: string;
  describedBy: string | undefined;
  invalid: boolean;
  disabled: boolean;
  value: unknown;
  setValue: (v: unknown) => void;
  /** Every current value of the form (read-only). */
  values: Readonly<Record<string, unknown>>;
}

export interface SettingField {
  /** Value key, field anchor id and the registry key or catalogue id. */
  id: string;
  label: string;
  /** Technical name after the label ("STARTTLS", "MW_LOG_LEVEL"). */
  technical?: string;
  /** One sentence: what it does, what it does not cover (UX4). */
  help: string;
  /** "Learn more" target, `docs/admin/<area>.md#<field-id>`. */
  docsHref?: string;
  kind: SettingFieldKind;
  required?: boolean;
  min?: number;
  max?: number;
  step?: number;
  maxLength?: number;
  /** A regular expression the whole value must match. */
  pattern?: string;
  /** The sentence shown when `pattern` does not match. */
  patternHint?: string;
  options?: readonly SettingOption[];
  placeholder?: string;
  source?: SourceInfo;
  /** Hot: applies within 30 s; restart: after a restart (UX18). */
  propagation?: 'hot' | 'restart';
  /** Audit target and last change (UX8). */
  history?: { target: string; updatedBy?: string | null; updatedAt?: string | null };
  /** Revert to default (UX7): the default as shown, and the action (deletes the stored value).
   *  With `value` instead of `isDefault`/`onRevert`, the form puts the default in the field
   *  (a change saved with the others) and hides the button while the field holds it. */
  revert?: { defaultDisplay: string; isDefault?: boolean; onRevert?: () => void | Promise<void>; value?: unknown };
  /** Extra client rule; returns the sentence to show, or null. */
  validate?: (value: unknown, values: Readonly<Record<string, unknown>>) => string | null;
  /** A control the kinds above do not cover. */
  render?: (ctx: FieldRenderContext) => JSX.Element;
  /** Show the field only while this holds for the current values. */
  when?: (values: Readonly<Record<string, unknown>>) => boolean;
}

export interface SettingsGroup {
  id: string;
  heading: string;
  help?: string;
  fields: readonly SettingField[];
  /** Show the group only while this holds for the current values. */
  when?: (values: Readonly<Record<string, unknown>>) => boolean;
}

export type SettingsLoad = { values: Record<string, unknown>; etag?: string | null } | 'unavailable';

export interface SettingsSaveContext {
  ifMatch: string | null;
  values: Readonly<Record<string, unknown>>;
}

export interface SettingsFormPageProps {
  title: string;
  description?: string;
  groups: readonly SettingsGroup[];
  /** Reads the values (and their tag). `unavailable` when the server has no such route yet. */
  load: () => Promise<SettingsLoad>;
  /** Writes the changed fields. May return the new tag and the stored values. */
  save: (
    changed: Record<string, unknown>,
    ctx: SettingsSaveContext,
  ) => Promise<{ etag?: string | null; values?: Record<string, unknown> } | void>;
  /** "Not available — <reason>" when `load` says `unavailable`. */
  unavailableReason?: string;
  /** Map a 422 JSON pointer to a field id when `/a/b` → `a.b` does not fit. */
  fieldFor?: (pointer: string) => string | undefined;
  actions?: JSX.Element;
  saveLabel?: string;
  /** `none`: no page heading or description, for a form inside a detail pane. Default `page`. */
  frame?: 'page' | 'none';
  /** With `frame: 'none'`, the id of the heading that names the form. */
  labelledBy?: string;
  testId?: string;
}

const same = (a: unknown, b: unknown): boolean => a === b || JSON.stringify(a) === JSON.stringify(b);

/** Client validation of one field (UX5). Returns the sentence to show, or null. */
export function validateSettingField(f: SettingField, value: unknown, values: Readonly<Record<string, unknown>> = {}): string | null {
  const empty = value === undefined || value === null || value === '';
  if (f.kind === 'secret') {
    // Write-only: empty means "keep the stored secret".
    if (!empty && f.maxLength !== undefined && String(value).length > f.maxLength) return t('admin-ux-err-max-length', { max: f.maxLength });
    return f.validate?.(value, values) ?? null;
  }
  if (empty) return f.required ? t('admin-ux-err-required') : (f.validate?.(value, values) ?? null);
  if (f.kind === 'number') {
    const n = typeof value === 'number' ? value : Number(value);
    if (!Number.isFinite(n)) return t('admin-ux-err-number');
    if (f.step !== undefined && f.step >= 1 && !Number.isInteger(n)) return t('admin-ux-err-whole');
    if (f.min !== undefined && n < f.min) return t('admin-ux-err-min', { min: f.min });
    if (f.max !== undefined && n > f.max) return t('admin-ux-err-max', { max: f.max });
  }
  if (typeof value === 'string') {
    if (f.maxLength !== undefined && value.length > f.maxLength) return t('admin-ux-err-max-length', { max: f.maxLength });
    if (f.kind === 'url') {
      let ok = false;
      try {
        const u = new URL(value);
        ok = u.protocol === 'https:' || u.protocol === 'http:';
      } catch {
        ok = false;
      }
      if (!ok) return t('admin-ux-err-url');
    }
    if (f.kind === 'email' && !/^[^\s@]+@[^\s@]+$/.test(value)) return t('admin-ux-err-email');
    if (f.pattern !== undefined && !new RegExp(`^(?:${f.pattern})$`, 'u').test(value)) return f.patternHint ?? t('admin-ux-err-pattern');
  }
  if (f.kind === 'select' && f.options !== undefined && !f.options.some((o) => o.value === value)) return t('admin-ux-err-option');
  return f.validate?.(value, values) ?? null;
}

export function SettingsFormPage(props: SettingsFormPageProps): JSX.Element {
  const id = createUniqueId();
  const [state, setState] = createSignal<'loading' | 'ready' | 'error' | 'unavailable'>('loading');
  const [loadError, setLoadError] = createSignal<unknown>(null);
  const [initial, setInitial] = createSignal<Record<string, unknown>>({});
  const [draft, setDraft] = createSignal<Record<string, unknown>>({});
  const [saving, setSaving] = createSignal(false);
  const [result, setResult] = createSignal<string | null>(null);

  const fields = createMemo(() => props.groups.flatMap((g) => g.fields));
  // The fields on screen for the current values (`when` on the field and its group).
  const visible = createMemo(() =>
    props.groups.flatMap((g) => (g.when && !g.when(draft()) ? [] : g.fields.filter((f) => !f.when || f.when(draft())))),
  );
  const errors = useFieldErrors({
    ...(props.fieldFor ? { fieldFor: props.fieldFor } : {}),
    known: () => fields().map((f) => f.id),
  });
  const etag = useEtag();

  const changedIds = createMemo(() =>
    visible()
      .filter((f) => (f.kind === 'secret' ? typeof draft()[f.id] === 'string' && draft()[f.id] !== '' : !same(draft()[f.id], initial()[f.id])))
      .map((f) => f.id),
  );
  useDirtyGuard(() => changedIds().length > 0);

  async function load(): Promise<void> {
    setState('loading');
    setResult(null);
    errors.clear();
    try {
      const r = await props.load();
      if (r === 'unavailable') {
        setState('unavailable');
        return;
      }
      setInitial({ ...r.values });
      setDraft({ ...r.values });
      etag.set(r.etag ?? null);
      setState('ready');
    } catch (err) {
      setLoadError(err);
      setState('error');
    }
  }
  onMount(() => void load());

  function setValue(fieldId: string, v: unknown): void {
    setDraft((d) => ({ ...d, [fieldId]: v }));
    setResult(null);
    if (errors.error(fieldId) !== null) errors.set(fieldId, null);
  }

  function discard(): void {
    setDraft({ ...initial() });
    errors.clear();
    etag.dismiss();
    setResult(null);
  }

  async function save(e?: Event): Promise<void> {
    e?.preventDefault();
    if (saving() || changedIds().length === 0) return;
    setResult(null);
    // Validate every field first (UX5): no request while any value is wrong.
    const found: Record<string, string> = {};
    for (const f of visible()) {
      if (sourceBlocksEdit(f.source)) continue;
      const msg = validateSettingField(f, draft()[f.id], draft());
      if (msg !== null) found[f.id] = msg;
    }
    errors.replace(found);
    const order = fields().map((f) => f.id);
    if (Object.keys(found).length > 0) {
      setResult(t('admin-ux-form-fix', { count: Object.keys(found).length }));
      errors.focusFirst(order);
      return;
    }
    const changed: Record<string, unknown> = {};
    for (const fid of changedIds()) changed[fid] = draft()[fid];
    setSaving(true);
    try {
      const out = await etag.write(async (ifMatch) => (await props.save(changed, { ifMatch, values: draft() })) ?? {});
      applySaved(out.values, changed);
    } catch (err) {
      if (etag.conflict() === null) {
        const at = errors.fromError(err);
        if (at !== null) {
          setResult(t('admin-ux-form-fix', { count: 1 }));
          errors.focusFirst(order);
        }
      }
    } finally {
      setSaving(false);
    }
  }

  /** After a successful write: the stored values become the new baseline. */
  function applySaved(values: Record<string, unknown> | undefined, changed: Record<string, unknown>): void {
    const stored = values ?? { ...draft() };
    // Secrets are write-only: forget what was typed; the server now has one.
    for (const f of fields()) {
      if (f.kind !== 'secret') continue;
      if (typeof changed[f.id] === 'string') stored[f.id] = { set: true };
      else delete stored[f.id];
    }
    const next = { ...initial(), ...stored };
    setInitial(next);
    setDraft({ ...next });
    setResult(t('admin-ux-form-saved', { count: Object.keys(changed).length }));
  }

  async function keepMine(): Promise<void> {
    const changed: Record<string, unknown> = {};
    for (const fid of changedIds()) changed[fid] = draft()[fid];
    setSaving(true);
    try {
      const out = await etag.keepMine<{ etag?: string | null; values?: Record<string, unknown> }>();
      applySaved(out.values, changed);
    } catch (err) {
      if (etag.conflict() === null) errors.fromError(err);
    } finally {
      setSaving(false);
    }
  }

  const body = (): JSX.Element => (
    <Show
      when={state() === 'ready'}
      fallback={
        <PageState
          state={
            state() === 'loading'
              ? { kind: 'loading' }
              : state() === 'unavailable'
                ? { kind: 'unavailable', reason: props.unavailableReason ?? t('admin-ux-unavailable-default') }
                : { kind: 'error', error: loadError(), onRetry: () => void load() }
          }
        />
      }
    >
      <form class={ux.form} noValidate onSubmit={(e) => void save(e)} aria-labelledby={props.frame === 'none' ? props.labelledBy : `${id}-title`}>
        <For each={props.groups}>
          {(group) => (
            <Show when={!group.when || group.when(draft())}>
              <fieldset class={ux.fieldset} data-group={group.id}>
                <legend class={ux.legend}>{group.heading}</legend>
                <Show when={group.help}>{(h) => <p class={ux.fieldsetHelp}>{h()}</p>}</Show>
                <For each={group.fields}>
                  {(f) => (
                    <Show when={!f.when || f.when(draft())}>
                      <SettingFieldView
                        field={f}
                        value={draft()[f.id]}
                        stored={initial()[f.id]}
                        values={draft()}
                        error={errors.error(f.id)}
                        onInput={(v) => setValue(f.id, v)}
                      />
                    </Show>
                  )}
                </For>
              </fieldset>
            </Show>
          )}
        </For>
        <Feedback kind="error" message={errors.formError()} />
        <StaleNotice conflict={etag.conflict()} onReload={() => load()} onKeepMine={() => keepMine()} />
        <div class={ux.saveBar} data-testid="save-bar">
          <p class={ux.saveCount} aria-live="polite">
            {changedIds().length === 0 ? t('admin-ux-form-no-changes') : t('admin-ux-form-changes', { count: changedIds().length })}
          </p>
          <div class={css.rowActions}>
            <button type="button" class="btn btn--ghost" disabled={saving() || changedIds().length === 0} onClick={discard}>
              {t('admin-ux-form-discard')}
            </button>
            <button type="submit" class="btn btn--primary" disabled={saving() || changedIds().length === 0}>
              {saving() ? t('admin-ux-form-saving') : (props.saveLabel ?? t('admin-shell-save'))}
            </button>
          </div>
        </div>
        <p class={css.srOnly} role="status" data-testid="save-result">
          {result() ?? ''}
        </p>
      </form>
    </Show>
  );

  return (
    <Show
      when={props.frame !== 'none'}
      fallback={
        <div class={ux.form} data-testid={props.testId}>
          {body()}
        </div>
      }
    >
      <AdminPage title={props.title} titleId={`${id}-title`} {...(props.description ? { description: props.description } : {})} actions={props.actions} testId={props.testId}>
        {body()}
      </AdminPage>
    </Show>
  );
}

interface FieldViewProps {
  field: SettingField;
  value: unknown;
  stored: unknown;
  values: Readonly<Record<string, unknown>>;
  error: string | null;
  onInput: (v: unknown) => void;
}

function SettingFieldView(props: FieldViewProps): JSX.Element {
  const id = createUniqueId();
  const f = (): SettingField => props.field;
  const blocked = (): boolean => sourceBlocksEdit(f().source);
  const reason = (): string | null => (f().source ? sourceReason(f().source!) : null);
  const describedBy = (): string | undefined => {
    const ids = [`${id}-help`, reason() ? `${id}-reason` : '', props.error ? `${id}-error` : ''].filter((s) => s !== '');
    return ids.join(' ');
  };
  // A secret's stored value is `{set: true}` (DC3), never text: the field shows only what is typed.
  const text = (): string =>
    f().kind === 'secret' ? (typeof props.value === 'string' ? props.value : '') : props.value === undefined || props.value === null ? '' : String(props.value);
  const common = () => ({
    id,
    'aria-describedby': describedBy(),
    'aria-invalid': props.error ? (true as const) : undefined,
    'aria-required': f().required ? (true as const) : undefined,
    disabled: blocked(),
    class: ux.control,
  });

  const control = (): JSX.Element => {
    const field = f();
    if (field.render) {
      // Getters, read inside the control's own JSX: the control is built once and
      // keeps its focus while the value changes.
      const render = field.render;
      return untrack(() =>
        render({
          id,
          get describedBy() {
            return describedBy();
          },
          get invalid() {
            return Boolean(props.error);
          },
          get disabled() {
            return blocked();
          },
          get value() {
            return props.value;
          },
          setValue: props.onInput,
          get values() {
            return props.values;
          },
        }),
      );
    }
    switch (field.kind) {
      case 'boolean':
        return null;
      case 'number':
        return (
          <input
            {...common()}
            type="number"
            inputMode="decimal"
            min={field.min}
            max={field.max}
            step={field.step}
            value={text()}
            onInput={(e) => {
              const raw = e.currentTarget.value;
              props.onInput(raw === '' ? null : Number.isFinite(Number(raw)) ? Number(raw) : raw);
            }}
          />
        );
      case 'select':
        return (
          <select {...common()} value={text()} onChange={(e) => props.onInput(e.currentTarget.value)}>
            <For each={field.options ?? []}>{(o) => <option value={o.value}>{o.label}</option>}</For>
          </select>
        );
      case 'textarea':
        return <textarea {...common()} rows={4} value={text()} onInput={(e) => props.onInput(e.currentTarget.value)} />;
      case 'secret':
        return (
          <PasswordInput
            {...common()}
            label={field.label}
            autocomplete="new-password"
            placeholder={props.stored === true || (typeof props.stored === 'object' && props.stored !== null && (props.stored as { set?: boolean }).set) ? t('admin-ux-secret-set') : t('admin-ux-secret-unset')}
            value={text()}
            onInput={(e) => props.onInput(e.currentTarget.value)}
          />
        );
      default:
        return (
          <input
            {...common()}
            type={field.kind === 'url' ? 'url' : field.kind === 'email' ? 'email' : 'text'}
            spellcheck={false}
            placeholder={field.placeholder}
            maxLength={field.maxLength}
            value={text()}
            onInput={(e) => props.onInput(e.currentTarget.value)}
          />
        );
    }
  };

  return (
    <div class={ux.fieldBlock} {...fieldAnchorAttrs(f().id)} data-testid={`setting-${f().id}`}>
      <Show
        when={f().kind === 'boolean' && !f().render}
        fallback={
          <>
            <label class={css.fieldLabel} for={id}>
              {f().label}
              <Show when={f().technical}>{(tech) => <span class={css.fieldTech}> — {tech()}</span>}</Show>
            </label>
            {control()}
          </>
        }
      >
        <div class={ux.checkboxRow}>
          <input
            id={id}
            type="checkbox"
            checked={props.value === true}
            disabled={blocked()}
            aria-describedby={describedBy()}
            aria-invalid={props.error ? true : undefined}
            onChange={(e) => props.onInput(e.currentTarget.checked)}
          />
          <label for={id}>
            {f().label}
            <Show when={f().technical}>{(tech) => <span class={css.fieldTech}> — {tech()}</span>}</Show>
          </label>
        </div>
      </Show>
      <HelpNote id={`${id}-help`}>
        <>
          {f().help}
          <Show when={f().docsHref}>
            {(href) => (
              <>
                {' '}
                <a class={ux.learnMore} href={href()} target="_blank" rel="noopener noreferrer">
                  {t('admin-ux-learn-more')}
                  <span class={css.srOnly}> {t('admin-ux-learn-more-field', { label: f().label })}</span>
                </a>
              </>
            )}
          </Show>
        </>
      </HelpNote>
      <Show when={reason()}>
        {(r) => (
          <p id={`${id}-reason`} class={css.help}>
            {r()}
          </p>
        )}
      </Show>
      <Show when={props.error}>
        {(error) => (
          <p id={`${id}-error`} class={css.fieldError}>
            <Icon name="alert" />
            <span>{error()}</span>
          </p>
        )}
      </Show>
      <div class={ux.fieldMeta}>
        <Show when={f().source}>{(s) => <SourceBadge source={s()} />}</Show>
        <Show when={f().propagation}>
          {(p) => <span>{p() === 'hot' ? t('admin-ux-propagation-hot') : t('admin-ux-propagation-restart')}</span>}
        </Show>
        <Show when={f().history}>
          {(h) => <FieldHistory target={h().target} updatedBy={h().updatedBy} updatedAt={h().updatedAt} />}
        </Show>
        <Show when={f().revert}>
          {(r) => (
            <RevertToDefault
              label={f().label}
              defaultDisplay={r().defaultDisplay}
              isDefault={'value' in r() ? same(props.value, r().value) : (r().isDefault ?? false)}
              onRevert={() => ('value' in r() ? props.onInput(r().value) : r().onRevert?.())}
              disabledReason={reason()}
            />
          )}
        </Show>
      </div>
    </div>
  );
}
