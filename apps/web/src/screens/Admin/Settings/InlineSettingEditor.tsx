// One registry setting, edited in place (inline editor of the Privacy and
// Attachments screens). The same pieces SettingDetail uses, in a compact card:
// the value (a generic ValueInput, or a caller's own editor for lists), "Users
// may change it", the lock, per-domain values behind a disclosure, the effective
// value with its source badge, Save (If-Match; 412 -> Reload / Keep mine), Reset
// (discard edits), Revert to default, a dry-run impact preview before a lock or
// an override change that narrows what users may do, errors under the field,
// and a read-only render without settings.write or when an env variable pins
// the key. Writes go through api/admin/settings.ts, which the server audits.

import { Show, createEffect, createMemo, createSignal, on, onCleanup, useContext, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { deleteGlobal, getAdminSetting, previewGlobal, putGlobal, type AdminSetting, type GlobalWrite } from '../../../api/admin/settings.ts';
import { AdminProblemError } from '../shell/adminApi.ts';
import { ShellContext } from '../shell/context.ts';
import { fieldAnchorAttrs } from '../shell/fieldAnchors.ts';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { FormField, type FieldWiring } from '../shell/FormField.tsx';
import { ImpactPreview } from '../shell/ImpactPreview.tsx';
import { LockSwitch } from '../shell/LockSwitch.tsx';
import { RevertToDefault } from '../shell/RevertToDefault.tsx';
import { SourceBadge, sourceReason } from '../shell/SourceBadge.tsx';
import { StaleNotice } from '../shell/StaleNotice.tsx';
import { useDirtyGuard } from '../shell/useDirtyGuard.ts';
import { useEtag } from '../shell/useEtag.ts';
import { useSavingToast } from '../shell/useSavingToast.ts';
import { ScopedValues } from './ScopedValues.tsx';
import { ValueInput } from './ValueInput.tsx';
import { impactLabels } from './impact.ts';
import { settingAnchorId } from './keys.ts';
import { settingHelp, settingLabel } from './labels.ts';
import { sourceOf } from './SettingDetail.tsx';
import { displayValue, parseDraft, toDraft, type Draft } from './value.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from './settings.css.ts';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('admin-settings');

/** What a caller's own value editor receives. */
export interface InlineEditorContext {
  setting: AdminSetting;
  draft: Draft;
  onDraft: (d: Draft) => void;
  wiring: FieldWiring;
  disabled: boolean;
}

export interface InlineSettingEditorProps {
  settingKey: string;
  /** The row from the settings list; saves a request. Loaded by key when absent. */
  setting?: AdminSetting | undefined;
  /** Replaces the generic value input (chip lists, byte sizes). The draft stays the registry's JSON text. */
  editor?: ((ctx: InlineEditorContext) => JSX.Element) | undefined;
  /** Text shown for the effective value in the header; default `displayValue`. */
  format?: ((s: AdminSetting) => string) | undefined;
  /** Overrides the registry label. */
  label?: string | undefined;
  /** Called with the saved row after any write. */
  onChanged?: ((s: AdminSetting) => void) | undefined;
  /** Called when the card gains or loses unsaved edits. */
  onDirty?: ((key: string, dirty: boolean) => void) | undefined;
  /** Scroll to and focus the card on mount (deep links). */
  focus?: boolean | undefined;
  /** Injected for tests. */
  load?: ((key: string) => Promise<{ setting: AdminSetting; etag: string | null }>) | undefined;
  /** Hide the per-domain entry point (a screen that shows it elsewhere). */
  noScoped?: boolean | undefined;
}

export function InlineSettingEditor(props: InlineSettingEditorProps): JSX.Element {
  const shell = useContext(ShellContext);
  const can = (cap: Parameters<NonNullable<typeof shell>['can']['can']>[0]): boolean => shell?.can.can(cap) ?? true;
  const load = props.load ?? getAdminSetting;
  const [row, setRow] = createSignal<AdminSetting | undefined>(props.setting);
  const [loadError, setLoadError] = createSignal<string | null>(null);
  const tag = useEtag({ refresh: async () => (await load(props.settingKey)).etag });

  const [draft, setDraft] = createSignal<Draft>('');
  const [override, setOverride] = createSignal(true);
  const [locked, setLocked] = createSignal(false);
  const [valueError, setValueError] = createSignal<string | null>(null);
  const [formError, setFormError] = createSignal<string | null>(null);
  const [done, setDone] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [preview, setPreview] = createSignal<GlobalWrite | null>(null);
  const [open, setOpen] = createSignal(false);
  useSavingToast(busy, () => t('admin-ux-form-saving'));

  function reset(s: AdminSetting, etag: string | null): void {
    setDraft(toDraft(s.kind, s.globalValue ?? s.effective.value));
    setOverride(s.allowUserOverride);
    setLocked(s.globalLocked);
    setValueError(null);
    setFormError(null);
    tag.set(etag);
  }
  async function reload(): Promise<void> {
    try {
      const out = await load(props.settingKey);
      setRow(out.setting);
      reset(out.setting, out.etag);
      setLoadError(null);
    } catch (err) {
      setLoadError(describeAdminError(err));
    }
  }
  // A row handed in by the list seeds the form; a new row object (the list
  // refreshed after another card saved) reseeds only while this card is clean.
  if (props.setting) reset(props.setting, props.setting.etag ?? null);
  else void reload();
  createEffect(on(() => props.setting, (s) => {
    if (s && !dirty()) {
      setRow(s);
      reset(s, s.etag ?? null);
    }
  }, { defer: true }));

  const userScoped = (): boolean => row()?.scopes === 'global+domain+user';
  const lockable = (): boolean => row()?.scopes !== 'global';
  const pinned = (): boolean => {
    const e = row()?.envState;
    return e?.set === true && e.precedence !== 'fallback';
  };
  const dedicated = (): string | null => row()?.dedicatedRoute ?? null;
  const writable = (): boolean => can('settings.write') && !pinned() && dedicated() === null;
  const blockedReason = (): string | null => {
    const s = row();
    if (!s) return null;
    if (pinned()) return sourceReason(sourceOf(s));
    if (!can('settings.write')) return shell?.can.reason('settings.write') ?? null;
    return null;
  };
  const storedDraft = createMemo(() => {
    const s = row();
    return s ? toDraft(s.kind, s.globalValue ?? s.effective.value) : '';
  });
  const dirty = (): boolean => {
    const s = row();
    if (!s) return false;
    return draft() !== storedDraft() || override() !== s.allowUserOverride || locked() !== s.globalLocked;
  };
  useDirtyGuard(dirty);
  createEffect(on(dirty, (d) => props.onDirty?.(props.settingKey, d), { defer: true }));
  onCleanup(() => props.onDirty?.(props.settingKey, false));

  function changes(s: AdminSetting): GlobalWrite | null {
    const body: GlobalWrite = {};
    if (draft() !== storedDraft()) {
      const parsed = parseDraft(s.kind, draft(), s.nullable);
      if (!parsed.ok) {
        setValueError(parsed.error);
        return null;
      }
      body.globalValue = parsed.value;
    }
    if (override() !== s.allowUserOverride) body.allowUserOverride = override();
    if (locked() !== s.globalLocked) body.locked = locked();
    return body;
  }
  function showError(err: unknown): void {
    if (err instanceof AdminProblemError && err.field === '/globalValue') setValueError(describeAdminError(err));
    else if (!(err instanceof AdminProblemError && err.stale)) setFormError(describeAdminError(err));
  }
  async function write(body: GlobalWrite): Promise<void> {
    setBusy(true);
    setFormError(null);
    try {
      const out = await tag.write((ifMatch) => putGlobal(props.settingKey, body, ifMatch));
      setRow(out.setting);
      reset(out.setting, out.etag);
      setDone(t(out.setting.hot ? 'admin-settings-saved' : 'admin-settings-saved-restart'));
      props.onChanged?.(out.setting);
    } catch (err) {
      showError(err);
    } finally {
      setBusy(false);
    }
  }
  async function save(e: Event): Promise<void> {
    e.preventDefault();
    const s = row();
    if (!s) return;
    setDone(null);
    setValueError(null);
    const body = changes(s);
    if (body === null || Object.keys(body).length === 0) return;
    const narrower = Object.values(s.overrides).reduce((a, b) => a + (b ?? 0), 0);
    const tightening = (body.locked === true && !s.globalLocked) || (body.allowUserOverride === false && s.allowUserOverride);
    if (tightening && narrower > 0) {
      setPreview(body);
      return;
    }
    await write(body);
  }
  function discard(): void {
    const s = row();
    if (s) reset(s, tag.etag());
    setDone(null);
  }

  let card: HTMLElement | undefined;
  createEffect(on(() => row() !== undefined && props.focus, (ready) => {
    if (!ready || !card) return;
    card.scrollIntoView?.({ block: 'center' });
    card.querySelector<HTMLElement>('input, select, textarea, button')?.focus?.();
  }));

  const label = (): string => props.label ?? settingLabel(props.settingKey);
  const help = (): string => settingHelp(props.settingKey);
  const helpId = (): string => `inline-${props.settingKey}-help`;

  return (
    <article ref={(el) => (card = el)} class={css.inlineCard} data-testid="inline-setting" data-setting-key={props.settingKey} aria-labelledby={`inline-${props.settingKey}-title`} {...fieldAnchorAttrs(settingAnchorId(props.settingKey))}>
      <Show when={row()} fallback={<p role={loadError() ? 'alert' : 'status'}>{loadError() ?? t('admin-shell-loading')}</p>}>
        {(s) => (
          <form noValidate onSubmit={(e) => void save(e)} aria-describedby={help() ? helpId() : undefined}>
            <header class={css.inlineHead}>
              <h4 id={`inline-${props.settingKey}-title`} class={css.inlineTitle}>{label()}</h4>
              <code class={css.keyName}>{s().key}</code>
              <SourceBadge source={sourceOf(s())} id={`inline-${props.settingKey}-source`} />
              <span class={css.inlineEffective} data-testid="inline-effective">
                {t('admin-settings-inline-effective')} <strong>{props.format ? props.format(s()) : displayValue(s().kind, s().effective.value)}</strong>
              </span>
            </header>
            <Show when={help()}><p class={ux.fieldsetHelp} id={helpId()}>{help()}</p></Show>
            <Show when={blockedReason()}>{(reason) => <p class={ux.fieldsetHelp} data-testid="inline-readonly">{reason()}</p>}</Show>
            <Show when={dedicated()}><p class={ux.fieldsetHelp}>{t('admin-settings-dedicated', { section: isolate(dedicated() ?? '') })}</p></Show>
            <FormField label={label()} technical={s().key} error={valueError() ?? blockedReason()}>
              {(w) => (
                <Show
                  when={props.editor}
                  fallback={<ValueInput kind={s().kind} draft={draft()} onDraft={setDraft} wiring={w} disabled={!writable()} nullable={s().nullable} {...(s().kind.type === 'bool' ? { checkboxLabel: label() } : {})} />}
                >
                  {(render) => render()({
                    get setting() { return s(); },
                    get draft() { return draft(); },
                    onDraft: setDraft,
                    get wiring() { return w; },
                    get disabled() { return !writable(); },
                  })}
                </Show>
              )}
            </FormField>
            <div class={css.switches}>
              <Show when={userScoped()}>
                <div class={css.switchOption}>
                  <label class={ux.checkboxRow}>
                    <input type="checkbox" checked={override()} disabled={!writable() || locked()} onChange={(e) => setOverride(e.currentTarget.checked)} />
                    <span>{t('admin-settings-user-override')}</span>
                  </label>
                </div>
              </Show>
              <Show when={lockable()}>
                <div class={css.switchOption}>
                  <LockSwitch locked={locked()} onChange={setLocked} settingLabel={label()} disabled={!writable()} />
                </div>
              </Show>
            </div>
            <StaleNotice
              conflict={tag.conflict()}
              onReload={async () => { tag.dismiss(); await reload(); }}
              onKeepMine={async () => {
                try {
                  const out = await tag.keepMine<{ setting: AdminSetting; etag: string | null }>();
                  setRow(out.setting);
                  reset(out.setting, out.etag);
                  props.onChanged?.(out.setting);
                } catch (err) { showError(err); }
              }}
            />
            <div class={ux.saveBar}>
              <button type="submit" class="btn btn--primary" aria-busy={busy()} disabled={busy() || !writable() || !dirty()}>{t('admin-settings-save')}</button>
              <button type="button" class="btn btn--ghost" disabled={busy() || !dirty()} onClick={discard}>{t('admin-settings-inline-reset')}</button>
              <RevertToDefault
                compact
                label={label()}
                defaultDisplay={displayValue(s().kind, s().default)}
                isDefault={(s().globalValue === null || s().globalValue === undefined) && !s().globalLocked}
                disabledReason={writable() ? null : blockedReason()}
                onRevert={async () => {
                  const out = await tag.write((ifMatch) => deleteGlobal(props.settingKey, ifMatch));
                  setRow(out.setting);
                  reset(out.setting, out.etag);
                  props.onChanged?.(out.setting);
                }}
              />
              <Show when={s().scopes !== 'global' && dedicated() === null && !props.noScoped}>
                <button type="button" class="btn btn--ghost" aria-expanded={open()} onClick={() => setOpen(!open())}>{t('admin-settings-inline-domains')}</button>
              </Show>
            </div>
            <div aria-live="polite"><Feedback kind="success" message={done()} /></div>
            <Feedback kind="error" as="toast" message={formError()} />
            <Show when={open()}>
              <ScopedValues setting={s()} label={label()} scope="domain" canWrite={can('settings.write') && !pinned()} disabledReason={blockedReason()} onChanged={() => void reload()} />
            </Show>
            <ImpactPreview
              open={preview() !== null}
              title={t('admin-settings-lock-title', { setting: isolate(label()) })}
              body={t('admin-settings-lock-body')}
              loadImpact={() => { const p = preview(); return p ? previewGlobal(props.settingKey, p) : Promise.resolve(null); }}
              labels={impactLabels()}
              confirmLabel={t('admin-settings-lock-confirm')}
              danger={false}
              onConfirm={async () => { const p = preview(); if (p) await write(p); setPreview(null); }}
              onCancel={() => setPreview(null)}
            />
          </form>
        )}
      </Show>
    </article>
  );
}
