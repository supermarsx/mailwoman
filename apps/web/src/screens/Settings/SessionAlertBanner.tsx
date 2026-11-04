import { createResource, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { fetchSessionAlerts } from './sessionAlertsModel.ts';
import * as css from './styles.css.ts';

export interface SessionAlertBannerProps {
  /** Called when the user chooses to review the notices. */
  onReview?: () => void;
}

/**
 * One line shown when unread session security notices exist. It loads on mount
 * ("on next load"), shows nothing while loading, on error or with nothing unread.
 */
export function SessionAlertBanner(props: SessionAlertBannerProps): JSX.Element {
  void loadCatalog('security');
  const [alerts] = createResource(() => fetchSessionAlerts().catch(() => null));
  return (
    <Show when={(alerts()?.unread ?? 0) > 0}>
      <p class={css.prose} role="status" data-testid="session-alert-banner">
        {t('session-alerts-banner', { count: alerts()!.unread })}{' '}
        <Show when={props.onReview}>
          <button type="button" onClick={() => props.onReview?.()}>
            {t('session-alerts-banner-review')}
          </button>
        </Show>
      </p>
    </Show>
  );
}
export default SessionAlertBanner;
