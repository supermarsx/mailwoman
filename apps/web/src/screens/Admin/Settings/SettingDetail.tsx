// One setting of the settings model (worker f01; t32-admin §2.4, UX4-UX9, UX18,
// UX19). Shows what decides the value now (source badge), the value for the whole
// server with its own editor, "Users may change it" and "Lock for users", Save
// (If-Match; 412 → Reload / Keep mine), Revert to default, the history link, and
// the domain and per-user values. A lock over narrower values shows what it
// overrules first (dry run). A pinned key is shown with the variable that pins
// it; a `signin.*` key is read-only here with a link to the section that owns it.

import { Show, createEffect, createMemo, createResource, createSignal, on, useContext, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { deleteGlobal, getAdminSetting, previewGlobal, putGlobal, type AdminSetting, type GlobalWrite } from '../../../api/admin/settings.ts';
import { AdminProblemError } from '../shell/adminApi.ts';
import { ShellContext } from '../shell/context.ts';
import { adminDocsHref } from '../shell/docsLink.ts';
import { fieldAnchorAttrs } from '../shell/fieldAnchors.ts';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { FieldHistory } from '../shell/FieldHistory.tsx';
import { FormField, type FieldWiring } from '../shell/FormField.tsx';
import { HelpNote } from '../shell/HelpNote.tsx';
import { ImpactPreview } from '../shell/ImpactPreview.tsx';
import { LockSwitch } from '../shell/LockSwitch.tsx';
import { RevertToDefault } from '../shell/RevertToDefault.tsx';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { SourceBadge, sourceReason, type SourceInfo } from '../shell/SourceBadge.tsx';
import { StaleNotice } from '../shell/StaleNotice.tsx';
import { useDirtyGuard } from '../shell/useDirtyGuard.ts';
import { useEtag } from '../shell/useEtag.ts';
import { PageState } from '../shell/templates/PageFrame.tsx';
import { resourceValue } from '../../../components/resourceValue.ts';
import { ScopedValues } from './ScopedValues.tsx';
import { ValueInput } from './ValueInput.tsx';
import { impactLabels } from './impact.ts';
import { settingHelp, settingLabel } from './labels.ts';
import { settingAnchorId } from './keys.ts';
import { displayValue, parseDraft, toDraft, type Draft } from './value.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from './settings.css.ts';

const AREA = 'settings';

/** The console section that owns a key's writes, from its dedicated route. */
function dedicatedSection(route: string): string {
  return route.replace(/^\/+/, '').split('/')[0] ?? route;
}

/** What decides a key's value, as a badge. */
export function sourceOf(s: AdminSetting): SourceInfo {
  if (s.envState?.set && s.envState.precedence === 'pin') return { kind: 'env-pin', envVar: s.envState.var };
  if (s.envState?.set && s.envState.precedence === 'boot') {
    return { kind: 'env-pin', envVar: s.envState.var, restartNeeded: true };
  }
  if (s.globalLocked) return { kind: 'locked', lockedBy: 'global' };
  if (s.globalValue !== null && s.globalValue !== undefined) return { kind: 'stored' };
  if (s.envState?.set) return { kind: 'env-fallback', envVar: s.envState.var };
  return { kind: 'default' };
}

function isoOf(unix: number | null): string | null {
  return unix === null ? null : new Date(unix * 1000).toISOString();
}

export interface SettingDetailProps {
  settingKey: string;
  /** Called after any write, so the list refreshes. */
  onChanged?: () => void;
  /** Injected for tests; default `GET /admin/settings/<key>`. */
  load?: (key: string) => Promise<{ setting: AdminSetting; etag: string | null }>;
}

export function SettingDetail(props: SettingDetailProps): JSX.Element {
  const shell = useContext(ShellContext);
  const can = (cap: Parameters<NonNullable<typeof shell>['can']['can']>[0]): boolean => shell?.can.can(cap) ?? true;
  const load = props.load ?? getAdminSetting;
  const [data, { refetch, mutate }] = createResource(() => props.settingKey, load);
  const tag = useEtag({ refresh: async () => (await load(props.settingKey)).etag });

  const [draft, setDraft] = createSignal<Draft>('');
  const [override, setOverride] = createSignal(true);
  const [locked, setLocked] = createSignal(false);
  const [valueError, setValueError] = createSignal<string | null>(null);
  const [formError, setFormError] = createSignal<string | null>(null);
  const [done, setDone] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [lossConfirmation, setLossConfirmation] = createSignal<GlobalWrite | null>(null);
  const [preview, setPreview] = createSignal<GlobalWrite | null>(null);

  // Never `data()` (t75): read while a reload is in flight (Reload after a 412, a
  // domain or per-user value written) it suspends the console's page boundary
  // and the form is swapped for the page skeleton and back. The last value
  // stays on screen while it refreshes.
  const setting = (): AdminSetting | undefined => resourceValue(data)?.setting;
  const label = (): string => settingLabel(props.settingKey);

  function reset(s: AdminSetting, etag: string | null): void {
    setDraft(toDraft(s.kind, s.globalValue ?? s.effective.value));
    setOverride(s.allowUserOverride);
    setLocked(s.globalLocked);
    setValueError(null);
    setFormError(null);
    tag.set(etag);
  }
  createEffect(
    on(
      () => resourceValue(data),
      (d) => {
        if (d) reset(d.setting, d.etag);
      },
    ),
  );

  const userScoped = (): boolean => setting()?.scopes === 'global+domain+user';
  const lockable = (): boolean => setting()?.scopes !== 'global';
  const pinned = (): boolean => {
    const e = setting()?.envState;
    return e?.set === true && e.precedence !== 'fallback';
  };
  const dedicated = (): string | null => setting()?.dedicatedRoute ?? null;
  const writable = (): boolean => can('settings.write') && !pinned() && dedicated() === null;
  const blockedReason = (): string | null => {
    const s = setting();
    if (!s) return null;
    if (pinned()) return sourceReason(sourceOf(s));
    if (!can('settings.write')) return shell?.can.reason('settings.write') ?? null;
    return null;
  };

  const storedDraft = createMemo(() => {
    const s = setting();
    return s ? toDraft(s.kind, s.globalValue ?? s.effective.value) : '';
  });
  const dirty = (): boolean => {
    const s = setting();
    if (!s) return false;
    return draft() !== storedDraft() || override() !== s.allowUserOverride || locked() !== s.globalLocked;
  };
  useDirtyGuard(dirty);

  /** The members that changed, or null when the value does not pass the checks. */
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

  async function write(body: GlobalWrite): Promise<void> {
    setBusy(true);
    setFormError(null);
    try {
      const out = await tag.write((ifMatch) => putGlobal(props.settingKey, body, ifMatch));
      mutate(out);
      reset(out.setting, out.etag);
      setDone(t(out.setting.hot ? 'admin-settings-saved' : 'admin-settings-saved-restart'));
      props.onChanged?.();
    } catch (err) {
      showError(err);
      throw err;
    } finally {
      setBusy(false);
    }
  }

  /** A refusal at the field its pointer names, otherwise under the form (a 412 has its own notice). */
  function showError(err: unknown): void {
    if (err instanceof AdminProblemError && err.field === '/globalValue') setValueError(describeAdminError(err));
    else if (!(err instanceof AdminProblemError && err.stale)) setFormError(describeAdminError(err));
  }

  async function save(e: Event): Promise<void> {
    e.preventDefault();
    const s = setting();
    if (!s) return;
    setDone(null);
    setValueError(null);
    const body = changes(s);
    if (body === null || Object.keys(body).length === 0) return;
    if (props.settingKey === 'admin.allowLastSuperadminLoss' && body.globalValue === true) {
      setLossConfirmation(body);
      return;
    }
    const narrower = Object.values(s.overrides).reduce((a, b) => a + (b ?? 0), 0);
    const tightening = (body.locked === true && !s.globalLocked) || (body.allowUserOverride === false && s.allowUserOverride);
    if (tightening && narrower > 0) {
      setPreview(body);
      return;
    }
    await write(body).catch(() => undefined);
  }

  const helpId = (): string => `setting-${props.settingKey}-help`;

  return (
    <Show
      when={setting()}
      fallback={
        <PageState
          state={
            data.error
              ? { kind: 'error', error: data.error, onRetry: () => void refetch() }
              : { kind: 'loading' }
          }
        />
      }
    >
      {(s) => (
        <div class={ux.form} data-testid="setting-detail">
          <p class={css.keyName}>{t('admin-settings-detail-key', { key: s().key })}</p>
          <p class={ux.fieldsetHelp} id={helpId()}>
            {settingHelp(props.settingKey)}{' '}
            <a class={ux.learnMore} href={adminDocsHref(AREA, s().key)} target="_blank" rel="noopener noreferrer">
              {t('admin-ux-learn-more')}
            </a>
          </p>
          <div class={ux.fieldMeta}>
            <SourceBadge source={sourceOf(s())} id={`setting-${props.settingKey}-source`} />
            <FieldHistory target={`setting:${s().key}`} updatedBy={s().updatedBy} updatedAt={isoOf(s().updatedAt)} />
          </div>

          <Show when={dedicated()}>
            {(route) => (
              <HelpNote>
                {t('admin-settings-dedicated', { section: isolate(dedicatedSection(route())) })}{' '}
                <a class={ux.learnMore} href={hrefFor(sectionPath(dedicatedSection(route())))}>
                  {t('admin-settings-dedicated-link', { section: isolate(dedicatedSection(route())) })}
                </a>
              </HelpNote>
            )}
          </Show>

          <form class={ux.fieldset} noValidate onSubmit={(e) => void save(e)} aria-labelledby="setting-global-heading">
            <h3 class={ux.legend} id="setting-global-heading">
              {t('admin-settings-global')}
            </h3>
            <p class={ux.fieldsetHelp}>{t('admin-settings-global-help')}</p>
            <div {...fieldAnchorAttrs(settingAnchorId(s().key))}>
              <FormField
                label={label()}
                technical={s().key}
                help={s().kind.type === 'folderRole' || s().kind.type === 'localeMap' || s().kind.type === 'intList' || s().kind.type === 'quietHours' || s().kind.type === 'links' ? t('admin-settings-json-hint') : ''}
                error={valueError() ?? blockedReason()}
              >
                {(w) => (
                  <ValueInputOf setting={s()} draft={draft()} onDraft={setDraft} wiring={w} disabled={!writable()} />
                )}
              </FormField>
            </div>
            <div class={css.switches}>
              <Show when={userScoped()}>
                <div class={css.switchOption}>
                <label class={ux.checkboxRow}>
                  <input
                    type="checkbox"
                    checked={override()}
                    disabled={!writable() || locked()}
                    aria-describedby="setting-override-help"
                    onChange={(e) => setOverride(e.currentTarget.checked)}
                  />
                  <span>{t('admin-settings-user-override')}</span>
                </label>
                <p class={ux.fieldsetHelp} id="setting-override-help">{t('admin-settings-user-override-help')}</p>
                </div>
              </Show>
              <Show when={lockable()}>
                <div class={css.switchOption}>
                <LockSwitch
                  locked={locked()}
                  onChange={setLocked}
                  settingLabel={label()}
                  disabled={!writable()}
                  describedBy="setting-lock-help"
                />
                <p class={ux.fieldsetHelp} id="setting-lock-help">{t('admin-settings-lock-help')}</p>
                </div>
              </Show>
            </div>
            <p class={ux.fieldsetHelp}>{t(s().hot ? 'admin-ux-propagation-hot' : 'admin-ux-propagation-restart')}</p>
            <StaleNotice
              conflict={tag.conflict()}
              onReload={async () => {
                tag.dismiss();
                await refetch();
              }}
              onKeepMine={async () => {
                try {
                  const out = await tag.keepMine<{ setting: AdminSetting; etag: string | null }>();
                  mutate(out);
                  reset(out.setting, out.etag);
                  props.onChanged?.();
                } catch (err) {
                  showError(err);
                }
              }}
            />
            <div class={ux.saveBar}>
              <button type="submit" class="btn btn--primary" disabled={busy() || !writable() || !dirty()}>
                {busy() ? t('admin-ux-form-saving') : t('admin-settings-save')}
              </button>
              <RevertToDefault
                label={label()}
                defaultDisplay={displayValue(s().kind, s().default)}
                isDefault={(s().globalValue === null || s().globalValue === undefined) && !s().globalLocked}
                disabledReason={writable() ? null : blockedReason()}
                onRevert={async () => {
                  const out = await tag.write((ifMatch) => deleteGlobal(props.settingKey, ifMatch));
                  mutate(out);
                  reset(out.setting, out.etag);
                  props.onChanged?.();
                }}
              />
            </div>
            <div aria-live="polite">
              <Feedback kind="success" message={done()} />
            </div>
            <Feedback kind="error" message={formError()} />
          </form>

          <Show when={s().scopes !== 'global' && dedicated() === null}>
            <ScopedValues
              setting={s()}
              label={label()}
              scope="domain"
              canWrite={can('settings.write') && !pinned()}
              disabledReason={blockedReason()}
              onChanged={() => {
                void refetch();
                props.onChanged?.();
              }}
            />
          </Show>
          <Show when={(s().scopes === 'global+domain+admin_user' || s().scopes === 'global+domain+user') && dedicated() === null}>
            <ScopedValues
              setting={s()}
              label={label()}
              scope="admin_user"
              canWrite={can('settings.user_values') && !pinned()}
              disabledReason={pinned() ? blockedReason() : (shell?.can.reason('settings.user_values') ?? null)}
              onChanged={() => {
                void refetch();
                props.onChanged?.();
              }}
            />
          </Show>

          <ConfirmDialog open={lossConfirmation() !== null}
            title={t('admin-settings-last-superadmin-title')}
            body={t('admin-settings-last-superadmin-body')}
            typedConfirmation="ALLOW LAST SUPERADMIN LOSS"
            confirmLabel={t('admin-settings-last-superadmin-confirm')}
            onCancel={() => setLossConfirmation(null)}
            onConfirm={async () => { const body = lossConfirmation(); if (body) await write({ ...body, confirmLastSuperadminLoss: true }); setLossConfirmation(null); }} />
          <ImpactPreview
            open={preview() !== null}
            title={t('admin-settings-lock-title', { setting: isolate(label()) })}
            body={t('admin-settings-lock-body')}
            loadImpact={() => {
              const p = preview();
              return p ? previewGlobal(props.settingKey, p) : Promise.resolve(null);
            }}
            labels={impactLabels()}
            confirmLabel={t('admin-settings-lock-confirm')}
            danger={false}
            onConfirm={async () => {
              const p = preview();
              if (p) await write(p);
              setPreview(null);
            }}
            onCancel={() => setPreview(null)}
          />
        </div>
      )}
    </Show>
  );
}


function ValueInputOf(props: {
  setting: AdminSetting;
  draft: Draft;
  onDraft: (d: Draft) => void;
  wiring: FieldWiring;
  disabled: boolean;
}): JSX.Element {
  return (
    <ValueInput
      kind={props.setting.kind}
      draft={props.draft}
      onDraft={props.onDraft}
      wiring={props.wiring}
      disabled={props.disabled}
      nullable={props.setting.nullable}
    />
  );
}
