// Mounts the new-version notice for a signed-in session (see newVersionNotice.ts).

import { createEffect, onCleanup, useContext, type JSX } from 'solid-js';
import { settings } from '../api/settings.ts';
import { buildId, fetchWebVersion } from '../api/webVersion.ts';
import { toast } from '../components/toast/index.ts';
import { t } from '../i18n/index.ts';
import { AppContext } from './context.ts';
import { createNewVersionNotice } from './newVersionNotice.ts';

export const NEW_VERSION_TOAST_ID = 'new-version-available';
/** Longer than the composer's 800 ms draft autosave debounce. */
const DRAFT_SETTLE_MS = 1000;

/** The setting is on unless the effective value is explicitly false. */
export const notifyNewVersionOn = (): boolean => settings()['ui.notifyNewVersion']?.value !== false;

/** Reload; when a composer is open, first let its debounced draft save finish. */
export function reloadNow(): void {
  const composing = typeof document !== 'undefined' && document.querySelector('form.compose') !== null;
  if (composing) setTimeout(() => location.reload(), DRAFT_SETTLE_MS);
  else location.reload();
}

export function NewVersionToast(): JSX.Element {
  const app = useContext(AppContext);
  const notice = createNewVersionNotice({
    fetchId: async () => {
      const v = await fetchWebVersion();
      return v === null ? null : buildId(v);
    },
    active: () => app?.me() != null && notifyNewVersionOn(),
    show: ({ reload, later }) => {
      toast.info(t('common-new-version'), {
        id: NEW_VERSION_TOAST_ID,
        duration: Infinity,
        action: {
          label: t('common-new-version-reload'),
          run: () => {
            reload();
            reloadNow();
          },
        },
        onClose: (reason) => {
          if (reason !== 'action') later();
        },
      });
    },
    hide: () => toast.dismiss(NEW_VERSION_TOAST_ID),
  });
  createEffect(() => {
    if (app?.me() != null && notifyNewVersionOn()) notice.start();
    else notice.stop();
  });
  onCleanup(() => notice.stop());
  return <></>;
}
