// Remote-images banner (t16 §S8/S9, e14b; compact single line since t55) — the
// reader affordance for the image proxy's deny-by-default remote images (SPEC
// §7.2). STANDALONE and presentational: it takes the blocked-content report,
// the message context, the policy mode and the grant/revoke callbacks as props
// and fetches nothing, so the reader wires the real `RemoteImageApi`
// (`api/remote-images.ts`). The callbacks default to no-ops so it renders and
// tests on its own.
//
// One line, always:
//   ask, blocked  — "Remote images blocked · 3 trackers removed  [Show images] [▾]"
//                   The menu holds the other grant scopes that exist today
//                   (sender / domain / all mail) and a Privacy settings link.
//   ask, allowed  — "Images shown · Undo" (revokes the grant that covers it).
//   proxy/direct  — what the policy did, plus the tracker count when known.
//   block         — blocked by policy; only the settings link (a grant would
//                   not take effect in block mode, so none is offered).
// The tracker count is shown only when it is above zero. The explanation and
// the blocked host list live in the summary's tooltip, not in the row.
// At narrow container widths the text ellipsises (full text in its tooltip)
// and the actions collapse to icon buttons with tooltips and aria-labels.
//
// Copy is factual (no hype). Untrusted host names are bidi-isolated (SPEC §24).

import { Show, createMemo, createSignal, type JSX } from 'solid-js';
import { t } from '../i18n/index.ts';
import { toast } from './toast/toast.ts';
import { Icon, type IconName } from './Icon.tsx';
import { createContextMenu, type ContextAction } from './ContextMenu.tsx';
import * as css from './remote-content-bar.css.ts';
import type { RemoteMode } from '../api/remote-content.ts';
import {
  scopeFor,
  senderDomain,
  type BlockedContentReport,
  type GrantScope,
  type GrantScopeKind,
  type RemoteImageGrant,
} from '../api/remote-images.ts';

export interface RemoteContentBarProps {
  mode?: RemoteMode;
  /** The open message id (the `single`-scope grant value). */
  emailId: string;
  /** The sender address (drives the per-sender / per-domain scopes). */
  sender: string;
  /** What the sanitizer blocked in this body (from `analyzeBlockedContent`). */
  report: BlockedContentReport;
  /** An active grant already covering this message, or `null` when blocked. */
  activeGrant?: RemoteImageGrant | null;
  /** Settled count of trackers removed beyond the HTML ones (server-decoded). */
  decodedTrackers?: { count: number; limited: boolean } | null;
  /** Persist a grant for a scope. Defaults to a no-op. */
  onGrant?: (scope: GrantScope) => Promise<void> | void;
  /** Revoke a grant. Defaults to a no-op. */
  onRevoke?: (scope: GrantScope) => Promise<void> | void;
  /** Open Settings › Privacy. The menu item is shown only when given. */
  onOpenPrivacySettings?: () => void;
}

const MENU_KINDS: GrantScopeKind[] = ['per-sender', 'per-domain', 'all'];
const isolate = (s: string): string => `⁨${s}⁩`;
const SEP = ' · ';

async function settle(v: Promise<void> | void): Promise<void> {
  if (v instanceof Promise) await v;
}

export function RemoteContentBar(props: RemoteContentBarProps): JSX.Element {
  const [pending, setPending] = createSignal(false);
  const mode = (): RemoteMode => props.mode ?? 'ask';
  const granted = (): boolean => props.activeGrant !== null && props.activeGrant !== undefined;
  const domain = createMemo(() => senderDomain(props.sender));
  const ctx = (): { emailId: string; sender: string } => ({ emailId: props.emailId, sender: props.sender });

  // Trackers removed: the ones the sanitizer classified in the HTML plus the
  // ones the server recognised after decoding. Unknown or zero: not shown.
  const trackers = createMemo(() => {
    const decoded = props.decodedTrackers;
    const count = props.report.trackerCount + (decoded?.count ?? 0);
    return count > 0 ? { count, limited: decoded?.limited === true } : null;
  });
  const trackerText = (): string | null => {
    const tr = trackers();
    return tr === null ? null : t(tr.limited ? 'remote-banner-trackers-limited' : 'remote-banner-trackers', { count: tr.count });
  };

  const headline = createMemo(() => {
    if (mode() === 'block') return t('remote-banner-blocked-policy');
    if (mode() === 'proxy') return t('remote-banner-proxied');
    if (mode() === 'direct') return t('remote-banner-direct');
    return granted() ? t('remote-banner-shown') : t('remote-banner-blocked');
  });
  const icon = (): IconName => (mode() === 'ask' && !granted()) || mode() === 'block' ? 'shield' : 'image';

  /** The explanation moved out of the row: the tooltip of the summary text. */
  const help = createMemo(() => {
    const parts = [mode() === 'ask' && !granted() ? t('remote-banner-blocked-help') : t(`remote-banner-help-${mode()}`)];
    const hosts = props.report.blockedHosts;
    if (hosts.length > 0 && !granted() && mode() !== 'proxy' && mode() !== 'direct') {
      const shown = hosts.slice(0, 8).map(isolate).join(', ');
      parts.push(t('remote-banner-hosts', { hosts: hosts.length > 8 ? `${shown}, …` : shown }));
    }
    if (trackers() !== null) parts.push(t('remote-banner-trackers-help'));
    return parts.join('\n');
  });
  const fullText = (): string => [headline(), trackerText()].filter(Boolean).join(SEP);

  async function grant(kind: GrantScopeKind): Promise<void> {
    if (pending()) return;
    setPending(true);
    try {
      await settle((props.onGrant ?? (() => undefined))(scopeFor(kind, ctx())));
    } catch {
      toast.error(t('remote-change-failed'));
    } finally {
      setPending(false);
    }
  }

  async function undo(): Promise<void> {
    const g = props.activeGrant;
    if (g === null || g === undefined || pending()) return;
    setPending(true);
    try {
      await settle((props.onRevoke ?? (() => undefined))({ kind: g.scopeKind, value: g.scopeValue }));
    } catch {
      toast.error(t('remote-change-failed'));
    } finally {
      setPending(false);
    }
  }

  const undoHelp = (): string => {
    const g = props.activeGrant;
    if (!g) return '';
    if (g.scopeKind === 'per-sender') return t('remote-banner-undo-help-per-sender', { sender: isolate(g.scopeValue) });
    if (g.scopeKind === 'per-domain') return t('remote-banner-undo-help-per-domain', { domain: isolate(g.scopeValue) });
    return t(`remote-banner-undo-help-${g.scopeKind}`);
  };

  const menuActions = createMemo<ContextAction[]>(() => {
    const items: ContextAction[] = [];
    if (mode() === 'ask' && !granted()) {
      for (const kind of MENU_KINDS) {
        if (kind === 'per-sender' && props.sender === '') continue;
        if (kind === 'per-domain' && domain() === '') continue;
        const label = kind === 'per-sender'
          ? t('remote-banner-always-sender', { sender: isolate(props.sender) })
          : kind === 'per-domain'
            ? t('remote-banner-always-domain', { domain: isolate(domain()) })
            : t('remote-banner-always-all');
        items.push({ id: kind, label, icon: kind === 'all' ? 'image' : kind === 'per-domain' ? 'domains' : 'users', disabled: pending(), run: () => grant(kind) });
      }
    }
    const open = props.onOpenPrivacySettings;
    if (open) items.push({ id: 'settings', label: t('remote-banner-settings'), icon: 'privacy', separatorBefore: items.length > 0, run: () => open() });
    return items;
  });
  const menu = createContextMenu({
    label: () => t('remote-banner-more'),
    scope: () => props.emailId,
    actions: menuActions,
    onError: () => toast.error(t('remote-change-failed')),
  });

  return (
    <section class={css.root} aria-label={t('remote-bar-label')} data-testid="remote-content-bar" data-state={mode() === 'ask' ? (granted() ? 'shown' : 'blocked') : mode()}>
      <div class={css.row}>
        <span class={css.mark} aria-hidden="true"><Icon name={icon()} /></span>
        {/* Polite: announces "Images shown" after Show images, never takes focus. */}
        <span class={css.summary} role="status" aria-live="polite" data-tooltip={`${fullText()}\n${help()}`} tabIndex={0}>
          <span class={css.headline}>{headline()}</span>
          <Show when={trackerText()}>{(text) => <span class={css.trackers}>{SEP}{text()}</span>}</Show>
        </span>
        <span class={css.actions}>
          <Show when={mode() === 'ask' && !granted()}>
            <button type="button" class={`${css.btn} ${css.btnPrimary}`} disabled={pending()} data-tooltip={t('remote-banner-show-help')} aria-label={t('remote-banner-show')} onClick={() => void grant('single')}>
              <Icon name="eye" /><span class={css.label}>{t('remote-banner-show')}</span>
            </button>
          </Show>
          <Show when={mode() === 'ask' && granted()}>
            <button type="button" class={css.btn} disabled={pending()} data-tooltip={undoHelp()} aria-label={`${t('remote-banner-undo')}: ${undoHelp()}`} onClick={() => void undo()}>
              <Icon name="restore" /><span class={css.label}>{t('remote-banner-undo')}</span>
            </button>
          </Show>
          <Show when={menuActions().length > 0}>
            <button type="button" class={`${css.btn} ${css.iconBtn}`} aria-haspopup="menu" aria-expanded={menu.isOpen()} aria-label={t('remote-banner-more')} data-tooltip={t('remote-banner-more')} onClick={(e) => menu.open(e)} onKeyDown={menu.keydown}>
              <span class={css.chevron}><Icon name="chevron" /></span>
            </button>
          </Show>
        </span>
      </div>
      {menu.menu()}
    </section>
  );
}
