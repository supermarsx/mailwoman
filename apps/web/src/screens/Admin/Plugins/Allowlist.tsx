// Admin → Plugins → Third-party allowlist panel (§7.2 / t15 26.15, plan §3 e7).
//
// The trust surface for the ONLY security-core loosening in 26.15: a NON-first-party
// component loads only after an administrator approves its exact SHA-256 digest. This
// panel lists the components an operator has placed in the third-party plugin directory
// with the digest the SERVER computed over their on-disk bytes (the value the admin is
// approving), and lets the admin approve that exact digest, revoke a pin, or uninstall a
// plugin (which also purges its stored key/value data).
//
// Every state-changing action is a security action, so each goes through an explicit,
// focus-trapped confirmation dialog that names exactly what it does (the approve dialog
// shows the exact digest being trusted). Copy is factual — neither alarmist nor
// reassuring-marketing (memory: no-hype-wording).
//
// It surfaces two rules honestly rather than letting the admin discover them by a
// rejected request:
//   * High-power capabilities (the account-backend / send-as-user class) are refused to
//     third-party plugins at grant time by the server regardless of admin action, so they
//     are shown as first-party-only here.
//   * A component admitted by digest pin without a signature is expected — a neutral
//     informational note, not a warning. The digest pin is what authorizes loading.

import { createEffect, For, onMount, Show, createSignal, type JSX } from 'solid-js';
import {
  createPluginsSlice,
  createHttpPluginsApi,
  HIGH_POWER_CAPABILITIES,
  type PluginsApi,
  type PluginsSlice,
  type AllowlistPresent,
  type AllowlistPin,
} from '../../../state/slices/plugins.ts';
import { createFocusTrap } from '../../../components/a11y';
import { t, loadCatalog } from '../../../i18n';
import { vars } from '../../../theme/contract.css.ts';
import * as css from './styles.css.ts';

export interface AllowlistPanelProps {
  /** Tests / the admin shell inject a slice or a client; production defaults to HTTP. */
  slice?: PluginsSlice;
  api?: PluginsApi;
}

/** A pending confirmation: which action, and the row it targets. */
type Pending =
  | { readonly kind: 'approve'; readonly pluginId: string; readonly digestHex: string }
  | { readonly kind: 'revoke'; readonly pluginId: string; readonly digestHex: string }
  | { readonly kind: 'uninstall'; readonly pluginId: string };

/** The comma-joined high-power capability list shown in the not-grantable note. */
const HIGH_POWER_LABEL = HIGH_POWER_CAPABILITIES.join(', ');

export function AllowlistPanel(props: AllowlistPanelProps): JSX.Element {
  const slice = props.slice ?? createPluginsSlice(props.api ?? createHttpPluginsApi());

  onMount(() => void loadCatalog('admin'));
  createEffect(() => {
    void slice.loadAllowlist();
  });

  const [pending, setPending] = createSignal<Pending | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [failed, setFailed] = createSignal(false);

  const [dialogEl, setDialogEl] = createSignal<HTMLDivElement>();
  createFocusTrap(dialogEl, { active: () => pending() !== null, onEscape: () => setPending(null) });

  async function runPending(): Promise<void> {
    const p = pending();
    if (p === null) return;
    setBusy(true);
    setFailed(false);
    try {
      if (p.kind === 'approve') await slice.approveDigest(p.pluginId, p.digestHex);
      else if (p.kind === 'revoke') await slice.revokeDigest(p.pluginId, p.digestHex);
      else await slice.uninstall(p.pluginId);
      setPending(null);
    } catch {
      setFailed(true);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.screen} data-screen="admin-allowlist" aria-label={t('admin-allowlist-title')}>
      <div>
        <h2 class={css.heading}>{t('admin-allowlist-title')}</h2>
        <p class={css.prose}>{t('admin-allowlist-intro')}</p>
      </div>

      <Show when={failed()}>
        <p class={css.error} role="alert" data-testid="allowlist-error">
          {t('admin-allowlist-load-error')}
        </p>
      </Show>

      <div>
        <h3 class={css.heading}>{t('admin-allowlist-present-heading')}</h3>
        <Show
          when={!slice.allowlistLoading() && slice.allowlist().present.length === 0}
        >
          <p class={css.meta}>{t('admin-allowlist-present-empty')}</p>
        </Show>
        <ul class={css.list}>
          <For each={slice.allowlist().present}>
            {(row) => <PresentCard row={row} onAct={setPending} />}
          </For>
        </ul>
      </div>

      <PinsList pins={slice.allowlist().pins} />

      <Show when={pending()}>
        {(p) => (
          <ConfirmDialog
            pending={p()}
            busy={busy()}
            setRef={setDialogEl}
            onCancel={() => setPending(null)}
            onConfirm={() => void runPending()}
          />
        )}
      </Show>
    </section>
  );
}

/** One present-on-disk component: its computed digest and the actions available for it. */
function PresentCard(props: {
  row: AllowlistPresent;
  onAct: (p: Pending) => void;
}): JSX.Element {
  const r = (): AllowlistPresent => props.row;

  return (
    <li class={css.card} data-plugin-id={r().pluginId} data-testid="allowlist-present-card">
      <div class={css.cardHead}>
        <div>
          <p class={css.title} dir="auto">
            {r().pluginId}
          </p>
          <div class={css.row}>
            <Show
              when={r().firstParty}
              fallback={
                <Show
                  when={r().approved}
                  fallback={
                    <span class={css.chip} data-testid="allowlist-status">
                      {t('admin-allowlist-status-pending')}
                    </span>
                  }
                >
                  <span class={`${css.chip} ${css.signedChip}`} data-testid="allowlist-status">
                    {t('admin-allowlist-status-approved')}
                  </span>
                </Show>
              }
            >
              <span class={css.chip} data-testid="allowlist-status">
                {t('admin-allowlist-status-firstparty')}
              </span>
            </Show>
          </div>
        </div>
        <div class={css.row}>
          <Show when={!r().firstParty && !r().approved}>
            <button
              type="button"
              class={css.button}
              aria-label={t('admin-allowlist-approve-for', { id: r().pluginId })}
              onClick={() => props.onAct({ kind: 'approve', pluginId: r().pluginId, digestHex: r().computedDigest })}
            >
              {t('admin-allowlist-approve')}
            </button>
          </Show>
          <Show when={!r().firstParty && r().approved}>
            <button
              type="button"
              class={css.danger}
              aria-label={t('admin-allowlist-revoke-for', { id: r().pluginId })}
              onClick={() => props.onAct({ kind: 'revoke', pluginId: r().pluginId, digestHex: r().computedDigest })}
            >
              {t('admin-allowlist-revoke')}
            </button>
            <button
              type="button"
              class={css.danger}
              aria-label={t('admin-allowlist-uninstall-for', { id: r().pluginId })}
              onClick={() => props.onAct({ kind: 'uninstall', pluginId: r().pluginId })}
            >
              {t('admin-allowlist-uninstall')}
            </button>
          </Show>
        </div>
      </div>

      <div>
        <p class={css.fieldLabel}>{t('admin-allowlist-digest-label')}</p>
        <p class={css.digest} data-testid="allowlist-digest">
          {r().computedDigest}
        </p>
      </div>

      <Show
        when={r().firstParty}
        fallback={
          <>
            <p class={css.infoNote} data-testid="allowlist-unsigned-note">
              {t('admin-allowlist-unsigned-note')}
            </p>
            <p class={css.infoNote} data-testid="allowlist-highpower-note">
              {t('admin-allowlist-highpower-note', { caps: HIGH_POWER_LABEL })}
            </p>
          </>
        }
      >
        <p class={css.infoNote} data-testid="allowlist-firstparty-note">
          {t('admin-allowlist-firstparty-note')}
        </p>
      </Show>
    </li>
  );
}

/** The stored pins (approved + revoked) for oversight. */
function PinsList(props: { pins: AllowlistPin[] }): JSX.Element {
  return (
    <div>
      <h3 class={css.heading}>{t('admin-allowlist-pins-heading')}</h3>
      <Show when={props.pins.length === 0}>
        <p class={css.meta}>{t('admin-allowlist-pins-empty')}</p>
      </Show>
      <ul class={css.list}>
        <For each={props.pins}>
          {(pin) => (
            <li
              class={`${css.card} ${pin.revoked ? css.revokedRow : ''}`}
              data-plugin-id={pin.pluginId}
              data-testid="allowlist-pin"
            >
              <div class={css.cardHead}>
                <p class={css.title} dir="auto">
                  {pin.name ?? pin.pluginId}
                  <Show when={pin.version}>
                    {' '}
                    <span class={css.meta}>{t('admin-plugins-version', { version: pin.version ?? '' })}</span>
                  </Show>
                </p>
                <Show when={pin.revoked}>
                  <span class={`${css.chip} ${css.unsignedChip}`}>{t('admin-allowlist-pin-revoked')}</span>
                </Show>
              </div>
              <p class={css.digest}>{pin.digestHex}</p>
              <p class={css.meta}>
                {t('admin-allowlist-pin-approved-by', { by: pin.approvedBy, at: pin.approvedAt })}
              </p>
            </li>
          )}
        </For>
      </ul>
    </div>
  );
}

/** The focus-trapped confirmation for a security action. The approve variant shows the
 *  exact digest being trusted; revoke/uninstall name exactly what they change/delete. */
function ConfirmDialog(props: {
  pending: Pending;
  busy: boolean;
  setRef: (el: HTMLDivElement) => void;
  onCancel: () => void;
  onConfirm: () => void;
}): JSX.Element {
  const p = (): Pending => props.pending;
  const titleKey = (): string =>
    p().kind === 'approve'
      ? 'admin-allowlist-approve-title'
      : p().kind === 'revoke'
        ? 'admin-allowlist-revoke-title'
        : 'admin-allowlist-uninstall-title';
  const detailKey = (): string =>
    p().kind === 'approve'
      ? 'admin-allowlist-approve-detail'
      : p().kind === 'revoke'
        ? 'admin-allowlist-revoke-detail'
        : 'admin-allowlist-uninstall-detail';
  const confirmKey = (): string =>
    p().kind === 'approve'
      ? 'admin-allowlist-approve-confirm'
      : p().kind === 'revoke'
        ? 'admin-allowlist-revoke-confirm'
        : 'admin-allowlist-uninstall-confirm';

  return (
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
        ref={props.setRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="allowlist-confirm-title"
        aria-describedby="allowlist-confirm-detail"
        tabindex="-1"
        data-testid="allowlist-dialog"
        style={{
          display: 'flex',
          'flex-direction': 'column',
          gap: vars.space[4],
          'max-width': '520px',
          width: '100%',
          padding: vars.space[5],
          background: vars.color.surface,
          color: vars.color.text,
          border: `1px solid ${vars.color.border}`,
          'border-radius': vars.radius.lg,
        }}
      >
        <h3 id="allowlist-confirm-title" class={css.heading}>
          {t(titleKey())}
        </h3>
        <p id="allowlist-confirm-detail" class={css.prose}>
          {t(detailKey())}
        </p>
        <p class={css.meta} dir="auto">
          {p().pluginId}
        </p>
        <Show when={p().kind === 'approve'}>
          <div>
            <p class={css.fieldLabel}>{t('admin-allowlist-digest-label')}</p>
            <p class={css.digest} data-testid="allowlist-dialog-digest">
              {p().kind === 'approve' ? (p() as { digestHex: string }).digestHex : ''}
            </p>
          </div>
        </Show>
        <div style={{ display: 'flex', gap: vars.space[3], 'justify-content': 'flex-end' }}>
          <button
            type="button"
            class={css.ghost}
            data-testid="allowlist-cancel"
            onClick={() => props.onCancel()}
          >
            {t('admin-allowlist-cancel')}
          </button>
          <button
            type="button"
            class={css.danger}
            data-testid="allowlist-confirm"
            disabled={props.busy}
            onClick={() => props.onConfirm()}
          >
            {t(confirmKey())}
          </button>
        </div>
      </div>
    </div>
  );
}

export default AllowlistPanel;
