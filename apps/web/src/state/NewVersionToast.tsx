import {notifyEvent} from '../realtime/notificationEvents.ts';
// Mounts the new-version notice for a signed-in session (see newVersionNotice.ts).

import { createEffect, onCleanup, useContext, type JSX } from 'solid-js';
import { settings } from '../api/settings.ts';
import { buildId, fetchWebVersion } from '../api/webVersion.ts';
import { toast } from '../components/toast/index.ts';
import { t } from '../i18n/index.ts';
import { AppContext } from './context.ts';
import { createNewVersionNotice } from './newVersionNotice.ts';
import {publishAvailableUpdate} from './notificationCenter.ts';

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
      const id=v===null?null:buildId(v);
      if(id&&notice.booted()!==null&&id!==notice.booted())publishAvailableUpdate(id);
      return id;
    },
    active: () => app?.me() != null && notifyNewVersionOn(),
    show: ({ reload, later }) => {
      notifyEvent('update',t('common-new-version'),'info',NEW_VERSION_TOAST_ID,()=>toast.info(t('common-new-version'), {
        respectPreferences:true,
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
      }));
    },
    hide: () => toast.dismiss(NEW_VERSION_TOAST_ID),
  });
  createEffect(() => {
    if (app?.me() != null && notifyNewVersionOn()) notice.start();
    else notice.stop();
  });
  // t52w: the stream's first WebVersion state is only this page's starting point
  // (the load already asked /api/version): it runs the ordinary check, which the
  // 30 s gap skips. A state that moves after that forces a check.
  let webVersion:string|undefined;
  const stopRealtime=app?.onRealtimeChange?.(change=>{
    const states=Object.values(change.changed).map(state=>state.WebVersion).filter(value=>value!==undefined);
    if(!states.length)return;const next=String(states[0]),moved=webVersion!==undefined&&next!==webVersion;webVersion=next;
    void notice.check(moved);
  });
  onCleanup(()=>{stopRealtime?.();notice.stop();});
  return <></>;
}
