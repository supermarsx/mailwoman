// Pieces the Plugins list, its detail and the UI plugins screen share: the
// refusal and not-loaded sentences, and the trust and status badges.

import { Show, type JSX } from 'solid-js';
import { PluginsApiError, type NotLoadedReason, type PluginInfo } from '../../../state/slices/plugins.ts';
import { t } from '../../../i18n';
import { StatusBadge } from '../shell/StatusBadge.tsx';
import * as css from '../shell/extras.css.ts';

/** Split a comma-separated host list into its non-empty entries. */
export function parseHosts(text: string): string[] {
  return text
    .split(',')
    .map((h) => h.trim())
    .filter((h) => h.length > 0);
}

/** The line explaining why a plugin is not loaded. */
export function reasonText(reason: NotLoadedReason): string {
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
export function refusalText(error: PluginsApiError): string {
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

/** Running, restart needed, or not running: what the server reports. */
export function StatusCell(props: { plugin: PluginInfo }): JSX.Element {
  const p = (): PluginInfo => props.plugin;
  return (
    <Show
      when={!p().restartRequired}
      fallback={
        <StatusBadge tone="warn" status="restart" testId="status-chip">
          {t('plugins-admin-status-restart')}
        </StatusBadge>
      }
    >
      <StatusBadge tone={p().loaded ? 'ok' : 'off'} status={p().loaded ? 'loaded' : 'not-loaded'} testId="status-chip">
        {p().loaded ? t('plugins-admin-status-loaded') : t('plugins-admin-status-not-loaded')}
      </StatusBadge>
    </Show>
  );
}

/** First-party or approved third-party, and for a third-party one signed or not. */
export function TrustCell(props: { plugin: PluginInfo }): JSX.Element {
  const p = (): PluginInfo => props.plugin;
  return (
    <span class={css.badgeRow}>
      <StatusBadge tone="neutral" testId="trust-chip">
        {p().firstParty ? t('plugins-admin-trust-first-party') : t('plugins-admin-trust-third-party')}
      </StatusBadge>
      {/* A first-party component is trusted by its digest; signed or not is a third-party question. */}
      <Show when={!p().firstParty}>
        <StatusBadge tone={p().signed ? 'ok' : 'warn'} status={p().signed ? 'signed' : 'unsigned'} testId="sig-chip">
          {p().signed ? t('admin-plugins-signed') : t('admin-plugins-unsigned')}
        </StatusBadge>
      </Show>
    </span>
  );
}

