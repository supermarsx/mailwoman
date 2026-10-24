// Admin → Assist screen (SPEC §14/§19): the deployment's Assist endpoint, the
// capabilities it may use, the data limits, and the on/off control.
//
// What takes effect when is the server's behaviour, and the screen reports it from
// the server's answers rather than assuming it: turning Assist off stops the
// running gateway at once; everything else is read when the server starts
// (`crates/mw-server/src/v7_mount.rs`, `assist_status` / `store_assist_config`).
//
// Save is only offered once the stored configuration has been loaded. Saving a
// form that still holds defaults would overwrite the deployment's configuration.
//
// A local program is offered only from the list the operator set in
// `MW_ASSIST_LOCAL_PROGRAMS` where the server starts; the server reports that list
// in the status and refuses any other program (`local_program_refusal`).

import { createSignal, For, Match, onMount, Show, Switch, type JSX } from 'solid-js';
import { ASSIST_CAPABILITIES, type AssistCapability } from '../../../modules/assist/types.ts';
import { t, loadCatalog } from '../../../i18n';
import * as css from '../../../modules/assist/styles.css.ts';
import { PasswordInput } from '../../../components/PasswordInput.tsx';
import {
  AdminAssistApi,
  AdminAssistError,
  DEFAULT_ADMIN_ASSIST_CONFIG,
  type AdminAssistConfig,
  type AdminAssistStatus,
  type AssistAdapter,
  type AssistAdapterKind,
} from './service.ts';

export interface AdminAssistProps {
  /** The admin client. Defaults to the same-origin HTTP client; tests inject one over a mock fetch. */
  api?: AdminAssistApi;
}

/** One id per line → list, blank lines dropped. */
function lines(text: string): string[] {
  return text
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
}

/** The endpoint an adapter sends to: kind and base URL. The server keeps a stored key only while this is unchanged. */
function endpointOf(adapter: AssistAdapter | null): string {
  if (adapter === null || adapter.kind === 'local-process') return '';
  return `${adapter.kind} ${adapter.baseUrl ?? ''}`;
}

/**
 * The adapter as it is sent: an optional text field left empty is omitted so the
 * server applies its default, instead of storing an empty model name.
 *
 * The API key is write-only. An empty key field is left out, which keeps the stored
 * key, except when the stored key is being removed or the endpoint has moved: then
 * it is sent as `""`, because the server does not carry a key to a new endpoint and
 * an empty field there means "no key".
 */
function adapterForSave(adapter: AssistAdapter | null, sendEmptyKey: boolean): AssistAdapter | null {
  if (adapter === null) return null;
  const out: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(adapter)) {
    const optionalText = typeof value === 'string' && key !== 'kind' && key !== 'apiKey';
    const required = adapter.kind === 'open-ai-compatible' && key === 'baseUrl';
    if (optionalText && !required && value.trim() === '') continue;
    if (key === 'apiKey' && value === '') continue;
    out[key] = value;
  }
  if (sendEmptyKey && adapter.kind !== 'local-process' && out['apiKey'] === undefined) out['apiKey'] = '';
  return out as unknown as AssistAdapter;
}

export function AdminAssist(props: AdminAssistProps): JSX.Element {
  const api = props.api ?? new AdminAssistApi();
  const [config, setConfig] = createSignal<AdminAssistConfig>(DEFAULT_ADMIN_ASSIST_CONFIG);
  const [status, setStatus] = createSignal<AdminAssistStatus | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [loaded, setLoaded] = createSignal(false);
  // The adapter as last read from the server, to tell whether the endpoint moved.
  const [loadedAdapter, setLoadedAdapter] = createSignal<AssistAdapter | null>(null);
  // The administrator asked to remove the stored API key on the next save.
  const [removeKey, setRemoveKey] = createSignal(false);
  // One write at a time: a Save sent while the kill switch is still in flight would
  // carry the on/off state from before it and undo it.
  const [busy, setBusy] = createSignal(false);

  async function load(): Promise<void> {
    // Both or neither: the form is only usable against a config that was read.
    const [c, s] = await Promise.all([api.get(), api.status()]);
    setConfig(c);
    setLoadedAdapter(c.adapter);
    setStatus(s);
    setLoaded(true);
  }

  onMount(() => void loadCatalog('assist'));
  onMount(() => {
    void load().catch((e: unknown) => {
      // The stored key does not open with this server's key. There is nothing to
      // overwrite by mistake (the server cannot read it either), and saving a new
      // configuration is how it is replaced, so the form is offered empty.
      if (e instanceof AdminAssistError && e.code === 'config-unreadable') {
        setError(t('assist-admin-load-unreadable'));
        setLoaded(true);
        return;
      }
      setError(t('assist-admin-load-error'));
    });
  });

  function patch(next: Partial<AdminAssistConfig>): void {
    setConfig((prev) => ({ ...prev, ...next }));
    setNotice(null);
  }

  /** The programs the operator allows a local-process endpoint to run (empty until the status is read). */
  const localPrograms = (): readonly string[] => status()?.localPrograms ?? [];

  /** The configured local program, if the endpoint is one. */
  const localProgram = (): string => {
    const adapter = config().adapter;
    return adapter?.kind === 'local-process' ? adapter.program : '';
  };

  /** The configured local program's arguments, if the endpoint is one. */
  const localArgs = (): readonly string[] => {
    const adapter = config().adapter;
    return adapter?.kind === 'local-process' ? (adapter.args ?? []) : [];
  };

  const unlisted = (): boolean => localProgram() !== '' && !localPrograms().includes(localProgram());

  function setKind(kind: AssistAdapterKind | 'none'): void {
    if (kind === 'none') patch({ adapter: null });
    else if (kind === 'open-ai-compatible') patch({ adapter: { kind, baseUrl: '' } });
    else if (kind === 'anthropic') patch({ adapter: { kind } });
    else patch({ adapter: { kind, program: localPrograms()[0] ?? '', args: [] } });
  }

  /** Set one field of the current adapter; `undefined` removes it. */
  function setAdapterField(key: string, value: string | number | readonly string[] | undefined): void {
    const adapter = config().adapter;
    if (adapter === null) return;
    const next: Record<string, unknown> = { ...adapter, [key]: value };
    if (value === undefined) delete next[key];
    patch({ adapter: next as unknown as AssistAdapter });
  }

  function adapterText(key: string): string {
    const value = (config().adapter as Record<string, unknown> | null)?.[key];
    return typeof value === 'string' || typeof value === 'number' ? String(value) : '';
  }

  function toggleGrant(cap: AssistCapability, granted: boolean): void {
    const rest = config().capabilityGrants.filter((c) => c !== cap);
    patch({ capabilityGrants: granted ? [...rest, cap] : rest });
  }

  async function save(): Promise<void> {
    setError(null);
    setNotice(null);
    const adapter = config().adapter;
    if (adapter?.kind === 'open-ai-compatible' && !/^https?:\/\/\S+/.test(adapter.baseUrl.trim())) {
      setError(t('assist-admin-base-url-required'));
      return;
    }
    if (adapter?.kind === 'local-process' && adapter.program === '') {
      setError(t('assist-admin-local-program-required'));
      return;
    }
    setBusy(true);
    try {
      // A stored key the new endpoint will not get: say "no key" rather than leave
      // the server to refuse a move that did not mention the key.
      const loadedKey = loadedAdapter();
      const dropsKey =
        loadedKey !== null &&
        loadedKey.kind !== 'local-process' &&
        loadedKey.hasApiKey === true &&
        endpointOf(adapter) !== endpointOf(loadedKey);
      const next = await api.save({ ...config(), adapter: adapterForSave(adapter, removeKey() || dropsKey) });
      setStatus(next);
      // Read back what was stored, so fields the server defaulted are shown.
      const stored = await api.get();
      setConfig(stored);
      setLoadedAdapter(stored.adapter);
      setRemoveKey(false);
      setNotice(next.restartPending ? t('assist-admin-saved-pending') : t('assist-admin-saved'));
    } catch (e) {
      setError(
        e instanceof AdminAssistError && e.status === 422
          ? t('assist-admin-save-not-allowed')
          : t('assist-admin-save-error'),
      );
    } finally {
      setBusy(false);
    }
  }

  async function setKill(on: boolean): Promise<void> {
    setError(null);
    setNotice(null);
    setBusy(true);
    try {
      const next = await api.setKillSwitch(on);
      setStatus(next);
      setConfig((prev) => ({ ...prev, enabled: next.enabled }));
    } catch {
      setError(t('assist-admin-kill-error'));
    } finally {
      setBusy(false);
    }
  }

  // `labelId` rather than the label text, so `t` runs inside the JSX and the label
  // follows the catalog when it finishes loading.
  const field = (
    key: string,
    labelId: string,
    opts: { secret?: boolean; fallback?: boolean; placeholderId?: string } = {},
  ): JSX.Element => {
    const placeholder = (): string | undefined =>
      opts.placeholderId !== undefined
        ? t(opts.placeholderId)
        : opts.fallback
          ? t('assist-admin-default-placeholder')
          : undefined;
    return (
      <label class={css.field}>
        <span class={css.meta}>{t(labelId)}</span>
        <Show
          when={opts.secret}
          fallback={
            <input
              class={css.input}
              type="text"
              autocomplete="off"
              aria-label={t(labelId)}
              placeholder={placeholder()}
              value={adapterText(key)}
              onInput={(e) => setAdapterField(key, e.currentTarget.value)}
            />
          }
        >
          <PasswordInput
            class={css.input}
            label={t(labelId)}
            autocomplete="off"
            placeholder={placeholder()}
            value={adapterText(key)}
            onInput={(e) => setAdapterField(key, e.currentTarget.value)}
            hideWhen={busy}
          />
        </Show>
      </label>
    );
  };

  /**
   * A key is stored for the endpoint being edited (the server says so; it never
   * sends the key). Once the type or base URL is changed the stored key will not be
   * used, so the screen shows an empty key field for the new endpoint instead.
   */
  const keyStored = (): boolean => {
    const adapter = config().adapter;
    return (
      adapter !== null &&
      adapter.kind !== 'local-process' &&
      adapter.hasApiKey === true &&
      endpointOf(adapter) === endpointOf(loadedAdapter())
    );
  };

  // The API key: write-only. With a key stored, the screen says so and offers to
  // replace or remove it; it cannot show it.
  const apiKeyField = (): JSX.Element => (
    <Show when={keyStored()} fallback={field('apiKey', 'assist-admin-api-key', { secret: true })}>
      <p class={css.meta} data-testid="assist-api-key-stored">
        {removeKey() ? t('assist-admin-api-key-remove-pending') : t('assist-admin-api-key-stored')}
      </p>
      <Show when={!removeKey()}>
        {field('apiKey', 'assist-admin-api-key-replace', {
          secret: true,
          placeholderId: 'assist-admin-api-key-replace-placeholder',
        })}
      </Show>
      <div class={css.row}>
        <button
          type="button"
          class={css.ghost}
          onClick={() => {
            setAdapterField('apiKey', undefined);
            setRemoveKey(!removeKey());
          }}
        >
          {removeKey() ? t('assist-admin-api-key-keep') : t('assist-admin-api-key-remove')}
        </button>
      </div>
    </Show>
  );

  return (
    <section class={css.panel} data-screen="admin-assist" aria-label={t('assist-admin-title')}>
      <div class={css.section}>
        <h2 class={css.heading}>{t('assist-admin-title')}</h2>
        <p class={css.prose}>{t('assist-admin-intro')}</p>

        {/* What the running server is doing, as it reports it. */}
        <Show when={status()}>
          {(s) => (
            <p class={css.prose} role="status" data-testid="assist-status">
              <Switch>
                <Match when={s().running && s().restartPending}>{t('assist-admin-status-running-pending')}</Match>
                <Match when={s().running}>
                  {t('assist-admin-status-running', { host: s().endpointHost ?? '' })}
                </Match>
                <Match when={!s().enabled}>{t('assist-admin-status-off')}</Match>
                <Match when={s().stoppedReason === 'local-program-not-allowed'}>
                  {t('assist-admin-status-local-program-not-allowed', { program: localProgram() })}
                </Match>
                <Match when={s().restartPending}>{t('assist-admin-status-pending')}</Match>
                <Match when={true}>{t('assist-admin-status-no-endpoint')}</Match>
              </Switch>
            </p>
          )}
        </Show>

        {/* Kill switch (§19). Acts on the server directly; it does not save the form. */}
        <Show when={status()}>
          {(s) => (
            <div class={css.row}>
              <Show
                when={s().enabled}
                fallback={
                  <>
                    <button type="button" class={css.ghost} disabled={busy()} onClick={() => void setKill(false)}>
                      {t('assist-admin-resume')}
                    </button>
                    <span class={css.meta}>{t('assist-admin-resume-note')}</span>
                  </>
                }
              >
                <button type="button" class={css.button} disabled={busy()} onClick={() => void setKill(true)}>
                  {t('assist-admin-stop')}
                </button>
                <span class={css.meta}>{t('assist-admin-stop-note')}</span>
              </Show>
            </div>
          )}
        </Show>
      </div>

      {/* Endpoint. */}
      <div class={css.section}>
        <span class={css.subHeading}>{t('assist-admin-endpoint')}</span>
        <p class={css.prose}>{t('assist-admin-endpoint-note')}</p>
        <label class={css.field}>
          <span class={css.meta}>{t('assist-admin-kind')}</span>
          <select
            class={css.input}
            aria-label={t('assist-admin-kind')}
            value={config().adapter?.kind ?? 'none'}
            onChange={(e) => setKind(e.currentTarget.value as AssistAdapterKind | 'none')}
          >
            <option value="none">{t('assist-admin-kind-none')}</option>
            <option value="open-ai-compatible">{t('assist-admin-kind-open-ai-compatible')}</option>
            <option value="anthropic">{t('assist-admin-kind-anthropic')}</option>
            <Show when={localPrograms().length > 0 || config().adapter?.kind === 'local-process'}>
              <option value="local-process">{t('assist-admin-kind-local-process')}</option>
            </Show>
          </select>
        </label>
        <Show when={status() !== null && localPrograms().length === 0}>
          <p class={css.meta} data-testid="assist-local-program-enable">
            {t('assist-admin-local-program-enable')}
          </p>
        </Show>
        <Switch>
          <Match when={config().adapter?.kind === 'open-ai-compatible'}>
            {field('baseUrl', 'assist-admin-base-url')}
            {apiKeyField()}
            {field('chatModel', 'assist-admin-chat-model', { fallback: true })}
            {field('embedModel', 'assist-admin-embed-model', { fallback: true })}
            {field('sttModel', 'assist-admin-stt-model', { fallback: true })}
          </Match>
          <Match when={config().adapter?.kind === 'anthropic'}>
            {field('baseUrl', 'assist-admin-base-url', { fallback: true })}
            {apiKeyField()}
            {field('model', 'assist-admin-model', { fallback: true })}
            {field('anthropicVersion', 'assist-admin-anthropic-version', { fallback: true })}
            <label class={css.field}>
              <span class={css.meta}>{t('assist-admin-max-tokens')}</span>
              <input
                class={css.input}
                type="number"
                min="1"
                aria-label={t('assist-admin-max-tokens')}
                placeholder={t('assist-admin-default-placeholder')}
                value={adapterText('maxTokens')}
                onInput={(e) => {
                  const n = Number.parseInt(e.currentTarget.value, 10);
                  // Emptied or not a positive number: leave it to the server default.
                  setAdapterField('maxTokens', Number.isFinite(n) && n > 0 ? n : undefined);
                }}
              />
            </label>
          </Match>
          <Match when={config().adapter?.kind === 'local-process'}>
            <p class={css.prose}>{t('assist-admin-local-program-note')}</p>
            <label class={css.field}>
              <span class={css.meta}>{t('assist-admin-local-program')}</span>
              <select
                class={css.input}
                aria-label={t('assist-admin-local-program')}
                value={localProgram()}
                onChange={(e) => setAdapterField('program', e.currentTarget.value)}
              >
                {/* A stored program the operator no longer lists stays in the list, marked, so the form shows what is stored. */}
                <Show when={unlisted()}>
                  <option value={localProgram()}>
                    {t('assist-admin-local-program-unlisted', { program: localProgram() })}
                  </option>
                </Show>
                <For each={localPrograms()}>{(program) => <option value={program}>{program}</option>}</For>
              </select>
            </label>
            <Show when={unlisted()}>
              <p class={css.error} data-testid="assist-local-program-unlisted">
                {t('assist-admin-local-program-unlisted-note')}
              </p>
            </Show>
            <label class={css.field}>
              <span class={css.meta}>{t('assist-admin-local-args')}</span>
              <textarea
                class={css.input}
                rows="3"
                aria-label={t('assist-admin-local-args')}
                value={localArgs().join('\n')}
                onChange={(e) => setAdapterField('args', lines(e.currentTarget.value))}
              />
              <span class={css.meta}>{t('assist-admin-local-args-note')}</span>
            </label>
          </Match>
        </Switch>
      </div>

      {/* Capability grants. */}
      <div class={css.section}>
        <span class={css.subHeading}>{t('assist-admin-grants')}</span>
        <p class={css.prose}>{t('assist-admin-grants-note')}</p>
        <div class={css.field}>
          <For each={ASSIST_CAPABILITIES}>
            {(cap) => (
              <label class={css.check}>
                <input
                  type="checkbox"
                  checked={config().capabilityGrants.includes(cap)}
                  aria-label={t(`assist-admin-cap-${cap}`)}
                  onChange={(e) => toggleGrant(cap, e.currentTarget.checked)}
                />
                <span>{t(`assist-admin-cap-${cap}`)}</span>
              </label>
            )}
          </For>
        </div>
      </div>

      {/* Data limits. */}
      <div class={css.section}>
        <span class={css.subHeading}>{t('assist-admin-ceilings')}</span>
        <p class={css.prose}>{t('assist-admin-ceilings-note')}</p>
        <label class={css.field}>
          <span class={css.meta}>{t('assist-admin-accounts')}</span>
          <textarea
            class={css.input}
            rows="3"
            aria-label={t('assist-admin-accounts')}
            value={config().dataCeilings.accounts.join('\n')}
            onChange={(e) =>
              patch({ dataCeilings: { ...config().dataCeilings, accounts: lines(e.currentTarget.value) } })
            }
          />
          <span class={css.meta}>{t('assist-admin-accounts-note')}</span>
        </label>
        <label class={css.field}>
          <span class={css.meta}>{t('assist-admin-folders')}</span>
          <textarea
            class={css.input}
            rows="3"
            aria-label={t('assist-admin-folders')}
            value={config().dataCeilings.folders.join('\n')}
            onChange={(e) =>
              patch({ dataCeilings: { ...config().dataCeilings, folders: lines(e.currentTarget.value) } })
            }
          />
          <span class={css.meta}>{t('assist-admin-folders-note')}</span>
        </label>
        <label class={css.check}>
          <input
            type="checkbox"
            checked={config().dataCeilings.includeE2ee}
            aria-label={t('assist-admin-allow-e2ee')}
            onChange={(e) =>
              patch({ dataCeilings: { ...config().dataCeilings, includeE2ee: e.currentTarget.checked } })
            }
          />
          <span>{t('assist-admin-allow-e2ee')}</span>
        </label>
        <label class={css.check}>
          <input
            type="checkbox"
            checked={config().dataCeilings.includeAttachments}
            aria-label={t('assist-admin-allow-attachments')}
            onChange={(e) =>
              patch({
                dataCeilings: { ...config().dataCeilings, includeAttachments: e.currentTarget.checked },
              })
            }
          />
          <span>{t('assist-admin-allow-attachments')}</span>
        </label>
      </div>

      <div class={css.row}>
        <button type="button" class={css.button} disabled={!loaded() || busy()} onClick={() => void save()}>
          {t('assist-admin-save')}
        </button>
        <Show when={notice() !== null}>
          <span class={css.meta} role="status" data-testid="assist-notice">
            {notice()}
          </span>
        </Show>
        <Show when={error() !== null}>
          <span class={css.error} role="alert">
            {error()}
          </span>
        </Show>
      </div>
    </section>
  );
}

export default AdminAssist;
