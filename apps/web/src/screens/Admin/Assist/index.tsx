// Admin → AI assistant (SPEC §14/§19; rebuilt on SettingsFormPage by f33 for
// t32-cfg §4.12): the deployment's endpoint, the capabilities it may use, the data
// limits, the rate limit and the on/off control.
//
// What takes effect when is the server's behaviour, and the screen reports it from
// the server's answers rather than assuming it: turning the assistant off stops
// the running gateway at once; everything else is read when the server starts
// (`crates/mw-server/src/v7_mount.rs`, `assist_status` / `store_assist_config`),
// so the endpoint fields say "restart needed".
//
// The form is offered only once the stored configuration has been read: a form
// that still held defaults would overwrite the deployment's configuration when
// saved. When the stored API key does not open with this server's key the form
// starts empty, says why, and may be saved as it is to replace the configuration.
//
// The API key is write-only (`ApiKeyField`): with a key stored the field says so
// and offers to replace or remove it. Once the type or base URL changes the server
// does not carry the key over, so the field asks for a key for the new endpoint
// and saving without one sends "no key".
//
// A local program is offered only from `MW_ASSIST_LOCAL_PROGRAMS` (the status
// lists it); the server refuses any other (`local_program_refusal`). Account,
// folder and argument lists are list editors instead of one-per-line textareas.
// The rate limit is the settings-model key `assistant.rateLimitPerMin`
// (`GET/PUT /admin/settings/<key>`, f01), shown only when the server offers it.

import { Match, Show, Switch, createMemo, createSignal, onMount, type JSX } from 'solid-js';
import { ASSIST_CAPABILITIES, type AssistCapability } from '../../../modules/assist/types.ts';
import { t, loadCatalog } from '../../../i18n';
import { PasswordInput } from '../../../components/PasswordInput.tsx';
import { getAdminSetting, putGlobal, type AdminSetting } from '../../../api/admin/settings.ts';
import { AdminProblemError } from '../shell/adminApi.ts';
import { SettingsFormPage, type FieldRenderContext, type SettingsGroup, type SettingsLoad } from '../shell/templates/SettingsFormPage.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback } from '../shell/Feedback.tsx';
import { ListEditor } from '../shell/ListEditor.tsx';
import type { SourceInfo } from '../shell/SourceBadge.tsx';
import { adminDocsHref } from '../shell/docsLink.ts';
import * as shell from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from '../shell/extras.css.ts';
import { AssistProfiles, type AssignmentApi } from './Profiles.tsx';
import type { ProfilesApi } from './profilesService.ts';
import { AdminAssistApi, AdminAssistError, DEFAULT_ADMIN_ASSIST_CONFIG, type AdminAssistConfig, type AdminAssistStatus, type AssistAdapter } from './service.ts';

const AREA = 'ai-assistant';
const RATE_KEY = 'assistant.rateLimitPerMin';
type Kind = AssistAdapter['kind'] | 'none';

/** The settings-model client for the rate limit (tests inject one). */
export interface RateLimitApi {
  load(): Promise<{ kind: 'ok'; setting: AdminSetting; etag: string | null } | { kind: 'unavailable' }>;
  save(value: number, ifMatch: string | null): Promise<{ setting: AdminSetting; etag: string | null }>;
}

export const httpRateLimitApi: RateLimitApi = {
  async load() {
    try {
      const r = await getAdminSetting(RATE_KEY);
      return { kind: 'ok', setting: r.setting, etag: r.etag };
    } catch (err) {
      // No settings model on this server yet: the field is not offered.
      if (err instanceof AdminProblemError && [404, 405, 501].includes(err.status) && err.code === undefined) return { kind: 'unavailable' };
      throw err;
    }
  },
  save: (value, ifMatch) => putGlobal(RATE_KEY, { globalValue: value }, ifMatch),
};

/** The API key field's value: whether a key was stored at load, what is typed, and "remove it". */
export interface ApiKeyValue {
  stored: boolean;
  typed: string;
  remove: boolean;
}

/** The endpoint an adapter sends to: kind and base URL. The server keeps a stored key only while this is unchanged. */
function endpointOf(kind: unknown, baseUrl: unknown): string {
  if (kind !== 'open-ai-compatible' && kind !== 'anthropic') return '';
  return `${String(kind)} ${String(baseUrl ?? '').trim()}`;
}

const text = (v: unknown): string => (typeof v === 'string' ? v : '');

/** The form values of a stored configuration. */
export function assistValues(c: AdminAssistConfig): Record<string, unknown> {
  const a = c.adapter as Record<string, unknown> | null;
  const kind: Kind = c.adapter?.kind ?? 'none';
  return {
    kind,
    baseUrl: text(a?.baseUrl),
    apiKey: { stored: a?.hasApiKey === true, typed: '', remove: false } satisfies ApiKeyValue,
    chatModel: text(a?.chatModel),
    embedModel: text(a?.embedModel),
    sttModel: text(a?.sttModel),
    model: text(a?.model),
    anthropicVersion: text(a?.anthropicVersion),
    maxTokens: typeof a?.maxTokens === 'number' ? a.maxTokens : null,
    program: text(a?.program),
    args: Array.isArray(a?.args) ? [...(a.args as string[])] : [],
    capabilityGrants: [...c.capabilityGrants],
    accounts: [...c.dataCeilings.accounts],
    folders: [...c.dataCeilings.folders],
    includeE2ee: c.dataCeilings.includeE2ee,
    includeAttachments: c.dataCeilings.includeAttachments,
  };
}

/**
 * The `PUT /admin/assist` body. An optional text field left empty is omitted so the
 * server applies its default. The API key is sent when typed; as `""` when the
 * stored key is to be removed or the endpoint moved (the server does not carry a
 * key to a new endpoint, and an empty key there means "no key"); else left out,
 * which keeps the stored key.
 */
export function assistConfig(v: Readonly<Record<string, unknown>>, enabled: boolean, loadedEndpoint: string): AdminAssistConfig {
  const kind = v.kind as Kind;
  const opt = (key: string): Record<string, unknown> => (text(v[key]).trim() === '' ? {} : { [key]: text(v[key]).trim() });
  let adapter: Record<string, unknown> | null = null;
  if (kind === 'open-ai-compatible') adapter = { kind, baseUrl: text(v.baseUrl).trim(), ...opt('chatModel'), ...opt('embedModel'), ...opt('sttModel') };
  else if (kind === 'anthropic') {
    adapter = { kind, ...opt('baseUrl'), ...opt('model'), ...opt('anthropicVersion'), ...(typeof v.maxTokens === 'number' ? { maxTokens: v.maxTokens } : {}) };
  } else if (kind === 'local-process') adapter = { kind, program: text(v.program), args: [...((v.args as string[] | undefined) ?? [])] };
  if (adapter !== null && kind !== 'local-process') {
    const key = v.apiKey as ApiKeyValue;
    const moved = key.stored && endpointOf(kind, v.baseUrl) !== loadedEndpoint;
    if (key.typed !== '') adapter.apiKey = key.typed;
    else if (key.remove || moved) adapter.apiKey = '';
  }
  return {
    enabled,
    adapter: adapter as AssistAdapter | null,
    capabilityGrants: [...((v.capabilityGrants as AssistCapability[] | undefined) ?? [])],
    dataCeilings: {
      accounts: [...((v.accounts as string[] | undefined) ?? [])],
      folders: [...((v.folders as string[] | undefined) ?? [])],
      includeE2ee: v.includeE2ee === true,
      includeAttachments: v.includeAttachments === true,
    },
  };
}

export interface AdminAssistProps {
  /** The admin client. Defaults to the same-origin HTTP client; tests inject one over a mock fetch. */
  api?: AdminAssistApi;
  rateLimit?: RateLimitApi;
  /** Provider profiles (t39 N8); `false` leaves the panel out (tests of the form above). */
  profiles?: ProfilesApi | false;
  assignments?: AssignmentApi;
}

export function AdminAssist(props: AdminAssistProps): JSX.Element {
  const api = props.api ?? new AdminAssistApi();
  const rateApi = props.rateLimit ?? httpRateLimitApi;
  onMount(() => void loadCatalog('assist'));

  const [status, setStatus] = createSignal<AdminAssistStatus | null>(null);
  const [enabled, setEnabled] = createSignal(DEFAULT_ADMIN_ASSIST_CONFIG.enabled);
  const [loadedEndpoint, setLoadedEndpoint] = createSignal('');
  const [unreadable, setUnreadable] = createSignal(false);
  // The program the stored configuration names, for the status sentence.
  const [storedProgram, setStoredProgram] = createSignal('');
  const [rate, setRate] = createSignal<{ setting: AdminSetting; etag: string | null } | null>(null);
  const [killOpen, setKillOpen] = createSignal(false);
  const [killDone, setKillDone] = createSignal<string | null>(null);
  const [killError, setKillError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);

  const localPrograms = (): readonly string[] => status()?.localPrograms ?? [];
  const docs = (id: string): string => adminDocsHref(AREA, id);
  const isKind = (...kinds: Kind[]) => (v: Readonly<Record<string, unknown>>): boolean => kinds.includes(v.kind as Kind);

  function remember(c: AdminAssistConfig): Record<string, unknown> {
    setEnabled(c.enabled);
    setLoadedEndpoint(endpointOf(c.adapter?.kind, (c.adapter as { baseUrl?: string } | null)?.baseUrl));
    setStoredProgram(c.adapter?.kind === 'local-process' ? c.adapter.program : '');
    return assistValues(c);
  }

  async function loadRate(): Promise<number | null> {
    try {
      const r = await rateApi.load();
      if (r.kind === 'unavailable') {
        setRate(null);
        return null;
      }
      setRate({ setting: r.setting, etag: r.etag });
      return typeof r.setting.effective.value === 'number' ? r.setting.effective.value : Number(r.setting.globalValue ?? r.setting.default ?? 0);
    } catch {
      setRate(null);
      return null;
    }
  }

  async function load(): Promise<SettingsLoad> {
    setUnreadable(false);
    const rateValue = await loadRate();
    try {
      // Both or neither: the form is only usable against a config that was read.
      const [c, s] = await Promise.all([api.get(), api.status()]);
      setStatus(s);
      return { values: { ...remember(c), rateLimitPerMin: rateValue } };
    } catch (e) {
      // The stored key does not open with this server's key. Nothing can be
      // overwritten by mistake (the server cannot read it either), and saving a new
      // configuration is how it is replaced, so the form is offered empty.
      if (e instanceof AdminAssistError && e.code === 'config-unreadable') {
        setUnreadable(true);
        return { values: { ...remember(DEFAULT_ADMIN_ASSIST_CONFIG), rateLimitPerMin: rateValue } };
      }
      throw e;
    }
  }

  async function save(changed: Record<string, unknown>, ctx: { values: Readonly<Record<string, unknown>> }): Promise<{ values: Record<string, unknown> }> {
    const assistChanged = unreadable() || Object.keys(changed).some((k) => k !== 'rateLimitPerMin');
    let values: Record<string, unknown> = { ...ctx.values };
    if (killing) await killing;
    if (assistChanged) {
      try {
        setStatus(await api.save(assistConfig(ctx.values, enabled(), loadedEndpoint())));
      } catch (e) {
        if (e instanceof AdminAssistError && e.status === 422) {
          throw new AdminProblemError(422, { detail: t('assist-admin-save-not-allowed'), field: ctx.values.kind === 'local-process' ? '/program' : '/kind' }, null);
        }
        throw e;
      }
      // Read back what was stored, so fields the server defaulted are shown.
      values = { ...remember(await api.get()), rateLimitPerMin: ctx.values.rateLimitPerMin };
      setUnreadable(false);
    }
    if ('rateLimitPerMin' in changed && rate() !== null) {
      const out = await rateApi.save(Number(ctx.values.rateLimitPerMin), rate()?.etag ?? null);
      setRate(out);
    }
    return { values };
  }

  // A Save sent while the kill switch is in flight would carry the on/off state from
  // before it and undo it, so Save waits for it.
  let killing: Promise<void> | null = null;

  function setKill(on: boolean): Promise<void> {
    killing = runKill(on).finally(() => {
      killing = null;
    });
    return killing;
  }

  async function runKill(on: boolean): Promise<void> {
    setKillError(null);
    setKillDone(null);
    setBusy(true);
    try {
      const next = await api.setKillSwitch(on);
      setStatus(next);
      setEnabled(next.enabled);
      setKillDone(on ? t('assist-admin-stopped-done') : t('assist-admin-resumed-done'));
    } catch {
      setKillError(t('assist-admin-kill-error'));
    } finally {
      setBusy(false);
    }
  }

  const rateSource = (): SourceInfo | undefined => {
    const s = rate()?.setting;
    if (!s) return undefined;
    if (s.envState?.set && s.env?.precedence === 'pin') return { kind: 'env-pin', envVar: s.env.var };
    if (s.globalLocked) return { kind: 'locked', lockedBy: 'global' };
    return { kind: s.globalValue === null || s.globalValue === undefined ? 'default' : 'stored' };
  };

  const programOptions = (v: Readonly<Record<string, unknown>>): { value: string; label: string }[] => {
    const stored = text(v.program);
    const listed = localPrograms().map((p) => ({ value: p, label: p }));
    // A stored program the operator no longer lists stays in the list, marked, so the form shows what is stored.
    return stored !== '' && !localPrograms().includes(stored) ? [{ value: stored, label: t('assist-admin-local-program-unlisted', { program: stored }) }, ...listed] : listed;
  };

  const groups = createMemo((): SettingsGroup[] => [
    {
      id: 'endpoint',
      heading: t('assist-admin-endpoint'),
      help: t('assist-admin-endpoint-note'),
      fields: [
        {
          id: 'kind',
          label: t('assist-admin-kind'),
          help: localPrograms().length === 0 ? t('assist-admin-kind-help-no-local') : t('assist-admin-kind-help'),
          docsHref: docs('kind'),
          kind: 'text',
          propagation: 'restart',
          render: (ctx) => <KindSelect ctx={ctx} local={localPrograms().length > 0} />,
        },
        {
          id: 'baseUrl',
          label: t('assist-admin-base-url'),
          help: t('assist-admin-base-url-help'),
          docsHref: docs('baseUrl'),
          kind: 'url',
          propagation: 'restart',
          placeholder: t('assist-admin-default-placeholder'),
          when: isKind('open-ai-compatible', 'anthropic'),
          validate: (v, all) => (all.kind === 'open-ai-compatible' && text(v).trim() === '' ? t('assist-admin-base-url-required') : null),
        },
        {
          id: 'apiKey',
          label: t('assist-admin-api-key'),
          help: t('assist-admin-api-key-help'),
          docsHref: docs('apiKey'),
          kind: 'text',
          propagation: 'restart',
          when: isKind('open-ai-compatible', 'anthropic'),
          render: (ctx) => <ApiKeyField ctx={ctx} loadedEndpoint={loadedEndpoint()} hideWhen={busy} />,
        },
        { id: 'chatModel', label: t('assist-admin-chat-model'), help: t('assist-admin-chat-model-help'), docsHref: docs('models'), kind: 'text', placeholder: t('assist-admin-default-placeholder'), propagation: 'restart', when: isKind('open-ai-compatible') },
        { id: 'embedModel', label: t('assist-admin-embed-model'), help: t('assist-admin-embed-model-help'), docsHref: docs('models'), kind: 'text', placeholder: t('assist-admin-default-placeholder'), propagation: 'restart', when: isKind('open-ai-compatible') },
        { id: 'sttModel', label: t('assist-admin-stt-model'), help: t('assist-admin-stt-model-help'), docsHref: docs('models'), kind: 'text', placeholder: t('assist-admin-default-placeholder'), propagation: 'restart', when: isKind('open-ai-compatible') },
        { id: 'model', label: t('assist-admin-model'), help: t('assist-admin-model-help'), docsHref: docs('models'), kind: 'text', placeholder: t('assist-admin-default-placeholder'), propagation: 'restart', when: isKind('anthropic') },
        {
          id: 'anthropicVersion',
          label: t('assist-admin-anthropic-version'),
          help: t('assist-admin-anthropic-version-help'),
          docsHref: docs('models'),
          kind: 'text',
          placeholder: t('assist-admin-default-placeholder'),
          propagation: 'restart',
          when: isKind('anthropic'),
        },
        {
          id: 'maxTokens',
          label: t('assist-admin-max-tokens'),
          help: t('assist-admin-max-tokens-help'),
          docsHref: docs('models'),
          kind: 'number',
          min: 1,
          step: 1,
          placeholder: t('assist-admin-default-placeholder'),
          propagation: 'restart',
          when: isKind('anthropic'),
        },
        {
          id: 'program',
          label: t('assist-admin-local-program'),
          help: t('assist-admin-local-program-note'),
          docsHref: docs('program'),
          kind: 'select',
          propagation: 'restart',
          when: isKind('local-process'),
          validate: (v) => {
            if (text(v) === '') return t('assist-admin-local-program-required');
            return localPrograms().includes(text(v)) ? null : t('assist-admin-local-program-unlisted-note');
          },
          render: (ctx) => (
            <select id={ctx.id} class={ux.control} aria-describedby={ctx.describedBy} aria-invalid={ctx.invalid ? true : undefined} value={text(ctx.value)} onChange={(e) => ctx.setValue(e.currentTarget.value)}>
              {programOptions(ctx.values).map((o) => (
                <option value={o.value} selected={o.value === text(ctx.value)}>
                  {o.label}
                </option>
              ))}
            </select>
          ),
        },
        {
          id: 'args',
          label: t('assist-admin-local-args-label'),
          help: t('assist-admin-local-args-note'),
          docsHref: docs('program'),
          kind: 'text',
          propagation: 'restart',
          when: isKind('local-process'),
          render: (ctx) => (
            <ListEditor id={ctx.id} label={t('assist-admin-local-args-label')} items={(ctx.value as string[] | undefined) ?? []} onChange={(l) => ctx.setValue(l)} describedBy={ctx.describedBy} split="none" mono />
          ),
        },
      ],
    },
    {
      id: 'grants',
      heading: t('assist-admin-grants'),
      fields: [
        {
          id: 'capabilityGrants',
          label: t('assist-admin-grants'),
          help: t('assist-admin-grants-note'),
          docsHref: docs('capabilityGrants'),
          kind: 'text',
          propagation: 'restart',
          render: (ctx) => {
            const chosen = (): AssistCapability[] => (ctx.value as AssistCapability[] | undefined) ?? [];
            return (
              <fieldset class={css.checks} id={ctx.id} aria-describedby={ctx.describedBy}>
                <legend class={shell.srOnly}>{t('assist-admin-grants')}</legend>
                {ASSIST_CAPABILITIES.map((cap) => (
                  <div class={css.checkRow}>
                    <input
                      id={`${ctx.id}-${cap}`}
                      type="checkbox"
                      checked={chosen().includes(cap)}
                      onChange={(e) => ctx.setValue(e.currentTarget.checked ? [...chosen().filter((c) => c !== cap), cap] : chosen().filter((c) => c !== cap))}
                    />
                    <label for={`${ctx.id}-${cap}`}>{t(`assist-admin-cap-${cap}`)}</label>
                  </div>
                ))}
              </fieldset>
            );
          },
        },
      ],
    },
    {
      id: 'ceilings',
      heading: t('assist-admin-ceilings'),
      help: t('assist-admin-ceilings-note'),
      fields: [
        {
          id: 'accounts',
          label: t('assist-admin-accounts-label'),
          help: t('assist-admin-accounts-note'),
          docsHref: docs('accounts'),
          kind: 'text',
          propagation: 'restart',
          render: (ctx) => (
            <ListEditor
              id={ctx.id}
              label={t('assist-admin-accounts-label')}
              items={(ctx.value as string[] | undefined) ?? []}
              onChange={(l) => ctx.setValue(l)}
              describedBy={ctx.describedBy}
              mono
              emptyText={t('assist-admin-accounts-none')}
            />
          ),
        },
        {
          id: 'folders',
          label: t('assist-admin-folders-label'),
          help: t('assist-admin-folders-note'),
          docsHref: docs('folders'),
          kind: 'text',
          propagation: 'restart',
          render: (ctx) => (
            <ListEditor
              id={ctx.id}
              label={t('assist-admin-folders-label')}
              items={(ctx.value as string[] | undefined) ?? []}
              onChange={(l) => ctx.setValue(l)}
              describedBy={ctx.describedBy}
              split="none"
              mono
              emptyText={t('assist-admin-folders-none')}
            />
          ),
        },
        { id: 'includeE2ee', label: t('assist-admin-allow-e2ee'), help: t('assist-admin-allow-e2ee-help'), docsHref: docs('includeE2ee'), kind: 'boolean', propagation: 'restart', revert: { defaultDisplay: t('assist-admin-off'), value: false } },
        {
          id: 'includeAttachments',
          label: t('assist-admin-allow-attachments'),
          help: t('assist-admin-allow-attachments-help'),
          docsHref: docs('includeAttachments'),
          kind: 'boolean',
          propagation: 'restart',
          revert: { defaultDisplay: t('assist-admin-off'), value: false },
        },
      ],
    },
    {
      id: 'rate',
      heading: t('assist-admin-rate-heading'),
      help: rate() === null ? t('assist-admin-rate-unavailable') : t('assist-admin-rate-group-help'),
      fields: [
        {
          id: 'rateLimitPerMin',
          label: t('assist-admin-rate-limit'),
          technical: RATE_KEY,
          help: t('assist-admin-rate-limit-help'),
          docsHref: docs('rateLimitPerMin'),
          kind: 'number',
          required: true,
          min: 0,
          max: 10_000,
          step: 1,
          propagation: 'hot',
          when: () => rate() !== null,
          ...(rateSource() ? { source: rateSource()! } : {}),
          history: { target: `setting:${RATE_KEY}`, updatedBy: rate()?.setting.updatedBy ?? null, updatedAt: rate()?.setting.updatedAt ? new Date((rate()?.setting.updatedAt ?? 0) * 1000).toISOString() : null },
          revert: { defaultDisplay: t('assist-admin-rate-none'), value: 0 },
        },
      ],
    },
  ]);

  return (
    <div class={css.listEditor} data-screen="admin-assist">
      <Show when={status()}>
        {(s) => (
          <Feedback
            kind="info"
            testId="assist-status"
            message={
              s().running && s().restartPending
                ? t('assist-admin-status-running-pending')
                : s().running
                  ? t('assist-admin-status-running', { host: s().endpointHost ?? '' })
                  : !s().enabled
                    ? t('assist-admin-status-off')
                    : s().stoppedReason === 'local-program-not-allowed'
                      ? t('assist-admin-status-local-program-not-allowed', { program: storedProgram() })
                      : s().restartPending
                        ? t('assist-admin-status-pending')
                        : t('assist-admin-status-no-endpoint')
            }
          />
        )}
      </Show>
      <Show when={unreadable()}>
        <Feedback kind="error" message={t('assist-admin-load-unreadable')} testId="assist-unreadable" />
      </Show>
      <Feedback kind="success" message={killDone()} testId="assist-kill-done" />
      <Feedback kind="error" message={killError()} />
      <SettingsFormPage
        title={t('assist-admin-title')}
        description={t('assist-admin-intro')}
        groups={groups()}
        load={load}
        save={save}
        allowUnchanged={unreadable()}
        saveLabel={t('assist-admin-save')}
        unavailableReason={t('assist-admin-unavailable')}
        actions={
          <Show when={status()}>
            {(s) => (
              <Switch>
                <Match when={s().enabled}>
                  <button type="button" class="btn btn--danger" disabled={busy()} aria-describedby="assist-kill-note" onClick={() => setKillOpen(true)}>
                    {t('assist-admin-stop')}
                  </button>
                  <span id="assist-kill-note" class={shell.help}>
                    {t('assist-admin-stop-note')}
                  </span>
                </Match>
                <Match when={!s().enabled}>
                  <button type="button" class="btn" disabled={busy()} aria-describedby="assist-kill-note" onClick={() => void setKill(false)}>
                    {t('assist-admin-resume')}
                  </button>
                  <span id="assist-kill-note" class={shell.help}>
                    {t('assist-admin-resume-note')}
                  </span>
                </Match>
              </Switch>
            )}
          </Show>
        }
        testId="admin-assist"
      />
      <Show when={props.profiles !== false}>
        <AssistProfiles {...(props.profiles ? { api: props.profiles } : {})} {...(props.assignments ? { assignments: props.assignments } : {})} />
      </Show>
      <ConfirmDialog
        open={killOpen()}
        title={t('assist-admin-stop-title')}
        body={t('assist-admin-stop-body')}
        confirmLabel={t('assist-admin-stop')}
        onConfirm={async () => {
          await setKill(true);
          setKillOpen(false);
        }}
        onCancel={() => setKillOpen(false)}
      />
    </div>
  );
}

/** The endpoint type; a local program only while the operator lists one (or one is stored). */
function KindSelect(props: { ctx: FieldRenderContext; local: boolean }): JSX.Element {
  const ctx = props.ctx;
  return (
    <select id={ctx.id} class={ux.control} aria-describedby={ctx.describedBy} value={text(ctx.value)} onChange={(e) => ctx.setValue(e.currentTarget.value)}>
      {/* `selected` per option: a conditional option is added after the select's value is set. */}
      <option value="none" selected={ctx.value === 'none'}>
        {t('assist-admin-kind-none')}
      </option>
      <option value="open-ai-compatible" selected={ctx.value === 'open-ai-compatible'}>
        {t('assist-admin-kind-open-ai-compatible')}
      </option>
      <option value="anthropic" selected={ctx.value === 'anthropic'}>
        {t('assist-admin-kind-anthropic')}
      </option>
      <Show when={props.local || ctx.value === 'local-process'}>
        <option value="local-process" selected={ctx.value === 'local-process'}>
          {t('assist-admin-kind-local-process')}
        </option>
      </Show>
    </select>
  );
}

/**
 * The write-only API key. A key is stored for the endpoint being edited (the server
 * says so; it never sends the key): the field says so and offers to replace or
 * remove it. Once the type or base URL changes the stored key will not be used, so
 * an empty key field for the new endpoint is shown instead.
 */
function ApiKeyField(props: { ctx: FieldRenderContext; loadedEndpoint: string; hideWhen: () => unknown }): JSX.Element {
  const ctx = props.ctx;
  const v = (): ApiKeyValue => (ctx.value as ApiKeyValue | undefined) ?? { stored: false, typed: '', remove: false };
  const set = (patch: Partial<ApiKeyValue>): void => ctx.setValue({ ...v(), ...patch });
  const keyStored = (): boolean => v().stored && endpointOf(ctx.values.kind, ctx.values.baseUrl) === props.loadedEndpoint;
  return (
    <Show
      when={keyStored()}
      fallback={<PasswordInput id={ctx.id} class={ux.control} label={t('assist-admin-api-key')} autocomplete="off" aria-describedby={ctx.describedBy} value={v().typed} onInput={(e) => set({ typed: e.currentTarget.value })} hideWhen={props.hideWhen} />}
    >
      <div class={css.listEditor}>
        <p class={shell.help} data-testid="assist-api-key-stored">
          {v().remove ? t('assist-admin-api-key-remove-pending') : t('assist-admin-api-key-stored')}
        </p>
        <Show when={!v().remove}>
          <PasswordInput
            id={ctx.id}
            class={ux.control}
            label={t('assist-admin-api-key-replace')}
            autocomplete="off"
            aria-describedby={ctx.describedBy}
            placeholder={t('assist-admin-api-key-replace-placeholder')}
            value={v().typed}
            onInput={(e) => set({ typed: e.currentTarget.value })}
            hideWhen={props.hideWhen}
          />
        </Show>
        <div class={css.actions}>
          <button type="button" class="btn btn--ghost" onClick={() => set({ remove: !v().remove, typed: '' })}>
            {v().remove ? t('assist-admin-api-key-keep') : t('assist-admin-api-key-remove')}
          </button>
        </div>
      </div>
    </Show>
  );
}

export default AdminAssist;
