// Admin → Plugins screen (SPEC §22): register an engine plugin, approve it, grant
// it capabilities, enable it, and see whether it is running.
//
// The screen shows what the server reports and nothing more (`plugin_view` in
// crates/mw-server/src/plugins.rs): `enabled` is the stored setting, and the status
// chip and line come from `loaded`, `restartRequired` and `notLoadedReason`, so a
// plugin that is enabled but not running is shown as not running, with the reason.
// A refused change is shown with the server's refusal code.
//
// It takes an injected `PluginsSlice` or `PluginsApi` so it is unit-testable; the
// production default is the HTTP client.

import { createEffect, createMemo, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import {
  createPluginsSlice,
  createHttpPluginsApi,
  FIRST_PARTY_PLUGIN_IDS,
  PLUGIN_CAPABILITIES,
  PluginsApiError,
  isHighPowerCapability,
  type ClassifierTest,
  type NotLoadedReason,
  type PluginCapability,
  type PluginsApi,
  type PluginsSlice,
  type PluginInfo,
} from '../../../state/slices/plugins.ts';
import { t, loadCatalog } from '../../../i18n';
import { AllowlistPanel } from './Allowlist.tsx';
import * as css from './styles.css.ts';

export interface AdminPluginsProps {
  /** Tests inject a slice or a client; production defaults to the HTTP client. */
  slice?: PluginsSlice;
  api?: PluginsApi;
}

/** The `<select>` value that stands for "a third-party component". */
const THIRD_PARTY = '';

/** Split a comma-separated host list into its non-empty entries. */
export function parseHosts(text: string): string[] {
  return text
    .split(',')
    .map((h) => h.trim())
    .filter((h) => h.length > 0);
}

/** The line explaining why a plugin is not loaded. */
function reasonText(reason: NotLoadedReason): string {
  switch (reason) {
    case 'not-approved':
      return t('plugins-admin-reason-not-approved');
    case 'disabled':
      return t('plugins-admin-reason-disabled');
    case 'unsigned-not-allowed':
      return t('plugins-admin-reason-unsigned-not-allowed');
    case 'no-grant':
      return t('plugins-admin-reason-no-grant');
    case 'component-unavailable':
      return t('plugins-admin-reason-component-unavailable');
    case 'load-failed':
      return t('plugins-admin-reason-load-failed');
    case 'proxy-mode':
      return t('plugins-admin-reason-proxy-mode');
    case 'no-account-binding':
      return t('plugins-admin-reason-no-account-binding');
    case 'no-host-caller':
      return t('plugins-admin-reason-no-host-caller');
    case 'another-classifier-active':
      return t('plugins-admin-reason-another-classifier-active');
  }
}

/** The message for a refused change, by the server's refusal code. */
function refusalText(error: PluginsApiError): string {
  switch (error.code) {
    case 'already-registered':
      return t('plugins-admin-error-already-registered');
    case 'component-unavailable':
      return t('plugins-admin-error-component-unavailable');
    case 'digest-not-approved':
      return t('plugins-admin-error-digest-not-approved');
    case 'not-approved':
      return t('plugins-admin-error-not-approved');
    case 'unsigned-not-allowed':
      return t('plugins-admin-error-unsigned-not-allowed');
    case 'not-loaded':
      return t('plugins-admin-error-not-loaded');
    case 'classifier-error':
      return t('plugins-admin-error-classifier-error', { detail: error.message });
    default:
      return t('plugins-admin-error-other', { detail: error.message });
  }
}

export function AdminPlugins(props: AdminPluginsProps): JSX.Element {
  const slice = props.slice ?? createPluginsSlice(props.api ?? createHttpPluginsApi());

  onMount(() => void loadCatalog('admin'));
  onMount(() => void loadCatalog('plugins'));
  createEffect(() => {
    void slice.load();
  });
  // Cards are keyed by plugin id, not by the object a reload returns, so a card
  // (its unsaved grant ticks, its test result, focus) survives every re-read.
  const ids = createMemo(() => slice.plugins().map((p) => p.id), [] as string[], {
    equals: (a, b) => a.length === b.length && a.every((id, i) => id === b[i]),
  });

  return (
    <section class={css.screen} data-screen="admin-plugins" aria-label={t('admin-plugins-title')}>
      <div>
        <h2 class={css.heading}>{t('admin-plugins-title')}</h2>
        <p class={css.prose}>{t('plugins-admin-intro')}</p>
      </div>

      <Show when={slice.hasUnsignedLoaded()}>
        <p class={css.unsignedBanner} role="alert" data-testid="unsigned-banner">
          {t('plugins-admin-unsigned-banner')}
        </p>
      </Show>

      <Show when={slice.lastError()}>
        {(error) => (
          <p class={css.error} role="alert" data-testid="plugins-error">
            {refusalText(error())}
          </p>
        )}
      </Show>

      <RegisterForm slice={slice} />

      <Show when={slice.loadFailed()}>
        <p class={css.error} role="alert" data-testid="plugins-load-error">
          {t('plugins-admin-load-error')}
        </p>
      </Show>
      <Show when={!slice.loading() && !slice.loadFailed() && slice.plugins().length === 0}>
        <p class={css.meta} data-testid="plugins-empty">
          {t('admin-plugins-empty')}
        </p>
      </Show>

      <ul class={css.list}>
        <For each={ids()}>
          {(id) => (
            <Show when={slice.plugins().find((p) => p.id === id)}>
              {(plugin) => <PluginCard plugin={plugin()} slice={slice} />}
            </Show>
          )}
        </For>
      </ul>

      {/* Third-party allowlist: the digest pins a third-party registration needs.
          Shares this screen's slice (one client, one load lifecycle). */}
      <AllowlistPanel slice={slice} />
    </section>
  );
}

function RegisterForm(props: { slice: PluginsSlice }): JSX.Element {
  const slice = props.slice;
  const unregistered = (): string[] =>
    FIRST_PARTY_PLUGIN_IDS.filter((id) => !slice.plugins().some((p) => p.id === id));
  const [component, setComponent] = createSignal<string>(FIRST_PARTY_PLUGIN_IDS[0] ?? THIRD_PARTY);
  const [id, setId] = createSignal('');
  const [name, setName] = createSignal('');
  const [version, setVersion] = createSignal('');
  const [caps, setCaps] = createSignal<PluginCapability[]>([]);
  const [hosts, setHosts] = createSignal('');
  const thirdParty = (): boolean => component() === THIRD_PARTY;
  // Keep the selection on something that can still be registered.
  createEffect(() => {
    if (!thirdParty() && !unregistered().includes(component())) {
      setComponent(unregistered()[0] ?? THIRD_PARTY);
    }
  });

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    const netAllowlist = parseHosts(hosts());
    const accepted = thirdParty()
      ? await slice.register({
          id: id().trim(),
          name: name().trim(),
          version: version().trim(),
          capabilities: caps(),
          netAllowlist,
        })
      : // A first-party registration is the id, plus hosts only when some were given
        // (the server refuses any other manifest key for these ids).
        await slice.register(netAllowlist.length > 0 ? { id: component(), netAllowlist } : { id: component() });
    if (accepted) {
      setId('');
      setName('');
      setVersion('');
      setCaps([]);
      setHosts('');
    }
  }

  return (
    <form class={css.form} aria-label={t('plugins-admin-register-heading')} onSubmit={(e) => void submit(e)}>
      <h3 class={css.subheading}>{t('plugins-admin-register-heading')}</h3>
      <label class={css.field}>
        <span>{t('plugins-admin-register-component')}</span>
        <select
          class={css.input}
          aria-label={t('plugins-admin-register-component')}
          value={component()}
          onChange={(e) => setComponent(e.currentTarget.value)}
        >
          <For each={unregistered()}>{(fp) => <option value={fp}>{fp}</option>}</For>
          <option value={THIRD_PARTY}>{t('plugins-admin-register-third-party')}</option>
        </select>
      </label>

      <Show when={thirdParty()}>
        <label class={css.field}>
          <span>{t('plugins-admin-register-id')}</span>
          <input
            class={css.input}
            aria-label={t('plugins-admin-register-id')}
            value={id()}
            required
            onInput={(e) => setId(e.currentTarget.value)}
          />
          <span class={css.meta}>{t('plugins-admin-register-id-hint')}</span>
        </label>
        <label class={css.field}>
          <span>{t('plugins-admin-register-name')}</span>
          <input class={css.input} value={name()} required onInput={(e) => setName(e.currentTarget.value)} />
        </label>
        <label class={css.field}>
          <span>{t('plugins-admin-register-version')}</span>
          <input class={css.input} value={version()} required onInput={(e) => setVersion(e.currentTarget.value)} />
        </label>
        <fieldset class={css.field}>
          <legend>{t('plugins-admin-register-capabilities')}</legend>
          {/* The first-party-only capability is not offered: the server refuses it. */}
          <For each={PLUGIN_CAPABILITIES.filter((c) => !isHighPowerCapability(c))}>
            {(cap) => (
              <label class={css.check}>
                <input
                  type="checkbox"
                  checked={caps().includes(cap)}
                  onChange={(e) =>
                    setCaps(e.currentTarget.checked ? [...caps(), cap] : caps().filter((c) => c !== cap))
                  }
                />
                <span class={css.limits}>{cap}</span>
              </label>
            )}
          </For>
        </fieldset>
      </Show>

      <label class={css.field}>
        <span>{t('plugins-admin-register-hosts')}</span>
        <input
          class={css.input}
          aria-label={t('plugins-admin-register-hosts')}
          value={hosts()}
          onInput={(e) => setHosts(e.currentTarget.value)}
        />
        <span class={css.meta}>
          {thirdParty()
            ? t('plugins-admin-register-hosts-hint-third-party')
            : t('plugins-admin-register-hosts-hint-first-party')}
        </span>
      </label>
      <div class={css.row}>
        <button type="submit" class={css.button}>
          {t('plugins-admin-register-submit')}
        </button>
      </div>
    </form>
  );
}

function PluginCard(props: { plugin: PluginInfo; slice: PluginsSlice }): JSX.Element {
  const p = (): PluginInfo => props.plugin;
  const slice = props.slice;
  // The grant being edited; starts from, and follows, what the server has stored.
  const [draft, setDraft] = createSignal<PluginCapability[]>([]);
  createEffect(() => setDraft(p().granted));
  const [endpoint, setEndpoint] = createSignal('');
  createEffect(() => setEndpoint(p().endpoint ?? ''));
  const [tested, setTested] = createSignal<ClassifierTest | null>(null);
  const [testError, setTestError] = createSignal<PluginsApiError | null>(null);
  const [confirmUninstall, setConfirmUninstall] = createSignal(false);

  async function runTest(): Promise<void> {
    setTested(null);
    setTestError(null);
    try {
      setTested(await slice.api.testClassifier(p().id));
    } catch (e) {
      if (!(e instanceof PluginsApiError)) throw e;
      setTestError(e);
    }
  }

  return (
    <li class={css.card} data-plugin-id={p().id} data-testid="plugin-card">
      <div class={css.cardHead}>
        <div>
          <p class={css.title}>
            <span dir="auto">{p().name}</span>{' '}
            <span class={css.meta}>{t('admin-plugins-version', { version: p().version })}</span>
          </p>
          <div class={css.row}>
            <span class={css.chip} data-testid="trust-chip">
              {p().firstParty ? t('plugins-admin-trust-first-party') : t('plugins-admin-trust-third-party')}
            </span>
            {/* A first-party component is trusted by its digest; signed or not is a
                third-party question. */}
            <Show when={!p().firstParty}>
              <span
                class={`${css.chip} ${p().signed ? css.signedChip : css.unsignedChip}`}
                data-testid="sig-chip"
              >
                {p().signed ? t('admin-plugins-signed') : t('admin-plugins-unsigned')}
              </span>
            </Show>
            <Show when={p().approved}>
              <span class={css.chip}>{t('admin-plugins-approved')}</span>
            </Show>
            <span
              class={`${css.chip} ${p().loaded ? css.loadedChip : ''} ${p().restartRequired ? css.restartChip : ''}`}
              data-testid="status-chip"
            >
              {p().restartRequired
                ? t('plugins-admin-status-restart')
                : p().loaded
                  ? t('plugins-admin-status-loaded')
                  : t('plugins-admin-status-not-loaded')}
            </span>
          </div>
        </div>
        <div class={css.row}>
          <Show when={!p().approved}>
            <button type="button" class={css.button} onClick={() => void slice.approve(p().id)}>
              {t('admin-plugins-approve')}
            </button>
          </Show>
          <Show when={p().approved && !p().enabled}>
            <button type="button" class={css.button} onClick={() => void slice.enable(p().id)}>
              {t('admin-plugins-enable')}
            </button>
          </Show>
          <Show when={p().enabled}>
            <button type="button" class={css.danger} onClick={() => void slice.disable(p().id)}>
              {t('admin-plugins-disable')}
            </button>
          </Show>
        </div>
      </div>

      <Show when={p().restartRequired}>
        <p class={css.statusLine} role="status" data-testid="status-line">
          {p().loaded ? t('plugins-admin-restart-to-apply') : t('plugins-admin-restart-to-load')}
        </p>
      </Show>
      <Show when={!p().restartRequired ? p().notLoadedReason : null}>
        {(reason) => (
          <p class={css.statusLine} role="status" data-testid="status-line">
            {reasonText(reason())}
          </p>
        )}
      </Show>
      <Show when={p().loaded}>
        <p class={css.limits} data-testid="running-with">
          {t('plugins-admin-running-with', { capabilities: p().loadedCapabilities.join(', ') })}
        </p>
      </Show>

      <fieldset class={css.field}>
        <legend class={css.subheading}>{t('plugins-admin-grants-heading')}</legend>
        <span class={css.meta}>{t('plugins-admin-grants-hint')}</span>
        <For each={p().capabilities}>
          {(cap) => (
            <label class={css.check} data-testid="grant-check">
              <input
                type="checkbox"
                aria-label={t('plugins-admin-grant-for', { capability: cap, name: p().name })}
                checked={draft().includes(cap)}
                onChange={(e) =>
                  setDraft(e.currentTarget.checked ? [...draft(), cap] : draft().filter((c) => c !== cap))
                }
              />
              <span class={css.limits}>{cap}</span>
            </label>
          )}
        </For>
        <div class={css.row}>
          <button
            type="button"
            class={css.ghost}
            onClick={() => void slice.grant(p().id, { accountId: null, capabilities: draft() })}
          >
            {t('plugins-admin-grants-save')}
          </button>
        </div>
      </fieldset>

      <p class={css.limits}>
        {p().netAllowlist.length > 0
          ? t('plugins-admin-hosts', { hosts: p().netAllowlist.join(', ') })
          : t('plugins-admin-hosts-none')}
      </p>
      <p class={css.limits}>
        {p().limits.fuel !== null
          ? t('admin-plugins-limits-fuel', {
              memory: p().limits.memoryMb,
              deadline: p().limits.deadlineMs,
              fuel: p().limits.fuel ?? 0,
            })
          : t('admin-plugins-limits', { memory: p().limits.memoryMb, deadline: p().limits.deadlineMs })}
      </p>

      <Show when={p().role === 'spam-classifier'}>
        <label class={css.field}>
          <span>{t('plugins-admin-endpoint')}</span>
          <input
            class={css.input}
            aria-label={t('plugins-admin-endpoint-for', { name: p().name })}
            value={endpoint()}
            onInput={(e) => setEndpoint(e.currentTarget.value)}
          />
          <span class={css.meta}>{t('plugins-admin-endpoint-hint')}</span>
        </label>
        <div class={css.row}>
          <button
            type="button"
            class={css.ghost}
            onClick={() => void slice.setEndpoint(p().id, endpoint().trim() === '' ? null : endpoint().trim())}
          >
            {t('plugins-admin-endpoint-save')}
          </button>
          <button type="button" class={css.ghost} disabled={!p().loaded} onClick={() => void runTest()}>
            {t('plugins-admin-test')}
          </button>
        </div>
        <Show when={tested()}>
          {(result) => (
            <div data-testid="test-result">
              <p class={css.statusLine}>{t('plugins-admin-test-verdict', { verdict: result().verdict })}</p>
              <pre class={css.detail}>{JSON.stringify(result().detail, null, 2)}</pre>
            </div>
          )}
        </Show>
        <Show when={testError()}>
          {(error) => (
            <p class={css.error} role="alert">
              {refusalText(error())}
            </p>
          )}
        </Show>
      </Show>

      <Show when={!p().firstParty && !p().signed}>
        <label class={css.check} data-testid="allow-unsigned">
          <input
            type="checkbox"
            aria-label={t('admin-plugins-allow-unsigned-for', { name: p().name })}
            checked={p().allowUnsigned}
            onChange={(e) => void slice.setAllowUnsigned(p().id, e.currentTarget.checked)}
          />
          <span>{t('admin-plugins-allow-unsigned')}</span>
        </label>
      </Show>

      <div class={css.row}>
        <Show
          when={confirmUninstall()}
          fallback={
            <button type="button" class={css.ghost} onClick={() => setConfirmUninstall(true)}>
              {t('plugins-admin-uninstall')}
            </button>
          }
        >
          <span class={css.meta}>{t('plugins-admin-uninstall-note')}</span>
          <button type="button" class={css.danger} onClick={() => void slice.uninstall(p().id)}>
            {t('plugins-admin-uninstall-confirm')}
          </button>
          <button type="button" class={css.ghost} onClick={() => setConfirmUninstall(false)}>
            {t('common-cancel')}
          </button>
        </Show>
      </div>
    </li>
  );
}

export default AdminPlugins;
