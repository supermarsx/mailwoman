// Admin › Re-thread mailbox (JWZ backfill) (t14 26.14, plan §Workstream-3 E5/E-mount).
//
// A maintenance action that drives the admin-gated one-shot JWZ backfill (E5's
// engine driver, exposed by E-mount at `POST /admin/maintenance/rethread`). The
// admin SELECTS a provisioned account (the SAME picker pattern E4 introduced in
// `ServerMetadata.tsx` — populated from `AdminApi.listUsers()`), then presses
// "Re-thread mailbox". Because re-threading RE-KEYS conversation grouping (existing
// threads may merge or split), the POST NEVER fires directly: the button opens an
// explicit confirmation dialog that warns of the effect; only its confirm button
// issues the request. On success the returned summary (reassigned count etc.) is
// shown; on failure an honest error state.
//
// It follows the SSO/metadata local-flag mount pattern: the frozen `AdminSection`
// union is untouched — `index.tsx` layers this in with a `rethreadActive` signal,
// exactly like `metaActive`/`ssoActive`.
//
// WCAG 2.2 AA: the account `<select>` is labelled; the confirmation dialog is a
// focus-trapped `role="dialog"` (aria-modal, Esc-to-close, focus restore via the
// shared `createFocusTrap`); the warning carries `role="alert"` so it is announced.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { createHttpAdminApi, type AdminApi } from '../../state/slices/admin.ts';
import {
  createHttpMaintenanceApi,
  type MaintenanceApi,
  type RethreadSummary,
} from '../../api/maintenance.ts';
import { createFocusTrap } from '../../components/a11y';
import { t, loadCatalog } from '../../i18n';
import { vars } from '../../theme/contract.css.ts';
import * as css from './admin.css.ts';

export interface RethreadMaintenanceProps {
  /**
   * Lists provisioned accounts for the account picker. Defaults to the same-origin
   * admin HTTP client; index.tsx threads the panel's shared client, tests a mock.
   */
  api?: AdminApi;
  /**
   * The maintenance client the confirm action drives. Defaults to the same-origin
   * admin client (`POST /admin/maintenance/rethread`); tests inject a fake.
   */
  maintenance?: MaintenanceApi;
}

export function RethreadMaintenance(props: RethreadMaintenanceProps): JSX.Element {
  const api = props.api ?? createHttpAdminApi();
  const maintenance = props.maintenance ?? createHttpMaintenanceApi();

  onMount(() => void loadCatalog('admin'));

  const [users] = createResource(() => api.listUsers());
  const [accountId, setAccountId] = createSignal<string | null>(null);
  const [confirmOpen, setConfirmOpen] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [summary, setSummary] = createSignal<RethreadSummary | null>(null);
  const [failed, setFailed] = createSignal(false);

  // Focus-trapped confirmation dialog: the trap arms while `confirmOpen`, moves
  // focus in, restores it on close, and Esc dismisses (non-destructive default —
  // Esc/Cancel never fires the POST).
  const [dialogEl, setDialogEl] = createSignal<HTMLDivElement>();
  createFocusTrap(dialogEl, { active: confirmOpen, onEscape: () => setConfirmOpen(false) });

  function openConfirm(): void {
    // Clear any prior result so the dialog is a clean confirmation each time.
    setSummary(null);
    setFailed(false);
    setConfirmOpen(true);
  }

  async function runRethread(): Promise<void> {
    const id = accountId();
    if (id === null) return; // guarded: the button is disabled without a selection
    setBusy(true);
    setFailed(false);
    setSummary(null);
    try {
      const result = await maintenance.rethread(id);
      setSummary(result);
      setConfirmOpen(false);
    } catch {
      setFailed(true);
      setConfirmOpen(false);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.section} aria-label={t('admin-rethread-title')} data-testid="admin-rethread">
      <h2 class={css.heading}>{t('admin-rethread-title')}</h2>
      <p class={css.note}>{t('admin-rethread-intro')}</p>

      <Show when={users.error as unknown}>
        <p class={css.error} role="alert">
          {t('admin-rethread-load-error')}
        </p>
      </Show>

      <label class="field">
        <span>{t('admin-rethread-account')}</span>
        <select
          value={accountId() ?? ''}
          data-testid="admin-rethread-account"
          onChange={(e) => setAccountId(e.currentTarget.value === '' ? null : e.currentTarget.value)}
        >
          <option value="">{t('admin-rethread-select-option')}</option>
          <For each={users() ?? []}>
            {(u) => <option value={u.accountId}>{`${u.username}@${u.domain}`}</option>}
          </For>
        </select>
      </label>

      <Show when={(users()?.length ?? 0) === 0 && !users.loading}>
        <p class={css.note}>{t('admin-rethread-no-accounts')}</p>
      </Show>

      <div>
        <button
          type="button"
          class="btn btn--primary"
          data-testid="admin-rethread-run"
          disabled={accountId() === null}
          onClick={openConfirm}
        >
          {t('admin-rethread-run')}
        </button>
      </div>

      <Show when={summary()}>
        {(s) => (
          <p class={css.note} role="status" data-testid="admin-rethread-summary">
            {t('admin-rethread-summary', {
              accounts: s().accounts,
              messages: s().messages,
              threads: s().threads,
              reassigned: s().reassigned,
            })}
          </p>
        )}
      </Show>

      <Show when={failed()}>
        <p class={css.error} role="alert" data-testid="admin-rethread-error">
          {t('admin-rethread-error')}
        </p>
      </Show>

      <Show when={confirmOpen()}>
        <div
          style={{
            position: 'fixed',
            inset: '0',
            display: 'grid',
            'place-items': 'center',
            background: 'rgba(0, 0, 0, 0.5)',
            padding: vars.space[4],
            'z-index': '1000',
          }}
        >
          <div
            ref={setDialogEl}
            role="dialog"
            aria-modal="true"
            aria-labelledby="admin-rethread-confirm-title"
            aria-describedby="admin-rethread-confirm-warning"
            tabindex="-1"
            data-testid="admin-rethread-dialog"
            style={{
              display: 'flex',
              'flex-direction': 'column',
              gap: vars.space[4],
              'max-width': '480px',
              width: '100%',
              padding: vars.space[5],
              background: vars.color.surface,
              color: vars.color.text,
              border: `1px solid ${vars.color.border}`,
              'border-radius': vars.radius.lg,
            }}
          >
            <h3 id="admin-rethread-confirm-title" class={css.heading}>
              {t('admin-rethread-confirm-title')}
            </h3>
            <p id="admin-rethread-confirm-warning" class={css.error} role="alert">
              {t('admin-rethread-confirm-warning')}
            </p>
            <p class={css.note}>{t('admin-rethread-confirm-detail')}</p>
            <div style={{ display: 'flex', gap: vars.space[3], 'justify-content': 'flex-end' }}>
              <button
                type="button"
                class="btn btn--ghost"
                data-testid="admin-rethread-cancel"
                onClick={() => setConfirmOpen(false)}
              >
                {t('admin-rethread-cancel')}
              </button>
              <button
                type="button"
                class="btn btn--primary"
                data-testid="admin-rethread-confirm"
                disabled={busy()}
                onClick={() => void runRethread()}
              >
                {busy() ? t('admin-rethread-running') : t('admin-rethread-confirm')}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </section>
  );
}

export default RethreadMaintenance;
