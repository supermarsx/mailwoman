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
import * as css from '../../../modules/assist/styles.css.ts';
import {
  AdminAssistApi,
  DEFAULT_ADMIN_ASSIST_CONFIG,
  type AdminAssistConfig,
  type CapabilityLock,
} from './service.ts';

const CAP_LABELS: Record<AssistCapability, string> = {
  summarize: 'Summarize',
  draft: 'Draft & rewrite',
  grammar: 'Grammar',
  dictation: 'Dictation',
  'search-semantic': 'Semantic search',
  'auto-tag': 'Auto-tag',
  recap: 'Recap',
  assistant: 'Assistant (chat)',
};

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

  onMount(() => {
    void api
      .get()
      .then((c) => setConfig(c))
      .catch(() => setError('Could not load Assist policy.'))
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
      setStatus('Saved.');
    } catch {
      setError('Save failed.');
    }
  }

  async function toggleKill(on: boolean): Promise<void> {
    setError(null);
    patch({ enabled: on });
    try {
      await api.setKillSwitch(on);
      setStatus(on ? 'Assist enabled.' : 'Assist disabled tenant-wide (kill switch).');
    } catch {
      setError('Could not change the kill switch.');
    }
  }

  return (
    <section class={css.panel} data-screen="admin-assist" aria-label="Assist">
      <div class={css.section}>
        <h2 class={css.heading}>Assist</h2>
        <p class={css.prose}>
          Assist proxies selected message text to an AI endpoint you configure. It never sends, deletes, or
          accepts mail on a user's behalf — those always require a person. End-to-end-encrypted content and
          attachments are withheld unless you explicitly allow them below.
        </p>

        {/* Kill switch (§19) — the master gate. Off ⇒ every user sees no Assist UI. */}
        <label class={css.check}>
          <input
            type="checkbox"
            checked={config().enabled}
            aria-label="Enable Assist tenant-wide"
            onChange={(e) => void toggleKill(e.currentTarget.checked)}
          />
          <span>Assist enabled tenant-wide</span>
        </label>
        <Show when={!config().enabled}>
          <p class={css.meta}>Assist is off. The kill switch reports the gateway as disabled to every user.</p>
        </Show>
      </div>

      {/* Endpoint allowlist. */}
      <div class={css.section}>
        <span class={css.subHeading}>Endpoint allowlist</span>
        <p class={css.prose}>Only these hosts may receive proxied requests. Anything else is refused.</p>
        <form
          class={css.row}
          onSubmit={(e) => {
            e.preventDefault();
            addHost();
          }}
        >
          <input
            class={css.input}
            aria-label="Endpoint host"
            placeholder="api.openai.com"
            value={newHost()}
            onInput={(e) => setNewHost(e.currentTarget.value)}
          />
          <button type="submit" class={css.ghost}>
            Add host
          </button>
        </form>
        <div class={css.toolbar}>
          <For each={config().endpointAllowlist} fallback={<span class={css.meta}>No hosts yet.</span>}>
            {(host) => (
              <span class={css.badge} data-testid="allowlist-host">
                <span>{host}</span>
                <button
                  type="button"
                  class={css.ghost}
                  aria-label={`Remove ${host}`}
                  onClick={() => removeHost(host)}
                >
                  Remove
                </button>
              </span>
            )}
          </For>
        </div>
      </div>

      {/* Per-capability locks. */}
      <div class={css.section}>
        <span class={css.subHeading}>Capability locks</span>
        <p class={css.prose}>A locked capability is never offered, regardless of per-user grants.</p>
        <div class={css.field}>
          <For each={ASSIST_CAPABILITIES}>
            {(cap) => (
              <label class={css.check}>
                <input
                  type="checkbox"
                  checked={config().capabilityLocks[cap] === 'allowed'}
                  aria-label={CAP_LABELS[cap]}
                  onChange={(e) => setLock(cap, e.currentTarget.checked ? 'allowed' : 'locked')}
                />
                <span>{CAP_LABELS[cap]}</span>
                <Show when={config().capabilityLocks[cap] === 'locked'}>
                  <span class={css.meta}>Locked</span>
                </Show>
              </label>
            )}
          </For>
        </div>
      </div>

      {/* Data-class ceilings — default DENY. */}
      <div class={css.section}>
        <span class={css.subHeading}>Data-class ceilings</span>
        <p class={css.prose}>
          These are hard limits. Even a granted capability cannot exceed them. Both are off by default.
        </p>
        <label class={css.check}>
          <input
            type="checkbox"
            checked={config().dataCeilings.includeE2ee}
            aria-label="Allow end-to-end-encrypted content to be sent"
            onChange={(e) =>
              patch({ dataCeilings: { ...config().dataCeilings, includeE2ee: e.currentTarget.checked } })
            }
          />
          <span>Allow end-to-end-encrypted content to leave the deployment</span>
        </label>
        <label class={css.check}>
          <input
            type="checkbox"
            checked={config().dataCeilings.includeAttachments}
            aria-label="Allow attachments to be sent"
            onChange={(e) =>
              patch({
                dataCeilings: { ...config().dataCeilings, includeAttachments: e.currentTarget.checked },
              })
            }
          />
          <span>Allow attachments to leave the deployment</span>
        </label>
      </div>

      <div class={css.row}>
        <button type="button" class={css.button} disabled={!loaded()} onClick={() => void save()}>
          Save policy
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
