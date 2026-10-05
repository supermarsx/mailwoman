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

import { createSignal, For, Match, onMount, Show, Switch, type JSX } from 'solid-js';
import { ASSIST_CAPABILITIES, type AssistCapability } from '../../../modules/assist/types.ts';
import { t, loadCatalog } from '../../../i18n';
import * as css from '../../../modules/assist/styles.css.ts';
import {
  AdminAssistApi,
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

/**
 * The adapter as it is sent: an optional text field left empty is omitted so the
 * server applies its default, instead of storing an empty model name.
 */
function adapterForSave(adapter: AssistAdapter | null): AssistAdapter | null {
  if (adapter === null) return null;
  const out: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(adapter)) {
    const optionalText = typeof value === 'string' && key !== 'kind' && key !== 'apiKey';
    const required = adapter.kind === 'open-ai-compatible' && key === 'baseUrl';
    if (optionalText && !required && value.trim() === '') continue;
    out[key] = value;
  }
  return out as unknown as AssistAdapter;
}

export function AdminAssist(props: AdminAssistProps): JSX.Element {
  const api = props.api ?? new AdminAssistApi();
  const [config, setConfig] = createSignal<AdminAssistConfig>(DEFAULT_ADMIN_ASSIST_CONFIG);
  const [status, setStatus] = createSignal<AdminAssistStatus | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [loaded, setLoaded] = createSignal(false);
  // One write at a time: a Save sent while the kill switch is still in flight would
  // carry the on/off state from before it and undo it.
  const [busy, setBusy] = createSignal(false);

  async function load(): Promise<void> {
    // Both or neither: the form is only usable against a config that was read.
    const [c, s] = await Promise.all([api.get(), api.status()]);
    setConfig(c);
    setStatus(s);
    setLoaded(true);
  }

  onMount(() => void loadCatalog('assist'));
  onMount(() => {
    void load().catch(() => setError(t('assist-admin-load-error')));
  });

  function patch(next: Partial<AdminAssistConfig>): void {
    setConfig((prev) => ({ ...prev, ...next }));
    setNotice(null);
  }

  function setKind(kind: AssistAdapterKind | 'none'): void {
    if (kind === 'none') patch({ adapter: null });
    else if (kind === 'open-ai-compatible') patch({ adapter: { kind, baseUrl: '' } });
    else if (kind === 'anthropic') patch({ adapter: { kind } });
  }

  /** Set one field of the current adapter; `undefined` removes it. */
  function setAdapterField(key: string, value: string | number | undefined): void {
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
    setBusy(true);
    try {
      const next = await api.save({ ...config(), adapter: adapterForSave(adapter) });
      setStatus(next);
      // Read back what was stored, so fields the server defaulted are shown.
      setConfig(await api.get());
      setNotice(next.restartPending ? t('assist-admin-saved-pending') : t('assist-admin-saved'));
    } catch {
      setError(t('assist-admin-save-error'));
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
  const field = (key: string, labelId: string, opts: { secret?: boolean; fallback?: boolean } = {}): JSX.Element => (
    <label class={css.field}>
      <span class={css.meta}>{t(labelId)}</span>
      <input
        class={css.input}
        type={opts.secret ? 'password' : 'text'}
        autocomplete="off"
        aria-label={t(labelId)}
        placeholder={opts.fallback ? t('assist-admin-default-placeholder') : undefined}
        value={adapterText(key)}
        onInput={(e) => setAdapterField(key, e.currentTarget.value)}
      />
    </label>
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
            disabled={config().adapter?.kind === 'local-process'}
            onChange={(e) => setKind(e.currentTarget.value as AssistAdapterKind | 'none')}
          >
            <option value="none">{t('assist-admin-kind-none')}</option>
            <option value="open-ai-compatible">{t('assist-admin-kind-open-ai-compatible')}</option>
            <option value="anthropic">{t('assist-admin-kind-anthropic')}</option>
            <Show when={config().adapter?.kind === 'local-process'}>
              <option value="local-process">{t('assist-admin-kind-local-process')}</option>
            </Show>
          </select>
        </label>
        <Switch>
          <Match when={config().adapter?.kind === 'open-ai-compatible'}>
            {field('baseUrl', 'assist-admin-base-url')}
            {field('apiKey', 'assist-admin-api-key', { secret: true })}
            {field('chatModel', 'assist-admin-chat-model', { fallback: true })}
            {field('embedModel', 'assist-admin-embed-model', { fallback: true })}
            {field('sttModel', 'assist-admin-stt-model', { fallback: true })}
          </Match>
          <Match when={config().adapter?.kind === 'anthropic'}>
            {field('baseUrl', 'assist-admin-base-url', { fallback: true })}
            {field('apiKey', 'assist-admin-api-key', { secret: true })}
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
            <p class={css.meta}>
              {t('assist-admin-local-process-note', { program: adapterText('program') })}
            </p>
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
