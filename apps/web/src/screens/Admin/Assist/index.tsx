// V7 Admin → Assist governance screen (SPEC §14/§19, plan §2.6 / §3 e6). The
// tenant-wide policy surface: the master kill switch, the endpoint allowlist, the
// per-capability locks, and the data-class ceilings (E2EE / attachments — DENY by
// default). e14 mounts this as an admin section (see the wire-up note at the foot).
//
// This file does NOT touch the router or the shared Admin index/slice (ownership
// boundary — e7/e14 own those). It is a self-contained section component that
// fetches its own config over the admin session; tests inject a mock `api`.

import { createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { ASSIST_CAPABILITIES, type AssistCapability } from '../../../modules/assist/types.ts';
import { t, loadCatalog } from '../../../i18n';
import * as css from '../../../modules/assist/styles.css.ts';
import {
  AdminAssistApi,
  DEFAULT_ADMIN_ASSIST_CONFIG,
  type AdminAssistConfig,
  type CapabilityLock,
} from './service.ts';

/** Localised label for an Assist capability (source strings in admin.ftl). */
function capLabel(cap: AssistCapability): string {
  return t(`admin-assist-cap-${cap}`);
}

export interface AdminAssistProps {
  /** The governance client. Defaults to the same-origin admin HTTP client; tests inject a mock. */
  api?: AdminAssistApi;
}

export function AdminAssist(props: AdminAssistProps): JSX.Element {
  const api = props.api ?? new AdminAssistApi();
  const [config, setConfig] = createSignal<AdminAssistConfig>(DEFAULT_ADMIN_ASSIST_CONFIG);
  const [newHost, setNewHost] = createSignal('');
  const [status, setStatus] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [loaded, setLoaded] = createSignal(false);

  onMount(() => void loadCatalog('admin'));
  onMount(() => {
    void api
      .get()
      .then((c) => setConfig(c))
      .catch(() => setError(t('admin-assist-load-error')))
      .finally(() => setLoaded(true));
  });

  function patch(next: Partial<AdminAssistConfig>): void {
    setConfig((prev) => ({ ...prev, ...next }));
    setStatus(null);
  }

  function setLock(cap: AssistCapability, lock: CapabilityLock): void {
    patch({ capabilityLocks: { ...config().capabilityLocks, [cap]: lock } });
  }

  function addHost(): void {
    const host = newHost().trim().toLowerCase();
    if (host.length === 0) return;
    if (config().endpointAllowlist.includes(host)) {
      setNewHost('');
      return;
    }
    patch({ endpointAllowlist: [...config().endpointAllowlist, host] });
    setNewHost('');
  }

  function removeHost(host: string): void {
    patch({ endpointAllowlist: config().endpointAllowlist.filter((h) => h !== host) });
  }

  async function save(): Promise<void> {
    setError(null);
    try {
      await api.save(config());
      setStatus(t('admin-saved'));
    } catch {
      setError(t('admin-assist-save-error'));
    }
  }

  async function toggleKill(on: boolean): Promise<void> {
    setError(null);
    patch({ enabled: on });
    try {
      await api.setKillSwitch(on);
      setStatus(on ? t('admin-assist-enabled-status') : t('admin-assist-disabled-status'));
    } catch {
      setError(t('admin-assist-kill-error'));
    }
  }

  return (
    <section class={css.panel} data-screen="admin-assist" aria-label={t('admin-assist-title')}>
      <div class={css.section}>
        <h2 class={css.heading}>{t('admin-assist-title')}</h2>
        <p class={css.prose}>{t('admin-assist-intro')}</p>

        {/* Kill switch (§19) — the master gate. Off ⇒ every user sees no Assist UI. */}
        <label class={css.check}>
          <input
            type="checkbox"
            checked={config().enabled}
            aria-label={t('admin-assist-enable')}
            onChange={(e) => void toggleKill(e.currentTarget.checked)}
          />
          <span>{t('admin-assist-enabled')}</span>
        </label>
        <Show when={!config().enabled}>
          <p class={css.meta}>{t('admin-assist-off-note')}</p>
        </Show>
      </div>

      {/* Endpoint allowlist. */}
      <div class={css.section}>
        <span class={css.subHeading}>{t('admin-assist-allowlist')}</span>
        <p class={css.prose}>{t('admin-assist-allowlist-note')}</p>
        <form
          class={css.row}
          onSubmit={(e) => {
            e.preventDefault();
            addHost();
          }}
        >
          <input
            class={css.input}
            aria-label={t('admin-assist-host')}
            placeholder={t('admin-assist-host-placeholder')}
            value={newHost()}
            onInput={(e) => setNewHost(e.currentTarget.value)}
          />
          <button type="submit" class={css.ghost}>
            {t('admin-assist-add-host')}
          </button>
        </form>
        <div class={css.toolbar}>
          <For each={config().endpointAllowlist} fallback={<span class={css.meta}>{t('admin-assist-hosts-empty')}</span>}>
            {(host) => (
              <span class={css.badge} data-testid="allowlist-host">
                <span dir="auto">{host}</span>
                <button
                  type="button"
                  class={css.ghost}
                  aria-label={t('admin-assist-remove-host', { host })}
                  onClick={() => removeHost(host)}
                >
                  {t('admin-remove')}
                </button>
              </span>
            )}
          </For>
        </div>
      </div>

      {/* Per-capability locks. */}
      <div class={css.section}>
        <span class={css.subHeading}>{t('admin-assist-locks')}</span>
        <p class={css.prose}>{t('admin-assist-locks-note')}</p>
        <div class={css.field}>
          <For each={ASSIST_CAPABILITIES}>
            {(cap) => (
              <label class={css.check}>
                <input
                  type="checkbox"
                  checked={config().capabilityLocks[cap] === 'allowed'}
                  aria-label={capLabel(cap)}
                  onChange={(e) => setLock(cap, e.currentTarget.checked ? 'allowed' : 'locked')}
                />
                <span>{capLabel(cap)}</span>
                <Show when={config().capabilityLocks[cap] === 'locked'}>
                  <span class={css.meta}>{t('admin-assist-locked')}</span>
                </Show>
              </label>
            )}
          </For>
        </div>
      </div>

      {/* Data-class ceilings — default DENY. */}
      <div class={css.section}>
        <span class={css.subHeading}>{t('admin-assist-ceilings')}</span>
        <p class={css.prose}>{t('admin-assist-ceilings-note')}</p>
        <label class={css.check}>
          <input
            type="checkbox"
            checked={config().dataCeilings.includeE2ee}
            aria-label={t('admin-assist-allow-e2ee-label')}
            onChange={(e) =>
              patch({ dataCeilings: { ...config().dataCeilings, includeE2ee: e.currentTarget.checked } })
            }
          />
          <span>{t('admin-assist-allow-e2ee')}</span>
        </label>
        <label class={css.check}>
          <input
            type="checkbox"
            checked={config().dataCeilings.includeAttachments}
            aria-label={t('admin-assist-allow-attachments-label')}
            onChange={(e) =>
              patch({
                dataCeilings: { ...config().dataCeilings, includeAttachments: e.currentTarget.checked },
              })
            }
          />
          <span>{t('admin-assist-allow-attachments')}</span>
        </label>
      </div>

      <div class={css.row}>
        <button type="button" class={css.button} disabled={!loaded()} onClick={() => void save()}>
          {t('admin-assist-save')}
        </button>
        <Show when={status() !== null}>
          <span class={css.meta} role="status">
            {status()}
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

// ── e14 WIRE-UP NOTE (do NOT edit the shared Admin index/slice from here) ───────
// Add an 'assist' section to the admin panel by extending, in e14/e7-owned files:
//   • state/slices/admin.ts   ADMIN_SECTIONS += 'assist'; ADMIN_SECTION_LABELS.assist = 'Assist'
//   • screens/Admin/index.tsx SECTION_VIEWS.assist = () => <AdminAssist />   (default import here)
// Endpoints e9 must satisfy (admin session domain):
//   GET  /admin/assist        → WireAdminAssistConfig
//   PUT  /admin/assist        → save
//   POST /admin/assist/kill   → { on }   (kill switch)
