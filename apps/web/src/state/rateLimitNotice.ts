// The one toast for a server that stays busy (see `api/rateAware.ts`).
//
// A request that is waited out and then succeeds shows nothing. When a request
// that is safe to repeat has used its retries, one toast says so, with the
// seconds left until the server said to try again; it goes away as soon as a
// request succeeds again. A page that keeps being refused never stacks toasts:
// the id replaces the one on screen.

import { onRateEvent } from '../api/rateAware.ts';
import { t } from '../i18n/index.ts';
import { toast } from '../components/toast/index.ts';

export const RATE_LIMIT_TOAST_ID = 'rate-limited';
let installed = false;

/** Subscribe once; safe to call again. */
export function installRateLimitNotice(): void {
  if (installed) return;
  installed = true;
  onRateEvent((event) => {
    if (event.kind === 'exhausted') {
      const left = Math.max(3_000, Math.min(event.untilMs - Date.now(), 60_000));
      toast.warning(t('common-rate-limited'), { id: RATE_LIMIT_TOAST_ID, duration: left, countdown: true });
    } else if (event.kind === 'clear') {
      toast.dismiss(RATE_LIMIT_TOAST_ID);
    }
  });
}
